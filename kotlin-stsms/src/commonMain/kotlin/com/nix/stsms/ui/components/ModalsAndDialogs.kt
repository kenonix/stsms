package com.nix.stsms.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import com.nix.stsms.domain.*
import com.nix.stsms.report.ExportedReportPaths
import com.nix.stsms.ui.theme.StsmsColors
import java.time.LocalDate

sealed class ActiveModal {
    data object None : ActiveModal()
    data class CompleteTask(val task: TaskCapsule) : ActiveModal()
    data class RescheduleTask(val task: TaskCapsule) : ActiveModal()
    data class AttachEvidence(val task: TaskCapsule) : ActiveModal()
    data object WebhookSimulator : ActiveModal()
    data object AddOrInstantiateTask : ActiveModal()
    data class A4ReportPreview(val exported: ExportedReportPaths) : ActiveModal()
}

@Composable
fun CompleteTaskDialog(
    task: TaskCapsule,
    errorMessage: String?,
    onDismiss: () -> Unit,
    onConfirm: (actualCount: Int, summary: String, notes: String?) -> Unit
) {
    var actualCountText by remember(task) {
        mutableStateOf(
            if (task.reportActualCount > 0) task.reportActualCount.toString()
            else if (task.reportTargetCount > 0) task.reportTargetCount.toString()
            else "1"
        )
    }
    var summaryText by remember(task) { mutableStateOf(task.reportResultSummary ?: "") }
    var notesText by remember(task) { mutableStateOf(task.specialNotes ?: "") }

    Dialog(onDismissRequest = onDismiss) {
        Column(
            modifier = Modifier
                .widthIn(min = 340.dp, max = 520.dp)
                .clip(RoundedCornerShape(14.dp))
                .background(StsmsColors.BgCard)
                .border(1.5.dp, StsmsColors.AccentCyan, RoundedCornerShape(14.dp))
                .padding(18.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            Text(
                text = "✅ 업무 완료 보고 [Gate 1 & Gate 2 검증]",
                color = StsmsColors.AccentCyan,
                fontSize = 16.sp,
                fontWeight = FontWeight.Bold
            )
            Text(
                text = "[${task.taskId}] ${task.title}",
                color = StsmsColors.TextPrimary,
                fontSize = 13.sp,
                fontWeight = FontWeight.SemiBold
            )

            // Gate 1 & Gate 2 요구사항 안내 배지
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                val g1Color = if (task.requiresReport) StsmsColors.AccentAmber else StsmsColors.AccentEmerald
                val g2Color = if (task.requiresPhotoEvidence) {
                    if (task.attachments.isNotEmpty()) StsmsColors.AccentEmerald else StsmsColors.AccentCoral
                } else StsmsColors.AccentEmerald

                BadgePill(
                    text = if (task.requiresReport) "Gate 1: 실적/요약 필수 (목표 ${task.reportTargetCount}건)" else "Gate 1: 일반 업무",
                    color = g1Color
                )
                BadgePill(
                    text = if (task.requiresPhotoEvidence) "Gate 2: 사진필수 (현재 ${task.attachments.size}장)" else "Gate 2: 사진 선택",
                    color = g2Color
                )
            }

            if (errorMessage != null) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(8.dp))
                        .background(StsmsColors.AccentCoral.copy(alpha = 0.2f))
                        .border(1.dp, StsmsColors.AccentCoral, RoundedCornerShape(8.dp))
                        .padding(10.dp)
                ) {
                    Text(text = errorMessage, color = StsmsColors.AccentCoral, fontSize = 12.sp, fontWeight = FontWeight.Bold)
                }
            }

            OutlinedTextField(
                value = actualCountText,
                onValueChange = { actualCountText = it.filter { c -> c.isDigit() } },
                label = { Text("실적 건수 (${task.reportItemName ?: "수행 건수"}, 목표: ${task.reportTargetCount}건)") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )

            OutlinedTextField(
                value = summaryText,
                onValueChange = { summaryText = it },
                label = { Text("결과 요약 보고 (report_result_summary, 2자 이상 필수)") },
                placeholder = { Text("예: 4개소 센서 오차 0.08% 이내 정밀 교정 완료") },
                modifier = Modifier.fillMaxWidth()
            )

            // 빠른 입력 프리셋 버튼 (모바일 터치 편의)
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                SmallActionChip("⚡ 정상 교정 완료") {
                    summaryText = "${task.reportTargetCount.coerceAtLeast(1)}개소 센서 오차 0.08% 교정 완료"
                }
                SmallActionChip("⚡ 이상 없음 확인") {
                    summaryText = "전 구간 기준치 이내 정상 작동 확인 완료"
                }
            }

            OutlinedTextField(
                value = notesText,
                onValueChange = { notesText = it },
                label = { Text("현장 특이사항 (special_notes, 선택)") },
                modifier = Modifier.fillMaxWidth()
            )

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.End,
                verticalAlignment = Alignment.CenterVertically
            ) {
                TextButton(onClick = onDismiss) {
                    Text("취소", color = StsmsColors.TextSecondary)
                }
                Spacer(modifier = Modifier.width(8.dp))
                Button(
                    onClick = {
                        val cnt = actualCountText.toIntOrNull() ?: 0
                        onConfirm(cnt, summaryText, notesText)
                    },
                    colors = ButtonDefaults.buttonColors(containerColor = StsmsColors.AccentEmerald)
                ) {
                    Text("완료 확정 (Gate 검증)", color = Color.White, fontWeight = FontWeight.Bold)
                }
            }
        }
    }
}

@Composable
fun RescheduleTaskDialog(
    task: TaskCapsule,
    errorMessage: String?,
    onDismiss: () -> Unit,
    onConfirm: (newDate: LocalDate, reason: String) -> Unit
) {
    var targetDate by remember(task) { mutableStateOf(task.executionDate.plusDays(1)) }
    var reasonText by remember { mutableStateOf("") }

    Dialog(onDismissRequest = onDismiss) {
        Column(
            modifier = Modifier
                .widthIn(min = 340.dp, max = 520.dp)
                .clip(RoundedCornerShape(14.dp))
                .background(StsmsColors.BgCard)
                .border(1.5.dp, StsmsColors.AccentAmber, RoundedCornerShape(14.dp))
                .padding(18.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            Text(
                text = "📅 마스터 기준선 보존형 일정 연기 [Gate 3]",
                color = StsmsColors.AccentAmber,
                fontSize = 16.sp,
                fontWeight = FontWeight.Bold
            )
            Text(
                text = "[${task.taskId}] ${task.title}",
                color = StsmsColors.TextPrimary,
                fontSize = 13.sp,
                fontWeight = FontWeight.SemiBold
            )

            // 마스터 기준일 불변 보장 안내 박스
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(8.dp))
                    .background(StsmsColors.BgSecondary)
                    .padding(10.dp),
                verticalArrangement = Arrangement.spacedBy(4.dp)
            ) {
                Text(
                    text = "🔒 최초 마스터 기준일 (baseline_date): ${task.baselineDate} [절대 불변]",
                    color = StsmsColors.AccentCyan,
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Bold
                )
                Text(
                    text = "➡️ 현재 수행 예정일: ${task.executionDate}  →  변경 후 수행 예정일: $targetDate",
                    color = StsmsColors.TextPrimary,
                    fontSize = 12.sp
                )
            }

            Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                Text("수행일 빠른 선택:", color = StsmsColors.TextSecondary, fontSize = 11.sp)
                SmallActionChip("+1일(익일)") { targetDate = task.executionDate.plusDays(1) }
                SmallActionChip("+2일") { targetDate = task.executionDate.plusDays(2) }
                SmallActionChip("+3일") { targetDate = task.executionDate.plusDays(3) }
                SmallActionChip("+7일(차주)") { targetDate = task.executionDate.plusDays(7) }
            }

            if (errorMessage != null) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(8.dp))
                        .background(StsmsColors.AccentCoral.copy(alpha = 0.2f))
                        .border(1.dp, StsmsColors.AccentCoral, RoundedCornerShape(8.dp))
                        .padding(10.dp)
                ) {
                    Text(text = errorMessage, color = StsmsColors.AccentCoral, fontSize = 12.sp, fontWeight = FontWeight.Bold)
                }
            }

            OutlinedTextField(
                value = reasonText,
                onValueChange = { reasonText = it },
                label = { Text("일정 조정 사유 (reason, 2글자 이상 필수)") },
                placeholder = { Text("예: 우천 및 강풍으로 인한 현장 작업 순연") },
                modifier = Modifier.fillMaxWidth()
            )

            Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                SmallActionChip("🌧️ 우천/강풍 순연") { reasonText = "우천 및 강풍으로 인한 현장 작업 순연" }
                SmallActionChip("📦 자재 입고 지연") { reasonText = "교체용 센서 모듈 자재 입고 지연" }
                SmallActionChip("🏢 기관 출입 일정 조율") { reasonText = "방문 기관 보안 출입 승인 일정 조율" }
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.End
            ) {
                TextButton(onClick = onDismiss) {
                    Text("취소", color = StsmsColors.TextSecondary)
                }
                Spacer(modifier = Modifier.width(8.dp))
                Button(
                    onClick = { onConfirm(targetDate, reasonText) },
                    colors = ButtonDefaults.buttonColors(containerColor = StsmsColors.AccentAmber)
                ) {
                    Text("일정 조정 확정 (Gate 3)", color = Color.Black, fontWeight = FontWeight.Bold)
                }
            }
        }
    }
}

@Composable
fun AttachEvidenceDialog(
    task: TaskCapsule,
    onDismiss: () -> Unit,
    onAttach: (filePathOrName: String, channel: String) -> Unit
) {
    var filePathInput by remember { mutableStateOf("seocho_sensor_hub_photo.jpg") }
    var selectedChannel by remember { mutableStateOf("LOCAL_FS") }

    Dialog(onDismissRequest = onDismiss) {
        Column(
            modifier = Modifier
                .widthIn(min = 340.dp, max = 540.dp)
                .clip(RoundedCornerShape(14.dp))
                .background(StsmsColors.BgCard)
                .border(1.5.dp, StsmsColors.AccentCyan, RoundedCornerShape(14.dp))
                .padding(18.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            Text(
                text = "📷 현장 사진 및 파일시스템 증빙 첨부 [REQ-05 / EXT-03]",
                color = StsmsColors.AccentCyan,
                fontSize = 15.sp,
                fontWeight = FontWeight.Bold
            )
            Text(
                text = "대상 과업: [${task.taskId}] ${task.title}",
                color = StsmsColors.TextPrimary,
                fontSize = 12.sp
            )
            Text(
                text = "저장 경로 규칙: storage_root/${"%04d/%02d/%02d".format(task.executionDate.year, task.executionDate.monthValue, task.executionDate.dayOfMonth)}/${task.taskId}/<파일명> (SHA-256 자동 해시)",
                color = StsmsColors.TextSecondary,
                fontSize = 11.sp
            )

            // 채널 선택
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                Text("수집 채널:", color = StsmsColors.TextSecondary, fontSize = 11.sp)
                listOf("LOCAL_FS" to "📁 로컬 파일", "KAKAO" to "💬 카카오톡", "DISCORD" to "🤖 디스코드").forEach { (ch, label) ->
                    SmallActionChip(
                        label = label,
                        active = selectedChannel == ch,
                        onClick = { selectedChannel = ch }
                    )
                }
            }

            OutlinedTextField(
                value = filePathInput,
                onValueChange = { filePathInput = it },
                label = { Text("첨부할 사진/문서 파일명 또는 경로") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )

            Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                SmallActionChip("📷 seocho_sensor_hub_photo.jpg") {
                    filePathInput = "seocho_sensor_hub_photo.jpg"
                }
                SmallActionChip("📷 sangam_ids_audit_proof.jpg") {
                    filePathInput = "sangam_ids_audit_proof.jpg"
                }
                SmallActionChip("📄 calibration_report.pdf") {
                    filePathInput = "calibration_report.pdf"
                }
            }

            // 현재 첨부된 파일 목록
            if (task.attachments.isNotEmpty()) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(8.dp))
                        .background(StsmsColors.BgSecondary)
                        .padding(8.dp),
                    verticalArrangement = Arrangement.spacedBy(4.dp)
                ) {
                    Text(
                        text = "현재 첨부된 증빙 (${task.attachments.size}건):",
                        color = StsmsColors.AccentEmerald,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold
                    )
                    task.attachments.forEach { att ->
                        Text(
                            text = "• [${att.sourceChannel}] ${att.fileName} (${att.fileSizeBytes ?: 0}B) SHA256: ${att.sha256Hash?.take(16) ?: "-"}...",
                            color = StsmsColors.TextPrimary,
                            fontSize = 10.sp,
                            fontFamily = FontFamily.Monospace
                        )
                    }
                }
            }

            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                TextButton(onClick = onDismiss) {
                    Text("닫기", color = StsmsColors.TextSecondary)
                }
                Spacer(modifier = Modifier.width(8.dp))
                Button(
                    onClick = {
                        if (filePathInput.isNotBlank()) {
                            onAttach(filePathInput.trim(), selectedChannel)
                        }
                    },
                    colors = ButtonDefaults.buttonColors(containerColor = StsmsColors.AccentCyan)
                ) {
                    Text("스토리지 적재 & SHA-256 검증", color = Color.Black, fontWeight = FontWeight.Bold)
                }
            }
        }
    }
}

@Composable
fun WebhookSimulatorDialog(
    tasksOnDate: List<TaskCapsule>,
    onDismiss: () -> Unit,
    onExecuteWebhook: (platform: String, session: SessionSlot, messageText: String, fileName: String, forcedTaskId: String?) -> String
) {
    var platform by remember { mutableStateOf("KAKAO") }
    var currentSession by remember { mutableStateOf(SessionSlot.PM) }
    var messageText by remember { mutableStateOf("#TASK-004 상암 IDS 해시 대조 현장 사진 보고") }
    var fileName by remember { mutableStateOf("webhook_site_evidence.jpg") }
    var resultLog by remember { mutableStateOf<String?>(null) }
    var pendingCandidates by remember { mutableStateOf<List<TaskCapsule>>(emptyList()) }

    Dialog(onDismissRequest = onDismiss) {
        Column(
            modifier = Modifier
                .widthIn(min = 360.dp, max = 560.dp)
                .clip(RoundedCornerShape(14.dp))
                .background(StsmsColors.BgCard)
                .border(1.5.dp, StsmsColors.AccentPurple, RoundedCornerShape(14.dp))
                .padding(18.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            Text(
                text = "💬 카카오톡 / 디스코드 사진 Webhook 자동 매핑 [REQ-10, 6.2]",
                color = StsmsColors.AccentPurple,
                fontSize = 15.sp,
                fontWeight = FontWeight.Bold
            )
            Text(
                text = "3단계 우선순위: ① 메시지 #단축태그 매칭 → ② 현재 세션(AM/PM) 사진필수 단일 업무 자동 매칭 → ③ 다중 후보 시 퀵리플라이 선택",
                color = StsmsColors.TextSecondary,
                fontSize = 11.sp
            )

            Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                Text("플랫폼:", color = StsmsColors.TextSecondary, fontSize = 11.sp)
                SmallActionChip("💬 Kakao OpenBuilder", active = platform == "KAKAO") { platform = "KAKAO" }
                SmallActionChip("🤖 Discord Bot", active = platform == "DISCORD") { platform = "DISCORD" }
                Spacer(modifier = Modifier.width(8.dp))
                Text("현재 시간대:", color = StsmsColors.TextSecondary, fontSize = 11.sp)
                SmallActionChip("오전(AM)", active = currentSession == SessionSlot.AM) { currentSession = SessionSlot.AM }
                SmallActionChip("오후(PM)", active = currentSession == SessionSlot.PM) { currentSession = SessionSlot.PM }
            }

            OutlinedTextField(
                value = messageText,
                onValueChange = { messageText = it },
                label = { Text("메신저 수신 텍스트 (예: #TASK-004 현장완료 또는 태그 없음)") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )

            Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                SmallActionChip("1순위: #TASK-001 태그") {
                    messageText = "#TASK-001 서초 센서 사진 업로드"
                    fileName = "kakao_seocho_sensor.jpg"
                }
                SmallActionChip("2순위: 태그 없음(현재 세션 자동)") {
                    messageText = "현장 점검 사진 올립니다"
                    fileName = "discord_session_auto.jpg"
                }
            }

            OutlinedTextField(
                value = fileName,
                onValueChange = { fileName = it },
                label = { Text("업로드 이미지 파일명") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )

            if (resultLog != null) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(8.dp))
                        .background(StsmsColors.BgSecondary)
                        .border(1.dp, StsmsColors.AccentCyan, RoundedCornerShape(8.dp))
                        .padding(10.dp)
                ) {
                    Text(text = resultLog!!, color = StsmsColors.AccentCyan, fontSize = 11.sp)
                }
            }

            if (pendingCandidates.isNotEmpty()) {
                Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Text("3순위 대화형 선택 버튼 (Quick Reply):", color = StsmsColors.AccentAmber, fontSize = 11.sp, fontWeight = FontWeight.Bold)
                    pendingCandidates.forEach { cand ->
                        SmallActionChip("[${cand.taskId}] ${cand.title.take(24)}") {
                            resultLog = onExecuteWebhook(platform, currentSession, messageText, fileName, cand.taskId)
                            pendingCandidates = emptyList()
                        }
                    }
                }
            }

            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                TextButton(onClick = onDismiss) {
                    Text("닫기", color = StsmsColors.TextSecondary)
                }
                Spacer(modifier = Modifier.width(8.dp))
                Button(
                    onClick = {
                        when (val match = ValidationGates.matchMessengerWebhookTask(tasksOnDate, currentSession, messageText)) {
                            is ValidationGates.WebhookMatchResult.MatchedSingle -> {
                                pendingCandidates = emptyList()
                                resultLog = onExecuteWebhook(platform, currentSession, messageText, fileName, match.task.taskId)
                            }
                            is ValidationGates.WebhookMatchResult.RequiresInteractiveChoice -> {
                                pendingCandidates = match.candidates
                                resultLog = "[3순위 대화형 선택 요청] ${match.promptMessage}"
                            }
                            is ValidationGates.WebhookMatchResult.NoCandidateFound -> {
                                pendingCandidates = emptyList()
                                resultLog = match.reason
                            }
                        }
                    },
                    colors = ButtonDefaults.buttonColors(containerColor = StsmsColors.AccentPurple)
                ) {
                    Text("웹훅 사진 전송 시뮬레이션", color = Color.White, fontWeight = FontWeight.Bold)
                }
            }
        }
    }
}

@Composable
fun AddTaskOrTemplateDialog(
    selectedDate: LocalDate,
    templates: List<ScheduleTemplate>,
    onDismiss: () -> Unit,
    onCreateAdHocTask: (title: String, session: SessionSlot, layer: CalendarLayer, requiresReport: Boolean, requiresPhoto: Boolean, placeName: String) -> Unit,
    onInstantiateTemplate: (templateId: String) -> Unit
) {
    var title by remember { mutableStateOf("") }
    var placeName by remember { mutableStateOf("강남 통합관제센터") }
    var session by remember { mutableStateOf(SessionSlot.AM) }
    var layer by remember { mutableStateOf(CalendarLayer.WORK) }
    var requiresReport by remember { mutableStateOf(true) }
    var requiresPhoto by remember { mutableStateOf(false) }

    Dialog(onDismissRequest = onDismiss) {
        Column(
            modifier = Modifier
                .widthIn(min = 350.dp, max = 540.dp)
                .clip(RoundedCornerShape(14.dp))
                .background(StsmsColors.BgCard)
                .border(1.5.dp, StsmsColors.AccentCyan, RoundedCornerShape(14.dp))
                .padding(18.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            Text(
                text = "➕ 돌발 과업 추가 [REQ-06] & D-Day 템플릿 생성 [REQ-02]",
                color = StsmsColors.AccentCyan,
                fontSize = 15.sp,
                fontWeight = FontWeight.Bold
            )

            // 1. D-Day 템플릿 원클릭 인스턴스화
            Text("D-Day 마스터 템플릿 일괄 생성 (기준일: $selectedDate):", color = StsmsColors.TextSecondary, fontSize = 11.sp)
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                templates.forEach { tmpl ->
                    SmallActionChip("📋 ${tmpl.templateName}") {
                        onInstantiateTemplate(tmpl.templateId)
                        onDismiss()
                    }
                }
            }

            HorizontalDivider(color = StsmsColors.BorderSubtle)

            // 2. 단일 돌발 과업 등록
            Text("신규 돌발/수동 과업 캡슐 생성:", color = StsmsColors.TextSecondary, fontSize = 11.sp)
            OutlinedTextField(
                value = title,
                onValueChange = { title = it },
                label = { Text("과업 제목") },
                placeholder = { Text("예: 강남 백본 라우터 긴급 패킷 점검") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )
            OutlinedTextField(
                value = placeName,
                onValueChange = { placeName = it },
                label = { Text("수행 장소") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth()
            )

            Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                SmallActionChip("오전(AM)", active = session == SessionSlot.AM) { session = SessionSlot.AM }
                SmallActionChip("오후(PM)", active = session == SessionSlot.PM) { session = SessionSlot.PM }
                Spacer(modifier = Modifier.width(6.dp))
                SmallActionChip("업무(WORK)", active = layer == CalendarLayer.WORK) { layer = CalendarLayer.WORK }
                SmallActionChip("개인(PERSONAL)", active = layer == CalendarLayer.PERSONAL) { layer = CalendarLayer.PERSONAL }
            }

            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                SmallActionChip("보고필수(Gate 1): ${if (requiresReport) "ON" else "OFF"}", active = requiresReport) {
                    requiresReport = !requiresReport
                }
                SmallActionChip("사진필수(Gate 2): ${if (requiresPhoto) "ON" else "OFF"}", active = requiresPhoto) {
                    requiresPhoto = !requiresPhoto
                }
            }

            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                TextButton(onClick = onDismiss) {
                    Text("취소", color = StsmsColors.TextSecondary)
                }
                Spacer(modifier = Modifier.width(8.dp))
                Button(
                    onClick = {
                        val finalTitle = title.ifBlank { "강남 백본 라우터 긴급 패킷 점검" }
                        onCreateAdHocTask(finalTitle, session, layer, requiresReport, requiresPhoto, placeName)
                        onDismiss()
                    },
                    colors = ButtonDefaults.buttonColors(containerColor = StsmsColors.AccentCyan)
                ) {
                    Text("과업 캡슐 생성", color = Color.Black, fontWeight = FontWeight.Bold)
                }
            }
        }
    }
}

@Composable
fun A4ReportPreviewDialog(
    exported: ExportedReportPaths,
    onDismiss: () -> Unit
) {
    var viewHtml by remember { mutableStateOf(false) }

    Dialog(onDismissRequest = onDismiss) {
        Column(
            modifier = Modifier
                .widthIn(min = 380.dp, max = 720.dp)
                .heightIn(min = 420.dp, max = 640.dp)
                .clip(RoundedCornerShape(14.dp))
                .background(StsmsColors.BgCard)
                .border(1.5.dp, StsmsColors.AccentEmerald, RoundedCornerShape(14.dp))
                .padding(18.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = "🖨️ A4 일일 업무 수행 보고서 출력 완료 [REQ-13]",
                    color = StsmsColors.AccentEmerald,
                    fontSize = 15.sp,
                    fontWeight = FontWeight.Bold
                )
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    SmallActionChip("Markdown 미리보기", active = !viewHtml) { viewHtml = false }
                    SmallActionChip("HTML (@media print)", active = viewHtml) { viewHtml = true }
                }
            }

            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(8.dp))
                    .background(StsmsColors.BgSecondary)
                    .padding(8.dp)
            ) {
                Text("저장된 Markdown: ${exported.markdownPath}", color = StsmsColors.AccentCyan, fontSize = 10.sp, fontFamily = FontFamily.Monospace)
                Text("저장된 A4 HTML: ${exported.htmlPath}", color = StsmsColors.AccentEmerald, fontSize = 10.sp, fontFamily = FontFamily.Monospace)
            }

            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .weight(1f)
                    .clip(RoundedCornerShape(8.dp))
                    .background(StsmsColors.BgPrimary)
                    .border(1.dp, StsmsColors.BorderSubtle, RoundedCornerShape(8.dp))
                    .padding(10.dp)
                    .verticalScroll(rememberScrollState())
            ) {
                Text(
                    text = if (viewHtml) exported.htmlContent else exported.markdownContent,
                    color = StsmsColors.TextPrimary,
                    fontSize = 11.sp,
                    fontFamily = FontFamily.Monospace
                )
            }

            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                Button(
                    onClick = onDismiss,
                    colors = ButtonDefaults.buttonColors(containerColor = StsmsColors.AccentEmerald)
                ) {
                    Text("확인", color = Color.White, fontWeight = FontWeight.Bold)
                }
            }
        }
    }
}

@Composable
fun BadgePill(
    text: String,
    color: Color
) {
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(5.dp))
            .background(color.copy(alpha = 0.18f))
            .border(1.dp, color.copy(alpha = 0.7f), RoundedCornerShape(5.dp))
            .padding(horizontal = 6.dp, vertical = 2.dp)
    ) {
        Text(
            text = text,
            color = color,
            fontSize = 10.sp,
            fontWeight = FontWeight.Bold
        )
    }
}
