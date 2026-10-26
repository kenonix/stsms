use chrono::{Datelike, Local, NaiveDate, Utc};
use rusqlite::{params, Result};

use crate::db::repository::Repository;
use crate::domain::gates::evaluate_weather_assist;
use crate::domain::models::*;

pub fn seed_sample_data(repo: &Repository) -> Result<()> {
    // 태스크가 이미 존재하면 스킵
    if let Ok(tasks) = repo.get_all_tasks() {
        if !tasks.is_empty() {
            return Ok(());
        }
    }

    let now = Utc::now();
    let user_id = "USR-001";
    repo.insert_user_profile(user_id, "Nix (시스템 엔지니어)", "SOLO_SPLIT")?;

    // 1. D-Day 템플릿 마스터 등록 (외래키 무결성 보장)
    let templates = [
        ("TMPL-01", "장비점검 D-Day 템플릿", "정기 시설 및 센서 교정"),
        ("TMPL-02", "정기감사 D-Day 템플릿", "보안 로그 무결성 대조"),
        ("TMPL-03", "운영보고회 템플릿", "분기 스마트 인프라 실적 종합"),
    ];
    for (tid, tname, tdesc) in templates {
        repo.conn.execute(
            "INSERT OR REPLACE INTO schedule_templates (template_id, user_id, template_name, description, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![tid, user_id, tname, tdesc, now.to_rfc3339()],
        )?;
    }

    // 2. 공휴일 및 휴무 등록
    let today = Local::now().date_naive();
    let holidays = vec![
        HolidayAndLeave {
            leave_id: None,
            user_id: user_id.to_string(),
            target_date: NaiveDate::from_ymd_opt(today.year(), 10, 3).unwrap_or(today),
            item_type: "PUBLIC_HOLIDAY".to_string(),
            title: "개천절".to_string(),
            is_work_disabled: true,
        },
        HolidayAndLeave {
            leave_id: None,
            user_id: user_id.to_string(),
            target_date: NaiveDate::from_ymd_opt(today.year(), 10, 9).unwrap_or(today),
            item_type: "PUBLIC_HOLIDAY".to_string(),
            title: "한글날".to_string(),
            is_work_disabled: true,
        },
        HolidayAndLeave {
            leave_id: None,
            user_id: user_id.to_string(),
            target_date: today + chrono::Duration::days(3),
            item_type: "ANNUAL_LEAVE".to_string(),
            title: "개인 정기 연차".to_string(),
            is_work_disabled: true,
        },
    ];
    for h in holidays {
        repo.insert_holiday(&h)?;
    }

    // 3. Subject 등록 (수도권 순회 점검)
    let subject = Subject {
        subject_id: "SUB-01".to_string(),
        user_id: user_id.to_string(),
        subject_name: "수도권 IoT 센서 및 네트워크 인프라 순회 점검".to_string(),
        color_hex: "#3B82F6".to_string(),
        total_distance_km: 42.8,
        estimated_duration_min: 110,
        created_at: now,
        updated_at: now,
    };
    repo.insert_subject(&subject)?;

    // 4. Task Capsules 등록 (오늘 날짜 기준)
    let base_date = today;

    let task1 = TaskCapsule {
        task_id: "TASK-001".to_string(),
        subject_id: Some("SUB-01".to_string()),
        user_id: user_id.to_string(),
        master_template_id: Some("TMPL-01".to_string()),
        title: "서초 센서 허브 4개소 정밀 교정 및 펌웨어 v2.4 패치".to_string(),
        description: Some("서초구 반포대로 수집 센서 데이터 오차율 0.2% 미만 유지 검증".to_string()),
        calendar_layer: CalendarLayer::Work,
        task_origin: TaskOrigin::TemplateScheduled,
        status: TaskStatus::InSession,
        visual_dday_tag: Some("[~장비점검] : D-21 = 센서 교정".to_string()),
        baseline_date: base_date,
        execution_date: base_date,
        assigned_session: SessionSlot::Am,
        session_order: 1,
        master_baseline_affected: false,
        recurrence: RecurrenceConfig::default(),
        requires_report: true,
        report_item_name: Some("센서 허브 교정 건수".to_string()),
        report_target_count: 4,
        report_actual_count: 0,
        report_result_summary: None,
        requires_photo_evidence: false,
        custom_checkboxes: vec![
            CheckItem {
                checklist_id: None,
                label: "전원 모듈 전압 측정 (3.3V / 5.0V 정상)".to_string(),
                is_checked: true,
                sort_order: 1,
            },
            CheckItem {
                checklist_id: None,
                label: "LoRaWAN 통신 패킷 수신율 99% 확인".to_string(),
                is_checked: false,
                sort_order: 2,
            },
        ],
        special_notes: Some("오전 9시 이전 통행량 적을 때 작업 권장".to_string()),
        place_name: Some("서초 센서 허브 A구역".to_string()),
        address: Some("서울특별시 서초구 반포대로 58".to_string()),
        latitude: Some(37.4837),
        longitude: Some(127.0124),
        sequence_in_subject: 1,
        distance_from_prev_km: 0.0,
        travel_time_from_prev_min: 0,
        weather_snapshot: Some(evaluate_weather_assist(14.5, 15, 2.1)),
        attachments: vec![],
        scheduled_fixed_at: now,
        created_at: now,
        last_modified_at: now,
    };

    let task2 = TaskCapsule {
        task_id: "TASK-002".to_string(),
        subject_id: Some("SUB-01".to_string()),
        user_id: user_id.to_string(),
        master_template_id: Some("TMPL-01".to_string()),
        title: "판교 IDC 메인 L3 스위치 백업 라인 바이패스 시험".to_string(),
        description: Some("판교 2테크노밸리 제어센터 무중단 절체 핫스탠바이 스위치 이중화 테스트".to_string()),
        calendar_layer: CalendarLayer::Work,
        task_origin: TaskOrigin::TemplateScheduled,
        status: TaskStatus::InSession,
        visual_dday_tag: Some("[~장비점검] : D-21 = 스위치 교체".to_string()),
        baseline_date: base_date,
        execution_date: base_date,
        assigned_session: SessionSlot::Am,
        session_order: 2,
        master_baseline_affected: false,
        recurrence: RecurrenceConfig::default(),
        requires_report: true,
        report_item_name: Some("바이패스 절체 시험 횟수".to_string()),
        report_target_count: 2,
        report_actual_count: 2,
        report_result_summary: Some("1/2차 절체 0.02초 이내 무중단 통과 완료".to_string()),
        requires_photo_evidence: true,
        custom_checkboxes: vec![],
        special_notes: Some("보안 출입증 사전 데스크 승인 필수. 랙 14번 전면 사진 촬영 필요.".to_string()),
        place_name: Some("판교 글로벌 R&D 센터".to_string()),
        address: Some("경기도 성남시 분당구 판교로 289".to_string()),
        latitude: Some(37.4020),
        longitude: Some(127.1086),
        sequence_in_subject: 2,
        distance_from_prev_km: 17.5,
        travel_time_from_prev_min: 35,
        weather_snapshot: Some(evaluate_weather_assist(16.0, 25, 3.2)),
        attachments: vec![TaskAttachment {
            attachment_id: None,
            task_id: "TASK-002".to_string(),
            attachment_type: "MESSENGER_PHOTO".to_string(),
            source_channel: "DISCORD".to_string(),
            file_name: "rack14_bypass_led_ok.jpg".to_string(),
            file_path: "/stsms_storage_root/evidence/rack14_bypass_led_ok.jpg".to_string(),
            file_size_bytes: Some(248912),
            mime_type: Some("image/jpeg".to_string()),
            sha256_hash: Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string()),
            attached_at: now,
        }],
        scheduled_fixed_at: now,
        created_at: now,
        last_modified_at: now,
    };

    let task3 = TaskCapsule {
        task_id: "TASK-003".to_string(),
        subject_id: Some("SUB-01".to_string()),
        user_id: user_id.to_string(),
        master_template_id: Some("TMPL-01".to_string()),
        title: "여의도 IFC 전용 백본망 광링크 레벨 측정 및 지터 분석".to_string(),
        description: Some("금융권 전용망 저지연 광케이블 감쇄율 dBm 전송 품질 정밀 진단".to_string()),
        calendar_layer: CalendarLayer::Work,
        task_origin: TaskOrigin::TemplateScheduled,
        status: TaskStatus::Scheduled,
        visual_dday_tag: Some("[~장비점검] : D-21 = 백본 진단".to_string()),
        baseline_date: base_date,
        execution_date: base_date,
        assigned_session: SessionSlot::Pm,
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
        special_notes: Some("오후 3시 장 마감 후 진단 세션 개시".to_string()),
        place_name: Some("여의도 국제금융센터(IFC)".to_string()),
        address: Some("서울특별시 영등포구 국제금융로 10".to_string()),
        latitude: Some(37.5253),
        longitude: Some(126.9255),
        sequence_in_subject: 3,
        distance_from_prev_km: 21.2,
        travel_time_from_prev_min: 45,
        weather_snapshot: Some(evaluate_weather_assist(13.2, 75, 4.8)), // 강수확률 75% -> 우산 필수!
        attachments: vec![],
        scheduled_fixed_at: now,
        created_at: now,
        last_modified_at: now,
    };

    let task4 = TaskCapsule {
        task_id: "TASK-004".to_string(),
        subject_id: Some("SUB-01".to_string()),
        user_id: user_id.to_string(),
        master_template_id: Some("TMPL-02".to_string()),
        title: "상암 디지털미디어시티 보안 감사 로그 무결성 대조".to_string(),
        description: Some("3분기 외부 접속 침입 탐지 시스템(IDS) 로그 SHA-256 해시 검증".to_string()),
        calendar_layer: CalendarLayer::Work,
        task_origin: TaskOrigin::TemplateScheduled,
        status: TaskStatus::Scheduled,
        visual_dday_tag: Some("[~정기감사] : D-10 = 로그 대조".to_string()),
        baseline_date: base_date,
        execution_date: base_date,
        assigned_session: SessionSlot::Pm,
        session_order: 2,
        master_baseline_affected: false,
        recurrence: RecurrenceConfig::default(),
        requires_report: true,
        report_item_name: Some("대조 로그 파일 수".to_string()),
        report_target_count: 12,
        report_actual_count: 0,
        report_result_summary: None,
        requires_photo_evidence: false,
        custom_checkboxes: vec![],
        special_notes: None,
        place_name: Some("상암 누리꿈스퀘어".to_string()),
        address: Some("서울특별시 마포구 월드컵북로 396".to_string()),
        latitude: Some(37.5794),
        longitude: Some(126.8926),
        sequence_in_subject: 4,
        distance_from_prev_km: 7.8,
        travel_time_from_prev_min: 20,
        weather_snapshot: Some(evaluate_weather_assist(12.8, 80, 5.0)),
        attachments: vec![],
        scheduled_fixed_at: now,
        created_at: now,
        last_modified_at: now,
    };

    let task5 = TaskCapsule {
        task_id: "TASK-005".to_string(),
        subject_id: None,
        user_id: user_id.to_string(),
        master_template_id: Some("TMPL-03".to_string()),
        title: "2026 스마트 인프라 운영 혁신 종합 보고서 초안 작성".to_string(),
        description: Some("경영진 정기 보고회 프레젠테이션 및 지표 통계 시각화 자료 취합".to_string()),
        calendar_layer: CalendarLayer::Work,
        task_origin: TaskOrigin::TemplateScheduled,
        status: TaskStatus::Scheduled,
        visual_dday_tag: Some("[~보고회] : D-03 = 최종 보고".to_string()),
        baseline_date: base_date,
        execution_date: base_date,
        assigned_session: SessionSlot::Unassigned,
        session_order: 0,
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
        place_name: Some("본사 기획실 대회의실".to_string()),
        address: Some("서울특별시 강남구 테헤란로 152".to_string()),
        latitude: Some(37.5000),
        longitude: Some(127.0360),
        sequence_in_subject: 0,
        distance_from_prev_km: 0.0,
        travel_time_from_prev_min: 0,
        weather_snapshot: Some(evaluate_weather_assist(15.0, 10, 1.5)),
        attachments: vec![],
        scheduled_fixed_at: now,
        created_at: now,
        last_modified_at: now,
    };

    let tomorrow = today.succ_opt().unwrap_or(today);
    let task6 = TaskCapsule {
        task_id: "TASK-006".to_string(),
        subject_id: None,
        user_id: user_id.to_string(),
        master_template_id: None,
        title: "개인 정밀 건강검진 사전 문진표 제출 및 금식 체크".to_string(),
        description: Some("종합병원 검진센터 온라인 문진표 18시 이전 제출".to_string()),
        calendar_layer: CalendarLayer::Personal,
        task_origin: TaskOrigin::Manual,
        status: TaskStatus::Scheduled,
        visual_dday_tag: Some("[개인] : D-01 = 건강검진".to_string()),
        baseline_date: tomorrow,
        execution_date: tomorrow,
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
        special_notes: Some("전일 21시 이후 금식 준수".to_string()),
        place_name: Some("강남 종합건강검진센터".to_string()),
        address: Some("서울특별시 강남구 도곡로 235".to_string()),
        latitude: Some(37.4920),
        longitude: Some(127.0450),
        sequence_in_subject: 0,
        distance_from_prev_km: 0.0,
        travel_time_from_prev_min: 0,
        weather_snapshot: Some(evaluate_weather_assist(11.0, 5, 1.0)),
        attachments: vec![],
        scheduled_fixed_at: now,
        created_at: now,
        last_modified_at: now,
    };

    repo.save_task(&task1)?;
    repo.save_task(&task2)?;
    repo.save_task(&task3)?;
    repo.save_task(&task4)?;
    repo.save_task(&task5)?;
    repo.save_task(&task6)?;

    // 초기 감사 로그
    let audit_log = TaskAuditLog {
        log_id: None,
        task_id: "TASK-001".to_string(),
        event_type: "CREATED".to_string(),
        field_name: None,
        old_value: None,
        new_value: Some("초기 D-Day 템플릿 업무 캡슐 생성".to_string()),
        source_channel: "SYSTEM".to_string(),
        occurred_at: now,
    };
    repo.insert_audit_log(&audit_log)?;

    Ok(())
}
