package com.nix.stsms.ui.components

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.nix.stsms.domain.MapRenderStyle
import com.nix.stsms.domain.Subject
import com.nix.stsms.domain.TaskCapsule
import com.nix.stsms.map.*
import com.nix.stsms.ui.theme.StsmsColors
import kotlin.math.hypot

@Composable
fun OsmMapView(
    dataset: OsmDataset,
    subject: Subject?,
    tasks: List<TaskCapsule>,
    selectedTaskId: String?,
    renderStyle: MapRenderStyle,
    onToggleRenderStyle: (MapRenderStyle) -> Unit,
    centerLat: Double,
    centerLon: Double,
    zoom: Double,
    onPan: (dLat: Double, dLon: Double) -> Unit,
    onZoom: (newZoom: Double) -> Unit,
    onCenterSelectedTask: () -> Unit,
    onSelectTask: (String) -> Unit,
    modifier: Modifier = Modifier
) {
    BoxWithConstraints(
        modifier = modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .background(StsmsColors.BgCard)
            .border(1.dp, StsmsColors.BorderSubtle, RoundedCornerShape(12.dp))
            .padding(10.dp)
    ) {
        val isCompactMap = maxWidth < 520.dp
        Column(modifier = Modifier.fillMaxSize()) {
            // 1. 지도 상단 컨트롤 바 (반응형: 모바일에서는 2줄 배치)
            if (isCompactMap) {
                Column(
                    modifier = Modifier.fillMaxWidth(),
                    verticalArrangement = Arrangement.spacedBy(4.dp)
                ) {
                    Text(
                        text = "🗺️ OSM 동선 & Subject 뷰 (${renderStyle.label})",
                        color = StsmsColors.TextPrimary,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.Bold
                    )
                    subject?.let { sub ->
                        Text(
                            text = "${sub.subjectName} · 총 ${"%.1f".format(sub.totalDistanceKm)}km / ${sub.estimatedDurationMin}분",
                            color = StsmsColors.AccentCyan,
                            fontSize = 10.sp
                        )
                    }
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(4.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        SmallActionChip(
                            label = "벡터 GIS",
                            active = renderStyle == MapRenderStyle.VECTOR_GIS,
                            onClick = { onToggleRenderStyle(MapRenderStyle.VECTOR_GIS) }
                        )
                        SmallActionChip(
                            label = "점자(Braille)",
                            active = renderStyle == MapRenderStyle.BRAILLE_MAPSCII,
                            onClick = { onToggleRenderStyle(MapRenderStyle.BRAILLE_MAPSCII) }
                        )
                        SmallActionChip("-") { onZoom((zoom / 1.25).coerceAtLeast(0.5)) }
                        Text(
                            text = "${"%.1f".format(zoom)}x",
                            color = StsmsColors.AccentAmber,
                            fontSize = 11.sp,
                            fontWeight = FontWeight.Bold
                        )
                        SmallActionChip("+") { onZoom((zoom * 1.25).coerceAtMost(6.0)) }
                        SmallActionChip("🎯 중심") { onCenterSelectedTask() }
                    }
                }
            } else {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text(
                            text = "🗺️ OpenStreetMap 동선 & Subject 뷰 (${renderStyle.label})",
                            color = StsmsColors.TextPrimary,
                            fontSize = 13.sp,
                            fontWeight = FontWeight.Bold
                        )
                        subject?.let { sub ->
                            Text(
                                text = "${sub.subjectName} · 총 ${"%.1f".format(sub.totalDistanceKm)}km / 예상 ${sub.estimatedDurationMin}분",
                                color = StsmsColors.AccentCyan,
                                fontSize = 11.sp
                            )
                        }
                    }

                    Row(horizontalArrangement = Arrangement.spacedBy(4.dp), verticalAlignment = Alignment.CenterVertically) {
                        SmallActionChip(
                            label = "벡터 GIS",
                            active = renderStyle == MapRenderStyle.VECTOR_GIS,
                            onClick = { onToggleRenderStyle(MapRenderStyle.VECTOR_GIS) }
                        )
                        SmallActionChip(
                            label = "점자(Braille)",
                            active = renderStyle == MapRenderStyle.BRAILLE_MAPSCII,
                            onClick = { onToggleRenderStyle(MapRenderStyle.BRAILLE_MAPSCII) }
                        )
                        SmallActionChip("-") { onZoom((zoom / 1.25).coerceAtLeast(0.5)) }
                        Text(
                            text = "${"%.1f".format(zoom)}x",
                            color = StsmsColors.AccentAmber,
                            fontSize = 11.sp,
                            fontWeight = FontWeight.Bold
                        )
                        SmallActionChip("+") { onZoom((zoom * 1.25).coerceAtMost(6.0)) }
                        SmallActionChip("🎯 중심") { onCenterSelectedTask() }
                    }
                }
            }

        Spacer(modifier = Modifier.height(6.dp))

        // 2. 메인 지도 뷰 영역 (드래그 패닝 + 핀 클릭 지원)
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f)
                .clip(RoundedCornerShape(8.dp))
                .background(StsmsColors.MapBg)
                .border(1.dp, StsmsColors.BorderSubtle, RoundedCornerShape(8.dp))
        ) {
            if (renderStyle == MapRenderStyle.VECTOR_GIS) {
                VectorOsmCanvas(
                    dataset = dataset,
                    tasks = tasks,
                    selectedTaskId = selectedTaskId,
                    centerLat = centerLat,
                    centerLon = centerLon,
                    zoom = zoom,
                    onPan = onPan,
                    onSelectTask = onSelectTask,
                    modifier = Modifier.fillMaxSize()
                )
            } else {
                BrailleOsmComposeView(
                    dataset = dataset,
                    tasks = tasks,
                    selectedTaskId = selectedTaskId,
                    centerLat = centerLat,
                    centerLon = centerLon,
                    zoom = zoom,
                    onPan = onPan,
                    modifier = Modifier.fillMaxSize()
                )
            }

            // 좌하단 범례 및 좌표 오버레이
            Row(
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .padding(6.dp)
                    .clip(RoundedCornerShape(6.dp))
                    .background(StsmsColors.BgPrimary.copy(alpha = 0.85f))
                    .padding(horizontal = 8.dp, vertical = 4.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                LegendDot("한강/탄천", StsmsColors.MapWater)
                LegendDot("고속화도로", StsmsColors.MapMotorway)
                LegendDot("간선도로", StsmsColors.MapPrimaryRoad)
                LegendDot("Subject동선(①→④)", StsmsColors.MapRouteLine)
                Text(
                    text = "(${"%.4f".format(centerLat)}, ${"%.4f".format(centerLon)})",
                    color = StsmsColors.TextMuted,
                    fontSize = 10.sp
                )
            }

            // 우하단 터치 방향키
            Row(
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .padding(6.dp),
                horizontalArrangement = Arrangement.spacedBy(3.dp)
            ) {
                val step = 0.018 / zoom
                SmallActionChip("◀") { onPan(0.0, -step) }
                SmallActionChip("▲") { onPan(step, 0.0) }
                SmallActionChip("▼") { onPan(-step, 0.0) }
                SmallActionChip("▶") { onPan(0.0, step) }
            }
        }

        Spacer(modifier = Modifier.height(6.dp))

        // 3. 하단 Subject 구간별 이동거리·소요시간 바 (① -> ② -> ③ -> ④)
        val routeTasks = tasks.filter { it.sequenceInSubject > 0 }.sortedBy { it.sequenceInSubject }
        if (routeTasks.isNotEmpty()) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(6.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                val badges = listOf("①", "②", "③", "④", "⑤", "⑥")
                routeTasks.forEachIndexed { index, t ->
                    val badge = badges.getOrElse(index) { "●" }
                    val isSelected = t.taskId == selectedTaskId
                    Row(
                        modifier = Modifier
                            .weight(1f)
                            .clip(RoundedCornerShape(6.dp))
                            .background(if (isSelected) StsmsColors.AccentCyan.copy(alpha = 0.2f) else StsmsColors.BgSecondary)
                            .border(
                                1.dp,
                                if (isSelected) StsmsColors.AccentCyan else StsmsColors.BorderSubtle,
                                RoundedCornerShape(6.dp)
                            )
                            .clickable { onSelectTask(t.taskId) }
                            .padding(horizontal = 6.dp, vertical = 4.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = "$badge ${t.placeName ?: t.title.take(8)}",
                                color = if (isSelected) StsmsColors.AccentCyan else StsmsColors.TextPrimary,
                                fontSize = 10.sp,
                                fontWeight = FontWeight.Bold,
                                maxLines = 1
                            )
                            Text(
                                text = if (index == 0) "출발 기점" else "+${"%.1f".format(t.distanceFromPrevKm)}km (${t.travelTimeFromPrevMin}분)",
                                color = StsmsColors.TextSecondary,
                                fontSize = 9.sp,
                                maxLines = 1
                            )
                        }
                    }
                }
            }
        }
        }
    }
}

@Composable
private fun VectorOsmCanvas(
    dataset: OsmDataset,
    tasks: List<TaskCapsule>,
    selectedTaskId: String?,
    centerLat: Double,
    centerLon: Double,
    zoom: Double,
    onPan: (dLat: Double, dLon: Double) -> Unit,
    onSelectTask: (String) -> Unit,
    modifier: Modifier = Modifier
) {
    var canvasWidth by remember { mutableStateOf(600f) }
    var canvasHeight by remember { mutableStateOf(300f) }

    val latSpan = 0.20 / zoom.coerceAtLeast(0.25)
    val lonSpan = 0.30 / zoom.coerceAtLeast(0.25)
    val minLat = centerLat - latSpan / 2.0
    val maxLat = centerLat + latSpan / 2.0
    val minLon = centerLon - lonSpan / 2.0
    val maxLon = centerLon + lonSpan / 2.0

    fun project(lat: Double, lon: Double, w: Float, h: Float): Offset {
        val nx = ((lon - minLon) / (maxLon - minLon)).toFloat()
        val ny = (1.0 - (lat - minLat) / (maxLat - minLat)).toFloat()
        return Offset(nx * w, ny * h)
    }

    Box(
        modifier = modifier
            .onSizeChanged { sz ->
                if (sz.width > 0 && sz.height > 0) {
                    canvasWidth = sz.width.toFloat()
                    canvasHeight = sz.height.toFloat()
                }
            }
            .pointerInput(zoom) {
                detectDragGestures { change, dragAmount ->
                    change.consume()
                    val dLon = -(dragAmount.x / canvasWidth.coerceAtLeast(1f)) * lonSpan
                    val dLat = (dragAmount.y / canvasHeight.coerceAtLeast(1f)) * latSpan
                    onPan(dLat, dLon)
                }
            }
            .pointerInput(tasks, centerLat, centerLon, zoom) {
                detectTapGestures { tapOffset ->
                    // 가장 가까운 과업 마커 클릭 감지
                    val hit = tasks.filter { it.latitude != null && it.longitude != null }
                        .minByOrNull { t ->
                            val pt = project(t.latitude!!, t.longitude!!, canvasWidth, canvasHeight)
                            hypot((pt.x - tapOffset.x).toDouble(), (pt.y - tapOffset.y).toDouble())
                        }
                    if (hit != null) {
                        val pt = project(hit.latitude!!, hit.longitude!!, canvasWidth, canvasHeight)
                        if (hypot((pt.x - tapOffset.x).toDouble(), (pt.y - tapOffset.y).toDouble()) <= 36.0) {
                            onSelectTask(hit.taskId)
                        }
                    }
                }
            }
    ) {
        Canvas(modifier = Modifier.fillMaxSize()) {
            canvasWidth = size.width
            canvasHeight = size.height

            // 배경 그리드 라인
            val gridStepX = size.width / 8f
            val gridStepY = size.height / 6f
            for (i in 1..7) {
                drawLine(
                    color = StsmsColors.BorderSubtle.copy(alpha = 0.28f),
                    start = Offset(i * gridStepX, 0f),
                    end = Offset(i * gridStepX, size.height),
                    strokeWidth = 1f
                )
            }
            for (j in 1..5) {
                drawLine(
                    color = StsmsColors.BorderSubtle.copy(alpha = 0.28f),
                    start = Offset(0f, j * gridStepY),
                    end = Offset(size.width, j * gridStepY),
                    strokeWidth = 1f
                )
            }

            // 1. OSM 도로 및 수계 폴리라인
            for (way in dataset.ways.sortedBy { it.wayType.priority }) {
                val (color, strokeW) = when (way.wayType) {
                    WayType.WATERWAY -> StsmsColors.MapWater.copy(alpha = 0.65f) to 7.5f
                    WayType.MOTORWAY -> StsmsColors.MapMotorway.copy(alpha = 0.78f) to 3.4f
                    WayType.PRIMARY -> StsmsColors.MapPrimaryRoad.copy(alpha = 0.75f) to 2.2f
                    else -> StsmsColors.MapPrimaryRoad.copy(alpha = 0.45f) to 1.4f
                }

                val path = Path()
                way.points.forEachIndexed { idx, gp ->
                    val pt = project(gp.lat, gp.lon, size.width, size.height)
                    if (idx == 0) path.moveTo(pt.x, pt.y) else path.lineTo(pt.x, pt.y)
                }
                drawPath(
                    path = path,
                    color = color,
                    style = Stroke(width = strokeW, cap = StrokeCap.Round)
                )
            }

            // 2. Subject 유향 동선 폴리라인 (① -> ② -> ③ -> ④)
            val routeTasks = tasks
                .filter { it.sequenceInSubject > 0 && it.latitude != null && it.longitude != null }
                .sortedBy { it.sequenceInSubject }

            for (i in 0 until routeTasks.size - 1) {
                val t0 = routeTasks[i]
                val t1 = routeTasks[i + 1]
                val p0 = project(t0.latitude!!, t0.longitude!!, size.width, size.height)
                val p1 = project(t1.latitude!!, t1.longitude!!, size.width, size.height)

                // 글로우 배경선
                drawLine(
                    color = StsmsColors.MapRouteLine.copy(alpha = 0.25f),
                    start = p0,
                    end = p1,
                    strokeWidth = 9f,
                    cap = StrokeCap.Round
                )
                // 대시 유향선
                drawLine(
                    color = StsmsColors.MapRouteLine,
                    start = p0,
                    end = p1,
                    strokeWidth = 3.2f,
                    pathEffect = PathEffect.dashPathEffect(floatArrayOf(12f, 6f), 0f),
                    cap = StrokeCap.Round
                )
            }

            // 3. 과업 마커 원 렌더링
            for (t in tasks) {
                val lat = t.latitude ?: continue
                val lon = t.longitude ?: continue
                val pt = project(lat, lon, size.width, size.height)
                val isSelected = t.taskId == selectedTaskId

                if (isSelected) {
                    drawCircle(
                        color = StsmsColors.MapPinActive.copy(alpha = 0.32f),
                        radius = 18f,
                        center = pt
                    )
                }
                drawCircle(
                    color = if (isSelected) StsmsColors.MapPinActive else StsmsColors.MapPinNormal,
                    radius = if (isSelected) 10f else 8f,
                    center = pt
                )
                drawCircle(
                    color = Color.White,
                    radius = 3.5f,
                    center = pt
                )
            }
        }

        // 마커 텍스트 배지 오버레이 (핀 실제 픽셀 좌표 정밀 매핑)
        val badges = listOf("①", "②", "③", "④", "⑤", "⑥", "⑦", "⑧")
        tasks.forEach { t ->
            val lat = t.latitude ?: return@forEach
            val lon = t.longitude ?: return@forEach
            val nx = ((lon - minLon) / (maxLon - minLon)).toFloat()
            val ny = (1.0 - (lat - minLat) / (maxLat - minLat)).toFloat()
            if (nx in 0.04f..0.94f && ny in 0.06f..0.92f) {
                val isSelected = t.taskId == selectedTaskId
                val seqBadge = if (t.sequenceInSubject in 1..8) badges[t.sequenceInSubject - 1] else "●"
                val pxX = (nx * canvasWidth).toInt() + 12
                val pxY = (ny * canvasHeight).toInt() - 10
                Box(
                    modifier = Modifier
                        .offset { IntOffset(pxX, pxY) }
                ) {
                    Text(
                        text = "$seqBadge ${(t.placeName ?: t.title).take(9)}",
                        color = if (isSelected) Color.White else StsmsColors.TextPrimary,
                        fontSize = 10.sp,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier
                            .clip(RoundedCornerShape(4.dp))
                            .background(
                                if (isSelected) StsmsColors.MapPinActive.copy(alpha = 0.92f)
                                else StsmsColors.BgPrimary.copy(alpha = 0.85f)
                            )
                            .border(
                                1.dp,
                                if (isSelected) Color.White.copy(alpha = 0.6f) else StsmsColors.BorderSubtle,
                                RoundedCornerShape(4.dp)
                            )
                            .clickable { onSelectTask(t.taskId) }
                            .padding(horizontal = 5.dp, vertical = 2.dp)
                    )
                }
            }
        }
    }
}

@Composable
private fun BrailleOsmComposeView(
    dataset: OsmDataset,
    tasks: List<TaskCapsule>,
    selectedTaskId: String?,
    centerLat: Double,
    centerLon: Double,
    zoom: Double,
    onPan: (dLat: Double, dLon: Double) -> Unit,
    modifier: Modifier = Modifier
) {
    val rows = remember(dataset, tasks, selectedTaskId, centerLat, centerLon, zoom) {
        BrailleCanvas.renderOsmToBraille(
            dataset = dataset,
            tasks = tasks,
            selectedTaskId = selectedTaskId,
            centerLat = centerLat,
            centerLon = centerLon,
            zoom = zoom,
            charWidth = 68,
            charHeight = 16
        )
    }

    val annotated = remember(rows) {
        buildAnnotatedString {
            rows.forEachIndexed { rIdx, row ->
                for (cell in row) {
                    val col = when (cell.role) {
                        BrailleColorRole.WATER -> StsmsColors.MapWater
                        BrailleColorRole.MOTORWAY -> StsmsColors.MapMotorway
                        BrailleColorRole.PRIMARY_ROAD -> StsmsColors.TextSecondary
                        BrailleColorRole.ROUTE_LINE -> StsmsColors.MapRouteLine
                        BrailleColorRole.MARKER_PIN -> StsmsColors.MapPinActive
                        BrailleColorRole.LABEL_TEXT -> StsmsColors.TextPrimary
                        BrailleColorRole.NONE -> StsmsColors.BorderSubtle.copy(alpha = 0.35f)
                    }
                    withStyle(SpanStyle(color = col)) {
                        append(cell.ch)
                    }
                }
                if (rIdx < rows.lastIndex) append('\n')
            }
        }
    }

    Box(
        modifier = modifier
            .pointerInput(zoom) {
                detectDragGestures { change, dragAmount ->
                    change.consume()
                    val step = 0.0005 / zoom
                    onPan(dragAmount.y * step, -dragAmount.x * step)
                }
            }
            .padding(8.dp),
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = annotated,
            fontFamily = FontFamily.Monospace,
            fontSize = 11.sp,
            lineHeight = 13.sp
        )
    }
}

@Composable
private fun LegendDot(label: String, color: Color) {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(3.dp)) {
        Box(modifier = Modifier.size(7.dp).clip(CircleShape).background(color))
        Text(text = label, color = StsmsColors.TextSecondary, fontSize = 9.sp)
    }
}
