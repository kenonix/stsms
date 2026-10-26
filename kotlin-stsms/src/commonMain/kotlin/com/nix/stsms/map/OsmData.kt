package com.nix.stsms.map

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.doubleOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.longOrNull

enum class WayType(val priority: Int) {
    WATERWAY(5),      // 한강, 탄천, 양재천
    MOTORWAY(4),      // 고속도로, 올림픽대로, 강변북로, 경부고속도로
    PRIMARY(3),       // 간선도로, 테헤란로, 강남대로, 반포대로, 여의대로
    SECONDARY(2),     // 보조간선도로
    RESIDENTIAL(1),   // 일반 도로
    PARK_BOUNDARY(0)  // 공원 경계
}

data class GeoPoint(val lat: Double, val lon: Double)

data class OsmWay(
    val id: Long,
    val wayType: WayType,
    val name: String? = null,
    val points: List<GeoPoint>
)

data class OsmDataset(
    val ways: List<OsmWay>
) {
    companion object {
        /**
         * Overpass API JSON 응답 파서
         */
        fun fromOverpassJson(jsonStr: String): Result<OsmDataset> = runCatching {
            val root = Json.parseToJsonElement(jsonStr).jsonObject
            val elements = root["elements"]?.jsonArray
                ?: throw IllegalArgumentException("elements array missing in Overpass JSON")

            val nodeMap = HashMap<Long, GeoPoint>()
            val ways = ArrayList<OsmWay>()

            // 1차 패스: 노드 좌표 수집
            for (el in elements) {
                val obj = el.jsonObject
                if (obj["type"]?.jsonPrimitive?.content == "node") {
                    val id = obj["id"]?.jsonPrimitive?.longOrNull ?: continue
                    val lat = obj["lat"]?.jsonPrimitive?.doubleOrNull ?: continue
                    val lon = obj["lon"]?.jsonPrimitive?.doubleOrNull ?: continue
                    nodeMap[id] = GeoPoint(lat, lon)
                }
            }

            // 2차 패스: Way 폴리라인 구성
            for (el in elements) {
                val obj = el.jsonObject
                if (obj["type"]?.jsonPrimitive?.content == "way") {
                    val id = obj["id"]?.jsonPrimitive?.longOrNull ?: 0L
                    val tags = obj["tags"]?.jsonObject
                    val highway = tags?.get("highway")?.jsonPrimitive?.content
                    val waterway = tags?.get("waterway")?.jsonPrimitive?.content
                    val natural = tags?.get("natural")?.jsonPrimitive?.content
                    val leisure = tags?.get("leisure")?.jsonPrimitive?.content
                    val name = tags?.get("name:ko")?.jsonPrimitive?.content
                        ?: tags?.get("name")?.jsonPrimitive?.content

                    val wayType = when {
                        waterway != null || natural == "water" -> WayType.WATERWAY
                        leisure == "park" -> WayType.PARK_BOUNDARY
                        highway == "motorway" || highway == "trunk" -> WayType.MOTORWAY
                        highway == "primary" -> WayType.PRIMARY
                        highway == "secondary" -> WayType.SECONDARY
                        highway != null -> WayType.RESIDENTIAL
                        else -> null
                    } ?: continue

                    val nodesArr = obj["nodes"]?.jsonArray ?: continue
                    val pts = nodesArr.mapNotNull { n ->
                        n.jsonPrimitive.longOrNull?.let { nodeMap[it] }
                    }
                    if (pts.size >= 2) {
                        ways.add(OsmWay(id = id, wayType = wayType, name = name, points = pts))
                    }
                }
            }

            OsmDataset(ways)
        }

        /**
         * 수도권(서울·여의도·서초·강남·상암·판교) 실제 OpenStreetMap 지리 좌표 내장 데이터셋
         */
        fun loadBuiltinSeoulMetropolitan(): OsmDataset {
            val ways = listOf(
                // 1. 한강 (Han River)
                OsmWay(
                    id = 1001,
                    wayType = WayType.WATERWAY,
                    name = "한강 (Han River)",
                    points = listOf(
                        GeoPoint(37.5682, 126.8320),
                        GeoPoint(37.5580, 126.8712),
                        GeoPoint(37.5450, 126.9020),
                        GeoPoint(37.5312, 126.9230),
                        GeoPoint(37.5270, 126.9380),
                        GeoPoint(37.5180, 126.9600),
                        GeoPoint(37.5130, 126.9800),
                        GeoPoint(37.5090, 127.0000),
                        GeoPoint(37.5170, 127.0120),
                        GeoPoint(37.5280, 127.0250),
                        GeoPoint(37.5340, 127.0420),
                        GeoPoint(37.5350, 127.0600),
                        GeoPoint(37.5320, 127.0850),
                        GeoPoint(37.5250, 127.1050),
                        GeoPoint(37.5220, 127.1250),
                        GeoPoint(37.5350, 127.1450)
                    )
                ),
                // 2. 탄천 (Tancheon: 판교 -> 잠실 한강 합류)
                OsmWay(
                    id = 1002,
                    wayType = WayType.WATERWAY,
                    name = "탄천 (Tancheon)",
                    points = listOf(
                        GeoPoint(37.3850, 127.1180),
                        GeoPoint(37.4010, 127.1130),
                        GeoPoint(37.4320, 127.1200),
                        GeoPoint(37.4650, 127.1150),
                        GeoPoint(37.4920, 127.0980),
                        GeoPoint(37.5150, 127.0750),
                        GeoPoint(37.5220, 127.0720)
                    )
                ),
                // 3. 올림픽대로 (Olympic-daero)
                OsmWay(
                    id = 2001,
                    wayType = WayType.MOTORWAY,
                    name = "올림픽대로",
                    points = listOf(
                        GeoPoint(37.5520, 126.8650),
                        GeoPoint(37.5380, 126.8950),
                        GeoPoint(37.5220, 126.9200),
                        GeoPoint(37.5140, 126.9600),
                        GeoPoint(37.5060, 126.9950),
                        GeoPoint(37.5120, 127.0150),
                        GeoPoint(37.5250, 127.0500),
                        GeoPoint(37.5180, 127.0800),
                        GeoPoint(37.5280, 127.1200)
                    )
                ),
                // 4. 강변북로 (Gangbyeonbuk-ro)
                OsmWay(
                    id = 2002,
                    wayType = WayType.MOTORWAY,
                    name = "강변북로",
                    points = listOf(
                        GeoPoint(37.5620, 126.8850),
                        GeoPoint(37.5480, 126.9180),
                        GeoPoint(37.5380, 126.9400),
                        GeoPoint(37.5260, 126.9750),
                        GeoPoint(37.5320, 127.0100),
                        GeoPoint(37.5390, 127.0450),
                        GeoPoint(37.5360, 127.0800)
                    )
                ),
                // 5. 경부고속도로 (Gyeongbu Expy: 한남 -> 서초 -> 양재 -> 판교)
                OsmWay(
                    id = 2003,
                    wayType = WayType.MOTORWAY,
                    name = "경부고속도로",
                    points = listOf(
                        GeoPoint(37.5280, 127.0200),
                        GeoPoint(37.5120, 127.0210),
                        GeoPoint(37.5010, 127.0220),
                        GeoPoint(37.4880, 127.0230),
                        GeoPoint(37.4720, 127.0340),
                        GeoPoint(37.4450, 127.0650),
                        GeoPoint(37.4080, 127.1020),
                        GeoPoint(37.3800, 127.1080)
                    )
                ),
                // 6. 테헤란로 (Teheran-ro)
                OsmWay(
                    id = 3001,
                    wayType = WayType.PRIMARY,
                    name = "테헤란로",
                    points = listOf(
                        GeoPoint(37.4979, 127.0276),
                        GeoPoint(37.5006, 127.0365),
                        GeoPoint(37.5045, 127.0490),
                        GeoPoint(37.5088, 127.0632)
                    )
                ),
                // 7. 강남대로 (Gangnam-daero)
                OsmWay(
                    id = 3002,
                    wayType = WayType.PRIMARY,
                    name = "강남대로",
                    points = listOf(
                        GeoPoint(37.5163, 127.0201),
                        GeoPoint(37.5075, 127.0237),
                        GeoPoint(37.4979, 127.0276),
                        GeoPoint(37.4842, 127.0348)
                    )
                ),
                // 8. 반포대로 (Banpo-daero)
                OsmWay(
                    id = 3003,
                    wayType = WayType.PRIMARY,
                    name = "반포대로",
                    points = listOf(
                        GeoPoint(37.5080, 127.0020),
                        GeoPoint(37.4920, 127.0080),
                        GeoPoint(37.4837, 127.0124),
                        GeoPoint(37.4760, 127.0160)
                    )
                ),
                // 9. 판교로 (Pangyo-ro)
                OsmWay(
                    id = 3004,
                    wayType = WayType.PRIMARY,
                    name = "판교로",
                    points = listOf(
                        GeoPoint(37.3950, 127.0950),
                        GeoPoint(37.4020, 127.1086),
                        GeoPoint(37.4060, 127.1220)
                    )
                ),
                // 10. 여의대로 (Yeoui-daero)
                OsmWay(
                    id = 3005,
                    wayType = WayType.PRIMARY,
                    name = "여의대로",
                    points = listOf(
                        GeoPoint(37.5253, 126.9255),
                        GeoPoint(37.5218, 126.9242),
                        GeoPoint(37.5180, 126.9230)
                    )
                ),
                // 11. 월드컵북로 (WorldCupbuk-ro)
                OsmWay(
                    id = 3006,
                    wayType = WayType.PRIMARY,
                    name = "월드컵북로",
                    points = listOf(
                        GeoPoint(37.5580, 126.9080),
                        GeoPoint(37.5700, 126.8980),
                        GeoPoint(37.5794, 126.8926),
                        GeoPoint(37.5850, 126.8850)
                    )
                )
            )
            return OsmDataset(ways)
        }
    }
}
