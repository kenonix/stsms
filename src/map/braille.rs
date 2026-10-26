use ratatui::style::Color;

/// 2x4 점자 도트 매트릭스 캔버스
/// 터미널 1셀 = 2열 x 4행의 8개 도트 서브픽셀
#[derive(Clone)]
pub struct BrailleCanvas {
    pub cell_width: usize,
    pub cell_height: usize,
    pub pixel_width: usize,
    pub pixel_height: usize,
    /// 각 셀의 활성화된 도트 비트마스크 (0..255)
    dots: Vec<u8>,
    /// 각 셀의 주요 색상
    colors: Vec<Option<Color>>,
    /// 텍스트 라벨 오버레이 (셀 x, 셀 y, 텍스트, 색상)
    labels: Vec<(usize, usize, String, Color)>,
}

impl BrailleCanvas {
    pub fn new(cell_width: usize, cell_height: usize) -> Self {
        let size = cell_width * cell_height;
        Self {
            cell_width,
            cell_height,
            pixel_width: cell_width * 2,
            pixel_height: cell_height * 4,
            dots: vec![0; size],
            colors: vec![None; size],
            labels: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.dots.fill(0);
        self.colors.fill(None);
        self.labels.clear();
    }

    pub fn resize(&mut self, cell_width: usize, cell_height: usize) {
        if self.cell_width != cell_width || self.cell_height != cell_height {
            self.cell_width = cell_width;
            self.cell_height = cell_height;
            self.pixel_width = cell_width * 2;
            self.pixel_height = cell_height * 4;
            let size = cell_width * cell_height;
            self.dots = vec![0; size];
            self.colors = vec![None; size];
            self.labels.clear();
        }
    }

    /// 서브픽셀 (x, y)에 점자 도트 찍기
    pub fn set_pixel(&mut self, px: i32, py: i32, color: Color) {
        if px < 0 || py < 0 {
            return;
        }
        let px = px as usize;
        let py = py as usize;
        if px >= self.pixel_width || py >= self.pixel_height {
            return;
        }

        let cell_x = px / 2;
        let cell_y = py / 4;
        let idx = cell_y * self.cell_width + cell_x;
        if idx >= self.dots.len() {
            return;
        }

        let sub_x = px % 2;
        let sub_y = py % 4;

        // 표준 유니코드 Braille 비트 매핑:
        // dot 1: (0, 0) -> 0x01
        // dot 2: (0, 1) -> 0x02
        // dot 3: (0, 2) -> 0x04
        // dot 4: (1, 0) -> 0x08
        // dot 5: (1, 1) -> 0x10
        // dot 6: (1, 2) -> 0x20
        // dot 7: (0, 3) -> 0x40
        // dot 8: (1, 3) -> 0x80
        let bit: u8 = match (sub_x, sub_y) {
            (0, 0) => 0x01,
            (0, 1) => 0x02,
            (0, 2) => 0x04,
            (1, 0) => 0x08,
            (1, 1) => 0x10,
            (1, 2) => 0x20,
            (0, 3) => 0x40,
            (1, 3) => 0x80,
            _ => 0,
        };

        self.dots[idx] |= bit;
        self.colors[idx] = Some(color);
    }

    /// Bresenham 알고리즘을 이용한 서브픽셀 직선 드로잉
    pub fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.set_pixel(x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// 연속된 점들을 연결하는 폴리라인 드로잉
    pub fn draw_polyline(&mut self, points: &[(i32, i32)], color: Color) {
        if points.len() < 2 {
            if let Some(&(x, y)) = points.first() {
                self.set_pixel(x, y, color);
            }
            return;
        }
        for window in points.windows(2) {
            let (x0, y0) = window[0];
            let (x1, y1) = window[1];
            self.draw_line(x0, y0, x1, y1, color);
        }
    }

    /// 서브픽셀 원형 마커 그리기
    pub fn draw_circle(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        let mut x = radius;
        let mut y = 0;
        let mut err = 0;

        while x >= y {
            self.set_pixel(cx + x, cy + y, color);
            self.set_pixel(cx + y, cy + x, color);
            self.set_pixel(cx - y, cy + x, color);
            self.set_pixel(cx - x, cy + y, color);
            self.set_pixel(cx - x, cy - y, color);
            self.set_pixel(cx - y, cy - x, color);
            self.set_pixel(cx + y, cy - x, color);
            self.set_pixel(cx + x, cy - y, color);

            y += 1;
            err += 1 + 2 * y;
            if 2 * (err - x) + 1 > 0 {
                x -= 1;
                err += 1 - 2 * x;
            }
        }
    }

    /// 텍스트 라벨 추가 (셀 단위 좌표)
    pub fn add_label(&mut self, cell_x: usize, cell_y: usize, text: String, color: Color) {
        if cell_x < self.cell_width && cell_y < self.cell_height {
            self.labels.push((cell_x, cell_y, text, color));
        }
    }

    /// 셀 (cx, cy)의 점자 문자 및 색상 반환
    pub fn get_cell(&self, cx: usize, cy: usize) -> (char, Option<Color>) {
        if cx >= self.cell_width || cy >= self.cell_height {
            return (' ', None);
        }
        let idx = cy * self.cell_width + cx;
        let dot_mask = self.dots[idx];
        if dot_mask == 0 {
            return (' ', None);
        }
        // U+2800 = 점자 빈 문자
        let c = std::char::from_u32(0x2800 + dot_mask as u32).unwrap_or(' ');
        (c, self.colors[idx])
    }

    pub fn labels(&self) -> &[(usize, usize, String, Color)] {
        &self.labels
    }
}
