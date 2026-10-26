use chrono::{DateTime, NaiveDate, Utc};
use crate::domain::models::{TaskAdjustment, TaskAuditLog, TaskCapsule, TaskStatus, WeatherSnapshot};

/// Gate 4: 기상 데이터 기반 복장 & 우산 자동 판단 매트릭스 (`CORE-02`)
pub fn evaluate_weather_assist(temp_c: f64, pop_pct: i32, wind_speed_ms: f64) -> WeatherSnapshot {
    let clothing = if temp_c <= 4.0 {
        "한파/영하권(<=4°C): 패딩, 헤비코트, 방한내의, 장갑·목도리 착용 필수"
    } else if (5.0..=9.0).contains(&temp_c) {
        "쌀쌀함(5~9°C): 울코트, 경량패딩, 플리스, 니트 레이어링 권장"
    } else if (10.0..=16.0).contains(&temp_c) {
        "선선함(10~16°C): 자켓, 가디건, 맨투맨, 긴팔 셔츠 및 외투 권장"
    } else if (17.0..=22.0).contains(&temp_c) {
        "쾌적함(17~22°C): 얇은 가디건, 긴팔 셔츠, 면바지 최적 활동복"
    } else if (23.0..=27.0).contains(&temp_c) {
        "더움(23~27°C): 반팔 셔츠, 얇은 면의류 및 통기성 복장 권장"
    } else {
        "폭염/무더위(>=28°C): 쿨링 기능성 반팔, 모자, 수분 섭취 안내"
    };

    let (umbrella_req, umbrella_msg) = if pop_pct >= 60 || wind_speed_ms >= 9.0 {
        if wind_speed_ms >= 9.0 {
            (
                true,
                "🔴 [방수우비 권장] 강풍 동반 우천: 우산 파손 주의, 방수자켓/우비 권장".to_string(),
            )
        } else {
            (
                true,
                "🔴 [우산 필수] 강수확률 60% 이상: 우산 및 장비 방수 커버 필수 지참".to_string(),
            )
        }
    } else if (30..60).contains(&pop_pct) {
        (
            true,
            "🟡 [우산 권장] 강수확률 30~59%: 휴대용 접이식 우산 구비 권장".to_string(),
        )
    } else {
        (
            false,
            "🟢 [우산 불필요] 강수확률 30% 미만: 야외 이동 원활".to_string(),
        )
    };

    WeatherSnapshot {
        temperature_celsius: temp_c,
        precipitation_prob_percent: pop_pct,
        weather_sky_code: if pop_pct >= 60 {
            "RAINY".to_string()
        } else if pop_pct >= 30 {
            "CLOUDY".to_string()
        } else {
            "CLEAR".to_string()
        },
        clothing_advice: clothing.to_string(),
        umbrella_required: umbrella_req,
        umbrella_alert: umbrella_msg,
    }
}

/// Gate 1 & Gate 2: 업무 완료 유효성 검증
pub fn validate_and_complete_task(
    task: &mut TaskCapsule,
    actual_count: i32,
    report_summary: Option<String>,
    now: DateTime<Utc>,
) -> Result<TaskAuditLog, String> {
    // Gate 1: 보고필요 업무 결과 입력 강제 검증 (REQ-15가)
    if task.requires_report {
        if actual_count < 1 {
            return Err("보고필요 업무는 실적 횟수(1 이상) 입력이 필수입니다.".to_string());
        }
        match &report_summary {
            Some(s) if s.trim().len() >= 2 => (),
            _ => {
                return Err(
                    "보고 결과 요약 텍스트를 반드시 기입해야 완료할 수 있습니다.".to_string(),
                );
            }
        }
    }

    // Gate 2: 필수 사진 증빙 첨부 검증 (REQ-15다)
    if task.requires_photo_evidence {
        let has_photo = task.attachments.iter().any(|att| {
            att.attachment_type == "MESSENGER_PHOTO"
                || att
                    .mime_type
                    .as_deref()
                    .map_or(false, |m| m.starts_with("image/"))
                || att.file_name.ends_with(".png")
                || att.file_name.ends_with(".jpg")
                || att.file_name.ends_with(".jpeg")
        });
        if !has_photo {
            return Err("사진 보고 필수 업무입니다. 완료 처리 전 현장 증빙 사진을 1장 이상 첨부하십시오.".to_string());
        }
    }

    // 통과 시 상태 갱신
    let old_status = format!("{:?}", task.status);
    task.status = TaskStatus::Completed;
    task.report_actual_count = actual_count;
    task.report_result_summary = report_summary;
    task.last_modified_at = now;

    let audit_log = TaskAuditLog {
        log_id: None,
        task_id: task.task_id.clone(),
        event_type: "COMPLETED".to_string(),
        field_name: Some("status".to_string()),
        old_value: Some(old_status),
        new_value: Some("COMPLETED".to_string()),
        source_channel: "TUI".to_string(),
        occurred_at: now,
    };

    Ok(audit_log)
}

/// Gate 3: 마스터 스케줄 보존형 일정 조정 (REQ-07, REQ-08)
pub fn validate_and_reschedule_task(
    task: &mut TaskCapsule,
    new_date: NaiveDate,
    reason: String,
    now: DateTime<Utc>,
) -> Result<(TaskAdjustment, TaskAuditLog), String> {
    if reason.trim().len() < 2 {
        return Err("일정 조정 사유(2글자 이상)를 반드시 입력해야 합니다.".to_string());
    }

    let prev_date = task.execution_date;
    // baseline_date는 절대 변경하지 않음!
    task.execution_date = new_date;
    task.status = TaskStatus::RescheduledHold;
    task.master_baseline_affected = false;
    task.last_modified_at = now;

    let adjustment = TaskAdjustment {
        adjustment_id: None,
        task_id: task.task_id.clone(),
        previous_date: prev_date,
        adjusted_date: new_date,
        reason: reason.clone(),
        master_baseline_affected: false,
        recorded_at: now,
    };

    let audit_log = TaskAuditLog {
        log_id: None,
        task_id: task.task_id.clone(),
        event_type: "RESCHEDULED".to_string(),
        field_name: Some("execution_date".to_string()),
        old_value: Some(prev_date.to_string()),
        new_value: Some(format!("{} (사유: {})", new_date, reason)),
        source_channel: "TUI".to_string(),
        occurred_at: now,
    };

    Ok((adjustment, audit_log))
}

/// EXT-03: 파일 무결성 검증용 표준 SHA-256 해시 계산 (FIPS 180-4)
pub fn compute_sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    format!(
        "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]
    )
}
