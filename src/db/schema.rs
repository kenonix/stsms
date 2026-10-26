use rusqlite::{Connection, Result};

pub fn init_database(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS users_profiles (
            user_id TEXT PRIMARY KEY,
            user_name TEXT NOT NULL,
            operation_mode TEXT DEFAULT 'SOLO_SPLIT',
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS subjects (
            subject_id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            subject_name TEXT NOT NULL,
            color_hex TEXT DEFAULT '#3B82F6',
            total_distance_km REAL DEFAULT 0.0,
            estimated_duration_min INTEGER DEFAULT 0,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users_profiles(user_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS schedule_templates (
            template_id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            template_name TEXT NOT NULL,
            description TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users_profiles(user_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS task_capsules (
            task_id TEXT PRIMARY KEY,
            subject_id TEXT,
            user_id TEXT NOT NULL,
            master_template_id TEXT,
            title TEXT NOT NULL,
            description TEXT,
            calendar_layer TEXT DEFAULT 'WORK',
            task_origin TEXT DEFAULT 'MANUAL',
            status TEXT DEFAULT 'SCHEDULED',
            visual_dday_tag TEXT,
            baseline_date TEXT NOT NULL,
            execution_date TEXT NOT NULL,
            assigned_session TEXT DEFAULT 'UNASSIGNED',
            session_order INTEGER DEFAULT 0,
            master_baseline_affected INTEGER DEFAULT 0,
            recurrence_type TEXT DEFAULT 'NONE',
            recurrence_config TEXT,
            requires_report INTEGER DEFAULT 0,
            report_item_name TEXT,
            report_target_count INTEGER DEFAULT 0,
            report_actual_count INTEGER DEFAULT 0,
            report_result_summary TEXT,
            requires_photo_evidence INTEGER DEFAULT 0,
            special_notes TEXT,
            place_name TEXT,
            address TEXT,
            latitude REAL,
            longitude REAL,
            sequence_in_subject INTEGER DEFAULT 0,
            distance_from_prev_km REAL DEFAULT 0.0,
            travel_time_from_prev_min INTEGER DEFAULT 0,
            weather_data TEXT,
            scheduled_fixed_at TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            last_modified_at TEXT DEFAULT CURRENT_TIMESTAMP,
            optional_extensions TEXT,
            FOREIGN KEY (user_id) REFERENCES users_profiles(user_id) ON DELETE CASCADE,
            FOREIGN KEY (subject_id) REFERENCES subjects(subject_id) ON DELETE SET NULL,
            FOREIGN KEY (master_template_id) REFERENCES schedule_templates(template_id) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_task_execution ON task_capsules(execution_date, assigned_session);
        CREATE INDEX IF NOT EXISTS idx_task_subject ON task_capsules(subject_id, sequence_in_subject);

        CREATE TABLE IF NOT EXISTS task_checklists (
            checklist_id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL,
            label TEXT NOT NULL,
            is_checked INTEGER DEFAULT 0,
            sort_order INTEGER DEFAULT 0,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (task_id) REFERENCES task_capsules(task_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS task_adjustments (
            adjustment_id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL,
            previous_date TEXT NOT NULL,
            adjusted_date TEXT NOT NULL,
            reason TEXT NOT NULL,
            master_baseline_affected INTEGER DEFAULT 0,
            recorded_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (task_id) REFERENCES task_capsules(task_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS task_audit_logs (
            log_id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            field_name TEXT,
            old_value TEXT,
            new_value TEXT,
            source_channel TEXT DEFAULT 'TUI',
            occurred_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (task_id) REFERENCES task_capsules(task_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS task_attachments (
            attachment_id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL,
            attachment_type TEXT NOT NULL,
            source_channel TEXT DEFAULT 'LOCAL_FS',
            file_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_size_bytes INTEGER,
            mime_type TEXT,
            sha256_hash TEXT,
            attached_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (task_id) REFERENCES task_capsules(task_id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS holidays_and_leaves (
            leave_id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT NOT NULL,
            target_date TEXT NOT NULL,
            item_type TEXT NOT NULL,
            title TEXT NOT NULL,
            is_work_disabled INTEGER DEFAULT 1,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users_profiles(user_id) ON DELETE CASCADE
        );
        "#,
    )?;
    Ok(())
}
