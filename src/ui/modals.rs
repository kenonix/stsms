use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::ui::app::{App, ModalState};

pub fn render_modals(frame: &mut Frame, app: &mut App) {
    app.layout_areas.modal_buttons.clear();
    match &app.modal {
        ModalState::None => {
            app.layout_areas.modal_area = None;
        }
        ModalState::Help => {
            let area = centered_rect(70, 75, frame.area());
            app.layout_areas.modal_area = Some(area);
            render_help_modal(frame, area);
        }
        ModalState::TaskDetail(task_id) => {
            let area = centered_rect(75, 80, frame.area());
            app.layout_areas.modal_area = Some(area);
            render_task_detail_modal(frame, app, task_id, area);
        }
        ModalState::Reschedule {
            task_id,
            new_date_input,
            reason_input,
            error_msg,
            active_field,
        } => {
            let area = centered_rect(60, 48, frame.area());
            app.layout_areas.modal_area = Some(area);
            render_reschedule_modal(
                frame,
                app,
                task_id,
                new_date_input,
                reason_input,
                error_msg.as_deref(),
                *active_field,
                area,
            );
        }
        ModalState::Complete {
            task_id,
            count_input,
            summary_input,
            error_msg,
            active_field,
        } => {
            let area = centered_rect(65, 54, frame.area());
            app.layout_areas.modal_area = Some(area);
            render_complete_modal(
                frame,
                app,
                task_id,
                count_input,
                summary_input,
                error_msg.as_deref(),
                *active_field,
                area,
            );
        }
        ModalState::A4Report {
            content,
            saved_path,
            scroll,
        } => {
            let area = centered_rect(80, 85, frame.area());
            app.layout_areas.modal_area = Some(area);
            render_report_modal(frame, content, saved_path.as_deref(), *scroll, area);
        }
        ModalState::AddAttachment {
            task_id,
            file_name_input,
        } => {
            let area = centered_rect(65, 42, frame.area());
            app.layout_areas.modal_area = Some(area);
            render_add_attachment_modal(frame, app, task_id, file_name_input, area);
        }
        ModalState::Message(msg) => {
            let area = centered_rect(50, 25, frame.area());
            app.layout_areas.modal_area = Some(area);
            render_message_modal(frame, msg, area);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_help_modal(frame: &mut Frame, area: Rect) {
    frame.render_widget(Clear, area);

    let text = vec![
        Line::from(Span::styled(
            "=== STSMS v1.0 TUI 단축키 가이드 ===",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("[1]~[5] ", Style::default().fg(Color::Cyan)),
            Span::raw("패널 직접 이동 (1:캘린더, 2:타임라인, 3:점자지도, 4:할일, 5:작업보드)"),
        ]),
        Line::from(vec![
            Span::styled("[Tab] / [Shift+Tab] ", Style::default().fg(Color::Cyan)),
            Span::raw("패널 순환 포커스 전환"),
        ]),
        Line::from(vec![
            Span::styled("[방향키] ", Style::default().fg(Color::Cyan)),
            Span::raw("캘린더 날짜 이동 / 지도 팬(이동) / 목록 탐색"),
        ]),
        Line::from(vec![
            Span::styled("[+ / -] ", Style::default().fg(Color::Cyan)),
            Span::raw("지도 줌 인 / 줌 아웃 (Mapscii 점자 OSM)"),
        ]),
        Line::from(vec![
            Span::styled("[c] ", Style::default().fg(Color::Cyan)),
            Span::raw("지도 중심을 현재 선택된 업무 좌표로 리셋"),
        ]),
        Line::from(vec![
            Span::styled("[o] ", Style::default().fg(Color::Cyan)),
            Span::raw("현재 지도 영역의 최신 OpenStreetMap 데이터 Overpass API 실시간 갱신"),
        ]),
        Line::from(vec![
            Span::styled("[m] 또는 [t] ", Style::default().fg(Color::Cyan)),
            Span::raw("선택된 업무를 오전(AM) <-> 오후(PM) 세션으로 즉시 이동"),
        ]),
        Line::from(vec![
            Span::styled("[Space] / [Enter] ", Style::default().fg(Color::Cyan)),
            Span::raw("업무 완료 처리 (Gate 1 실적/결과 요약 및 Gate 2 사진 검증 모달)"),
        ]),
        Line::from(vec![
            Span::styled("[r] ", Style::default().fg(Color::Cyan)),
            Span::raw("일정 연기 (Gate 3 사유 필수 입력 모달, baseline_date 불변 보존)"),
        ]),
        Line::from(vec![
            Span::styled("[a] ", Style::default().fg(Color::Cyan)),
            Span::raw("현장 증빙 사진 캡슐 첨부 (Gate 2 통과용 모의 사진)"),
        ]),
        Line::from(vec![
            Span::styled("[v] ", Style::default().fg(Color::Cyan)),
            Span::raw("업무 상세 팝오버 및 초 단위 감사 로그 이력 보기"),
        ]),
        Line::from(vec![
            Span::styled("[p] ", Style::default().fg(Color::Cyan)),
            Span::raw("일일 정형 업무보고서 (REQ-13 A4 규격) 생성 및 디스크 저장"),
        ]),
        Line::from(vec![
            Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
            Span::raw("열려 있는 팝업 모달 닫기"),
        ]),
        Line::from(vec![
            Span::styled("[q] ", Style::default().fg(Color::Cyan)),
            Span::raw("프로그램 종료"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "창을 닫으려면 [Esc] 키를 누르세요.",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let block = Block::default()
        .title(" [ 시스템 도움말 & 키보드 단축키 안내 ] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let p = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

fn render_reschedule_modal(
    frame: &mut Frame,
    app: &App,
    task_id: &str,
    new_date: &str,
    reason: &str,
    error_msg: Option<&str>,
    active_field: usize,
    area: Rect,
) {
    frame.render_widget(Clear, area);

    let task = app.tasks.iter().find(|t| t.task_id == task_id);
    let title = task.map(|t| t.title.as_str()).unwrap_or("업무");
    let baseline = task
        .map(|t| t.baseline_date.to_string())
        .unwrap_or_default();
    let current_exec = task
        .map(|t| t.execution_date.to_string())
        .unwrap_or_default();

    let mut lines = vec![
        Line::from(Span::styled(
            format!("[Gate 3: 마스터 스케줄 보존형 일정 조정]"),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("대상 업무: {}", title)),
        Line::from(vec![
            Span::raw("마스터 기준일(불변): "),
            Span::styled(baseline, Style::default().fg(Color::Green)),
            Span::raw(" | 현재 실행일: "),
            Span::styled(current_exec, Style::default().fg(Color::Red)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "1. 변경 실행 예정일 (YYYY-MM-DD): ",
                if active_field == 0 {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Span::styled(
                format!(" [ {} ]", new_date),
                if active_field == 0 {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                },
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "2. 일정 조정 사유 (필수 입력): ",
                if active_field == 1 {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Span::styled(
                format!(" [ {}_ ]", reason),
                if active_field == 1 {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                },
            ),
        ]),
        Line::from(""),
    ];

    if let Some(err) = error_msg {
        lines.push(Line::from(Span::styled(
            err,
            Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "* 사유는 2글자 이상 입력 필수이며, 원본 마스터 기준선에는 왜곡을 주지 않습니다.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("[Tab] ", Style::default().fg(Color::Cyan)),
        Span::raw("입력칸 이동  |  "),
        Span::styled("[Enter] ", Style::default().fg(Color::Green)),
        Span::raw("연기 확정  |  "),
        Span::styled("[Esc] ", Style::default().fg(Color::Red)),
        Span::raw("취소"),
    ]));

    let block = Block::default()
        .title(" [ 일정 연기 및 사유 기록 (REQ-07) ] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

fn render_complete_modal(
    frame: &mut Frame,
    app: &App,
    task_id: &str,
    count_input: &str,
    summary_input: &str,
    error_msg: Option<&str>,
    active_field: usize,
    area: Rect,
) {
    frame.render_widget(Clear, area);

    let task = app.tasks.iter().find(|t| t.task_id == task_id);
    let title = task.map(|t| t.title.as_str()).unwrap_or("업무");
    let req_report = task.map(|t| t.requires_report).unwrap_or(false);
    let req_photo = task.map(|t| t.requires_photo_evidence).unwrap_or(false);
    let photo_count = task.map(|t| t.attachments.len()).unwrap_or(0);

    let mut lines = vec![
        Line::from(Span::styled(
            format!("[Gate 1 & Gate 2: 업무 완료 무결성 검증]"),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("대상 업무: {}", title)),
        Line::from(vec![
            Span::raw("보고필수: "),
            Span::styled(
                if req_report { "예 (REQ-15가)" } else { "아니오" },
                if req_report {
                    Style::default().fg(Color::LightRed)
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
            Span::raw(" | 사진필수: "),
            Span::styled(
                if req_photo { "예 (REQ-15다)" } else { "아니오" },
                if req_photo {
                    Style::default().fg(Color::LightRed)
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
            Span::raw(format!(" | 현재 증빙: {}건", photo_count)),
        ]),
        Line::from(""),
    ];

    if req_report {
        lines.push(Line::from(vec![
            Span::styled(
                "1. 실적 건수 (1 이상 필수): ",
                if active_field == 0 {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            ),
            Span::styled(
                format!(" [ {} ]", count_input),
                if active_field == 0 {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                },
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "2. 결과 요약 텍스트 (필수 입력): ",
                if active_field == 1 {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            ),
            Span::styled(
                format!(" [ {}_ ]", summary_input),
                if active_field == 1 {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                },
            ),
        ]));
        lines.push(Line::from(""));
    }

    if req_photo && photo_count == 0 {
        lines.push(Line::from(Span::styled(
            "⚠️ 사진 보고 필수 업무입니다! 현장 증빙 사진이 1장 이상 등록되어야 합니다.",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  -> 팁: 모달 밖에서 [a] 키를 누르면 사진 증빙이 즉시 첨부됩니다.",
            Style::default().fg(Color::Yellow),
        )));
    }

    if let Some(err) = error_msg {
        lines.push(Line::from(Span::styled(
            err,
            Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("[Tab] ", Style::default().fg(Color::Cyan)),
        Span::raw("입력칸 전환  |  "),
        Span::styled("[Enter] ", Style::default().fg(Color::Green)),
        Span::raw("완료 검증 실행  |  "),
        Span::styled("[Esc] ", Style::default().fg(Color::Red)),
        Span::raw("취소"),
    ]));

    let block = Block::default()
        .title(" [ 업무 완료 처리 (Validation Gate) ] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

fn render_task_detail_modal(frame: &mut Frame, app: &App, task_id: &str, area: Rect) {
    frame.render_widget(Clear, area);

    let task = match app.tasks.iter().find(|t| t.task_id == task_id) {
        Some(t) => t,
        None => return,
    };

    let mut lines = vec![
        Line::from(Span::styled(
            format!("[{}] {}", task.task_id, task.title),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "구분: {} | 출처: {:?} | 상태: {}",
            task.calendar_layer.label(),
            task.task_origin,
            task.status.label()
        )),
        Line::from(format!(
            "마스터 기준일: {} | 실제 실행일: {} | 세션: {}",
            task.baseline_date,
            task.execution_date,
            task.assigned_session.label()
        )),
        Line::from(""),
    ];

    if let Some(tag) = &task.visual_dday_tag {
        lines.push(Line::from(vec![
            Span::raw("D-Day 비주얼 태그: "),
            Span::styled(tag, Style::default().fg(Color::Magenta)),
        ]));
    }

    if let Some(desc) = &task.description {
        lines.push(Line::from(format!("상세 설명: {}", desc)));
    }

    if let Some(notes) = &task.special_notes {
        lines.push(Line::from(vec![
            Span::styled("현장 특이사항: ", Style::default().fg(Color::LightRed)),
            Span::raw(notes),
        ]));
    }

    // 위치 & 날씨
    if let (Some(place), Some(addr)) = (&task.place_name, &task.address) {
        lines.push(Line::from(""));
        lines.push(Line::from(format!("수행 장소: {} ({})", place, addr)));
        if let (Some(lat), Some(lon)) = (task.latitude, task.longitude) {
            lines.push(Line::from(format!("위경도 좌표: ({:.4}, {:.4}) | 이동순서: {}", lat, lon, task.sequence_in_subject)));
        }
    }

    if let Some(weather) = &task.weather_snapshot {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!(
                "기상 안내: {} ({:.1}°C, 강수확률 {}%)",
                weather.weather_sky_code, weather.temperature_celsius, weather.precipitation_prob_percent
            ),
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(format!("* 복장: {}", weather.clothing_advice)));
        lines.push(Line::from(format!("* 우산: {}", weather.umbrella_alert)));
    }

    // 체크리스트
    if !task.custom_checkboxes.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "동적 점검 체크리스트 [REQ-15나]:",
            Style::default().fg(Color::Yellow),
        )));
        for item in &task.custom_checkboxes {
            let mark = if item.is_checked { "[x]" } else { "[ ]" };
            lines.push(Line::from(format!("  {} {}", mark, item.label)));
        }
    }

    // 첨부 목록
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("증빙 첨부 파일 (총 {}건):", task.attachments.len()),
        Style::default().fg(Color::Yellow),
    )));
    if task.attachments.is_empty() {
        lines.push(Line::from("  (첨부된 파일이 없습니다)"));
    } else {
        for att in &task.attachments {
            lines.push(Line::from(format!(
                "  - [{}] {} ({}) - {}",
                att.source_channel,
                att.file_name,
                att.attachment_type,
                att.attached_at.format("%Y-%m-%d %H:%M:%S")
            )));
        }
    }

    // 초 단위 감사 로그
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "타임스탬프 감사 정보: 최초 확정 {} | 수정 {}",
            task.scheduled_fixed_at.format("%Y-%m-%d %H:%M:%S"),
            task.last_modified_at.format("%Y-%m-%d %H:%M:%S")
        ),
        Style::default().fg(Color::DarkGray),
    )));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "닫으려면 [Esc] 또는 [Enter] 키를 누르세요.",
        Style::default().fg(Color::Cyan),
    )));

    let block = Block::default()
        .title(" [ 업무 캡슐 상세 정보 (TaskCapsule Inspector) ] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

fn render_report_modal(
    frame: &mut Frame,
    content: &str,
    saved_path: Option<&str>,
    scroll: usize,
    area: Rect,
) {
    frame.render_widget(Clear, area);

    let mut lines = Vec::new();
    if let Some(path) = saved_path {
        lines.push(Line::from(Span::styled(
            format!("💾 A4 보고서가 파일로 저장되었습니다: {}", path),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
    }

    for l in content.lines() {
        lines.push(Line::from(l));
    }

    let block = Block::default()
        .title(" [ A4 일일 업무보고서 미리보기 (REQ-13) - 스크롤: ↑/↓, 닫기: Esc ] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let p = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));
    frame.render_widget(p, area);
}

fn render_add_attachment_modal(
    frame: &mut Frame,
    app: &App,
    task_id: &str,
    file_name_input: &str,
    area: Rect,
) {
    frame.render_widget(Clear, area);

    let task = app.tasks.iter().find(|t| t.task_id == task_id);
    let title = task.map(|t| t.title.as_str()).unwrap_or("업무");
    let exec_date = task
        .map(|t| t.execution_date.format("%Y/%m/%d").to_string())
        .unwrap_or_else(|| "2026/10/25".to_string());

    let lines = vec![
        Line::from(Span::styled(
            "[REQ-05 & REQ-15다: 현장 사진 / 로컬 파일시스템 증빙 첨부]",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("대상 업무: [{}] {}", task_id, title)),
        Line::from(format!(
            "자동 적재 스토리지 경로: storage_root/{}/{}/",
            exec_date, task_id
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "첨부할 파일 경로 입력: ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" [ {}_ ] ", file_name_input),
                Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "* 로컬 파일이 존재하면 실제 파일을 스토리지 디렉터리로 복사하고 SHA-256 해시를 검증합니다.",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "* 파일이 아직 없으면 현장 메타데이터가 포함된 유효한 JPEG(JFIF) 이미지 파일을 생성해 적재합니다.",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("파일 적재 및 첨부 확정  |  "),
            Span::styled("[Esc] ", Style::default().fg(Color::Red)),
            Span::raw("취소"),
        ]),
    ];

    let block = Block::default()
        .title(" [ 현장 사진 및 파일시스템 증빙 입력 (REQ-05) ] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

fn render_message_modal(frame: &mut Frame, msg: &str, area: Rect) {
    frame.render_widget(Clear, area);

    let text = vec![
        Line::from(msg),
        Line::from(""),
        Line::from(Span::styled(
            "확인하려면 [Enter] 또는 [Esc]를 누르세요.",
            Style::default().fg(Color::Cyan),
        )),
    ];

    let block = Block::default()
        .title(" [ 알림 ] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let p = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);
    frame.render_widget(p, area);
}
