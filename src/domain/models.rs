use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationMode {
    SoloSplit,
    TeamMulti,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalendarLayer {
    Work,
    Personal,
}

impl CalendarLayer {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Work => "업무",
            Self::Personal => "개인",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskOrigin {
    Manual,
    TemplateScheduled,
    AdHoc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Draft,
    Scheduled,
    InSession,
    Completed,
    RescheduledHold,
}

impl TaskStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Draft => "임시(DRAFT)",
            Self::Scheduled => "예정(SCHEDULED)",
            Self::InSession => "진행중(IN_SESSION)",
            Self::Completed => "완료(COMPLETED)",
            Self::RescheduledHold => "연기보류(HOLD)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionSlot {
    Unassigned,
    Am,
    Pm,
}

impl SessionSlot {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Unassigned => "미배정",
            Self::Am => "오전 (AM)",
            Self::Pm => "오후 (PM)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecurrenceType {
    None,
    Daily,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurrenceConfig {
    pub recurrence_type: RecurrenceType,
    pub month: Option<u32>,
    pub day: Option<u32>,
    pub calendar_base: String, // "SOLAR" | "LUNAR"
}

impl Default for RecurrenceConfig {
    fn default() -> Self {
        Self {
            recurrence_type: RecurrenceType::None,
            month: None,
            day: None,
            calendar_base: "SOLAR".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckItem {
    pub checklist_id: Option<i64>,
    pub label: String,
    pub is_checked: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherSnapshot {
    pub temperature_celsius: f64,
    pub precipitation_prob_percent: i32,
    pub weather_sky_code: String,
    pub clothing_advice: String,
    pub umbrella_required: bool,
    pub umbrella_alert: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAttachment {
    pub attachment_id: Option<i64>,
    pub task_id: String,
    pub attachment_type: String, // "EVIDENCE_DOC" | "MESSENGER_PHOTO"
    pub source_channel: String,  // "LOCAL_FS" | "DISCORD" | "KAKAO"
    pub file_name: String,
    pub file_path: String,
    pub file_size_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub sha256_hash: Option<String>,
    pub attached_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCapsule {
    pub task_id: String,
    pub subject_id: Option<String>,
    pub user_id: String,
    pub master_template_id: Option<String>,

    // 기본 정보
    pub title: String,
    pub description: Option<String>,
    pub calendar_layer: CalendarLayer,
    pub task_origin: TaskOrigin,
    pub status: TaskStatus,

    // D-Day 및 일정 제어
    pub visual_dday_tag: Option<String>, // e.g., "[~장비점검] : D-21 = 센서 교정"
    pub baseline_date: NaiveDate,        // 불변 마스터 기준일
    pub execution_date: NaiveDate,       // 실제 수행 예정일
    pub assigned_session: SessionSlot,
    pub session_order: i32,
    pub master_baseline_affected: bool,
    pub recurrence: RecurrenceConfig,

    // 보고 및 특성 [REQ-15가~라]
    pub requires_report: bool,
    pub report_item_name: Option<String>,
    pub report_target_count: i32,
    pub report_actual_count: i32,
    pub report_result_summary: Option<String>,
    pub requires_photo_evidence: bool,
    pub custom_checkboxes: Vec<CheckItem>,
    pub special_notes: Option<String>,

    // 위치 및 동선 메타데이터 [CORE-03]
    pub place_name: Option<String>,
    pub address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub sequence_in_subject: i32,
    pub distance_from_prev_km: f64,
    pub travel_time_from_prev_min: i32,
    pub weather_snapshot: Option<WeatherSnapshot>,

    // 증빙 및 감사 로그 [REQ-05, 08]
    pub attachments: Vec<TaskAttachment>,
    pub scheduled_fixed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub subject_id: String,
    pub user_id: String,
    pub subject_name: String,
    pub color_hex: String,
    pub total_distance_km: f64,
    pub estimated_duration_min: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTemplate {
    pub template_id: String,
    pub user_id: String,
    pub template_name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAdjustment {
    pub adjustment_id: Option<i64>,
    pub task_id: String,
    pub previous_date: NaiveDate,
    pub adjusted_date: NaiveDate,
    pub reason: String,
    pub master_baseline_affected: bool,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAuditLog {
    pub log_id: Option<i64>,
    pub task_id: String,
    pub event_type: String, // CREATED, RESCHEDULED, SESSION_MOVED, COMPLETED, NOTE_ADDED
    pub field_name: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub source_channel: String, // WEB, DISCORD_BOT, KAKAO_BOT, SYSTEM, TUI
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolidayAndLeave {
    pub leave_id: Option<i64>,
    pub user_id: String,
    pub target_date: NaiveDate,
    pub item_type: String, // PUBLIC_HOLIDAY, REGULAR_OFF, ANNUAL_LEAVE, HALF_LEAVE_AM, HALF_LEAVE_PM, SICK_LEAVE
    pub title: String,
    pub is_work_disabled: bool,
}
