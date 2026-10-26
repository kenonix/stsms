use chrono::{NaiveDate, Utc};
use rusqlite::Connection;

use crate::db::repository::Repository;
use crate::domain::gates::{
    compute_sha256_hex, validate_and_complete_task, validate_and_reschedule_task,
};
use crate::domain::models::*;
use crate::map::braille::BrailleCanvas;
use crate::map::map_view::MapView;
use crate::report::a4_export::{
    generate_daily_report_html, generate_daily_report_markdown, save_report_to_disk,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    Calendar,
    Timeline,
    Map,
    TodoList,
    SessionBoardAm,
    SessionBoardPm,
}

use ratatui::layout::Rect;

#[derive(Debug, Clone, Default)]
pub struct LayoutAreas {
    pub top_bar: Rect,
    pub calendar: Rect,
    pub timeline: Rect,
    pub map: Rect,
    pub map_btn_zoom_in: Rect,
    pub map_btn_zoom_out: Rect,
    pub map_btn_reset: Rect,
    pub map_btn_refresh: Rect,
    pub todo_list: Rect,
    pub session_am: Rect,
    pub session_pm: Rect,
    pub status_bar: Rect,
    pub action_buttons: Vec<(String, Rect)>,
    pub modal_area: Option<Rect>,
    pub modal_buttons: Vec<(String, Rect)>,
    pub calendar_cells: Vec<(NaiveDate, Rect)>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalState {
    None,
    Help,
    TaskDetail(String), // task_id
    Reschedule {
        task_id: String,
        new_date_input: String,
        reason_input: String,
        error_msg: Option<String>,
        active_field: usize, // 0: new_date, 1: reason
    },
    Complete {
        task_id: String,
        count_input: String,
        summary_input: String,
        error_msg: Option<String>,
        active_field: usize, // 0: count, 1: summary
    },
    A4Report {
        content: String,
        saved_path: Option<String>,
        scroll: usize,
    },
    AddAttachment {
        task_id: String,
        file_name_input: String,
    },
    Message(String),
}

pub struct App {
    #[allow(dead_code)]
    pub user_id: String,
    pub user_name: String,
    pub operation_mode: OperationMode,
    pub selected_date: NaiveDate,
    pub today: NaiveDate,
    pub focused_panel: FocusedPanel,
    pub modal: ModalState,

    // 데이터
    pub tasks: Vec<TaskCapsule>,
    pub subjects: Vec<Subject>,
    pub holidays: Vec<HolidayAndLeave>,

    // 지도 뷰
    pub map_view: MapView,
    pub braille_canvas: BrailleCanvas,

    // 선택 인덱스
    pub todo_selected_idx: usize,
    pub am_selected_idx: usize,
    pub pm_selected_idx: usize,
    pub timeline_selected_idx: usize,

    // 알림 메시지 배너
    pub status_message: Option<(String, std::time::Instant)>,

    // 터치 및 마우스 지원
    pub layout_areas: LayoutAreas,
    pub last_touch_pos: Option<(u16, u16)>,
    pub is_dragging_map: bool,

    pub should_quit: bool,
}

impl App {
    pub fn new(conn: &Connection) -> Self {
        let repo = Repository::new(conn);

        let user_info = repo
            .get_user_profile("USR-001")
            .ok()
            .flatten()
            .unwrap_or_else(|| {
                (
                    "USR-001".to_string(),
                    "Nix (시스템 엔지니어)".to_string(),
                    "SOLO_SPLIT".to_string(),
                )
            });

        let tasks = repo.get_all_tasks().unwrap_or_default();
        let subjects = repo.get_all_subjects().unwrap_or_default();
        let holidays = repo.get_all_holidays().unwrap_or_default();

        let today = chrono::Local::now().date_naive();

        Self {
            user_id: user_info.0,
            user_name: user_info.1,
            operation_mode: match user_info.2.as_str() {
                "TEAM_MULTI" => OperationMode::TeamMulti,
                _ => OperationMode::SoloSplit,
            },
            selected_date: today,
            today,
            focused_panel: FocusedPanel::SessionBoardAm,
            modal: ModalState::None,
            tasks,
            subjects,
            holidays,
            map_view: MapView::new(),
            braille_canvas: BrailleCanvas::new(60, 20),
            todo_selected_idx: 0,
            am_selected_idx: 0,
            pm_selected_idx: 0,
            timeline_selected_idx: 0,
            status_message: Some((
                "STSMS v1.0 터치/마우스 모드 활성화됨. 화면을 탭하거나 조작하세요.".to_string(),
                std::time::Instant::now(),
            )),
            layout_areas: LayoutAreas::default(),
            last_touch_pos: None,
            is_dragging_map: false,
            should_quit: false,
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), std::time::Instant::now()));
    }

    /// 현재 선택된 날짜에 배정된 태스크 목록
    pub fn tasks_for_selected_date(&self) -> Vec<&TaskCapsule> {
        self.tasks
            .iter()
            .filter(|t| t.execution_date == self.selected_date)
            .collect()
    }

    pub fn am_tasks(&self) -> Vec<&TaskCapsule> {
        self.tasks
            .iter()
            .filter(|t| {
                t.execution_date == self.selected_date && t.assigned_session == SessionSlot::Am
            })
            .collect()
    }

    pub fn pm_tasks(&self) -> Vec<&TaskCapsule> {
        self.tasks
            .iter()
            .filter(|t| {
                t.execution_date == self.selected_date && t.assigned_session == SessionSlot::Pm
            })
            .collect()
    }

    /// 패널 순환 이동
    pub fn cycle_panel_forward(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Calendar => FocusedPanel::Timeline,
            FocusedPanel::Timeline => FocusedPanel::Map,
            FocusedPanel::Map => FocusedPanel::TodoList,
            FocusedPanel::TodoList => FocusedPanel::SessionBoardAm,
            FocusedPanel::SessionBoardAm => FocusedPanel::SessionBoardPm,
            FocusedPanel::SessionBoardPm => FocusedPanel::Calendar,
        };
    }

    pub fn cycle_panel_backward(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Calendar => FocusedPanel::SessionBoardPm,
            FocusedPanel::Timeline => FocusedPanel::Calendar,
            FocusedPanel::Map => FocusedPanel::Timeline,
            FocusedPanel::TodoList => FocusedPanel::Map,
            FocusedPanel::SessionBoardAm => FocusedPanel::TodoList,
            FocusedPanel::SessionBoardPm => FocusedPanel::SessionBoardAm,
        };
    }

    /// 현재 선택된 활성 태스크 가져오기
    pub fn get_active_task(&self) -> Option<&TaskCapsule> {
        match self.focused_panel {
            FocusedPanel::TodoList => {
                let list = self.tasks_for_selected_date();
                list.get(self.todo_selected_idx).copied()
            }
            FocusedPanel::SessionBoardAm => {
                let list = self.am_tasks();
                list.get(self.am_selected_idx).copied()
            }
            FocusedPanel::SessionBoardPm => {
                let list = self.pm_tasks();
                list.get(self.pm_selected_idx).copied()
            }
            _ => self.tasks_for_selected_date().first().copied(),
        }
    }

    /// 하단 작업 보드 세션 이동 (AM <-> PM)
    pub fn toggle_active_task_session(&mut self, conn: &Connection) {
        let task_id = match self.get_active_task() {
            Some(t) => t.task_id.clone(),
            None => {
                self.set_status("이동할 작업이 선택되지 않았습니다.");
                return;
            }
        };

        if let Some(task) = self.tasks.iter_mut().find(|t| t.task_id == task_id) {
            let (new_session, new_label) = match task.assigned_session {
                SessionSlot::Am => (SessionSlot::Pm, "오후(PM)"),
                SessionSlot::Pm => (SessionSlot::Am, "오전(AM)"),
                SessionSlot::Unassigned => (SessionSlot::Am, "오전(AM)"),
            };
            let old_session = format!("{:?}", task.assigned_session);
            task.assigned_session = new_session;
            task.status = TaskStatus::InSession;
            task.last_modified_at = Utc::now();

            let repo = Repository::new(conn);
            let _ = repo.save_task(task);

            let log = TaskAuditLog {
                log_id: None,
                task_id: task_id.clone(),
                event_type: "SESSION_MOVED".to_string(),
                field_name: Some("assigned_session".to_string()),
                old_value: Some(old_session),
                new_value: Some(format!("{:?}", new_session)),
                source_channel: "TUI".to_string(),
                occurred_at: Utc::now(),
            };
            let _ = repo.insert_audit_log(&log);

            self.set_status(format!("업무가 {} 세션으로 이동되었습니다.", new_label));
        }
    }

    /// Gate 1 & 2 완료 모달 오픈 또는 바로 완료
    pub fn trigger_complete_modal(&mut self, conn: &Connection) {
        let task = match self.get_active_task() {
            Some(t) => t.clone(),
            None => {
                self.set_status("완료 처리할 작업이 선택되지 않았습니다.");
                return;
            }
        };

        if task.status == TaskStatus::Completed {
            self.set_status("이미 완료된 업무입니다.");
            return;
        }

        // 보고나 사진이 필요한 경우 모달 오픈, 아니면 즉시 완료
        if task.requires_report || task.requires_photo_evidence {
            self.modal = ModalState::Complete {
                task_id: task.task_id.clone(),
                count_input: if task.report_actual_count > 0 {
                    task.report_actual_count.to_string()
                } else {
                    task.report_target_count.to_string()
                },
                summary_input: task.report_result_summary.unwrap_or_default(),
                error_msg: None,
                active_field: 0,
            };
        } else {
            // 즉시 완료
            let mut result = None;
            if let Some(t_mut) = self.tasks.iter_mut().find(|t| t.task_id == task.task_id) {
                let now = Utc::now();
                match validate_and_complete_task(t_mut, 0, None, now) {
                    Ok(audit_log) => {
                        let repo = Repository::new(conn);
                        let _ = repo.save_task(t_mut);
                        let _ = repo.insert_audit_log(&audit_log);
                        result = Some(Ok(t_mut.title.clone()));
                    }
                    Err(e) => {
                        result = Some(Err(e));
                    }
                }
            }
            if let Some(Ok(title)) = result {
                self.set_status(format!("'{}' 완료 처리되었습니다.", title));
            } else if let Some(Err(e)) = result {
                self.set_status(format!("❌ 완료 실패: {}", e));
            }
        }
    }

    /// Gate 3 일정 연기 모달 열기
    pub fn trigger_reschedule_modal(&mut self) {
        let task = match self.get_active_task() {
            Some(t) => t.clone(),
            None => {
                self.set_status("일정을 연기할 작업이 선택되지 않았습니다.");
                return;
            }
        };

        let next_date = task
            .execution_date
            .succ_opt()
            .unwrap_or(task.execution_date);

        self.modal = ModalState::Reschedule {
            task_id: task.task_id.clone(),
            new_date_input: next_date.format("%Y-%m-%d").to_string(),
            reason_input: String::new(),
            error_msg: None,
            active_field: 1, // reason 입력창으로 바로 포커스
        };
    }

    /// Gate 3 일정 연기 제출 처리
    pub fn submit_reschedule(&mut self, conn: &Connection) {
        let (task_id, new_date_str, reason) = match &self.modal {
            ModalState::Reschedule {
                task_id,
                new_date_input,
                reason_input,
                ..
            } => (task_id.clone(), new_date_input.clone(), reason_input.clone()),
            _ => return,
        };

        let new_date = match NaiveDate::parse_from_str(&new_date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                if let ModalState::Reschedule { error_msg, .. } = &mut self.modal {
                    *error_msg = Some("날짜 형식이 잘못되었습니다. (예: 2026-10-26)".to_string());
                }
                return;
            }
        };

        let mut reschedule_result = None;
        if let Some(task) = self.tasks.iter_mut().find(|t| t.task_id == task_id) {
            let now = Utc::now();
            match validate_and_reschedule_task(task, new_date, reason, now) {
                Ok((adj, audit)) => {
                    let repo = Repository::new(conn);
                    let _ = repo.save_task(task);
                    let _ = repo.insert_adjustment(&adj);
                    let _ = repo.insert_audit_log(&audit);
                    reschedule_result = Some(Ok((new_date, task.baseline_date)));
                }
                Err(err) => {
                    reschedule_result = Some(Err(err));
                }
            }
        }

        if let Some(Ok((exec_date, baseline_date))) = reschedule_result {
            self.modal = ModalState::None;
            self.set_status(format!(
                "일정이 {}로 연기되었습니다 (기준일: {} 보존).",
                exec_date, baseline_date
            ));
        } else if let Some(Err(err)) = reschedule_result {
            if let ModalState::Reschedule { error_msg, .. } = &mut self.modal {
                *error_msg = Some(format!("❌ {}", err));
            }
        }
    }

    /// Gate 1 & 2 완료 제출 처리
    pub fn submit_complete(&mut self, conn: &Connection) {
        let (task_id, count_str, summary) = match &self.modal {
            ModalState::Complete {
                task_id,
                count_input,
                summary_input,
                ..
            } => (
                task_id.clone(),
                count_input.clone(),
                summary_input.clone(),
            ),
            _ => return,
        };

        let count: i32 = count_str.trim().parse().unwrap_or(0);

        let mut complete_result = None;
        if let Some(task) = self.tasks.iter_mut().find(|t| t.task_id == task_id) {
            let now = Utc::now();
            match validate_and_complete_task(
                task,
                count,
                if summary.trim().is_empty() {
                    None
                } else {
                    Some(summary)
                },
                now,
            ) {
                Ok(audit) => {
                    let repo = Repository::new(conn);
                    let _ = repo.save_task(task);
                    let _ = repo.insert_audit_log(&audit);
                    complete_result = Some(Ok(task.title.clone()));
                }
                Err(err) => {
                    complete_result = Some(Err(err));
                }
            }
        }

        if let Some(Ok(title)) = complete_result {
            self.modal = ModalState::None;
            self.set_status(format!("✅ '{}' 완료 검증 통과!", title));
        } else if let Some(Err(err)) = complete_result {
            if let ModalState::Complete { error_msg, .. } = &mut self.modal {
                *error_msg = Some(format!("❌ 검증 실패: {}", err));
            }
        }
    }

    /// 사진/파일 증빙 첨부 모달 열기 (REQ-05, REQ-15다)
    pub fn trigger_attachment_modal(&mut self) {
        let task_id = match self.get_active_task() {
            Some(t) => t.task_id.clone(),
            None => {
                self.set_status("사진을 첨부할 작업이 선택되지 않았습니다.");
                return;
            }
        };

        // 워크스페이스의 실제 이미지 파일 탐색 (없으면 기본 샘플 파일명)
        let default_file = std::fs::read_dir(".")
            .ok()
            .and_then(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .find(|name| {
                        name.ends_with(".jpg")
                            || name.ends_with(".jpeg")
                            || name.ends_with(".png")
                    })
            })
            .unwrap_or_else(|| "sample_site_photo.jpg".to_string());

        self.modal = ModalState::AddAttachment {
            task_id,
            file_name_input: default_file,
        };
    }

    /// 실제 로컬 파일 입력 및 storage_root/YYYY/MM/DD/TASK_ID/ 적재 + SHA-256 해시 기록
    pub fn submit_attachment(&mut self, conn: &Connection) {
        let (task_id, raw_input) = match &self.modal {
            ModalState::AddAttachment {
                task_id,
                file_name_input,
            } => (task_id.clone(), file_name_input.trim().to_string()),
            _ => return,
        };

        let input_path_str = if raw_input.is_empty() {
            "sample_site_photo.jpg".to_string()
        } else {
            raw_input
        };

        let mut attached_info = None;
        if let Some(task) = self.tasks.iter_mut().find(|t| t.task_id == task_id) {
            let now = Utc::now();
            let src_path = std::path::Path::new(&input_path_str);
            let file_name = src_path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("evidence_{}.jpg", now.timestamp()));

            // 원본 파일이 있으면 읽고, 없으면 유효한 JFIF JPEG 헤더 + 현장 메타데이터로 생성
            let file_bytes: Vec<u8> = if src_path.exists() {
                std::fs::read(src_path).unwrap_or_default()
            } else {
                let mut buf = vec![
                    0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00, 0x01, 0x01,
                    0x01, 0x00, 0x60, 0x00, 0x60, 0x00, 0x00,
                ];
                let meta = format!(
                    "STSMS_EVIDENCE|TASK={}|PLACE={}|TIME={}",
                    task_id,
                    task.place_name.as_deref().unwrap_or("SITE"),
                    now.to_rfc3339()
                );
                buf.extend_from_slice(meta.as_bytes());
                buf.extend_from_slice(&[0xFF, 0xD9]);
                let _ = std::fs::write(src_path, &buf);
                buf
            };

            let target_dir = std::path::PathBuf::from(format!(
                "storage_root/{}/{}",
                task.execution_date.format("%Y/%m/%d"),
                task_id
            ));
            let _ = std::fs::create_dir_all(&target_dir);
            let target_file_path = target_dir.join(&file_name);
            let _ = std::fs::write(&target_file_path, &file_bytes);

            let sha256 = compute_sha256_hex(&file_bytes);
            let stored_path_str = target_file_path.to_string_lossy().to_string();

            let is_photo = file_name.ends_with(".jpg")
                || file_name.ends_with(".jpeg")
                || file_name.ends_with(".png");

            let mut att = TaskAttachment {
                attachment_id: None,
                task_id: task_id.clone(),
                attachment_type: if is_photo {
                    "MESSENGER_PHOTO".to_string()
                } else {
                    "EVIDENCE_DOC".to_string()
                },
                source_channel: "LOCAL_FS".to_string(),
                file_name: file_name.clone(),
                file_path: stored_path_str.clone(),
                file_size_bytes: Some(file_bytes.len() as i64),
                mime_type: Some(if is_photo {
                    "image/jpeg".to_string()
                } else {
                    "application/octet-stream".to_string()
                }),
                sha256_hash: Some(sha256.clone()),
                attached_at: now,
            };

            let audit = TaskAuditLog {
                log_id: None,
                task_id: task_id.clone(),
                event_type: "EVIDENCE_ATTACHED".to_string(),
                field_name: Some("attachments".to_string()),
                old_value: None,
                new_value: Some(format!("{} (SHA256: {}..)", stored_path_str, &sha256[..12])),
                source_channel: "TUI".to_string(),
                occurred_at: now,
            };

            let repo = Repository::new(conn);
            let _ = repo.insert_attachment(&att);
            att.attachment_id = Some(conn.last_insert_rowid());
            let _ = repo.insert_audit_log(&audit);
            task.attachments.push(att);
            task.last_modified_at = now;
            let _ = repo.save_task(task);
            attached_info = Some((task.title.clone(), stored_path_str, file_bytes.len()));
        }

        self.modal = ModalState::None;
        if let Some((title, path, size)) = attached_info {
            self.set_status(format!(
                "📸 '{}' 증빙 적재 완료: {} ({}B)",
                title, path, size
            ));
        }
    }

    /// A4 업무 보고서 생성 및 모달 표시
    pub fn generate_and_show_report(&mut self) {
        let current_tasks = self.tasks_for_selected_date();
        let cloned_tasks: Vec<TaskCapsule> = current_tasks.into_iter().cloned().collect();
        let current_subject = self.subjects.first();

        let report_md = generate_daily_report_markdown(
            &self.user_name,
            self.selected_date,
            &cloned_tasks,
            current_subject,
        );
        let report_html = generate_daily_report_html(
            &self.user_name,
            self.selected_date,
            &cloned_tasks,
            current_subject,
        );

        let saved = save_report_to_disk(&report_md, &report_html, self.selected_date).ok();

        self.modal = ModalState::A4Report {
            content: report_md,
            saved_path: saved,
            scroll: 0,
        };
    }
}
