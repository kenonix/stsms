use ratatui::style::Color;

use crate::domain::models::TaskCapsule;
use crate::map::braille::BrailleCanvas;
use crate::map::osm_data::{OsmDataset, WayType};

pub struct MapView {
    pub center_lat: f64,
    pub center_lon: f64,
    pub zoom_level: f64, // 배율: 1.0 (광역) ~ 30.0 (골목 상세)
    pub osm_dataset: OsmDataset,
}

impl Default for MapView {
    fn default() -> Self {
        Self {
            center_lat: 37.4800,
            center_lon: 127.0200,
            zoom_level: 1.2,
            osm_dataset: OsmDataset::load_builtin_seoul_metropolitan(),
        }
    }
}

impl MapView {
    pub fn new() -> Self {
        Self::default()
    }

    /// 두 위경도 좌표 간의 대원 거리(km) - Haversine Formula
    #[allow(dead_code)]
    pub fn haversine_distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        const R: f64 = 6371.0; // 지구 반경 (km)
        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();
        let a = (d_lat / 2.0).sin().powi(2)
            + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();
        R * c
    }

    /// (lat, lon)을 BrailleCanvas의 서브픽셀 (px, py) 좌표로 투영
    pub fn project_to_subpixel(
        &self,
        lat: f64,
        lon: f64,
        canvas_width: usize,
        canvas_height: usize,
    ) -> (i32, i32) {
        let center_x = (canvas_width as f64) / 2.0;
        let center_y = (canvas_height as f64) / 2.0;

        // 경도: 1도 = 약 88.8km (서울 37.5도 기준)
        // 위도: 1도 = 약 111km
        // 캔버스 가로 종횡비 보정
        let scale_x = (canvas_width as f64) * 2.8 * self.zoom_level;
        let scale_y = (canvas_height as f64) * 4.2 * self.zoom_level;

        let dx = (lon - self.center_lon) * scale_x;
        // 위도는 북쪽이 +이므로 캔버스 y축(아래쪽이 +)과 반대 방향
        let dy = (self.center_lat - lat) * scale_y;

        let px = (center_x + dx).round() as i32;
        let py = (center_y + dy).round() as i32;
        (px, py)
    }

    /// 점자 캔버스에 OpenStreetMap 도로망 및 Subject 동선 렌더링
    pub fn render_to_canvas(&self, canvas: &mut BrailleCanvas, tasks: &[TaskCapsule]) {
        canvas.clear();

        // 1. OpenStreetMap 레이어 렌더링 (WayType 우선순위에 따라 정렬)
        let mut sorted_ways = self.osm_dataset.ways.clone();
        sorted_ways.sort_by_key(|w| w.way_type.priority());

        for way in &sorted_ways {
            let color = match way.way_type {
                WayType::Waterway => Color::Cyan,
                WayType::Motorway => Color::Yellow,
                WayType::Primary => Color::White,
                WayType::Secondary => Color::Gray,
                WayType::Residential => Color::DarkGray,
                WayType::ParkBoundary => Color::Green,
            };

            let points: Vec<(i32, i32)> = way
                .points
                .iter()
                .map(|&(lat, lon)| {
                    self.project_to_subpixel(lat, lon, canvas.pixel_width, canvas.pixel_height)
                })
                .collect();

            canvas.draw_polyline(&points, color);

            // 주요 지명 라벨 오버레이 (예: 한강, 테헤란로)
            if let Some(name) = &way.name {
                if points.len() >= 2 {
                    let mid_pt = points[points.len() / 2];
                    if mid_pt.0 >= 0 && mid_pt.1 >= 0 {
                        let cell_x = (mid_pt.0 / 2) as usize;
                        let cell_y = (mid_pt.1 / 4) as usize;
                        if cell_x > 2
                            && cell_x.saturating_add(name.chars().count()) < canvas.cell_width
                            && cell_y < canvas.cell_height
                        {
                            canvas.add_label(cell_x, cell_y, name.clone(), color);
                        }
                    }
                }
            }
        }

        // 2. Subject 동선 렌더링 (CORE-03: ① -> ② -> ③ ...)
        let mut route_tasks: Vec<&TaskCapsule> = tasks
            .iter()
            .filter(|t| t.latitude.is_some() && t.longitude.is_some() && t.sequence_in_subject > 0)
            .collect();
        route_tasks.sort_by_key(|t| t.sequence_in_subject);

        if !route_tasks.is_empty() {
            let route_points: Vec<(i32, i32)> = route_tasks
                .iter()
                .map(|t| {
                    self.project_to_subpixel(
                        t.latitude.unwrap(),
                        t.longitude.unwrap(),
                        canvas.pixel_width,
                        canvas.pixel_height,
                    )
                })
                .collect();

            // 유향 폴리라인 (자홍색 굵은 경로 선)
            canvas.draw_polyline(&route_points, Color::Magenta);

            // 각 정류장 마커 (원 및 번호)
            for (idx, task) in route_tasks.iter().enumerate() {
                let (px, py) = route_points[idx];

                // 원형 마커
                canvas.draw_circle(px, py, 3, Color::LightRed);

                if px >= 0 && py >= 0 {
                    let cell_x = (px / 2) as usize;
                    let cell_y = (py / 4) as usize;

                    let marker_num = match task.sequence_in_subject {
                        1 => "①",
                        2 => "②",
                        3 => "③",
                        4 => "④",
                        5 => "⑤",
                        _ => "●",
                    };

                    let label_text = format!(
                        "{} {}",
                        marker_num,
                        task.place_name.as_deref().unwrap_or(&task.title)
                    );

                    if cell_x + 1 < canvas.cell_width && cell_y < canvas.cell_height {
                        canvas.add_label(cell_x + 1, cell_y, label_text, Color::LightYellow);
                    }
                }
            }
        }
    }

    /// 지도 이동 조작
    pub fn pan(&mut self, d_lat: f64, d_lon: f64) {
        let step = 0.05 / self.zoom_level;
        self.center_lat += d_lat * step;
        self.center_lon += d_lon * step;
    }

    pub fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level * 1.25).min(30.0);
    }

    pub fn zoom_out(&mut self) {
        self.zoom_level = (self.zoom_level / 1.25).max(0.5);
    }

    #[allow(dead_code)]
    pub fn focus_task(&mut self, task: &TaskCapsule) {
        if let (Some(lat), Some(lon)) = (task.latitude, task.longitude) {
            self.center_lat = lat;
            self.center_lon = lon;
            self.zoom_level = 2.5;
        }
    }

    pub fn reset_view(&mut self) {
        self.center_lat = 37.4800;
        self.center_lon = 127.0200;
        self.zoom_level = 1.2;
    }
}
