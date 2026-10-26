package com.nix.stsms.map

import com.nix.stsms.domain.TaskCapsule
import kotlin.math.abs

enum class BrailleColorRole {
    NONE,
    WATER,
    MOTORWAY,
    PRIMARY_ROAD,
    ROUTE_LINE,
    MARKER_PIN,
    LABEL_TEXT
}

data class BrailleCell(
    val ch: Char,
    val role: BrailleColorRole
)

/**
 * MapSCII 스타일 2x4 유니코드 점자(U+2800..U+28FF) 래스터 캔버스
 */
class BrailleCanvas(
    val charWidth: Int,
    val charHeight: Int
) {
    val pixelWidth: Int = charWidth * 2
    val pixelHeight: Int = charHeight * 4

    private val dots = IntArray(charWidth * charHeight)
    private val roles = Array(charWidth * charHeight) { BrailleColorRole.NONE }
    private val textOverlay = CharArray(charWidth * charHeight) { '\u0000' }

    fun setPixel(px: Int, py: Int, role: BrailleColorRole) {
        if (px !in 0 until pixelWidth || py !in 0 until pixelHeight) return
        val cx = px / 2
        val cy = py / 4
        val dx = px % 2
        val dy = py % 4

        val bit = when (dx to dy) {
            0 to 0 -> 0x01
            0 to 1 -> 0x02
            0 to 2 -> 0x04
            1 to 0 -> 0x08
            1 to 1 -> 0x10
            1 to 2 -> 0x20
            0 to 3 -> 0x40
            1 to 3 -> 0x80
            else -> 0
        }
        val idx = cy * charWidth + cx
        dots[idx] = dots[idx] or bit
        if (role.ordinal >= roles[idx].ordinal) {
            roles[idx] = role
        }
    }

    fun drawLine(x0: Int, y0: Int, x1: Int, y1: Int, role: BrailleColorRole, thick: Boolean = false) {
        var cx = x0
        var cy = y0
        val dx = abs(x1 - x0)
        val sx = if (x0 < x1) 1 else -1
        val dy = -abs(y1 - y0)
        val sy = if (y0 < y1) 1 else -1
        var err = dx + dy

        while (true) {
            setPixel(cx, cy, role)
            if (thick) {
                setPixel(cx + 1, cy, role)
                setPixel(cx, cy + 1, role)
            }
            if (cx == x1 && cy == y1) break
            val e2 = 2 * err
            if (e2 >= dy) {
                err += dy
                cx += sx
            }
            if (e2 <= dx) {
                err += dx
                cy += sy
            }
        }
    }

    fun drawCircle(centerX: Int, centerY: Int, radius: Int, role: BrailleColorRole) {
        for (dy in -radius..radius) {
            for (dx in -radius..radius) {
                if (dx * dx + dy * dy <= radius * radius) {
                    setPixel(centerX + dx, centerY + dy, role)
                }
            }
        }
    }

    fun putText(charX: Int, charY: Int, text: String, role: BrailleColorRole) {
        if (charY !in 0 until charHeight) return
        var cx = charX
        for (c in text) {
            if (cx in 0 until charWidth) {
                val idx = charY * charWidth + cx
                textOverlay[idx] = c
                roles[idx] = role
            }
            cx += 1
        }
    }

    fun toRows(): List<List<BrailleCell>> {
        return (0 until charHeight).map { cy ->
            (0 until charWidth).map { cx ->
                val idx = cy * charWidth + cx
                val overlay = textOverlay[idx]
                if (overlay != '\u0000') {
                    BrailleCell(overlay, roles[idx])
                } else {
                    val ch = (0x2800 + dots[idx]).toChar()
                    BrailleCell(ch, roles[idx])
                }
            }
        }
    }

    companion object {
        fun renderOsmToBraille(
            dataset: OsmDataset,
            tasks: List<TaskCapsule>,
            selectedTaskId: String?,
            centerLat: Double,
            centerLon: Double,
            zoom: Double,
            charWidth: Int,
            charHeight: Int
        ): List<List<BrailleCell>> {
            val canvas = BrailleCanvas(charWidth.coerceAtLeast(20), charHeight.coerceAtLeast(10))
            val latSpan = 0.22 / zoom.coerceAtLeast(0.25)
            val lonSpan = 0.32 / zoom.coerceAtLeast(0.25)
            val minLat = centerLat - latSpan / 2.0
            val maxLat = centerLat + latSpan / 2.0
            val minLon = centerLon - lonSpan / 2.0
            val maxLon = centerLon + lonSpan / 2.0

            fun project(lat: Double, lon: Double): Pair<Int, Int> {
                val nx = (lon - minLon) / (maxLon - minLon)
                val ny = 1.0 - (lat - minLat) / (maxLat - minLat)
                val px = (nx * canvas.pixelWidth).toInt()
                val py = (ny * canvas.pixelHeight).toInt()
                return px to py
            }

            // 1. OSM 도로 및 수계 렌더링
            for (way in dataset.ways.sortedBy { it.wayType.priority }) {
                val role = when (way.wayType) {
                    WayType.WATERWAY -> BrailleColorRole.WATER
                    WayType.MOTORWAY -> BrailleColorRole.MOTORWAY
                    else -> BrailleColorRole.PRIMARY_ROAD
                }
                val thick = way.wayType == WayType.WATERWAY || way.wayType == WayType.MOTORWAY
                for (i in 0 until way.points.size - 1) {
                    val p0 = project(way.points[i].lat, way.points[i].lon)
                    val p1 = project(way.points[i + 1].lat, way.points[i + 1].lon)
                    canvas.drawLine(p0.first, p0.second, p1.first, p1.second, role, thick)
                }
                if (zoom >= 1.3 && way.name != null && way.points.isNotEmpty()) {
                    val mid = way.points[way.points.size / 2]
                    val (mx, my) = project(mid.lat, mid.lon)
                    if (mx >= 0 && my >= 0) {
                        canvas.putText(mx / 2, my / 4, way.name, role)
                    }
                }
            }

            // 2. Subject 동선 폴리라인 (① -> ② -> ③ -> ④)
            val routeTasks = tasks
                .filter { it.sequenceInSubject > 0 && it.latitude != null && it.longitude != null }
                .sortedBy { it.sequenceInSubject }

            for (i in 0 until routeTasks.size - 1) {
                val t0 = routeTasks[i]
                val t1 = routeTasks[i + 1]
                val p0 = project(t0.latitude!!, t0.longitude!!)
                val p1 = project(t1.latitude!!, t1.longitude!!)
                canvas.drawLine(p0.first, p0.second, p1.first, p1.second, BrailleColorRole.ROUTE_LINE, thick = true)
            }

            // 3. 마커 및 번호 핀 오버레이
            val badges = arrayOf("①", "②", "③", "④", "⑤", "⑥", "⑦", "⑧", "⑨")
            for (t in tasks) {
                val lat = t.latitude ?: continue
                val lon = t.longitude ?: continue
                val (px, py) = project(lat, lon)
                if (px < 0 || py < 0) continue
                canvas.drawCircle(px, py, 2, BrailleColorRole.MARKER_PIN)
                val badge = if (t.sequenceInSubject in 1..9) badges[t.sequenceInSubject - 1] else "●"
                val placeShort = (t.placeName ?: t.title).take(7)
                val prefix = if (t.taskId == selectedTaskId) "▶$badge" else badge
                canvas.putText(
                    (px / 2).coerceAtLeast(0),
                    (py / 4).coerceAtLeast(0),
                    "$prefix$placeShort",
                    if (t.taskId == selectedTaskId) BrailleColorRole.MARKER_PIN else BrailleColorRole.LABEL_TEXT
                )
            }

            return canvas.toRows()
        }
    }
}
