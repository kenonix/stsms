use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection, Result};

use crate::domain::models::*;

pub struct Repository<'a> {
    pub conn: &'a Connection,
}

impl<'a> Repository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn get_user_profile(&self, user_id: &str) -> Result<Option<(String, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT user_id, user_name, operation_mode FROM users_profiles WHERE user_id = ?1",
        )?;
        let mut rows = stmt.query(params![user_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?)))
        } else {
            Ok(None)
        }
    }

    pub fn insert_user_profile(
        &self,
        user_id: &str,
        user_name: &str,
        mode: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO users_profiles (user_id, user_name, operation_mode) VALUES (?1, ?2, ?3)",
            params![user_id, user_name, mode],
        )?;
        Ok(())
    }

    pub fn get_all_subjects(&self) -> Result<Vec<Subject>> {
        let mut stmt = self.conn.prepare(
            "SELECT subject_id, user_id, subject_name, color_hex, total_distance_km, estimated_duration_min, created_at, updated_at FROM subjects",
        )?;
        let rows = stmt.query_map([], |row| {
            let created_str: String = row.get(6)?;
            let updated_str: String = row.get(7)?;
            Ok(Subject {
                subject_id: row.get(0)?,
                user_id: row.get(1)?,
                subject_name: row.get(2)?,
                color_hex: row.get(3)?,
                total_distance_km: row.get(4)?,
                estimated_duration_min: row.get(5)?,
                created_at: DateTime::parse_from_rfc3339(&created_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }
        Ok(res)
    }

    pub fn insert_subject(&self, sub: &Subject) -> Result<()> {
        self.conn.execute(
            r#"INSERT OR REPLACE INTO subjects 
               (subject_id, user_id, subject_name, color_hex, total_distance_km, estimated_duration_min, created_at, updated_at) 
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            params![
                sub.subject_id,
                sub.user_id,
                sub.subject_name,
                sub.color_hex,
                sub.total_distance_km,
                sub.estimated_duration_min,
                sub.created_at.to_rfc3339(),
                sub.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_all_holidays(&self) -> Result<Vec<HolidayAndLeave>> {
        let mut stmt = self.conn.prepare(
            "SELECT leave_id, user_id, target_date, item_type, title, is_work_disabled FROM holidays_and_leaves",
        )?;
        let rows = stmt.query_map([], |row| {
            let date_str: String = row.get(2)?;
            let target_date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
            let disabled: i32 = row.get(5)?;
            Ok(HolidayAndLeave {
                leave_id: Some(row.get(0)?),
                user_id: row.get(1)?,
                target_date,
                item_type: row.get(3)?,
                title: row.get(4)?,
                is_work_disabled: disabled != 0,
            })
        })?;

        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }
        Ok(res)
    }

    pub fn insert_holiday(&self, holiday: &HolidayAndLeave) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO holidays_and_leaves 
               (user_id, target_date, item_type, title, is_work_disabled) 
               VALUES (?1, ?2, ?3, ?4, ?5)"#,
            params![
                holiday.user_id,
                holiday.target_date.format("%Y-%m-%d").to_string(),
                holiday.item_type,
                holiday.title,
                if holiday.is_work_disabled { 1 } else { 0 },
            ],
        )?;
        Ok(())
    }

    pub fn get_all_tasks(&self) -> Result<Vec<TaskCapsule>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT 
                task_id, subject_id, user_id, master_template_id, title, description,
                calendar_layer, task_origin, status, visual_dday_tag, baseline_date,
                execution_date, assigned_session, session_order, master_baseline_affected,
                recurrence_type, recurrence_config, requires_report, report_item_name,
                report_target_count, report_actual_count, report_result_summary,
                requires_photo_evidence, special_notes, place_name, address,
                latitude, longitude, sequence_in_subject, distance_from_prev_km,
                travel_time_from_prev_min, weather_data, scheduled_fixed_at,
                created_at, last_modified_at
            FROM task_capsules ORDER BY execution_date ASC, session_order ASC"#,
        )?;

        let task_rows = stmt.query_map([], |row| {
            let task_id: String = row.get(0)?;
            let cal_layer_str: String = row.get(6)?;
            let calendar_layer = match cal_layer_str.as_str() {
                "PERSONAL" => CalendarLayer::Personal,
                _ => CalendarLayer::Work,
            };

            let origin_str: String = row.get(7)?;
            let task_origin = match origin_str.as_str() {
                "TEMPLATE_SCHEDULED" => TaskOrigin::TemplateScheduled,
                "AD_HOC" => TaskOrigin::AdHoc,
                _ => TaskOrigin::Manual,
            };

            let status_str: String = row.get(8)?;
            let status = match status_str.as_str() {
                "DRAFT" => TaskStatus::Draft,
                "IN_SESSION" => TaskStatus::InSession,
                "COMPLETED" => TaskStatus::Completed,
                "RESCHEDULED_HOLD" => TaskStatus::RescheduledHold,
                _ => TaskStatus::Scheduled,
            };

            let baseline_str: String = row.get(10)?;
            let baseline_date = NaiveDate::parse_from_str(&baseline_str, "%Y-%m-%d")
                .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());

            let exec_str: String = row.get(11)?;
            let execution_date = NaiveDate::parse_from_str(&exec_str, "%Y-%m-%d")
                .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());

            let session_str: String = row.get(12)?;
            let assigned_session = match session_str.as_str() {
                "AM" => SessionSlot::Am,
                "PM" => SessionSlot::Pm,
                _ => SessionSlot::Unassigned,
            };

            let baseline_affected: i32 = row.get(14)?;
            let req_report: i32 = row.get(17)?;
            let req_photo: i32 = row.get(22)?;

            let weather_json: Option<String> = row.get(31)?;
            let weather_snapshot = weather_json.and_then(|w| serde_json::from_str(&w).ok());

            let fixed_str: String = row.get(32)?;
            let created_str: String = row.get(33)?;
            let modified_str: String = row.get(34)?;

            let scheduled_fixed_at = DateTime::parse_from_rfc3339(&fixed_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let last_modified_at = DateTime::parse_from_rfc3339(&modified_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(TaskCapsule {
                task_id,
                subject_id: row.get(1)?,
                user_id: row.get(2)?,
                master_template_id: row.get(3)?,
                title: row.get(4)?,
                description: row.get(5)?,
                calendar_layer,
                task_origin,
                status,
                visual_dday_tag: row.get(9)?,
                baseline_date,
                execution_date,
                assigned_session,
                session_order: row.get(13)?,
                master_baseline_affected: baseline_affected != 0,
                recurrence: RecurrenceConfig::default(),
                requires_report: req_report != 0,
                report_item_name: row.get(18)?,
                report_target_count: row.get(19)?,
                report_actual_count: row.get(20)?,
                report_result_summary: row.get(21)?,
                requires_photo_evidence: req_photo != 0,
                custom_checkboxes: Vec::new(),
                special_notes: row.get(23)?,
                place_name: row.get(24)?,
                address: row.get(25)?,
                latitude: row.get(26)?,
                longitude: row.get(27)?,
                sequence_in_subject: row.get(28)?,
                distance_from_prev_km: row.get(29)?,
                travel_time_from_prev_min: row.get(30)?,
                weather_snapshot,
                attachments: Vec::new(),
                scheduled_fixed_at,
                created_at,
                last_modified_at,
            })
        })?;

        let mut tasks = Vec::new();
        for t in task_rows {
            let mut task = t?;
            // Attachments load
            task.attachments = self.get_attachments_for_task(&task.task_id)?;
            task.custom_checkboxes = self.get_checklists_for_task(&task.task_id)?;
            tasks.push(task);
        }

        Ok(tasks)
    }

    pub fn get_checklists_for_task(&self, task_id: &str) -> Result<Vec<CheckItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT checklist_id, label, is_checked, sort_order FROM task_checklists WHERE task_id = ?1 ORDER BY sort_order ASC",
        )?;
        let rows = stmt.query_map(params![task_id], |row| {
            let is_checked: i32 = row.get(2)?;
            Ok(CheckItem {
                checklist_id: Some(row.get(0)?),
                label: row.get(1)?,
                is_checked: is_checked != 0,
                sort_order: row.get(3)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_attachments_for_task(&self, task_id: &str) -> Result<Vec<TaskAttachment>> {
        let mut stmt = self.conn.prepare(
            "SELECT attachment_id, task_id, attachment_type, source_channel, file_name, file_path, file_size_bytes, mime_type, sha256_hash, attached_at FROM task_attachments WHERE task_id = ?1",
        )?;
        let rows = stmt.query_map(params![task_id], |row| {
            let attached_str: String = row.get(9)?;
            let attached_at = DateTime::parse_from_rfc3339(&attached_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            Ok(TaskAttachment {
                attachment_id: Some(row.get(0)?),
                task_id: row.get(1)?,
                attachment_type: row.get(2)?,
                source_channel: row.get(3)?,
                file_name: row.get(4)?,
                file_path: row.get(5)?,
                file_size_bytes: row.get(6)?,
                mime_type: row.get(7)?,
                sha256_hash: row.get(8)?,
                attached_at,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn save_task(&self, task: &TaskCapsule) -> Result<()> {
        let weather_json = task
            .weather_snapshot
            .as_ref()
            .and_then(|w| serde_json::to_string(w).ok());

        self.conn.execute(
            r#"INSERT INTO task_capsules (
                task_id, subject_id, user_id, master_template_id, title, description,
                calendar_layer, task_origin, status, visual_dday_tag, baseline_date,
                execution_date, assigned_session, session_order, master_baseline_affected,
                recurrence_type, recurrence_config, requires_report, report_item_name,
                report_target_count, report_actual_count, report_result_summary,
                requires_photo_evidence, special_notes, place_name, address,
                latitude, longitude, sequence_in_subject, distance_from_prev_km,
                travel_time_from_prev_min, weather_data, scheduled_fixed_at,
                created_at, last_modified_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28,
                ?29, ?30, ?31, ?32, ?33, ?34, ?35
            )
            ON CONFLICT(task_id) DO UPDATE SET
                status = excluded.status,
                execution_date = excluded.execution_date,
                assigned_session = excluded.assigned_session,
                session_order = excluded.session_order,
                master_baseline_affected = excluded.master_baseline_affected,
                report_actual_count = excluded.report_actual_count,
                report_result_summary = excluded.report_result_summary,
                special_notes = excluded.special_notes,
                weather_data = excluded.weather_data,
                last_modified_at = excluded.last_modified_at"#,
            params![
                task.task_id,
                task.subject_id,
                task.user_id,
                task.master_template_id,
                task.title,
                task.description,
                match task.calendar_layer {
                    CalendarLayer::Work => "WORK",
                    CalendarLayer::Personal => "PERSONAL",
                },
                match task.task_origin {
                    TaskOrigin::Manual => "MANUAL",
                    TaskOrigin::TemplateScheduled => "TEMPLATE_SCHEDULED",
                    TaskOrigin::AdHoc => "AD_HOC",
                },
                match task.status {
                    TaskStatus::Draft => "DRAFT",
                    TaskStatus::Scheduled => "SCHEDULED",
                    TaskStatus::InSession => "IN_SESSION",
                    TaskStatus::Completed => "COMPLETED",
                    TaskStatus::RescheduledHold => "RESCHEDULED_HOLD",
                },
                task.visual_dday_tag,
                task.baseline_date.format("%Y-%m-%d").to_string(),
                task.execution_date.format("%Y-%m-%d").to_string(),
                match task.assigned_session {
                    SessionSlot::Am => "AM",
                    SessionSlot::Pm => "PM",
                    SessionSlot::Unassigned => "UNASSIGNED",
                },
                task.session_order,
                if task.master_baseline_affected { 1 } else { 0 },
                "NONE",
                None::<String>,
                if task.requires_report { 1 } else { 0 },
                task.report_item_name,
                task.report_target_count,
                task.report_actual_count,
                task.report_result_summary,
                if task.requires_photo_evidence { 1 } else { 0 },
                task.special_notes,
                task.place_name,
                task.address,
                task.latitude,
                task.longitude,
                task.sequence_in_subject,
                task.distance_from_prev_km,
                task.travel_time_from_prev_min,
                weather_json,
                task.scheduled_fixed_at.to_rfc3339(),
                task.created_at.to_rfc3339(),
                task.last_modified_at.to_rfc3339(),
            ],
        )?;

        // Update attachments (신규 첨부만 등록)
        for att in &task.attachments {
            if att.attachment_id.is_none() {
                self.insert_attachment(att)?;
            }
        }

        Ok(())
    }

    pub fn insert_adjustment(&self, adj: &TaskAdjustment) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO task_adjustments (
                task_id, previous_date, adjusted_date, reason, master_baseline_affected, recorded_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
            params![
                adj.task_id,
                adj.previous_date.format("%Y-%m-%d").to_string(),
                adj.adjusted_date.format("%Y-%m-%d").to_string(),
                adj.reason,
                if adj.master_baseline_affected { 1 } else { 0 },
                adj.recorded_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn insert_audit_log(&self, log: &TaskAuditLog) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO task_audit_logs (
                task_id, event_type, field_name, old_value, new_value, source_channel, occurred_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![
                log.task_id,
                log.event_type,
                log.field_name,
                log.old_value,
                log.new_value,
                log.source_channel,
                log.occurred_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_audit_logs_for_task(&self, task_id: &str) -> Result<Vec<TaskAuditLog>> {
        let mut stmt = self.conn.prepare(
            "SELECT log_id, task_id, event_type, field_name, old_value, new_value, source_channel, occurred_at FROM task_audit_logs WHERE task_id = ?1 ORDER BY log_id DESC",
        )?;
        let rows = stmt.query_map(params![task_id], |row| {
            let occ_str: String = row.get(7)?;
            let occurred_at = DateTime::parse_from_rfc3339(&occ_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            Ok(TaskAuditLog {
                log_id: Some(row.get(0)?),
                task_id: row.get(1)?,
                event_type: row.get(2)?,
                field_name: row.get(3)?,
                old_value: row.get(4)?,
                new_value: row.get(5)?,
                source_channel: row.get(6)?,
                occurred_at,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn insert_attachment(&self, att: &TaskAttachment) -> Result<()> {
        self.conn.execute(
            r#"INSERT OR REPLACE INTO task_attachments (
                task_id, attachment_type, source_channel, file_name, file_path, file_size_bytes, mime_type, sha256_hash, attached_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
            params![
                att.task_id,
                att.attachment_type,
                att.source_channel,
                att.file_name,
                att.file_path,
                att.file_size_bytes,
                att.mime_type,
                att.sha256_hash,
                att.attached_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }
}
