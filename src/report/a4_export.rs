use std::fs;
use std::path::Path;
use chrono::NaiveDate;

use crate::domain::models::*;

pub fn generate_daily_report_markdown(
    user_name: &str,
    date: NaiveDate,
    tasks: &[TaskCapsule],
    subject: Option<&Subject>,
) -> String {
    let mut out = String::new();

    out.push_str("# [STSMS] 일일 업무 수행 보고서 (Daily Task Report)\n");
    out.push_str("====================================================================\n\n");
    out.push_str(&format!("* **보고 일자**: {}\n", date.format("%Y년 %m월 %d일")));
    out.push_str(&format!("* **작성 엔지니어**: {}\n", user_name));

    // 기상 요약
    if let Some(first_weather) = tasks.iter().find_map(|t| t.weather_snapshot.as_ref()) {
        out.push_str(&format!(
            "* **당일 기상 컨디션**: {} | 기온: {:.1}°C | 강수확률: {}%\n",
            first_weather.weather_sky_code,
            first_weather.temperature_celsius,
            first_weather.precipitation_prob_percent
        ));
        out.push_str(&format!("* **현장 환경 가이드**: {}\n", first_weather.clothing_advice));
        out.push_str(&format!("* **기상 보조 안내**: {}\n", first_weather.umbrella_alert));
    }
    out.push_str("\n--------------------------------------------------------------------\n\n");

    // 동선 요약
    if let Some(sub) = subject {
        out.push_str(&format!("## [현장 동선 요약: {}]\n", sub.subject_name));
        out.push_str(&format!(
            "- 총 순회 거리: {:.1} km | 예상 이동 소요: {}분\n",
            sub.total_distance_km, sub.estimated_duration_min
        ));

        let mut route_tasks: Vec<&TaskCapsule> = tasks
            .iter()
            .filter(|t| t.sequence_in_subject > 0)
            .collect();
        route_tasks.sort_by_key(|t| t.sequence_in_subject);

        for (idx, t) in route_tasks.iter().enumerate() {
            let place = t.place_name.as_deref().unwrap_or("지정장소");
            out.push_str(&format!(
                "  {}) {} ({}) - 구간 거리: {:.1}km / 이동: {}분\n",
                idx + 1,
                place,
                t.title,
                t.distance_from_prev_km,
                t.travel_time_from_prev_min
            ));
        }
        out.push_str("\n--------------------------------------------------------------------\n\n");
    }

    // 오전(AM) 세션
    out.push_str("## 1. 오전 작업 세션 (AM Session)\n\n");
    let am_tasks: Vec<&TaskCapsule> = tasks
        .iter()
        .filter(|t| t.assigned_session == SessionSlot::Am)
        .collect();

    if am_tasks.is_empty() {
        out.push_str("*(배정된 오전 업무가 없습니다)*\n\n");
    } else {
        for (i, t) in am_tasks.iter().enumerate() {
            render_task_detail(&mut out, i + 1, t);
        }
    }

    // 오후(PM) 세션
    out.push_str("## 2. 오후 작업 세션 (PM Session)\n\n");
    let pm_tasks: Vec<&TaskCapsule> = tasks
        .iter()
        .filter(|t| t.assigned_session == SessionSlot::Pm)
        .collect();

    if pm_tasks.is_empty() {
        out.push_str("*(배정된 오후 업무가 없습니다)*\n\n");
    } else {
        for (i, t) in pm_tasks.iter().enumerate() {
            render_task_detail(&mut out, i + 1, t);
        }
    }

    out.push_str("====================================================================\n");
    out.push_str("*문서 서명: Nix's Smart Task & Schedule Management System (v1.0)*\n");

    out
}

fn render_task_detail(out: &mut String, idx: usize, t: &TaskCapsule) {
    out.push_str(&format!("### [{}] {}\n", idx, t.title));
    out.push_str(&format!("- 상태: {} | 구분: {}\n", t.status.label(), t.calendar_layer.label()));
    if let Some(tag) = &t.visual_dday_tag {
        out.push_str(&format!("- D-Day 태그: {}\n", tag));
    }
    if let Some(desc) = &t.description {
        out.push_str(&format!("- 세부 내용: {}\n", desc));
    }
    if t.requires_report {
        out.push_str(&format!(
            "- 실적 보고: {}건 / 목표 {}건 (항목: {})\n",
            t.report_actual_count,
            t.report_target_count,
            t.report_item_name.as_deref().unwrap_or("단순건수")
        ));
        if let Some(summary) = &t.report_result_summary {
            out.push_str(&format!("  * 결과 요약: {}\n", summary));
        }
    }
    if let Some(notes) = &t.special_notes {
        out.push_str(&format!("- 현장 특이사항: {}\n", notes));
    }
    if !t.attachments.is_empty() {
        out.push_str("- 첨부 증빙 파일 목록:\n");
        for att in &t.attachments {
            out.push_str(&format!(
                "  * [{}][{}] {} ({}) [SHA256: {}]\n",
                att.source_channel,
                att.attachment_type,
                att.file_name,
                att.file_path,
                att.sha256_hash.as_deref().unwrap_or("N/A")
            ));
        }
    }
    out.push('\n');
}

/// 명세서 7.3절 A4 인쇄 및 PDF 내보내기 CSS 사양(@media print) 준수 HTML 보고서 생성
pub fn generate_daily_report_html(
    user_name: &str,
    date: NaiveDate,
    tasks: &[TaskCapsule],
    subject: Option<&Subject>,
) -> String {
    let mut rows_html = String::new();
    let mut photos_html = String::new();

    for t in tasks {
        let session_str = t.assigned_session.label();
        let status_str = t.status.label();
        let report_str = if t.requires_report {
            format!(
                "{} / {}건 ({})",
                t.report_actual_count,
                t.report_target_count,
                t.report_result_summary.as_deref().unwrap_or("미기입")
            )
        } else {
            "-".to_string()
        };
        let notes_str = t.special_notes.as_deref().unwrap_or("-");

        rows_html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td><strong>{}</strong><br/><small>{}</small></td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            t.task_id,
            session_str,
            t.title,
            t.visual_dday_tag.as_deref().unwrap_or(""),
            status_str,
            report_str,
            notes_str
        ));

        for att in &t.attachments {
            photos_html.push_str(&format!(
                "<div class=\"photo-card\"><strong>[{}] {}</strong><br/>파일명: {}<br/>경로: <code>{}</code><br/>크기: {} bytes | SHA-256: <code>{}</code></div>\n",
                t.task_id,
                att.attachment_type,
                att.file_name,
                att.file_path,
                att.file_size_bytes.unwrap_or(0),
                att.sha256_hash.as_deref().unwrap_or("N/A")
            ));
        }
    }

    let subject_title = subject
        .map(|s| format!("{} (총 {:.1}km, {}분)", s.subject_name, s.total_distance_km, s.estimated_duration_min))
        .unwrap_or_else(|| "일반 일정".to_string());

    format!(
        r#"<!DOCTYPE html>
<html lang="ko">
<head>
<meta charset="UTF-8">
<title>STSMS 일일 업무보고서 - {date}</title>
<style>
  body {{
    font-family: 'Pretendard', 'Malgun Gothic', sans-serif;
    margin: 20px;
    color: #111;
  }}
  h1 {{ border-bottom: 2px solid #222; padding-bottom: 8px; }}
  .meta-box {{ background: #f8f9fa; border: 1px solid #dee2e6; padding: 12px; margin-bottom: 16px; }}
  .report-table {{ width: 100%; border-collapse: collapse; margin-bottom: 20px; }}
  .report-table th, .report-table td {{ border: 1px solid #333; padding: 8px; text-align: left; font-size: 10.5pt; }}
  .report-table th {{ background: #e9ecef; }}
  .photo-grid {{
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }}
  .photo-card {{
    border: 1px solid #666;
    padding: 10px;
    background: #fafafa;
    font-size: 9.5pt;
  }}
  @media print {{
    header, nav, .dnd-handle, .btn-action, .map-controls {{
      display: none !important;
    }}
    @page {{
      size: A4 portrait;
      margin: 15mm;
    }}
    body {{
      background: #ffffff !important;
      color: #000000 !important;
      font-size: 11pt;
    }}
    .report-table tr {{
      page-break-inside: avoid;
    }}
    .photo-grid {{
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 8px;
      page-break-inside: avoid;
    }}
  }}
</style>
</head>
<body>
  <h1>[STSMS] 일일 업무 수행 보고서 (A4 Print Specification)</h1>
  <div class="meta-box">
    <p><strong>보고 일자:</strong> {date} &nbsp;|&nbsp; <strong>작성자:</strong> {user_name}</p>
    <p><strong>대상 프로젝트 동선(Subject):</strong> {subject_title}</p>
  </div>
  <h2>1. 당일 세션별 업무 수행 내역</h2>
  <table class="report-table">
    <thead>
      <tr>
        <th>업무ID</th>
        <th>세션</th>
        <th>업무명 / D-Day 태그</th>
        <th>상태</th>
        <th>실적 / 결과 요약</th>
        <th>현장 특이사항</th>
      </tr>
    </thead>
    <tbody>
      {rows_html}
    </tbody>
  </table>
  <h2>2. 현장 사진 및 파일시스템 증빙 목록</h2>
  <div class="photo-grid">
    {photos_html}
  </div>
</body>
</html>"#,
        date = date.format("%Y-%m-%d"),
        user_name = user_name,
        subject_title = subject_title,
        rows_html = rows_html,
        photos_html = if photos_html.is_empty() {
            "<p>첨부된 증빙 사진이 없습니다.</p>".to_string()
        } else {
            photos_html
        }
    )
}

pub fn save_report_to_disk(
    md_content: &str,
    html_content: &str,
    date: NaiveDate,
) -> std::io::Result<String> {
    let dir = Path::new("storage_root/reports");
    fs::create_dir_all(dir)?;

    let md_filename = format!("daily_report_{}.md", date.format("%Y%m%d"));
    let md_filepath = dir.join(&md_filename);
    fs::write(&md_filepath, md_content)?;

    let html_filename = format!("daily_report_{}.html", date.format("%Y%m%d"));
    let html_filepath = dir.join(&html_filename);
    fs::write(&html_filepath, html_content)?;

    Ok(format!(
        "{} & {}",
        md_filepath.to_string_lossy(),
        html_filepath.to_string_lossy()
    ))
}
