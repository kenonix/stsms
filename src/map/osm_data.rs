use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WayType {
    Waterway,     // 한강, 탄천, 양재천 등
    Motorway,     // 고속도로, 올림픽대로, 강변북로
    Primary,      // 간선도로, 테헤란로, 강남대로, 여의대로
    Secondary,    // 보조간선도로
    Residential,  // 일반 도로
    ParkBoundary, // 공원 경계
}

impl WayType {
    pub fn priority(&self) -> i32 {
        match self {
            Self::Waterway => 5,
            Self::Motorway => 4,
            Self::Primary => 3,
            Self::Secondary => 2,
            Self::Residential => 1,
            Self::ParkBoundary => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmNode {
    pub id: i64,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmWay {
    pub id: i64,
    pub way_type: WayType,
    pub name: Option<String>,
    pub points: Vec<(f64, f64)>, // (lat, lon)
}

#[derive(Debug, Clone, Default)]
pub struct OsmDataset {
    pub nodes: HashMap<i64, OsmNode>,
    pub ways: Vec<OsmWay>,
}

impl OsmDataset {
    pub fn new() -> Self {
        Self::default()
    }

    /// Overpass API JSON 응답 파싱
    pub fn from_overpass_json(json_str: &str) -> Result<Self, String> {
        let v: serde_json::Value =
            serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {}", e))?;

        let mut dataset = OsmDataset::new();
        let elements = v
            .get("elements")
            .and_then(|e| e.as_array())
            .ok_or_else(|| "elements array missing in Overpass response".to_string())?;

        // 1st pass: parse nodes
        for el in elements {
            if el.get("type").and_then(|t| t.as_str()) == Some("node") {
                if let (Some(id), Some(lat), Some(lon)) = (
                    el.get("id").and_then(|i| i.as_i64()),
                    el.get("lat").and_then(|l| l.as_f64()),
                    el.get("lon").and_then(|l| l.as_f64()),
                ) {
                    dataset.nodes.insert(id, OsmNode { id, lat, lon });
                }
            }
        }

        // 2nd pass: parse ways
        for el in elements {
            if el.get("type").and_then(|t| t.as_str()) == Some("way") {
                let id = el.get("id").and_then(|i| i.as_i64()).unwrap_or(0);
                let tags = el.get("tags");
                let highway = tags.and_then(|t| t.get("highway")).and_then(|h| h.as_str());
                let waterway = tags.and_then(|t| t.get("waterway")).and_then(|w| w.as_str());
                let natural = tags.and_then(|t| t.get("natural")).and_then(|n| n.as_str());
                let leisure = tags.and_then(|t| t.get("leisure")).and_then(|l| l.as_str());
                let name = tags
                    .and_then(|t| t.get("name:ko").or_else(|| t.get("name")))
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string());

                let way_type = if waterway.is_some() || natural == Some("water") {
                    Some(WayType::Waterway)
                } else if leisure == Some("park") {
                    Some(WayType::ParkBoundary)
                } else if let Some(h) = highway {
                    match h {
                        "motorway" | "trunk" => Some(WayType::Motorway),
                        "primary" => Some(WayType::Primary),
                        "secondary" => Some(WayType::Secondary),
                        _ => Some(WayType::Residential),
                    }
                } else {
                    None
                };

                if let Some(wtype) = way_type {
                    if let Some(nodes_arr) = el.get("nodes").and_then(|n| n.as_array()) {
                        let mut pts = Vec::new();
                        for n_id in nodes_arr {
                            if let Some(nid) = n_id.as_i64() {
                                if let Some(node) = dataset.nodes.get(&nid) {
                                    pts.push((node.lat, node.lon));
                                }
                            }
                        }
                        if pts.len() >= 2 {
                            dataset.ways.push(OsmWay {
                                id,
                                way_type: wtype,
                                name,
                                points: pts,
                            });
                        }
                    }
                }
            }
        }

        Ok(dataset)
    }

    /// 내장된 수도권(서울 및 판교, 여의도, 강남) 실제 OpenStreetMap 좌표 기반 데이터 로드
    pub fn load_builtin_seoul_metropolitan() -> Self {
        let mut ds = OsmDataset::new();

        // 1. 한강 (Han River) 실제 수계 경로
        ds.ways.push(OsmWay {
            id: 1001,
            way_type: WayType::Waterway,
            name: Some("한강 (Han River)".to_string()),
            points: vec![
                (37.5682, 126.8320), // 행주대교 방면
                (37.5580, 126.8712), // 가양대교
                (37.5450, 126.9020), // 양화대교 / 선유도
                (37.5312, 126.9230), // 여의도 북단 / 서강대교
                (37.5270, 126.9380), // 마포대교
                (37.5180, 126.9600), // 원효대교 / 한강철교
                (37.5130, 126.9800), // 노들섬 / 한강대교
                (37.5090, 127.0000), // 동작대교
                (37.5170, 127.0120), // 반포대교 / 세빛섬
                (37.5280, 127.0250), // 한남대교
                (37.5340, 127.0420), // 동호대교
                (37.5350, 127.0600), // 성수대교
                (37.5320, 127.0850), // 영동대교
                (37.5250, 127.1050), // 청담대교 / 잠실철교
                (37.5220, 127.1250), // 잠실대교
                (37.5350, 127.1450), // 올림픽대교 / 광진교
            ],
        });

        // 2. 탄천 (Tancheon) 수계 (판교 -> 잠실 합류)
        ds.ways.push(OsmWay {
            id: 1002,
            way_type: WayType::Waterway,
            name: Some("탄천 (Tancheon)".to_string()),
            points: vec![
                (37.3850, 127.1180), // 분당 정자
                (37.4010, 127.1130), // 판교 테크노밸리
                (37.4320, 127.1200), // 성남 태평
                (37.4650, 127.1150), // 복정
                (37.4920, 127.0980), // 수서
                (37.5150, 127.0750), // 잠실종합운동장
                (37.5220, 127.0720), // 한강 합류점
            ],
        });

        // 3. 올림픽대로 (Olympicdae-ro)
        ds.ways.push(OsmWay {
            id: 2001,
            way_type: WayType::Motorway,
            name: Some("올림픽대로".to_string()),
            points: vec![
                (37.5520, 126.8650),
                (37.5380, 126.8950),
                (37.5220, 126.9200), // 여의도 남단
                (37.5140, 126.9600),
                (37.5060, 126.9950),
                (37.5120, 127.0150), // 반포
                (37.5250, 127.0500), // 압구정
                (37.5180, 127.0800), // 잠실
                (37.5280, 127.1200),
            ],
        });

        // 4. 강변북로 (Gangbyeonbuk-ro)
        ds.ways.push(OsmWay {
            id: 2002,
            way_type: WayType::Motorway,
            name: Some("강변북로".to_string()),
            points: vec![
                (37.5620, 126.8850), // 상암 DMC 남단
                (37.5480, 126.9180), // 합정
                (37.5380, 126.9400), // 마포
                (37.5260, 126.9750), // 용산
                (37.5320, 127.0100), // 한남
                (37.5390, 127.0450), // 성수
                (37.5360, 127.0800), // 자양
            ],
        });

        // 5. 경부고속도로 (Gyeongbu Expressway: 한남 -> 서초 -> 양재 -> 판교)
        ds.ways.push(OsmWay {
            id: 2003,
            way_type: WayType::Motorway,
            name: Some("경부고속도로".to_string()),
            points: vec![
                (37.5280, 127.0200), // 한남IC
                (37.5120, 127.0210), // 잠원IC
                (37.5010, 127.0220), // 반포IC
                (37.4880, 127.0230), // 서초IC
                (37.4720, 127.0340), // 양재IC
                (37.4450, 127.0650), // 금토JCT
                (37.4080, 127.1020), // 판교IC
                (37.3800, 127.1080), // 분당
            ],
        });

        // 6. 테헤란로 (Teheran-ro: 강남역 -> 역삼 -> 선릉 -> 삼성)
        ds.ways.push(OsmWay {
            id: 3001,
            way_type: WayType::Primary,
            name: Some("테헤란로".to_string()),
            points: vec![
                (37.4979, 127.0276), // 강남역
                (37.5006, 127.0365), // 역삼역
                (37.5045, 127.0490), // 선릉역
                (37.5088, 127.0632), // 삼성역 (코엑스)
            ],
        });

        // 7. 강남대로 (Gangnam-daero: 신사 -> 논현 -> 강남 -> 양재)
        ds.ways.push(OsmWay {
            id: 3002,
            way_type: WayType::Primary,
            name: Some("강남대로".to_string()),
            points: vec![
                (37.5163, 127.0201), // 신사역
                (37.5075, 127.0237), // 논현역
                (37.4979, 127.0276), // 강남역
                (37.4842, 127.0348), // 양재역
            ],
        });

        // 8. 서초대로 / 반포대로 (서초 센서 허브 연결선)
        ds.ways.push(OsmWay {
            id: 3003,
            way_type: WayType::Primary,
            name: Some("반포대로".to_string()),
            points: vec![
                (37.5080, 127.0020), // 고속터미널
                (37.4920, 127.0080), // 교대역
                (37.4837, 127.0124), // 서초 센서 허브 구역
                (37.4760, 127.0160), // 예술의전당
            ],
        });

        // 9. 판교로 / 대왕판교로 (판교 테크노밸리)
        ds.ways.push(OsmWay {
            id: 3004,
            way_type: WayType::Primary,
            name: Some("판교로".to_string()),
            points: vec![
                (37.3950, 127.0950), // 서판교
                (37.4020, 127.1086), // 판교 제1/2 테크노밸리 R&D
                (37.4060, 127.1220), // 봇들마을
            ],
        });

        // 10. 여의대로 / 국회대로 (여의도 금융가)
        ds.ways.push(OsmWay {
            id: 3005,
            way_type: WayType::Primary,
            name: Some("여의대로".to_string()),
            points: vec![
                (37.5253, 126.9255), // 여의도 IFC / 파크원
                (37.5218, 126.9242), // 금융감독원 / 여의도역
                (37.5180, 126.9230), // 여의도 광장 남단
            ],
        });

        // 11. 월드컵북로 (상암 DMC 방면)
        ds.ways.push(OsmWay {
            id: 3006,
            way_type: WayType::Primary,
            name: Some("월드컵북로".to_string()),
            points: vec![
                (37.5580, 126.9080), // 홍대 방면
                (37.5700, 126.8980), // 디지털미디어시티역
                (37.5794, 126.8926), // 상암 누리꿈스퀘어
                (37.5850, 126.8850), // 구룡사거리
            ],
        });

        ds
    }

    /// Overpass API로부터 임의의 Bounding Box OSM 데이터를 온라인으로 페치
    pub async fn fetch_overpass_bbox(
        min_lat: f64,
        min_lon: f64,
        max_lat: f64,
        max_lon: f64,
    ) -> Result<Self, String> {
        let overpass_url = "https://overpass-api.de/api/interpreter";
        let query = format!(
            r#"[out:json][timeout:15];
(
  way["highway"~"motorway|trunk|primary|secondary"]({min_lat},{min_lon},{max_lat},{max_lon});
  way["waterway"~"river|canal"]({min_lat},{min_lon},{max_lat},{max_lon});
  way["natural"="water"]({min_lat},{min_lon},{max_lat},{max_lon});
);
out body;
>;
out skel qt;"#
        );

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(12))
            .build()
            .map_err(|e| e.to_string())?;

        let resp = client
            .post(overpass_url)
            .body(query)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Overpass server returned error: {}", resp.status()));
        }

        let body = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        Self::from_overpass_json(&body)
    }
}
