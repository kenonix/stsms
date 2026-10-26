use std::io::{self, stdout};
use std::time::Duration;

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
        KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};
use rusqlite::Connection;

mod calendar;
mod db;
mod domain;
mod map;
mod report;
mod ui;

use db::schema::init_database;
use db::seed::seed_sample_data;
use map::osm_data::OsmDataset;
use ui::app::{App, FocusedPanel, ModalState};
use ui::layout::render_ui;
use ui::modals::render_modals;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. SQLite 데이터베이스 초기화 및 시드 데이터 로드
    let conn = Connection::open("stsms_data.db")?;
    init_database(&conn)?;
    let repo = db::repository::Repository::new(&conn);
    seed_sample_data(&repo)?;

    // 2. 터미널 설정 (Raw mode, Alternate Screen, 마우스/터치 이벤트 활성화)
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. 애플리케이션 상태 생성
    let mut app = App::new(&conn);

    // 4. 메인 이벤트 루프
    let tick_rate = Duration::from_millis(40);
    let res = run_loop(&mut terminal, &mut app, &conn, tick_rate).await;

    // 5. 터미널 복원
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error running STSMS: {:?}", err);
    }

    Ok(())
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    conn: &Connection,
    tick_rate: Duration,
) -> io::Result<()> {
    loop {
        // 화면 그리기
        terminal.draw(|frame| {
            render_ui(frame, app);
            render_modals(frame, app);
        })?;

        // 6초 지난 상태 메시지 자동 클리어
        if let Some((_, time)) = &app.status_message {
            if time.elapsed() > Duration::from_secs(6) {
                app.status_message = None;
            }
        }

        // 이벤트 폴링 (키보드 및 마우스/터치)
        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        if app.modal != ModalState::None {
                            handle_modal_key_event(app, conn, key).await;
                        } else {
                            if handle_main_key_event(app, conn, key).await {
                                break;
                            }
                        }
                    }
                }
                Event::Mouse(mouse_event) => {
                    if handle_mouse_event(app, conn, mouse_event).await {
                        break;
                    }
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn rect_contains(r: Rect, x: u16, y: u16) -> bool {
    x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height
}

async fn handle_mouse_event(
    app: &mut App,
    conn: &Connection,
    mouse: MouseEvent,
) -> bool {
    let col = mouse.column;
    let row = mouse.row;

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // 1. 모달이 열려 있는 경우
            if let Some(modal_r) = app.layout_areas.modal_area {
                // 모달 바깥 터치 시 모달 닫기
                if !rect_contains(modal_r, col, row) {
                    app.modal = ModalState::None;
                    return false;
                }
                // 모달 내부 하단 버튼 영역 터치 시 제출/닫기 처리
                if row >= modal_r.y + modal_r.height.saturating_sub(4) {
                    if col < modal_r.x + (modal_r.width * 2 / 3) {
                        match &app.modal {
                            ModalState::Reschedule { .. } => app.submit_reschedule(conn),
                            ModalState::Complete { .. } => app.submit_complete(conn),
                            ModalState::AddAttachment { .. } => app.submit_attachment(conn),
                            _ => app.modal = ModalState::None,
                        }
                    } else {
                        app.modal = ModalState::None;
                    }
                }
                return false;
            }

            // 2. 하단 터치 액션 버튼 바 클릭
            for (action_id, r) in &app.layout_areas.action_buttons {
                if rect_contains(*r, col, row) {
                    match action_id.as_str() {
                        "BTN_CAL" => {
                            app.focused_panel = FocusedPanel::Calendar;
                            app.set_status("포커스: 1. 듀얼 캘린더");
                        }
                        "BTN_MAP" => {
                            app.focused_panel = FocusedPanel::Map;
                            app.set_status("포커스: 3. Mapscii 점자 OSM 지도");
                        }
                        "BTN_AM" => {
                            app.focused_panel = FocusedPanel::SessionBoardAm;
                            app.set_status("포커스: 5. 오전 세션");
                        }
                        "BTN_PM" => {
                            app.focused_panel = FocusedPanel::SessionBoardPm;
                            app.set_status("포커스: 5. 오후 세션");
                        }
                        "BTN_MOVE" => {
                            app.toggle_active_task_session(conn);
                        }
                        "BTN_COMPLETE" => {
                            app.trigger_complete_modal(conn);
                        }
                        "BTN_RESCHEDULE" => {
                            app.trigger_reschedule_modal();
                        }
                        "BTN_PHOTO" => {
                            app.trigger_attachment_modal();
                        }
                        "BTN_DETAIL" => {
                            if let Some(t) = app.get_active_task() {
                                app.modal = ModalState::TaskDetail(t.task_id.clone());
                            }
                        }
                        "BTN_REPORT" => {
                            app.generate_and_show_report();
                        }
                        "BTN_HELP" => {
                            app.modal = ModalState::Help;
                        }
                        "BTN_QUIT" => {
                            return true;
                        }
                        _ => {}
                    }
                    return false;
                }
            }

            // 3. 지도 컨트롤 버튼 터치 (확대, 축소, 중심, OSM갱신)
            if rect_contains(app.layout_areas.map_btn_zoom_in, col, row) {
                app.map_view.zoom_in();
                app.set_status(format!("지도 확대: {:.1}x", app.map_view.zoom_level));
                return false;
            }
            if rect_contains(app.layout_areas.map_btn_zoom_out, col, row) {
                app.map_view.zoom_out();
                app.set_status(format!("지도 축소: {:.1}x", app.map_view.zoom_level));
                return false;
            }
            if rect_contains(app.layout_areas.map_btn_reset, col, row) {
                let target = app.get_active_task().map(|t| (t.latitude, t.longitude, t.title.clone()));
                if let Some((Some(lat), Some(lon), title)) = target {
                    app.map_view.center_lat = lat;
                    app.map_view.center_lon = lon;
                    app.map_view.zoom_level = 2.5;
                    app.set_status(format!("'{}' 위치로 지도 중심 이동", title));
                } else {
                    app.map_view.reset_view();
                    app.set_status("지도 수도권 중심 뷰 리셋");
                }
                return false;
            }
            if rect_contains(app.layout_areas.map_btn_refresh, col, row) {
                app.set_status("📡 Overpass API로부터 최신 OSM 도로망 수신 중...");
                let min_lat = app.map_view.center_lat - 0.15;
                let max_lat = app.map_view.center_lat + 0.15;
                let min_lon = app.map_view.center_lon - 0.20;
                let max_lon = app.map_view.center_lon + 0.20;

                match OsmDataset::fetch_overpass_bbox(min_lat, min_lon, max_lat, max_lon).await {
                    Ok(new_ds) => {
                        let count = new_ds.ways.len();
                        app.map_view.osm_dataset = new_ds;
                        app.set_status(format!("✅ OSM 도로망 갱신 완료 (총 {}개 노선)", count));
                    }
                    Err(e) => {
                        app.set_status(format!("⚠️ OSM 다운로드 실패: {}", e));
                    }
                }
                return false;
            }

            // 4. 캘린더 날짜 셀 터치 선택
            for (date, cell_r) in &app.layout_areas.calendar_cells {
                if rect_contains(*cell_r, col, row) {
                    app.selected_date = *date;
                    app.focused_panel = FocusedPanel::Calendar;
                    app.set_status(format!("날짜 선택: {}", date));
                    return false;
                }
            }

            // 5. 지도 내부 터치 -> 드래그 모드 시작
            if rect_contains(app.layout_areas.map, col, row) {
                app.focused_panel = FocusedPanel::Map;
                app.is_dragging_map = true;
                app.last_touch_pos = Some((col, row));
                return false;
            }

            // 6. 오전 세션 보드 카드 터치
            if rect_contains(app.layout_areas.session_am, col, row) {
                app.focused_panel = FocusedPanel::SessionBoardAm;
                let offset_y = row.saturating_sub(app.layout_areas.session_am.y + 1) as usize;
                let count = app.am_tasks().len();
                if offset_y < count {
                    app.am_selected_idx = offset_y;
                }
                return false;
            }

            // 7. 오후 세션 보드 카드 터치
            if rect_contains(app.layout_areas.session_pm, col, row) {
                app.focused_panel = FocusedPanel::SessionBoardPm;
                let offset_y = row.saturating_sub(app.layout_areas.session_pm.y + 1) as usize;
                let count = app.pm_tasks().len();
                if offset_y < count {
                    app.pm_selected_idx = offset_y;
                }
                return false;
            }

            // 8. 당일 할 일 목록 터치
            if rect_contains(app.layout_areas.todo_list, col, row) {
                app.focused_panel = FocusedPanel::TodoList;
                let offset_y = row.saturating_sub(app.layout_areas.todo_list.y + 1) as usize;
                let count = app.tasks_for_selected_date().len();
                if offset_y < count {
                    app.todo_selected_idx = offset_y;
                }
                return false;
            }

            // 9. D-Day 타임라인 터치
            if rect_contains(app.layout_areas.timeline, col, row) {
                app.focused_panel = FocusedPanel::Timeline;
                let offset_y = row.saturating_sub(app.layout_areas.timeline.y + 1) as usize;
                let count = app.tasks.iter().filter(|t| t.visual_dday_tag.is_some()).count();
                if offset_y < count {
                    app.timeline_selected_idx = offset_y;
                }
                return false;
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            // 지도 드래그 패닝
            if app.is_dragging_map {
                if let Some((last_x, last_y)) = app.last_touch_pos {
                    let dx = (col as f64) - (last_x as f64);
                    let dy = (row as f64) - (last_y as f64);

                    if dx.abs() > 0.0 || dy.abs() > 0.0 {
                        // 드래그 방향에 맞춰 지도 중심 이동
                        let scale = 0.005 / app.map_view.zoom_level;
                        app.map_view.pan(dy * scale * 2.0, -dx * scale * 2.0);
                        app.last_touch_pos = Some((col, row));
                    }
                }
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            app.is_dragging_map = false;
            app.last_touch_pos = None;
        }
        MouseEventKind::ScrollUp => {
            // 제스처 스크롤 업: 지도 줌 인 또는 목록 위로 스크롤
            if rect_contains(app.layout_areas.map, col, row) {
                app.map_view.zoom_in();
                app.set_status(format!("지도 확대: {:.1}x", app.map_view.zoom_level));
            } else if rect_contains(app.layout_areas.session_am, col, row) {
                app.am_selected_idx = app.am_selected_idx.saturating_sub(1);
            } else if rect_contains(app.layout_areas.session_pm, col, row) {
                app.pm_selected_idx = app.pm_selected_idx.saturating_sub(1);
            } else if rect_contains(app.layout_areas.todo_list, col, row) {
                app.todo_selected_idx = app.todo_selected_idx.saturating_sub(1);
            } else if rect_contains(app.layout_areas.calendar, col, row) {
                app.selected_date = app.selected_date - chrono::Duration::days(7);
            }
        }
        MouseEventKind::ScrollDown => {
            // 제스처 스크롤 다운: 지도 줌 아웃 또는 목록 아래로 스크롤
            if rect_contains(app.layout_areas.map, col, row) {
                app.map_view.zoom_out();
                app.set_status(format!("지도 축소: {:.1}x", app.map_view.zoom_level));
            } else if rect_contains(app.layout_areas.session_am, col, row) {
                let count = app.am_tasks().len();
                if count > 0 && app.am_selected_idx + 1 < count {
                    app.am_selected_idx += 1;
                }
            } else if rect_contains(app.layout_areas.session_pm, col, row) {
                let count = app.pm_tasks().len();
                if count > 0 && app.pm_selected_idx + 1 < count {
                    app.pm_selected_idx += 1;
                }
            } else if rect_contains(app.layout_areas.todo_list, col, row) {
                let count = app.tasks_for_selected_date().len();
                if count > 0 && app.todo_selected_idx + 1 < count {
                    app.todo_selected_idx += 1;
                }
            } else if rect_contains(app.layout_areas.calendar, col, row) {
                app.selected_date = app.selected_date + chrono::Duration::days(7);
            }
        }
        _ => {}
    }

    false
}

async fn handle_main_key_event(
    app: &mut App,
    conn: &Connection,
    key: event::KeyEvent,
) -> bool {
    if key.code == KeyCode::Char('q')
        || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
    {
        return true;
    }

    match key.code {
        KeyCode::Char('1') => {
            app.focused_panel = FocusedPanel::Calendar;
            app.set_status("포커스: 1. 듀얼 캘린더");
            return false;
        }
        KeyCode::Char('2') => {
            app.focused_panel = FocusedPanel::Timeline;
            app.set_status("포커스: 2. D-Day 타임라인");
            return false;
        }
        KeyCode::Char('3') => {
            app.focused_panel = FocusedPanel::Map;
            app.set_status("포커스: 3. Mapscii 점자 OSM 지도");
            return false;
        }
        KeyCode::Char('4') => {
            app.focused_panel = FocusedPanel::TodoList;
            app.set_status("포커스: 4. 당일 할 일 목록");
            return false;
        }
        KeyCode::Char('5') => {
            app.focused_panel = match app.focused_panel {
                FocusedPanel::SessionBoardAm => FocusedPanel::SessionBoardPm,
                _ => FocusedPanel::SessionBoardAm,
            };
            app.set_status("포커스: 5. 작업 세션 드롭존");
            return false;
        }
        KeyCode::Tab => {
            app.cycle_panel_forward();
            return false;
        }
        KeyCode::BackTab => {
            app.cycle_panel_backward();
            return false;
        }
        KeyCode::Char('?') => {
            app.modal = ModalState::Help;
            return false;
        }
        KeyCode::Char('r') => {
            app.trigger_reschedule_modal();
            return false;
        }
        KeyCode::Char(' ') => {
            app.trigger_complete_modal(conn);
            return false;
        }
        KeyCode::Char('m') | KeyCode::Char('t') => {
            app.toggle_active_task_session(conn);
            return false;
        }
        KeyCode::Char('a') => {
            app.trigger_attachment_modal();
            return false;
        }
        KeyCode::Char('p') => {
            app.generate_and_show_report();
            return false;
        }
        KeyCode::Char('v') | KeyCode::Enter => {
            if let Some(t) = app.get_active_task() {
                app.modal = ModalState::TaskDetail(t.task_id.clone());
            }
            return false;
        }
        _ => {}
    }

    match app.focused_panel {
        FocusedPanel::Calendar => match key.code {
            KeyCode::Left => {
                app.selected_date = app.selected_date.pred_opt().unwrap_or(app.selected_date);
            }
            KeyCode::Right => {
                app.selected_date = app.selected_date.succ_opt().unwrap_or(app.selected_date);
            }
            KeyCode::Up => {
                let d = app.selected_date - chrono::Duration::days(7);
                app.selected_date = d;
            }
            KeyCode::Down => {
                let d = app.selected_date + chrono::Duration::days(7);
                app.selected_date = d;
            }
            _ => {}
        },
        FocusedPanel::Timeline => match key.code {
            KeyCode::Up => {
                if app.timeline_selected_idx > 0 {
                    app.timeline_selected_idx -= 1;
                }
            }
            KeyCode::Down => {
                let count = app.tasks.iter().filter(|t| t.visual_dday_tag.is_some()).count();
                if app.timeline_selected_idx + 1 < count {
                    app.timeline_selected_idx += 1;
                }
            }
            _ => {}
        },
        FocusedPanel::Map => match key.code {
            KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Char('z') => {
                app.map_view.zoom_in();
                app.set_status(format!("지도 확대: {:.1}x", app.map_view.zoom_level));
            }
            KeyCode::Char('-') | KeyCode::Char('_') | KeyCode::Char('x') => {
                app.map_view.zoom_out();
                app.set_status(format!("지도 축소: {:.1}x", app.map_view.zoom_level));
            }
            KeyCode::Left | KeyCode::Char('h') => {
                app.map_view.pan(0.0, -0.05);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                app.map_view.pan(0.0, 0.05);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.map_view.pan(0.05, 0.0);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.map_view.pan(-0.05, 0.0);
            }
            KeyCode::Char('c') => {
                let target = app.get_active_task().map(|t| (t.latitude, t.longitude, t.title.clone()));
                if let Some((Some(lat), Some(lon), title)) = target {
                    app.map_view.center_lat = lat;
                    app.map_view.center_lon = lon;
                    app.map_view.zoom_level = 2.5;
                    app.set_status(format!("'{}' 위치로 지도 중심 이동", title));
                } else {
                    app.map_view.reset_view();
                    app.set_status("지도 수도권 중심 뷰 리셋");
                }
            }
            KeyCode::Char('o') => {
                app.set_status("📡 Overpass API로부터 최신 OSM 도로망 데이터 수신 중...");
                let min_lat = app.map_view.center_lat - 0.15;
                let max_lat = app.map_view.center_lat + 0.15;
                let min_lon = app.map_view.center_lon - 0.20;
                let max_lon = app.map_view.center_lon + 0.20;

                match OsmDataset::fetch_overpass_bbox(min_lat, min_lon, max_lat, max_lon).await {
                    Ok(new_ds) => {
                        let count = new_ds.ways.len();
                        app.map_view.osm_dataset = new_ds;
                        app.set_status(format!("✅ OSM 도로망 데이터 갱신 완료 (총 {}개 노선)", count));
                    }
                    Err(e) => {
                        app.set_status(format!("⚠️ OSM 다운로드 실패: {}", e));
                    }
                }
            }
            _ => {}
        },
        FocusedPanel::TodoList => match key.code {
            KeyCode::Up => {
                if app.todo_selected_idx > 0 {
                    app.todo_selected_idx -= 1;
                }
            }
            KeyCode::Down => {
                let count = app.tasks_for_selected_date().len();
                if count > 0 && app.todo_selected_idx + 1 < count {
                    app.todo_selected_idx += 1;
                }
            }
            _ => {}
        },
        FocusedPanel::SessionBoardAm => match key.code {
            KeyCode::Up => {
                if app.am_selected_idx > 0 {
                    app.am_selected_idx -= 1;
                }
            }
            KeyCode::Down => {
                let count = app.am_tasks().len();
                if count > 0 && app.am_selected_idx + 1 < count {
                    app.am_selected_idx += 1;
                }
            }
            _ => {}
        },
        FocusedPanel::SessionBoardPm => match key.code {
            KeyCode::Up => {
                if app.pm_selected_idx > 0 {
                    app.pm_selected_idx -= 1;
                }
            }
            KeyCode::Down => {
                let count = app.pm_tasks().len();
                if count > 0 && app.pm_selected_idx + 1 < count {
                    app.pm_selected_idx += 1;
                }
            }
            _ => {}
        },
    }

    false
}

fn is_enter_key(key: &event::KeyEvent) -> bool {
    key.code == KeyCode::Enter
        || (key.modifiers.contains(KeyModifiers::CONTROL)
            && (key.code == KeyCode::Char('j') || key.code == KeyCode::Char('m')))
}

fn is_backspace_key(key: &event::KeyEvent) -> bool {
    key.code == KeyCode::Backspace
        || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('h'))
}

async fn handle_modal_key_event(
    app: &mut App,
    conn: &Connection,
    key: event::KeyEvent,
) {
    let enter = is_enter_key(&key);
    let backspace = is_backspace_key(&key);

    match &mut app.modal {
        ModalState::Help | ModalState::TaskDetail(_) | ModalState::Message(_) => {
            if key.code == KeyCode::Esc || enter || key.code == KeyCode::Char('q') {
                app.modal = ModalState::None;
            }
        }
        ModalState::A4Report { scroll, .. } => {
            if key.code == KeyCode::Esc || enter || key.code == KeyCode::Char('q') {
                app.modal = ModalState::None;
            } else if key.code == KeyCode::Up || key.code == KeyCode::Char('k') {
                *scroll = scroll.saturating_sub(1);
            } else if key.code == KeyCode::Down || key.code == KeyCode::Char('j') {
                *scroll += 1;
            }
        }
        ModalState::Reschedule {
            new_date_input,
            reason_input,
            active_field,
            ..
        } => {
            if key.code == KeyCode::Esc {
                app.modal = ModalState::None;
            } else if key.code == KeyCode::Tab {
                *active_field = (*active_field + 1) % 2;
            } else if enter {
                app.submit_reschedule(conn);
            } else if backspace {
                if *active_field == 0 {
                    new_date_input.pop();
                } else {
                    reason_input.pop();
                }
            } else if let KeyCode::Char(c) = key.code {
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT)
                {
                    if *active_field == 0 {
                        new_date_input.push(c);
                    } else {
                        reason_input.push(c);
                    }
                }
            }
        }
        ModalState::Complete {
            count_input,
            summary_input,
            active_field,
            ..
        } => {
            if key.code == KeyCode::Esc {
                app.modal = ModalState::None;
            } else if key.code == KeyCode::Tab {
                *active_field = (*active_field + 1) % 2;
            } else if enter {
                app.submit_complete(conn);
            } else if backspace {
                if *active_field == 0 {
                    count_input.pop();
                } else {
                    summary_input.pop();
                }
            } else if let KeyCode::Char(c) = key.code {
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT)
                {
                    if *active_field == 0 {
                        if c.is_ascii_digit() {
                            count_input.push(c);
                        }
                    } else {
                        summary_input.push(c);
                    }
                }
            }
        }
        ModalState::AddAttachment {
            file_name_input,
            ..
        } => {
            if key.code == KeyCode::Esc {
                app.modal = ModalState::None;
            } else if key.code == KeyCode::Tab {
                file_name_input.clear();
            } else if enter {
                app.submit_attachment(conn);
            } else if backspace {
                file_name_input.pop();
            } else if let KeyCode::Char(c) = key.code {
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT)
                {
                    file_name_input.push(c);
                }
            }
        }
        _ => {
            if key.code == KeyCode::Esc || enter {
                app.modal = ModalState::None;
            }
        }
    }
}
