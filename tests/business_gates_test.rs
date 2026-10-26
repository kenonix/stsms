use chrono::{NaiveDate, Utc};
use stsms::domain::gates::{
    evaluate_weather_assist, validate_and_complete_task, validate_and_reschedule_task,
};
use stsms::domain::models::*;
use stsms::map::braille::BrailleCanvas;
use stsms::map::osm_data::OsmDataset;

#[test]
fn test_gate1_report_requirement() {
    let base_date = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();
    let mut task = TaskCapsule {
        task_id: "TASK-TEST-01".to_string(),
        subject_id: None,
        user_id: "USR-001".to_string(),
        master_template_id: None,
        title: "보고 필요 작업".to_string(),
        description: None,
        calendar_layer: CalendarLayer::Work,
        task_origin: TaskOrigin::Manual,
        status: TaskStatus::InSession,
        visual_dday_tag: None,
        baseline_date: base_date,
        execution_date: base_date,
        assigned_session: SessionSlot::Am,
        session_order: 1,
        master_baseline_affected: false,
        recurrence: RecurrenceConfig::default(),
        requires_report: true,
        report_item_name: Some("점검건수".to_string()),
        report_target_count: 5,
        report_actual_count: 0,
        report_result_summary: None,
        requires_photo_evidence: false,
        custom_checkboxes: vec![],
        special_notes: None,
        place_name: None,
        address: None,
        latitude: None,
        longitude: None,
        sequence_in_subject: 0,
        distance_from_prev_km: 0.0,
        travel_time_from_prev_min: 0,
        weather_snapshot: None,
        attachments: vec![],
        scheduled_fixed_at: Utc::now(),
        created_at: Utc::now(),
        last_modified_at: Utc::now(),
    };

    // 1. 실적 0건일 때 완료 시도 -> 차단되어야 함
    let res = validate_and_complete_task(&mut task, 0, Some("정상 완료".to_string()), Utc::now());
    assert!(res.is_err(), "실적 0건이면 실패해야 함");

    // 2. 결과 요약이 없을 때 완료 시도 -> 차단되어야 함
    let res = validate_and_complete_task(&mut task, 5, None, Utc::now());
    assert!(res.is_err(), "요약 텍스트 없으면 실패해야 함");

    // 3. 정상 입력 시 -> 통과하고 상태가 COMPLETED로 변경되어야 함
    let res = validate_and_complete_task(&mut task, 5, Some("이상 없이 완료함".to_string()), Utc::now());
    assert!(res.is_ok());
    assert_eq!(task.status, TaskStatus::Completed);
    assert_eq!(task.report_actual_count, 5);
}

#[test]
fn test_gate2_photo_evidence_requirement() {
    let base_date = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();
    let mut task = TaskCapsule {
        task_id: "TASK-TEST-02".to_string(),
        subject_id: None,
        user_id: "USR-001".to_string(),
        master_template_id: None,
        title: "사진 보고 필수 작업".to_string(),
        description: None,
        calendar_layer: CalendarLayer::Work,
        task_origin: TaskOrigin::Manual,
        status: TaskStatus::InSession,
        visual_dday_tag: None,
        baseline_date: base_date,
        execution_date: base_date,
        assigned_session: SessionSlot::Am,
        session_order: 1,
        master_baseline_affected: false,
        recurrence: RecurrenceConfig::default(),
        requires_report: false,
        report_item_name: None,
        report_target_count: 0,
        report_actual_count: 0,
        report_result_summary: None,
        requires_photo_evidence: true,
        custom_checkboxes: vec![],
        special_notes: None,
        place_name: None,
        address: None,
        latitude: None,
        longitude: None,
        sequence_in_subject: 0,
        distance_from_prev_km: 0.0,
        travel_time_from_prev_min: 0,
        weather_snapshot: None,
        attachments: vec![],
        scheduled_fixed_at: Utc::now(),
        created_at: Utc::now(),
        last_modified_at: Utc::now(),
    };

    // 사진 첨부 없을 때 -> 에러
    let res = validate_and_complete_task(&mut task, 0, None, Utc::now());
    assert!(res.is_err(), "사진 필수인데 첨부 없으면 실패해야 함");

    // 사진 첨부 후 -> 통과
    task.attachments.push(TaskAttachment {
        attachment_id: None,
        task_id: task.task_id.clone(),
        attachment_type: "MESSENGER_PHOTO".to_string(),
        source_channel: "DISCORD".to_string(),
        file_name: "site_photo.jpg".to_string(),
        file_path: "/storage/site_photo.jpg".to_string(),
        file_size_bytes: Some(1024),
        mime_type: Some("image/jpeg".to_string()),
        sha256_hash: None,
        attached_at: Utc::now(),
    });

    let res = validate_and_complete_task(&mut task, 0, None, Utc::now());
    assert!(res.is_ok());
    assert_eq!(task.status, TaskStatus::Completed);
}

#[test]
fn test_gate3_reschedule_baseline_isolation() {
    let base_date = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();
    let new_date = NaiveDate::from_ymd_opt(2026, 10, 29).unwrap();
    let mut task = TaskCapsule {
        task_id: "TASK-TEST-03".to_string(),
        subject_id: None,
        user_id: "USR-001".to_string(),
        master_template_id: None,
        title: "일정 연기 테스트".to_string(),
        description: None,
        calendar_layer: CalendarLayer::Work,
        task_origin: TaskOrigin::TemplateScheduled,
        status: TaskStatus::Scheduled,
        visual_dday_tag: Some("[~테스트] : D-21".to_string()),
        baseline_date: base_date,
        execution_date: base_date,
        assigned_session: SessionSlot::Am,
        session_order: 1,
        master_baseline_affected: false,
        recurrence: RecurrenceConfig::default(),
        requires_report: false,
        report_item_name: None,
        report_target_count: 0,
        report_actual_count: 0,
        report_result_summary: None,
        requires_photo_evidence: false,
        custom_checkboxes: vec![],
        special_notes: None,
        place_name: None,
        address: None,
        latitude: None,
        longitude: None,
        sequence_in_subject: 0,
        distance_from_prev_km: 0.0,
        travel_time_from_prev_min: 0,
        weather_snapshot: None,
        attachments: vec![],
        scheduled_fixed_at: Utc::now(),
        created_at: Utc::now(),
        last_modified_at: Utc::now(),
    };

    // 1. 사유 없이 연기 시도 -> 실패해야 함
    let res = validate_and_reschedule_task(&mut task, new_date, "".to_string(), Utc::now());
    assert!(res.is_err());

    // 2. 사유 기입 시 -> baseline_date는 유지되고 execution_date만 변경되어야 함
    let res = validate_and_reschedule_task(&mut task, new_date, "협력사 자재 배송 지연".to_string(), Utc::now());
    assert!(res.is_ok());
    assert_eq!(task.baseline_date, base_date, "마스터 기준일은 불변이어야 함!");
    assert_eq!(task.execution_date, new_date, "수행 예정일만 변경되어야 함");
    assert!(!task.master_baseline_affected, "master_baseline_affected는 false이어야 함");
    assert_eq!(task.status, TaskStatus::RescheduledHold);
}

#[test]
fn test_gate4_weather_evaluation() {
    // 1. 기온 3도, 비 70% -> 한파 + 우산 필수
    let w1 = evaluate_weather_assist(3.0, 70, 2.0);
    assert!(w1.umbrella_required);
    assert!(w1.clothing_advice.contains("한파"));

    // 2. 기온 20도, 비 10% -> 쾌적 + 우산 불필요
    let w2 = evaluate_weather_assist(20.0, 10, 1.5);
    assert!(!w2.umbrella_required);
    assert!(w2.clothing_advice.contains("쾌적함"));
}

#[test]
fn test_braille_canvas_subpixels() {
    let mut canvas = BrailleCanvas::new(10, 5);
    // (0, 0) 서브픽셀 찍기 -> dot 1 (0x01)
    canvas.set_pixel(0, 0, ratatui::style::Color::White);
    let (ch, _) = canvas.get_cell(0, 0);
    // U+2800 + 0x01 = U+2801 ('⠁')
    assert_eq!(ch, '⠁');
}

#[test]
fn test_osm_dataset_builtin() {
    let ds = OsmDataset::load_builtin_seoul_metropolitan();
    assert!(!ds.ways.is_empty(), "내장 OSM 데이터셋에 노선이 있어야 함");
    let has_han_river = ds.ways.iter().any(|w| w.name.as_deref().unwrap_or("").contains("한강"));
    assert!(has_han_river, "한강 수계가 포함되어 있어야 함");
}
