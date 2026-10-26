use chrono::{Datelike, NaiveDate};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::calendar::dual_calendar::solar_to_lunar;
use crate::domain::models::*;
use crate::ui::app::{App, FocusedPanel};

pub fn render_ui(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // 초기화
    app.layout_areas.action_buttons.clear();
    app.layout_areas.calendar_cells.clear();

    // 전체 수직 분할: [TopBar: 3] [MainMiddle: Min 12] [BottomDock: 10] [StatusBar: 2]
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(10),
            Constraint::Length(2),
        ])
        .split(size);

    app.layout_areas.top_bar = vertical_chunks[0];
    app.layout_areas.status_bar = vertical_chunks[3];

    render_top_bar(frame, app, vertical_chunks[0]);
    render_middle_panels(frame, app, vertical_chunks[1]);
    render_bottom_dock(frame, app, vertical_chunks[2]);
    render_status_bar(frame, app, vertical_chunks[3]);
}

fn render_top_bar(frame: &mut Frame, app: &mut App, area: Rect) {
    let mode_str = match app.operation_mode {
        OperationMode::SoloSplit => "1인용 [업무/개인]",
        OperationMode::TeamMulti => "다수용 [팀 멀티]",
    };

    let lunar_str = solar_to_lunar(app.selected_date)
        .map(|l| format!(" (음력 {})", l.format_short()))
        .unwrap_or_default();

    let weather_summary = app
        .tasks_for_selected_date()
        .iter()
        .find_map(|t| t.weather_snapshot.as_ref())
        .map(|w| {
            format!(
                "{} {:.1}°C | 강수 {}% | {}",
                w.weather_sky_code,
                w.temperature_celsius,
                w.precipitation_prob_percent,
                if w.umbrella_required {
                    "🔴 우산 필수"
                } else {
                    "🟢 쾌적"
                }
            )
        })
        .unwrap_or_else(|| "맑음 14.5°C | 🟢 활동 원활".to_string());

    let header_line = Line::from(vec![
        Span::styled(" STSMS ", Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(
            format!("선택일: {}{}", app.selected_date.format("%Y-%m-%d (%a)"), lunar_str),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  모드: "),
        Span::styled(mode_str, Style::default().fg(Color::Cyan)),
        Span::raw("  |  날씨: "),
        Span::styled(weather_summary, Style::default().fg(Color::LightGreen)),
        Span::raw("  |  [❓ 도움말]"),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let p = Paragraph::new(header_line).block(block);
    frame.render_widget(p, area);
}

fn render_middle_panels(frame: &mut Frame, app: &mut App, area: Rect) {
    // 3분할: [좌측 캘린더: 24%] [중앙 타임라인+지도: 52%] [우측 할일: 24%]
    let horiz_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(24),
            Constraint::Percentage(52),
            Constraint::Percentage(24),
        ])
        .split(area);

    app.layout_areas.calendar = horiz_chunks[0];
    app.layout_areas.todo_list = horiz_chunks[2];

    render_calendar_panel(frame, app, horiz_chunks[0]);
    render_center_panel(frame, app, horiz_chunks[1]);
    render_todo_panel(frame, app, horiz_chunks[2]);
}

fn render_calendar_panel(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::Calendar;
    let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };

    let mut lines = Vec::new();
    let sel_year = app.selected_date.year();
    let sel_month = app.selected_date.month();

    lines.push(Line::from(vec![
        Span::styled("  ◀ ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!("{}년 {}월", sel_year, sel_month),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ▶  ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("[오늘]", Style::default().bg(Color::DarkGray).fg(Color::White)),
    ]));
    lines.push(Line::from(" 일  월  화  수  목  금  토"));

    // 달력 날짜 계산
    let first_day = NaiveDate::from_ymd_opt(sel_year, sel_month, 1).unwrap();
    let start_weekday = first_day.weekday().num_days_from_sunday();

    let mut current_day = first_day;
    let mut day_spans = Vec::new();

    let mut current_row_y = area.y + 3;

    for _ in 0..start_weekday {
        day_spans.push(Span::raw("    "));
    }

    while current_day.month() == sel_month {
        let is_sel = current_day == app.selected_date;
        let is_today = current_day == app.today;
        let is_holiday = app.holidays.iter().any(|h| h.target_date == current_day);
        let has_task = app.tasks.iter().any(|t| t.execution_date == current_day);

        let day_num = current_day.day();
        let num_str = format!("{:2}", day_num);

        let style = if is_sel {
            Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
        } else if is_today {
            Style::default().bg(Color::DarkGray).fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else if is_holiday {
            Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)
        } else if has_task {
            Style::default().fg(Color::LightGreen)
        } else if current_day.weekday().num_days_from_sunday() == 0 {
            Style::default().fg(Color::Red)
        } else if current_day.weekday().num_days_from_sunday() == 6 {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };

        let col_idx = current_day.weekday().num_days_from_sunday();
        let cell_rect = Rect::new(area.x + 1 + (col_idx as u16) * 4, current_row_y, 4, 1);
        app.layout_areas.calendar_cells.push((current_day, cell_rect));

        day_spans.push(Span::styled(format!("{} ", num_str), style));

        if current_day.weekday().num_days_from_sunday() == 6 {
            lines.push(Line::from(day_spans.clone()));
            day_spans.clear();
            current_row_y += 1;
        }

        current_day = current_day.succ_opt().unwrap_or(current_day);
    }

    if !day_spans.is_empty() {
        lines.push(Line::from(day_spans));
    }

    // 하단 음력 및 공휴일 정보
    lines.push(Line::from("-----------------------"));
    let lunar_info = solar_to_lunar(app.selected_date)
        .map(|l| format!("음력: {}월 {}일{}", l.month, l.day, if l.is_leap_month { "(윤)" } else { "" }))
        .unwrap_or_default();
    lines.push(Line::from(Span::styled(lunar_info, Style::default().fg(Color::DarkGray))));

    for h in &app.holidays {
        if h.target_date == app.selected_date {
            lines.push(Line::from(vec![
                Span::styled("● ", Style::default().fg(Color::Red)),
                Span::styled(&h.title, Style::default().fg(Color::Yellow)),
            ]));
        }
    }

    let block = Block::default()
        .title(" [ 1. 듀얼 캘린더 (터치선택 가능) ] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let p = Paragraph::new(lines).block(block);
    frame.render_widget(p, area);
}

fn render_center_panel(frame: &mut Frame, app: &mut App, area: Rect) {
    let center_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(26), Constraint::Percentage(74)])
        .split(area);

    app.layout_areas.timeline = center_chunks[0];
    app.layout_areas.map = center_chunks[1];

    render_timeline_widget(frame, app, center_chunks[0]);
    render_braille_map_widget(frame, app, center_chunks[1]);
}

fn render_timeline_widget(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::Timeline;
    let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };

    let mut items = Vec::new();
    let tasks_with_tags: Vec<&TaskCapsule> = app
        .tasks
        .iter()
        .filter(|t| t.visual_dday_tag.is_some())
        .collect();

    for (idx, t) in tasks_with_tags.iter().enumerate() {
        let tag = t.visual_dday_tag.as_deref().unwrap_or("");
        let is_selected = idx == app.timeline_selected_idx;

        let d_days = (t.execution_date - app.today).num_days();
        let d_str = if d_days == 0 {
            "D-DAY".to_string()
        } else if d_days > 0 {
            format!("D-{}", d_days)
        } else {
            format!("D+{}", d_days.abs())
        };

        let style = if is_selected {
            Style::default().bg(Color::DarkGray).fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let line = Line::from(vec![
            Span::styled(format!("[{}] ", d_str), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled(tag, style),
            Span::raw(format!(" (예정: {})", t.execution_date)),
        ]);
        items.push(ListItem::new(line));
    }

    let block = Block::default()
        .title(" [ 2. D-Day 타임라인 ] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_braille_map_widget(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::Map;
    let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };

    let inner_width = area.width.saturating_sub(2) as usize;
    let inner_height = area.height.saturating_sub(2) as usize;

    if inner_width > 0 && inner_height > 0 {
        app.braille_canvas.resize(inner_width, inner_height);
        let tasks_copy = app.tasks.clone();
        app.map_view.render_to_canvas(&mut app.braille_canvas, &tasks_copy);
    }

    // 터치 전용 줌/이동 컨트롤 버튼 영역 등록
    let btn_y = area.y;
    let btn_x = area.x + area.width.saturating_sub(44);
    app.layout_areas.map_btn_zoom_in = Rect::new(btn_x, btn_y, 7, 1);
    app.layout_areas.map_btn_zoom_out = Rect::new(btn_x + 8, btn_y, 7, 1);
    app.layout_areas.map_btn_reset = Rect::new(btn_x + 16, btn_y, 7, 1);
    app.layout_areas.map_btn_refresh = Rect::new(btn_x + 24, btn_y, 10, 1);

    let mut map_lines = Vec::new();
    for cy in 0..app.braille_canvas.cell_height {
        let mut row_spans = Vec::new();
        let mut cx = 0;

        while cx < app.braille_canvas.cell_width {
            if let Some((_, _, text, color)) = app
                .braille_canvas
                .labels()
                .iter()
                .find(|(lx, ly, _, _)| *lx == cx && *ly == cy)
            {
                row_spans.push(Span::styled(
                    text.clone(),
                    Style::default().fg(*color).add_modifier(Modifier::BOLD),
                ));
                let char_len = text.chars().count();
                cx += char_len.max(1);
            } else {
                let (ch, color) = app.braille_canvas.get_cell(cx, cy);
                let style = match color {
                    Some(c) => Style::default().fg(c),
                    None => Style::default().fg(Color::DarkGray),
                };
                row_spans.push(Span::styled(ch.to_string(), style));
                cx += 1;
            }
        }
        map_lines.push(Line::from(row_spans));
    }

    let title_line = Line::from(vec![
        Span::raw(format!(" 3. 점자 지도({:.1}x) ", app.map_view.zoom_level)),
        Span::styled("[➕확대]", Style::default().bg(Color::Blue).fg(Color::White)),
        Span::raw(" "),
        Span::styled("[➖축소]", Style::default().bg(Color::Blue).fg(Color::White)),
        Span::raw(" "),
        Span::styled("[🎯중심]", Style::default().bg(Color::DarkGray).fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled("[🔄OSM갱신]", Style::default().bg(Color::DarkGray).fg(Color::Cyan)),
        Span::raw(" (터치 드래그/스크롤) "),
    ]);

    let block = Block::default()
        .title(title_line)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let p = Paragraph::new(map_lines).block(block);
    frame.render_widget(p, area);
}

fn render_todo_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_panel == FocusedPanel::TodoList;
    let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };

    let tasks = app.tasks_for_selected_date();
    let mut items = Vec::new();

    for (idx, t) in tasks.iter().enumerate() {
        let is_selected = idx == app.todo_selected_idx;
        let status_color = match t.status {
            TaskStatus::Completed => Color::Green,
            TaskStatus::InSession => Color::Yellow,
            TaskStatus::RescheduledHold => Color::LightRed,
            _ => Color::White,
        };

        let status_badge = format!("[{}]", t.status.label());

        let line = Line::from(vec![
            Span::styled(
                if is_selected { "▶ " } else { "  " },
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                format!("{}. ", idx + 1),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(status_badge, Style::default().fg(status_color)),
            Span::raw(" "),
            Span::styled(
                &t.title,
                if is_selected {
                    Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                },
            ),
        ]);
        items.push(ListItem::new(line));
    }

    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "선택한 날짜에 등록된 업무가 없습니다.",
            Style::default().fg(Color::DarkGray),
        ))));
    }

    let block = Block::default()
        .title(" [ 4. 당일 할 일 (탭하여 선택) ] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_bottom_dock(frame: &mut Frame, app: &mut App, area: Rect) {
    let dock_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    app.layout_areas.session_am = dock_chunks[0];
    app.layout_areas.session_pm = dock_chunks[1];

    render_session_board(frame, app, SessionSlot::Am, dock_chunks[0]);
    render_session_board(frame, app, SessionSlot::Pm, dock_chunks[1]);
}

fn render_session_board(frame: &mut Frame, app: &App, session: SessionSlot, area: Rect) {
    let is_am = session == SessionSlot::Am;
    let is_focused = if is_am {
        app.focused_panel == FocusedPanel::SessionBoardAm
    } else {
        app.focused_panel == FocusedPanel::SessionBoardPm
    };

    let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };
    let tasks = if is_am { app.am_tasks() } else { app.pm_tasks() };
    let sel_idx = if is_am { app.am_selected_idx } else { app.pm_selected_idx };

    let mut items = Vec::new();

    for (idx, t) in tasks.iter().enumerate() {
        let is_selected = is_focused && idx == sel_idx;

        let mut tags = Vec::new();
        if t.requires_report {
            tags.push(Span::styled("[보고필수]", Style::default().fg(Color::LightRed)));
        }
        if t.requires_photo_evidence {
            tags.push(Span::styled("[사진필수]", Style::default().fg(Color::LightYellow)));
        }
        if let Some(w) = &t.weather_snapshot {
            if w.umbrella_required {
                tags.push(Span::styled("[우산필수]", Style::default().fg(Color::Red)));
            }
        }
        if t.attachments.iter().any(|a| a.attachment_type == "MESSENGER_PHOTO") {
            tags.push(Span::styled("[📸사진완료]", Style::default().fg(Color::Green)));
        }

        let status_color = match t.status {
            TaskStatus::Completed => Color::Green,
            TaskStatus::InSession => Color::Yellow,
            TaskStatus::RescheduledHold => Color::LightRed,
            _ => Color::White,
        };

        let mut spans = vec![
            Span::styled(if is_selected { "▶ " } else { "  " }, Style::default().fg(Color::Cyan)),
            Span::styled(format!("[{}] ", t.status.label()), Style::default().fg(status_color)),
            Span::styled(
                &t.title,
                if is_selected {
                    Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            ),
            Span::raw(" "),
        ];
        spans.extend(tags);

        items.push(ListItem::new(Line::from(spans)));
    }

    if items.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "(배정된 세션 업무 없음 - 탭하여 선택 후 [이동])",
            Style::default().fg(Color::DarkGray),
        ))));
    }

    let title = if is_am {
        " [ 5. 오전 세션 (AM) - 터치선택 ] "
    } else {
        " [ 5. 오후 세션 (PM) - 터치선택 ] "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_status_bar(frame: &mut Frame, app: &mut App, area: Rect) {
    let msg_span = if let Some((msg, _)) = &app.status_message {
        Span::styled(msg, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("터치 또는 마우스 클릭/드래그 지원 중", Style::default().fg(Color::Green))
    };

    let line1 = Line::from(vec![
        Span::styled(" [알림] ", Style::default().bg(Color::DarkGray).fg(Color::White)),
        Span::raw(" "),
        msg_span,
    ]);

    // 하단 터치 액션 버튼 바
    let buttons = [
        ("BTN_CAL", " [📅달력] "),
        ("BTN_MAP", " [🗺️지도] "),
        ("BTN_AM", " [☀️오전] "),
        ("BTN_PM", " [🌙오후] "),
        ("BTN_MOVE", " [🔄AM↔PM이동] "),
        ("BTN_COMPLETE", " [✅완료] "),
        ("BTN_RESCHEDULE", " [⏩연기] "),
        ("BTN_PHOTO", " [📸사진] "),
        ("BTN_DETAIL", " [🔍상세] "),
        ("BTN_REPORT", " [🖨️보고서] "),
        ("BTN_HELP", " [❓도움] "),
        ("BTN_QUIT", " [❌종료] "),
    ];

    let mut line2_spans = Vec::new();
    let mut current_x = area.x;
    let button_y = area.y + 1;

    for (id, label) in buttons {
        let len = label.chars().count() as u16;
        let btn_rect = Rect::new(current_x, button_y, len, 1);
        app.layout_areas.action_buttons.push((id.to_string(), btn_rect));

        let style = match id {
            "BTN_COMPLETE" => Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD),
            "BTN_RESCHEDULE" => Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD),
            "BTN_QUIT" => Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD),
            "BTN_MOVE" => Style::default().bg(Color::Magenta).fg(Color::White),
            "BTN_MAP" | "BTN_CAL" => Style::default().bg(Color::Blue).fg(Color::White),
            _ => Style::default().bg(Color::DarkGray).fg(Color::White),
        };

        line2_spans.push(Span::styled(label, style));
        line2_spans.push(Span::raw(" "));
        current_x += len + 1;
    }

    let p = Paragraph::new(vec![line1, Line::from(line2_spans)]);
    frame.render_widget(p, area);
}
