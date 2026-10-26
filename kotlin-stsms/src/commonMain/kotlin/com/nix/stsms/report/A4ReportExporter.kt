package com.nix.stsms.report

import com.nix.stsms.domain.SessionSlot
import com.nix.stsms.domain.Subject
import com.nix.stsms.domain.TaskCapsule
import java.io.File
import java.time.LocalDate
import java.time.format.DateTimeFormatter

data class ExportedReportPaths(
    val markdownPath: String,
    val htmlPath: String,
    val markdownContent: String,
    val htmlContent: String
)

object A4ReportExporter {

    fun generateDailyReportMarkdown(
        userName: String,
        date: LocalDate,
        tasks: List<TaskCapsule>,
        subject: Subject?
    ): String {
        val sb = StringBuilder()
        val dateKor = date.format(DateTimeFormatter.ofPattern("yyyy년 MM월 dd일"))

        sb.appendLine("# [STSMS] 일일 업무 수행 보고서 (Daily Task Report)")
        sb.appendLine("====================================================================")
        sb.appendLine()
        sb.appendLine("* **보고 일자**: $dateKor")
        sb.appendLine("* **작성 엔지니어**: $userName")

        tasks.firstNotNullOfOrNull { it.weatherSnapshot }?.let { w ->
            sb.appendLine(
                "* **당일 기상 컨디션**: ${w.weatherSkyCode} | 기온: ${"%.1f".format(w.temperatureCelsius)}°C | 강수확률: ${w.precipitationProbPercent}%"
            )
            sb.appendLine("* **현장 환경 가이드**: ${w.clothingAdvice}")
            sb.appendLine("* **기상 보조 안내**: ${w.umbrellaAlert}")
        }
        sb.appendLine()
        sb.appendLine("--------------------------------------------------------------------")
        sb.appendLine()

        if (subject != null) {
            sb.appendLine("## [현장 동선 요약: ${subject.subjectName}]")
            sb.appendLine("- 총 순회 거리: ${"%.1f".format(subject.totalDistanceKm)} km | 예상 이동 소요: ${subject.estimatedDurationMin}분")
            val routeTasks = tasks.filter { it.sequenceInSubject > 0 }.sortedBy { it.sequenceInSubject }
            routeTasks.forEachIndexed { idx, t ->
                val place = t.placeName ?: "지정장소"
                sb.appendLine(
                    "  ${idx + 1}) $place (${t.title}) - 구간 거리: ${"%.1f".format(t.distanceFromPrevKm)}km / 이동: ${t.travelTimeFromPrevMin}분"
                )
            }
            sb.appendLine()
            sb.appendLine("--------------------------------------------------------------------")
            sb.appendLine()
        }

        sb.appendLine("## 1. 오전 작업 세션 (AM Session)")
        sb.appendLine()
        val amTasks = tasks.filter { it.assignedSession == SessionSlot.AM }.sortedBy { it.sessionOrder }
        if (amTasks.isEmpty()) {
            sb.appendLine("*(배정된 오전 업무가 없습니다)*")
            sb.appendLine()
        } else {
            amTasks.forEachIndexed { idx, t -> renderTaskMarkdown(sb, idx + 1, t) }
        }

        sb.appendLine("## 2. 오후 작업 세션 (PM Session)")
        sb.appendLine()
        val pmTasks = tasks.filter { it.assignedSession == SessionSlot.PM }.sortedBy { it.sessionOrder }
        if (pmTasks.isEmpty()) {
            sb.appendLine("*(배정된 오후 업무가 없습니다)*")
            sb.appendLine()
        } else {
            pmTasks.forEachIndexed { idx, t -> renderTaskMarkdown(sb, idx + 1, t) }
        }

        sb.appendLine("====================================================================")
        sb.appendLine("*문서 서명: Nix's Smart Task & Schedule Management System (v1.0 - Kotlin Multiplatform)*")
        return sb.toString()
    }

    private fun renderTaskMarkdown(sb: StringBuilder, idx: Int, t: TaskCapsule) {
        sb.appendLine("### [$idx] ${t.title}")
        sb.appendLine("- 상태: ${t.status.label} | 구분: ${t.calendarLayer.label} | 기준일(불변): ${t.baselineDate}")
        t.visualDdayTag?.let { sb.appendLine("- D-Day 태그: $it") }
        t.description?.let { sb.appendLine("- 세부 내용: $it") }
        if (t.requiresReport) {
            sb.appendLine(
                "- 실적 보고: ${t.reportActualCount}건 / 목표 ${t.reportTargetCount}건 (항목: ${t.reportItemName ?: "단순건수"})"
            )
            t.reportResultSummary?.let { sb.appendLine("  * 결과 요약: $it") }
        }
        t.specialNotes?.let { sb.appendLine("- 현장 특이사항: $it") }
        if (t.attachments.isNotEmpty()) {
            sb.appendLine("- 첨부 증빙 파일 목록:")
            for (att in t.attachments) {
                sb.appendLine(
                    "  * [${att.sourceChannel}][${att.attachmentType}] ${att.fileName} (${att.filePath}) [SHA256: ${att.sha256Hash ?: "N/A"}]"
                )
            }
        }
        sb.appendLine()
    }

    /**
     * 명세서 7.3절 A4 인쇄 및 PDF 내보내기 CSS 사양 (@media print & @page A4 portrait) 준수 HTML 생성
     */
    fun generateDailyReportHtml(
        userName: String,
        date: LocalDate,
        tasks: List<TaskCapsule>,
        subject: Subject?
    ): String {
        val rowsHtml = StringBuilder()
        val photosHtml = StringBuilder()

        for (t in tasks) {
            val reportStr = if (t.requiresReport) {
                "${t.reportActualCount} / ${t.reportTargetCount}건 (${t.reportResultSummary ?: "미기입"})"
            } else {
                "-"
            }
            val notesStr = t.specialNotes ?: "-"
            rowsHtml.appendLine(
                "<tr><td>${t.taskId}</td><td>${t.assignedSession.label}</td><td><strong>${t.title}</strong><br/><small>${t.visualDdayTag ?: ""}</small></td><td>${t.status.label}</td><td>$reportStr</td><td>$notesStr</td></tr>"
            )

            for (att in t.attachments) {
                photosHtml.appendLine(
                    """<div class="photo-card"><strong>[${t.taskId}] ${att.attachmentType} (${att.sourceChannel})</strong><br/>파일명: ${att.fileName}<br/>경로: <code>${att.filePath}</code><br/>크기: ${att.fileSizeBytes ?: 0} bytes | SHA-256: <code>${att.sha256Hash ?: "N/A"}</code></div>"""
                )
            }
        }

        val subjectTitle = subject?.let {
            "${it.subjectName} (총 ${"%.1f".format(it.totalDistanceKm)}km, ${it.estimatedDurationMin}분)"
        } ?: "일반 일정"

        val photoSection = if (photosHtml.isEmpty()) {
            "<p>첨부된 증빙 사진이 없습니다.</p>"
        } else {
            photosHtml.toString()
        }

        return """<!DOCTYPE html>
<html lang="ko">
<head>
<meta charset="UTF-8">
<title>STSMS 일일 업무보고서 - $date</title>
<style>
  body {
    font-family: 'Pretendard', 'Malgun Gothic', sans-serif;
    margin: 20px;
    color: #111;
  }
  h1 { border-bottom: 2px solid #222; padding-bottom: 8px; }
  .meta-box { background: #f8f9fa; border: 1px solid #dee2e6; padding: 12px; margin-bottom: 16px; }
  .report-table { width: 100%; border-collapse: collapse; margin-bottom: 20px; }
  .report-table th, .report-table td { border: 1px solid #333; padding: 8px; text-align: left; font-size: 10.5pt; }
  .report-table th { background: #e9ecef; }
  .photo-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }
  .photo-card {
    border: 1px solid #666;
    padding: 10px;
    background: #fafafa;
    font-size: 9.5pt;
  }
  @media print {
    header, nav, .dnd-handle, .btn-action, .map-controls {
      display: none !important;
    }
    @page {
      size: A4 portrait;
      margin: 15mm;
    }
    body {
      background: #ffffff !important;
      color: #000000 !important;
      font-size: 11pt;
    }
    .report-table tr {
      page-break-inside: avoid;
    }
    .photo-grid {
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 8px;
      page-break-inside: avoid;
    }
  }
</style>
</head>
<body>
  <h1>[STSMS] 일일 업무 수행 보고서 (A4 Print Specification)</h1>
  <div class="meta-box">
    <p><strong>보고 일자:</strong> $date &nbsp;|&nbsp; <strong>작성자:</strong> $userName</p>
    <p><strong>대상 프로젝트 동선(Subject):</strong> $subjectTitle</p>
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
      $rowsHtml
    </tbody>
  </table>
  <h2>2. 현장 사진 및 파일시스템 증빙 목록</h2>
  <div class="photo-grid">
    $photoSection
  </div>
</body>
</html>"""
    }

    fun exportDailyReportFiles(
        storageRootDir: File,
        userName: String,
        date: LocalDate,
        tasks: List<TaskCapsule>,
        subject: Subject?
    ): ExportedReportPaths {
        val reportsDir = File(storageRootDir, "reports")
        reportsDir.mkdirs()

        val compactDate = date.format(DateTimeFormatter.ofPattern("yyyyMMdd"))
        val mdFile = File(reportsDir, "daily_report_$compactDate.md")
        val htmlFile = File(reportsDir, "daily_report_$compactDate.html")

        val mdContent = generateDailyReportMarkdown(userName, date, tasks, subject)
        val htmlContent = generateDailyReportHtml(userName, date, tasks, subject)

        mdFile.writeText(mdContent, Charsets.UTF_8)
        htmlFile.writeText(htmlContent, Charsets.UTF_8)

        return ExportedReportPaths(
            markdownPath = mdFile.absolutePath,
            htmlPath = htmlFile.absolutePath,
            markdownContent = mdContent,
            htmlContent = htmlContent
        )
    }
}
