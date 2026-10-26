package com.nix.stsms.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.nix.stsms.calendar.DualCalendarEngine
import com.nix.stsms.db.StsmsRepository
import com.nix.stsms.domain.*
import com.nix.stsms.map.OsmDataset
import com.nix.stsms.report.A4ReportExporter
import com.nix.stsms.report.ExportedReportPaths
import com.nix.stsms.ui.components.*
import com.nix.stsms.ui.theme.StsmsColors
import com.nix.stsms.ui.theme.StsmsTheme
import kotlinx.coroutines.delay
import java.time.LocalDate
import java.time.temporal.ChronoUnit

class StsmsController(
    val repo: StsmsRepository,
    initialDate: LocalDate? = null
) {
    val osmDataset: OsmDataset = OsmDataset.loadBuiltinSeoulMetropolitan()
    val today: LocalDate = initialDate ?: LocalDate.now()

    var allTasks by mutableStateOf(repo.getAllTasks())
        private set
    var subjects by mutableStateOf(repo.getSubjects())
        private set
    var holidays by mutableStateOf(repo.getHolidays())
        private set
    var templates by mutableStateOf(repo.getTemplates())
        private set
    var auditLogs by mutableStateOf(repo.getRecentAuditLogs(40))
        private set
    var adjustments by mutableStateOf(repo.getAdjustments())
        private set

    var selectedDate by mutableStateOf(
        initialDate ?: allTasks.firstOrNull()?.executionDate ?: LocalDate.now()
    )
    var selectedTaskId by mutableStateOf(allTasks.firstOrNull()?.taskId)
    var hoveredTaskId by mutableStateOf<String?>(null)

    var operationMode by mutableStateOf(OperationMode.SOLO_SPLIT)
    var layerFilter by mutableStateOf<CalendarLayer?>(null) // null = 전체
    var showLunar by mutableStateOf(true)
    var viewportMode by mutableStateOf(ViewportLayoutMode.AUTO_ADAPTIVE)
    var mobileTabIndex by mutableStateOf(0) // 0:보드, 1:달력, 2:지도, 3:D-Day/로그, 4:보고서/웹훅

    var mapRenderStyle by mutableStateOf(MapRenderStyle.VECTOR_GIS)
    var mapCenterLat by mutableStateOf(37.4950)
    var mapCenterLon by mutableStateOf(126.9950)
    var mapZoom by mutableStateOf(1.35)

    var activeModal by mutableStateOf<ActiveModal>(ActiveModal.None)
    var modalError by mutableStateOf<String?>(null)
    var statusBanner by mutableStateOf("STSMS v1.0 준비 완료 — 데스크탑 4분할 & 모바일 터치 반응형 UI")

    fun refreshFromDb() {
        allTasks = repo.getAllTasks()
        subjects = repo.getSubjects()
        holidays = repo.getHolidays()
        templates = repo.getTemplates()
        auditLogs = repo.getRecentAuditLogs(40)
        adjustments = repo.getAdjustments()
        if (selectedTaskId == null || allTasks.none { it.taskId == selectedTaskId }) {
            selectedTaskId = tasksForSelectedDate().firstOrNull()?.taskId ?: allTasks.firstOrNull()?.taskId
        }
    }

    fun tasksForSelectedDate(): List<TaskCapsule> {
        return allTasks.filter { t ->
            t.executionDate == selectedDate && (layerFilter == null || t.calendarLayer == layerFilter)
        }
    }

    fun selectedTask(): TaskCapsule? =
        allTasks.find { it.taskId == selectedTaskId } ?: tasksForSelectedDate().firstOrNull()

    fun selectDate(date: LocalDate) {
        selectedDate = date
        val firstOnDate = tasksForSelectedDate().firstOrNull()
        if (firstOnDate != null) {
            selectedTaskId = firstOnDate.taskId
        }
        statusBanner = "📅 선택 일자 변경: $date (${DualCalendarEngine.solarToLunar(date)?.formatShort() ?: ""})"
    }

    fun selectTask(taskId: String) {
        selectedTaskId = taskId
        val t = allTasks.find { it.taskId == taskId }
        if (t?.latitude != null && t.longitude != null) {
            mapCenterLat = t.latitude
            mapCenterLon = t.longitude
        }
    }

    fun centerMapOnSelectedTask() {
        val t = selectedTask()
        if (t?.latitude != null && t.longitude != null) {
            mapCenterLat = t.latitude
            mapCenterLon = t.longitude
            statusBanner = "🎯 지도 중심 이동: ${t.placeName ?: t.title} (${t.latitude}, ${t.longitude})"
        }
    }

    fun moveTaskSession(taskId: String, targetSession: SessionSlot) {
        val task = allTasks.find { it.taskId == taskId } ?: return
        if (task.assignedSession == targetSession) return
        val now = repo.nowIsoSecondPrecision()
        val oldSession = task.assignedSession
        val newStatus = if (targetSession != SessionSlot.UNASSIGNED && task.status == TaskStatus.SCHEDULED) {
            TaskStatus.IN_SESSION
        } else task.status

        val updated = task.copy(
            assignedSession = targetSession,
            status = newStatus,
            lastModifiedAt = now
        )
        repo.saveTask(updated)
        repo.insertAuditLog(
            TaskAuditLog(
                taskId = taskId,
                eventType = "SESSION_MOVED",
                fieldName = "assigned_session",
                oldValue = oldSession.dbValue,
                newValue = targetSession.dbValue,
                sourceChannel = "KOTLIN_UI",
                occurredAt = now
            )
        )
        refreshFromDb()
        statusBanner = "🔄 [${task.taskId}] 세션 이동 완료: ${oldSession.label} → ${targetSession.label}"
    }

    fun toggleChecklist(taskId: String, sortOrder: Int) {
        val task = allTasks.find { it.taskId == taskId } ?: return
        val now = repo.nowIsoSecondPrecision()
        val updatedChecks = task.customCheckboxes.map { chk ->
            if (chk.sortOrder == sortOrder) chk.copy(isChecked = !chk.isChecked) else chk
        }
        val updated = task.copy(customCheckboxes = updatedChecks, lastModifiedAt = now)
        repo.saveTask(updated)
        repo.insertAuditLog(
            TaskAuditLog(
                taskId = taskId,
                eventType = "CHECKLIST_TOGGLED",
                fieldName = "custom_checkboxes",
                oldValue = "sortOrder=$sortOrder",
                newValue = updatedChecks.find { it.sortOrder == sortOrder }?.isChecked.toString(),
                sourceChannel = "KOTLIN_UI",
                occurredAt = now
            )
        )
        refreshFromDb()
        statusBanner = "☑️ [${task.taskId}] 체크리스트 상태가 업데이트되었습니다."
    }

    fun submitCompleteTask(
        taskId: String,
        actualCount: Int,
        summary: String,
        notes: String? = null
    ): Result<TaskCapsule> {
        val task = allTasks.find { it.taskId == taskId }
            ?: return Result.failure(IllegalArgumentException("과업을 찾을 수 없습니다: $taskId"))
        val now = repo.nowIsoSecondPrecision()

        val validated = ValidationGates.validateAndCompleteTask(
            task = task,
            actualCount = actualCount,
            reportSummary = summary,
            overrideNotes = notes,
            nowIso = now,
            sourceChannel = "KOTLIN_UI"
        )
        return validated.onSuccess { (updatedTask, auditLog) ->
            repo.saveTask(updatedTask)
            repo.insertAuditLog(auditLog)
            modalError = null
            activeModal = ActiveModal.None
            refreshFromDb()
            statusBanner = "✅ [Gate 1·2 통과] ${updatedTask.taskId} 완료 처리됨 (요약: ${updatedTask.reportResultSummary ?: "-"})"
        }.onFailure { err ->
            modalError = err.message
            statusBanner = "⛔ ${err.message}"
        }.map { it.first }
    }

    fun submitRescheduleTask(
        taskId: String,
        newDate: LocalDate,
        reason: String
    ): Result<TaskCapsule> {
        val task = allTasks.find { it.taskId == taskId }
            ?: return Result.failure(IllegalArgumentException("과업을 찾을 수 없습니다: $taskId"))
        val now = repo.nowIsoSecondPrecision()

        val validated = ValidationGates.validateAndRescheduleTask(
            task = task,
            newDate = newDate,
            reason = reason,
            nowIso = now,
            sourceChannel = "KOTLIN_UI"
        )
        return validated.onSuccess { (updatedTask, adjustment, auditLog) ->
            repo.saveTask(updatedTask)
            repo.insertAdjustment(adjustment)
            repo.insertAuditLog(auditLog)
            modalError = null
            activeModal = ActiveModal.None
            refreshFromDb()
            statusBanner = "📅 [Gate 3 통과] ${updatedTask.taskId} 실행일 $newDate 연기 (마스터 기준일 ${updatedTask.baselineDate} 불변 유지)"
        }.onFailure { err ->
            modalError = err.message
            statusBanner = "⛔ ${err.message}"
        }.map { it.first }
    }

    fun submitAttachment(
        taskId: String,
        filePathOrName: String,
        channel: String = "LOCAL_FS"
    ): Result<TaskAttachment> {
        val task = allTasks.find { it.taskId == taskId }
            ?: return Result.failure(IllegalArgumentException("과업을 찾을 수 없습니다: $taskId"))

        return repo.attachFileToTask(task, filePathOrName, channel).map { (_, att) ->
            modalError = null
            activeModal = ActiveModal.None
            refreshFromDb()
            statusBanner = "📷 [${task.taskId}] 증빙 저장 완료: ${att.filePath} (SHA256: ${att.sha256Hash?.take(12)}...)"
            att
        }
    }

    fun executeMessengerWebhook(
        platform: String,
        session: SessionSlot,
        messageText: String,
        fileName: String,
        forcedTaskId: String? = null
    ): String {
        val targetTask = if (forcedTaskId != null) {
            allTasks.find { it.taskId == forcedTaskId }
        } else {
            when (val res = ValidationGates.matchMessengerWebhookTask(tasksForSelectedDate(), session, messageText)) {
                is ValidationGates.WebhookMatchResult.MatchedSingle -> res.task
                else -> null
            }
        } ?: return "매핑할 대상 과업을 찾지 못했습니다."

        val attResult = repo.attachFileToTask(targetTask, fileName, platform)
        refreshFromDb()
        return attResult.fold(
            onSuccess = { (_, att) ->
                val msg = "✅ [$platform 웹훅 매핑 완료] ${targetTask.taskId} (${targetTask.title.take(16)}) → ${att.filePath} [SHA256: ${att.sha256Hash?.take(12)}...]"
                statusBanner = msg
                msg
            },
            onFailure = { "오류: ${it.message}" }
        )
    }

    fun createAdHocTask(
        title: String,
        session: SessionSlot,
        layer: CalendarLayer,
        requiresReport: Boolean,
        requiresPhoto: Boolean,
        placeName: String
    ): TaskCapsule {
        val now = repo.nowIsoSecondPrecision()
        val nextNum = allTasks.size + 1
        val newId = "TASK-%03d".format(nextNum)
        val newTask = TaskCapsule(
            taskId = newId,
            subjectId = "SUB-01",
            userId = "USR-001",
            title = title,
            description = "현장 돌발/추가 등록 과업 캡슐 ($now 생성)",
            calendarLayer = layer,
            taskOrigin = TaskOrigin.AD_HOC,
            status = if (session != SessionSlot.UNASSIGNED) TaskStatus.IN_SESSION else TaskStatus.SCHEDULED,
            visualDdayTag = "[돌발추가] : D-00 = 긴급대응",
            baselineDate = selectedDate,
            executionDate = selectedDate,
            assignedSession = session,
            sessionOrder = tasksForSelectedDate().count { it.assignedSession == session } + 1,
            requiresReport = requiresReport,
            reportItemName = if (requiresReport) "긴급 점검 건수" else null,
            reportTargetCount = if (requiresReport) 1 else 0,
            requiresPhotoEvidence = requiresPhoto,
            placeName = placeName,
            latitude = 37.4979,
            longitude = 127.0276,
            weatherSnapshot = ValidationGates.evaluateWeatherAssist(15.2, 20, 2.0),
            scheduledFixedAt = now,
            createdAt = now,
            lastModifiedAt = now
        )
        repo.saveTask(newTask)
        repo.insertAuditLog(
            TaskAuditLog(
                taskId = newId,
                eventType = "CREATED",
                newValue = "돌발 과업 캡슐 생성: $title",
                sourceChannel = "KOTLIN_UI",
                occurredAt = now
            )
        )
        refreshFromDb()
        selectedTaskId = newId
        statusBanner = "➕ [REQ-06] 돌발 과업 생성 완료: [$newId] $title"
        return newTask
    }

    fun instantiateTemplate(templateId: String): TaskCapsule {
        val tmpl = templates.find { it.templateId == templateId }
        val now = repo.nowIsoSecondPrecision()
        val nextNum = allTasks.size + 1
        val newId = "TASK-%03d".format(nextNum)
        val task = TaskCapsule(
            taskId = newId,
            subjectId = "SUB-01",
            userId = "USR-001",
            masterTemplateId = templateId,
            title = "${tmpl?.templateName ?: "템플릿"} 하위 실행 캡슐 #$nextNum",
            description = tmpl?.description ?: "D-Day 템플릿 기반 자동 생성 캡슐",
            calendarLayer = CalendarLayer.WORK,
            taskOrigin = TaskOrigin.TEMPLATE_SCHEDULED,
            status = TaskStatus.SCHEDULED,
            visualDdayTag = "[~${(tmpl?.templateName ?: "템플릿").take(4)}] : D-07 = 정기실행",
            baselineDate = selectedDate,
            executionDate = selectedDate,
            assignedSession = SessionSlot.AM,
            sessionOrder = tasksForSelectedDate().count { it.assignedSession == SessionSlot.AM } + 1,
            requiresReport = true,
            reportItemName = "템플릿 표준 점검 건수",
            reportTargetCount = 2,
            placeName = "강남 테헤란로 데이터허브",
            latitude = 37.5006,
            longitude = 127.0365,
            weatherSnapshot = ValidationGates.evaluateWeatherAssist(16.0, 15, 1.8),
            scheduledFixedAt = now,
            createdAt = now,
            lastModifiedAt = now
        )
        repo.saveTask(task)
        repo.insertAuditLog(
            TaskAuditLog(
                taskId = newId,
                eventType = "CREATED",
                newValue = "템플릿($templateId) 인스턴스화 생성",
                sourceChannel = "KOTLIN_UI",
                occurredAt = now
            )
        )
        refreshFromDb()
        selectedTaskId = newId
        statusBanner = "📋 [REQ-02] 템플릿($templateId) 기반 과업 [$newId] 생성 완료"
        return task
    }

    fun exportA4DailyReport(openPreviewModal: Boolean = true): ExportedReportPaths {
        val exported = A4ReportExporter.exportDailyReportFiles(
            storageRootDir = repo.storageRootDir,
            userName = "Nix (시스템 엔지니어)",
            date = selectedDate,
            tasks = tasksForSelectedDate(),
            subject = subjects.firstOrNull()
        )
        if (openPreviewModal) {
            activeModal = ActiveModal.A4ReportPreview(exported)
        }
        statusBanner = "🖨️ [REQ-13] A4 일간 보고서 출력 완료: ${exported.markdownPath} & ${exported.htmlPath}"
        return exported
    }
}

@Composable
fun StsmsApp(controller: StsmsController) {
    StsmsTheme {
        BoxWithConstraints(
            modifier = Modifier
                .fillMaxSize()
                .background(StsmsColors.BgPrimary)
        ) {
            val rootMaxWidth = maxWidth
            val isMobileViewport = when (controller.viewportMode) {
                ViewportLayoutMode.DESKTOP_4SPLIT -> false
                ViewportLayoutMode.MOBILE_COMPACT -> true
                ViewportLayoutMode.AUTO_ADAPTIVE -> rootMaxWidth < 920.dp
            }

            Column(modifier = Modifier.fillMaxSize()) {
                // 상단 글로벌 네비게이션 바
                StsmsTopAppBar(
                    controller = controller,
                    isMobileViewport = isMobileViewport
                )

                // 메인 콘텐츠 영역 (데스크탑 4분할 vs 모바일 터치 레이아웃)
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .weight(1f)
                        .padding(horizontal = 8.dp, vertical = 6.dp),
                    contentAlignment = Alignment.TopCenter
                ) {
                    if (isMobileViewport) {
                        // 데스크탑 넓은 화면에서 모바일 뷰를 켰을 때 스마트폰 베젤 프레임으로 감싸서 보기 좋게 렌더링
                        val usePhoneFrame = rootMaxWidth >= 600.dp
                        Box(
                            modifier = if (usePhoneFrame) {
                                Modifier
                                    .width(430.dp)
                                    .fillMaxHeight()
                                    .clip(RoundedCornerShape(24.dp))
                                    .border(2.dp, StsmsColors.AccentCyan.copy(alpha = 0.6f), RoundedCornerShape(24.dp))
                                    .background(StsmsColors.BgPrimary)
                                    .padding(6.dp)
                            } else {
                                Modifier.fillMaxSize()
                            }
                        ) {
                            MobileTouchDashboard(controller = controller)
                        }
                    } else {
                        DesktopFourSplitDashboard(controller = controller)
                    }
                }

                // 하단 상태 표시줄
                StsmsStatusFooter(controller = controller)
            }

            // 모달 다이얼로그 렌더링
            when (val modal = controller.activeModal) {
                is ActiveModal.None -> {}
                is ActiveModal.CompleteTask -> {
                    CompleteTaskDialog(
                        task = modal.task,
                        errorMessage = controller.modalError,
                        onDismiss = {
                            controller.modalError = null
                            controller.activeModal = ActiveModal.None
                        },
                        onConfirm = { cnt, summary, notes ->
                            controller.submitCompleteTask(modal.task.taskId, cnt, summary, notes)
                        }
                    )
                }
                is ActiveModal.RescheduleTask -> {
                    RescheduleTaskDialog(
                        task = modal.task,
                        errorMessage = controller.modalError,
                        onDismiss = {
                            controller.modalError = null
                            controller.activeModal = ActiveModal.None
                        },
                        onConfirm = { newDate, reason ->
                            controller.submitRescheduleTask(modal.task.taskId, newDate, reason)
                        }
                    )
                }
                is ActiveModal.AttachEvidence -> {
                    AttachEvidenceDialog(
                        task = modal.task,
                        onDismiss = {
                            controller.modalError = null
                            controller.activeModal = ActiveModal.None
                        },
                        onAttach = { path, channel ->
                            controller.submitAttachment(modal.task.taskId, path, channel)
                        }
                    )
                }
                is ActiveModal.WebhookSimulator -> {
                    WebhookSimulatorDialog(
                        tasksOnDate = controller.tasksForSelectedDate(),
                        onDismiss = { controller.activeModal = ActiveModal.None },
                        onExecuteWebhook = { platform, session, msg, file, forcedId ->
                            controller.executeMessengerWebhook(platform, session, msg, file, forcedId)
                        }
                    )
                }
                is ActiveModal.AddOrInstantiateTask -> {
                    AddTaskOrTemplateDialog(
                        selectedDate = controller.selectedDate,
                        templates = controller.templates,
                        onDismiss = { controller.activeModal = ActiveModal.None },
                        onCreateAdHocTask = { title, session, layer, reqRep, reqPhoto, place ->
                            controller.createAdHocTask(title, session, layer, reqRep, reqPhoto, place)
                        },
                        onInstantiateTemplate = { tmplId ->
                            controller.instantiateTemplate(tmplId)
                        }
                    )
                }
                is ActiveModal.A4ReportPreview -> {
                    A4ReportPreviewDialog(
                        exported = modal.exported,
                        onDismiss = { controller.activeModal = ActiveModal.None }
                    )
                }
            }
        }
    }
}

@Composable
private fun StsmsTopAppBar(
    controller: StsmsController,
    isMobileViewport: Boolean
) {
    val lunar = DualCalendarEngine.solarToLunar(controller.selectedDate)

    if (isMobileViewport) {
        // 모바일 전용 컴팩트 2단 상단바
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .background(StsmsColors.BgSecondary)
                .border(1.dp, StsmsColors.BorderSubtle)
                .padding(horizontal = 10.dp, vertical = 6.dp),
            verticalArrangement = Arrangement.spacedBy(5.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(6.dp)
                ) {
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(6.dp))
                            .background(StsmsColors.AccentCyan.copy(alpha = 0.2f))
                            .border(1.dp, StsmsColors.AccentCyan, RoundedCornerShape(6.dp))
                            .padding(horizontal = 7.dp, vertical = 3.dp)
                    ) {
                        Text(
                            text = "📱 STSMS 모바일",
                            color = StsmsColors.AccentCyan,
                            fontSize = 11.sp,
                            fontWeight = FontWeight.ExtraBold
                        )
                    }
                    SmallActionChip("◀") { controller.selectDate(controller.selectedDate.minusDays(1)) }
                    Text(
                        text = "${controller.selectedDate} (${lunar?.formatShort() ?: "음력"})",
                        color = StsmsColors.TextPrimary,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold
                    )
                    SmallActionChip("▶") { controller.selectDate(controller.selectedDate.plusDays(1)) }
                }

                SmallActionChip(
                    label = "🖥️ 4분할 전환",
                    onClick = { controller.viewportMode = ViewportLayoutMode.DESKTOP_4SPLIT }
                )
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                SmallActionChip(
                    label = when (controller.layerFilter) {
                        null -> "전체"
                        CalendarLayer.WORK -> "업무만"
                        CalendarLayer.PERSONAL -> "개인만"
                    },
                    active = controller.layerFilter != null,
                    onClick = {
                        controller.layerFilter = when (controller.layerFilter) {
                            null -> CalendarLayer.WORK
                            CalendarLayer.WORK -> CalendarLayer.PERSONAL
                            CalendarLayer.PERSONAL -> null
                        }
                    }
                )
                SmallActionChip(
                    label = "➕ 과업추가",
                    accentColor = StsmsColors.AccentCyan,
                    onClick = { controller.activeModal = ActiveModal.AddOrInstantiateTask }
                )
                SmallActionChip(
                    label = "💬 카톡/디스코드 웹훅",
                    accentColor = StsmsColors.AccentPurple,
                    onClick = { controller.activeModal = ActiveModal.WebhookSimulator }
                )
                SmallActionChip(
                    label = "🖨️ A4보고서",
                    accentColor = StsmsColors.AccentEmerald,
                    onClick = { controller.exportA4DailyReport(openPreviewModal = true) }
                )
            }
        }
    } else {
        // 데스크탑 와이드 상단바
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .background(StsmsColors.BgSecondary)
                .border(1.dp, StsmsColors.BorderSubtle)
                .padding(horizontal = 12.dp, vertical = 8.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(6.dp))
                        .background(StsmsColors.AccentCyan.copy(alpha = 0.2f))
                        .border(1.dp, StsmsColors.AccentCyan, RoundedCornerShape(6.dp))
                        .padding(horizontal = 8.dp, vertical = 4.dp)
                ) {
                    Text(
                        text = "STSMS v1.0 통합관제",
                        color = StsmsColors.AccentCyan,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.ExtraBold
                    )
                }

                SmallActionChip("◀") { controller.selectDate(controller.selectedDate.minusDays(1)) }
                Text(
                    text = "${controller.selectedDate} (${lunar?.formatShort() ?: "음력"})",
                    color = StsmsColors.TextPrimary,
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Bold
                )
                SmallActionChip("▶") { controller.selectDate(controller.selectedDate.plusDays(1)) }
            }

            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                SmallActionChip(
                    label = controller.operationMode.label,
                    active = controller.operationMode == OperationMode.TEAM_MULTI,
                    accentColor = StsmsColors.AccentPurple,
                    onClick = {
                        controller.operationMode = if (controller.operationMode == OperationMode.SOLO_SPLIT) {
                            OperationMode.TEAM_MULTI
                        } else {
                            OperationMode.SOLO_SPLIT
                        }
                    }
                )

                SmallActionChip(
                    label = when (controller.layerFilter) {
                        null -> "레이어: 전체"
                        CalendarLayer.WORK -> "레이어: 업무"
                        CalendarLayer.PERSONAL -> "레이어: 개인"
                    },
                    active = controller.layerFilter != null,
                    onClick = {
                        controller.layerFilter = when (controller.layerFilter) {
                            null -> CalendarLayer.WORK
                            CalendarLayer.WORK -> CalendarLayer.PERSONAL
                            CalendarLayer.PERSONAL -> null
                        }
                    }
                )

                SmallActionChip(
                    label = "🖥️ 데스크탑 4분할",
                    active = true,
                    onClick = { controller.viewportMode = ViewportLayoutMode.DESKTOP_4SPLIT }
                )
                SmallActionChip(
                    label = "📱 모바일 뷰",
                    active = false,
                    accentColor = StsmsColors.AccentEmerald,
                    onClick = { controller.viewportMode = ViewportLayoutMode.MOBILE_COMPACT }
                )

                SmallActionChip(
                    label = "➕ 과업/템플릿",
                    accentColor = StsmsColors.AccentCyan,
                    onClick = { controller.activeModal = ActiveModal.AddOrInstantiateTask }
                )
                SmallActionChip(
                    label = "💬 웹훅 사진",
                    accentColor = StsmsColors.AccentPurple,
                    onClick = { controller.activeModal = ActiveModal.WebhookSimulator }
                )
                SmallActionChip(
                    label = "🖨️ A4 보고서",
                    accentColor = StsmsColors.AccentEmerald,
                    onClick = { controller.exportA4DailyReport(openPreviewModal = true) }
                )
            }
        }
    }
}

/**
 * 명세서 7.1절 데스크탑 4분할 레이아웃 (좌측 25% 달력/로그 | 중앙 48% D-Day & OSM 지도 | 우측 27% 할일 & 150ms 호버 팝오버 | 하단 AM/PM 보드)
 */
@Composable
private fun DesktopFourSplitDashboard(controller: StsmsController) {
    val tasksOnDate = controller.tasksForSelectedDate()

    Column(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        // 상단 3열 (좌측 25% / 중앙 48% / 우측 27%)
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .weight(0.70f),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            // [1. 좌측 25%] 양·음력 듀얼 캘린더 + 초 단위 불변 감사 로그
            Column(
                modifier = Modifier
                    .weight(0.25f)
                    .fillMaxHeight(),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                DualCalendarCard(
                    selectedDate = controller.selectedDate,
                    today = controller.today,
                    showLunar = controller.showLunar,
                    onToggleLunar = { controller.showLunar = !controller.showLunar },
                    onSelectDate = { controller.selectDate(it) },
                    allTasks = controller.allTasks,
                    holidays = controller.holidays
                )

                AuditLogCard(
                    auditLogs = controller.auditLogs,
                    adjustments = controller.adjustments,
                    modifier = Modifier.weight(1f)
                )
            }

            // [2. 중앙 48%] 상단 D-Day 타임라인 + 하단 OSM 동선 지도 (벡터 GIS / 점자 Braille 전환)
            Column(
                modifier = Modifier
                    .weight(0.48f)
                    .fillMaxHeight(),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                DDayTimelinePanel(
                    tasks = controller.allTasks,
                    selectedTaskId = controller.selectedTaskId,
                    onSelectTask = { controller.selectTask(it) },
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(150.dp)
                )

                OsmMapView(
                    dataset = controller.osmDataset,
                    subject = controller.subjects.firstOrNull(),
                    tasks = tasksOnDate,
                    selectedTaskId = controller.selectedTaskId,
                    renderStyle = controller.mapRenderStyle,
                    onToggleRenderStyle = { controller.mapRenderStyle = it },
                    centerLat = controller.mapCenterLat,
                    centerLon = controller.mapCenterLon,
                    zoom = controller.mapZoom,
                    onPan = { dLat, dLon ->
                        controller.mapCenterLat += dLat
                        controller.mapCenterLon += dLon
                    },
                    onZoom = { controller.mapZoom = it },
                    onCenterSelectedTask = { controller.centerMapOnSelectedTask() },
                    onSelectTask = { controller.selectTask(it) },
                    modifier = Modifier.weight(1f)
                )
            }

            // [3. 우측 27%] 당일/예정 할 일 리스트 + 150ms 디바운스 호버 상세 팝오버 & 기상 가이드
            TaskListAndHoverInspectorPane(
                controller = controller,
                tasksOnDate = tasksOnDate,
                modifier = Modifier
                    .weight(0.27f)
                    .fillMaxHeight()
            )
        }

        // [4. 하단 도크 30%] 오전(AM) / 오후(PM) 2열 드래그·전환 작업 보드 [REQ-09]
        AmPmSessionDockBoard(
            controller = controller,
            tasksOnDate = tasksOnDate,
            modifier = Modifier
                .fillMaxWidth()
                .weight(0.30f)
        )
    }
}

/**
 * 모바일 전용 터치 최적화 5탭 대시보드
 */
@Composable
private fun MobileTouchDashboard(controller: StsmsController) {
    val tasksOnDate = controller.tasksForSelectedDate()
    val activeTask = controller.selectedTask()

    Column(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.SpaceBetween
    ) {
        // 모바일 상단 기상·복장·우산 요약 배너 (CORE-02)
        activeTask?.weatherSnapshot?.let { w ->
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(10.dp))
                    .background(
                        if (w.umbrellaRequired) StsmsColors.AccentCoral.copy(alpha = 0.16f)
                        else StsmsColors.AccentEmerald.copy(alpha = 0.14f)
                    )
                    .border(
                        1.dp,
                        if (w.umbrellaRequired) StsmsColors.AccentCoral else StsmsColors.AccentEmerald,
                        RoundedCornerShape(10.dp)
                    )
                    .padding(horizontal = 10.dp, vertical = 6.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "${w.umbrellaAlert} (${"%.1f".format(w.temperatureCelsius)}°C / 강수 ${w.precipitationProbPercent}%)",
                        color = if (w.umbrellaRequired) StsmsColors.AccentCoral else StsmsColors.AccentEmerald,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold
                    )
                    Text(
                        text = w.clothingAdvice,
                        color = StsmsColors.TextPrimary,
                        fontSize = 10.sp,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
            }
            Spacer(modifier = Modifier.height(6.dp))
        }

        // 모바일 탭 콘텐츠 영역
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f)
        ) {
            when (controller.mobileTabIndex) {
                0 -> {
                    // 탭 0: 오전/오후 터치 작업 보드
                    LazyColumn(
                        modifier = Modifier.fillMaxSize(),
                        verticalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        item {
                            Text(
                                text = "🌅 오전 작업 세션 (AM) — ${tasksOnDate.count { it.assignedSession == SessionSlot.AM }}건",
                                color = StsmsColors.AccentCyan,
                                fontSize = 13.sp,
                                fontWeight = FontWeight.Bold
                            )
                        }
                        items(tasksOnDate.filter { it.assignedSession == SessionSlot.AM }) { task ->
                            SessionTaskCard(controller = controller, task = task)
                        }
                        item {
                            Spacer(modifier = Modifier.height(4.dp))
                            Text(
                                text = "🌇 오후 작업 세션 (PM) — ${tasksOnDate.count { it.assignedSession == SessionSlot.PM }}건",
                                color = StsmsColors.AccentAmber,
                                fontSize = 13.sp,
                                fontWeight = FontWeight.Bold
                            )
                        }
                        items(tasksOnDate.filter { it.assignedSession == SessionSlot.PM }) { task ->
                            SessionTaskCard(controller = controller, task = task)
                        }
                        val unassigned = tasksOnDate.filter { it.assignedSession == SessionSlot.UNASSIGNED }
                        if (unassigned.isNotEmpty()) {
                            item {
                                Text(
                                    text = "📌 미배정 대기 과업 — ${unassigned.size}건",
                                    color = StsmsColors.TextSecondary,
                                    fontSize = 12.sp,
                                    fontWeight = FontWeight.Bold
                                )
                            }
                            items(unassigned) { task ->
                                SessionTaskCard(controller = controller, task = task)
                            }
                        }
                    }
                }
                1 -> {
                    // 탭 1: 양·음력 듀얼 달력 + 당일 과업 요약
                    Column(
                        modifier = Modifier.fillMaxSize(),
                        verticalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        DualCalendarCard(
                            selectedDate = controller.selectedDate,
                            today = controller.today,
                            showLunar = controller.showLunar,
                            onToggleLunar = { controller.showLunar = !controller.showLunar },
                            onSelectDate = { controller.selectDate(it) },
                            allTasks = controller.allTasks,
                            holidays = controller.holidays,
                            compact = true
                        )
                        LazyColumn(
                            modifier = Modifier.weight(1f),
                            verticalArrangement = Arrangement.spacedBy(6.dp)
                        ) {
                            items(tasksOnDate) { task ->
                                SessionTaskCard(controller = controller, task = task)
                            }
                        }
                    }
                }
                2 -> {
                    // 탭 2: 터치 OSM 동선 지도 (벡터 GIS + 점자 Braille)
                    OsmMapView(
                        dataset = controller.osmDataset,
                        subject = controller.subjects.firstOrNull(),
                        tasks = tasksOnDate,
                        selectedTaskId = controller.selectedTaskId,
                        renderStyle = controller.mapRenderStyle,
                        onToggleRenderStyle = { controller.mapRenderStyle = it },
                        centerLat = controller.mapCenterLat,
                        centerLon = controller.mapCenterLon,
                        zoom = controller.mapZoom,
                        onPan = { dLat, dLon ->
                            controller.mapCenterLat += dLat
                            controller.mapCenterLon += dLon
                        },
                        onZoom = { controller.mapZoom = it },
                        onCenterSelectedTask = { controller.centerMapOnSelectedTask() },
                        onSelectTask = { controller.selectTask(it) },
                        modifier = Modifier.fillMaxSize()
                    )
                }
                3 -> {
                    // 탭 3: D-Day 타임라인 & 초 단위 감사 로그
                    Column(
                        modifier = Modifier.fillMaxSize(),
                        verticalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        DDayTimelinePanel(
                            tasks = controller.allTasks,
                            selectedTaskId = controller.selectedTaskId,
                            onSelectTask = { controller.selectTask(it) },
                            modifier = Modifier
                                .fillMaxWidth()
                                .weight(0.45f)
                        )
                        AuditLogCard(
                            auditLogs = controller.auditLogs,
                            adjustments = controller.adjustments,
                            modifier = Modifier
                                .fillMaxWidth()
                                .weight(0.55f)
                        )
                    }
                }
                4 -> {
                    // 탭 4: A4 일일 보고서 출력 & 메신저 웹훅 빠른 실행 센터
                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .clip(RoundedCornerShape(12.dp))
                            .background(StsmsColors.BgCard)
                            .border(1.dp, StsmsColors.BorderSubtle, RoundedCornerShape(12.dp))
                            .padding(14.dp),
                        verticalArrangement = Arrangement.spacedBy(10.dp)
                    ) {
                        Text(
                            text = "📄 모바일 현장 보고서 & 메신저 사진 연동 센터",
                            color = StsmsColors.AccentCyan,
                            fontSize = 15.sp,
                            fontWeight = FontWeight.Bold
                        )
                        Button(
                            onClick = { controller.exportA4DailyReport(openPreviewModal = true) },
                            colors = ButtonDefaults.buttonColors(containerColor = StsmsColors.AccentEmerald),
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            Text("🖨️ 당일 A4 업무보고서 (.md & .html) 즉시 생성", color = Color.White, fontWeight = FontWeight.Bold)
                        }
                        Button(
                            onClick = { controller.activeModal = ActiveModal.WebhookSimulator },
                            colors = ButtonDefaults.buttonColors(containerColor = StsmsColors.AccentPurple),
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            Text("💬 카카오톡 / 디스코드 현장 사진 웹훅 전송", color = Color.White, fontWeight = FontWeight.Bold)
                        }
                        Button(
                            onClick = { controller.activeModal = ActiveModal.AddOrInstantiateTask },
                            colors = ButtonDefaults.buttonColors(containerColor = StsmsColors.AccentBlue),
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            Text("➕ 돌발 과업 추가 / D-Day 템플릿 인스턴스화", color = Color.White, fontWeight = FontWeight.Bold)
                        }
                    }
                }
            }
        }

        Spacer(modifier = Modifier.height(6.dp))

        // 모바일 하단 5탭 네비게이션 바
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(12.dp))
                .background(StsmsColors.BgSecondary)
                .border(1.dp, StsmsColors.BorderSubtle, RoundedCornerShape(12.dp))
                .padding(vertical = 6.dp, horizontal = 4.dp),
            horizontalArrangement = Arrangement.SpaceEvenly
        ) {
            val tabs = listOf(
                "📋 보드",
                "📅 달력",
                "🗺️ 동선지도",
                "⏱️ D-Day/로그",
                "📄 보고/웹훅"
            )
            tabs.forEachIndexed { idx, label ->
                SmallActionChip(
                    label = label,
                    active = controller.mobileTabIndex == idx,
                    onClick = { controller.mobileTabIndex = idx }
                )
            }
        }
    }
}

@Composable
private fun DDayTimelinePanel(
    tasks: List<TaskCapsule>,
    selectedTaskId: String?,
    onSelectTask: (String) -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(12.dp))
            .background(StsmsColors.BgCard)
            .border(1.dp, StsmsColors.BorderSubtle, RoundedCornerShape(12.dp))
            .padding(10.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = "⏱️ D-Day 마스터 타임라인 & 기준일 보존 트래커 [REQ-02, REQ-03, REQ-07]",
                color = StsmsColors.TextPrimary,
                fontSize = 13.sp,
                fontWeight = FontWeight.Bold
            )
            Text(
                text = "총 ${tasks.size}개 캡슐 (마스터 기준일 왜곡 0건)",
                color = StsmsColors.AccentEmerald,
                fontSize = 11.sp,
                fontWeight = FontWeight.SemiBold
            )
        }

        Spacer(modifier = Modifier.height(6.dp))

        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            verticalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            items(tasks) { t ->
                val isSelected = t.taskId == selectedTaskId
                val delayedDays = ChronoUnit.DAYS.between(t.baselineDate, t.executionDate)
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(6.dp))
                        .background(if (isSelected) StsmsColors.BgCardHover else StsmsColors.BgSecondary)
                        .border(
                            1.dp,
                            if (isSelected) StsmsColors.AccentCyan else Color.Transparent,
                            RoundedCornerShape(6.dp)
                        )
                        .clickable { onSelectTask(t.taskId) }
                        .padding(horizontal = 8.dp, vertical = 5.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                        modifier = Modifier.weight(1f)
                    ) {
                        BadgePill(
                            text = t.visualDdayTag ?: "[일반] : D-Day",
                            color = if (t.calendarLayer == CalendarLayer.WORK) StsmsColors.AccentCyan else StsmsColors.AccentPurple
                        )
                        Text(
                            text = "${t.taskId} · ${t.title}",
                            color = StsmsColors.TextPrimary,
                            fontSize = 11.sp,
                            fontWeight = FontWeight.Medium,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }

                    Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                        Text(
                            text = if (delayedDays == 0L) {
                                "기준=실행: ${t.executionDate}"
                            } else {
                                "기준:${t.baselineDate}(불변) → 실행:${t.executionDate}(+${delayedDays}일)"
                            },
                            color = if (delayedDays == 0L) StsmsColors.TextSecondary else StsmsColors.AccentAmber,
                            fontSize = 10.sp,
                            fontWeight = FontWeight.SemiBold
                        )
                        StatusBadge(t.status)
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
private fun TaskListAndHoverInspectorPane(
    controller: StsmsController,
    tasksOnDate: List<TaskCapsule>,
    modifier: Modifier = Modifier
) {
    // 150ms 디바운스 호버 상태 [REQ-04]
    var candidateHoverId by remember { mutableStateOf<String?>(null) }
    LaunchedEffect(candidateHoverId) {
        if (candidateHoverId != null) {
            delay(150L) // 명세서 7.2절: 150ms 디바운스 후 팝오버 표시
            controller.hoveredTaskId = candidateHoverId
        } else {
            controller.hoveredTaskId = null
        }
    }

    val inspectedTask = controller.allTasks.find { it.taskId == (controller.hoveredTaskId ?: controller.selectedTaskId) }
        ?: tasksOnDate.firstOrNull()

    Column(
        modifier = modifier
            .clip(RoundedCornerShape(12.dp))
            .background(StsmsColors.BgCard)
            .border(1.dp, StsmsColors.BorderSubtle, RoundedCornerShape(12.dp))
            .padding(10.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        Text(
            text = "📋 당일 과업 캡슐 (${tasksOnDate.size}건) & 150ms 호버 인스펙터",
            color = StsmsColors.TextPrimary,
            fontSize = 13.sp,
            fontWeight = FontWeight.Bold
        )

        // 상단 과업 목록
        LazyColumn(
            modifier = Modifier
                .fillMaxWidth()
                .weight(0.46f),
            verticalArrangement = Arrangement.spacedBy(5.dp)
        ) {
            items(tasksOnDate) { task ->
                val isSelected = task.taskId == controller.selectedTaskId
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(8.dp))
                        .background(if (isSelected) StsmsColors.BgCardHover else StsmsColors.BgSecondary)
                        .border(
                            1.dp,
                            if (isSelected) StsmsColors.AccentCyan else StsmsColors.BorderSubtle,
                            RoundedCornerShape(8.dp)
                        )
                        .pointerInput(task.taskId) {
                            awaitPointerEventScope {
                                while (true) {
                                    val event = awaitPointerEvent()
                                    if (event.type == PointerEventType.Enter) {
                                        candidateHoverId = task.taskId
                                    } else if (event.type == PointerEventType.Exit && candidateHoverId == task.taskId) {
                                        candidateHoverId = null
                                    }
                                }
                            }
                        }
                        .clickable { controller.selectTask(task.taskId) }
                        .padding(8.dp),
                    verticalArrangement = Arrangement.spacedBy(4.dp)
                ) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Row(horizontalArrangement = Arrangement.spacedBy(4.dp), verticalAlignment = Alignment.CenterVertically) {
                            BadgePill(
                                text = task.calendarLayer.label,
                                color = if (task.calendarLayer == CalendarLayer.WORK) StsmsColors.AccentBlue else StsmsColors.AccentPurple
                            )
                            Text(
                                text = task.taskId,
                                color = StsmsColors.AccentCyan,
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Bold
                            )
                            Text(
                                text = "[${task.assignedSession.label}]",
                                color = StsmsColors.TextSecondary,
                                fontSize = 10.sp
                            )
                        }
                        StatusBadge(task.status)
                    }

                    Text(
                        text = task.title,
                        color = StsmsColors.TextPrimary,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.SemiBold,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
            }
        }

        // 하단 150ms 디바운스 호버 상세 팝오버 카드 [REQ-04, 7.2]
        if (inspectedTask != null) {
            HoverDetailInspectorCard(
                task = inspectedTask,
                isHovered = controller.hoveredTaskId == inspectedTask.taskId,
                onOpenComplete = { controller.activeModal = ActiveModal.CompleteTask(inspectedTask) },
                onOpenReschedule = { controller.activeModal = ActiveModal.RescheduleTask(inspectedTask) },
                onOpenAttach = { controller.activeModal = ActiveModal.AttachEvidence(inspectedTask) },
                modifier = Modifier
                    .fillMaxWidth()
                    .weight(0.54f)
            )
        }
    }
}

@Composable
private fun HoverDetailInspectorCard(
    task: TaskCapsule,
    isHovered: Boolean,
    onOpenComplete: () -> Unit,
    onOpenReschedule: () -> Unit,
    onOpenAttach: () -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(10.dp))
            .background(StsmsColors.BgSecondary)
            .border(
                1.dp,
                if (isHovered) StsmsColors.AccentCyan else StsmsColors.BorderSubtle,
                RoundedCornerShape(10.dp)
            )
            .padding(10.dp),
        verticalArrangement = Arrangement.SpaceBetween
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = if (isHovered) "🔍 [150ms 호버 팝오버] ${task.taskId}" else "📌 [선택 캡슐 상세] ${task.taskId}",
                    color = StsmsColors.AccentCyan,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Bold
                )
                Text(
                    text = "최종수정: ${task.lastModifiedAt.take(19)}",
                    color = StsmsColors.TextMuted,
                    fontSize = 9.sp,
                    fontFamily = FontFamily.Monospace
                )
            }

            Text(
                text = task.title,
                color = StsmsColors.TextPrimary,
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold
            )

            task.description?.let {
                Text(
                    text = it,
                    color = StsmsColors.TextSecondary,
                    fontSize = 11.sp,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )
            }

            // 보고 진행률 및 증빙 상태
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                BadgePill(
                    text = if (task.requiresReport) "실적 ${task.reportActualCount}/${task.reportTargetCount}건" else "보고 불필요",
                    color = if (task.requiresReport) StsmsColors.AccentAmber else StsmsColors.TextSecondary
                )
                BadgePill(
                    text = "📎 증빙 ${task.attachments.size}건${if (task.requiresPhotoEvidence) "(사진필수)" else ""}",
                    color = if (task.attachments.isNotEmpty()) StsmsColors.AccentEmerald else StsmsColors.AccentCoral
                )
                task.placeName?.let {
                    BadgePill(text = "📍 $it", color = StsmsColors.AccentBlue)
                }
            }

            // 기상·복장·우산 가이드 배너 [CORE-02, Gate 4]
            task.weatherSnapshot?.let { w ->
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(6.dp))
                        .background(
                            if (w.umbrellaRequired) StsmsColors.AccentCoral.copy(alpha = 0.15f)
                            else StsmsColors.AccentEmerald.copy(alpha = 0.12f)
                        )
                        .padding(6.dp),
                    verticalArrangement = Arrangement.spacedBy(2.dp)
                ) {
                    Text(
                        text = "${w.umbrellaAlert} (${"%.1f".format(w.temperatureCelsius)}°C / 강수 ${w.precipitationProbPercent}%)",
                        color = if (w.umbrellaRequired) StsmsColors.AccentCoral else StsmsColors.AccentEmerald,
                        fontSize = 10.sp,
                        fontWeight = FontWeight.Bold
                    )
                    Text(
                        text = "👕 ${w.clothingAdvice}",
                        color = StsmsColors.TextPrimary,
                        fontSize = 10.sp,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
            }
        }

        // 하단 즉시 액션 버튼 바
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(6.dp)
        ) {
            SmallActionChip("📷 사진/증빙 첨부", accentColor = StsmsColors.AccentCyan, onClick = onOpenAttach)
            SmallActionChip("📅 일정 연기(Gate3)", accentColor = StsmsColors.AccentAmber, onClick = onOpenReschedule)
            SmallActionChip("✅ 완료 보고(Gate1·2)", accentColor = StsmsColors.AccentEmerald, onClick = onOpenComplete)
        }
    }
}

/**
 * 명세서 7.1절 하단 고정 도크: 오전(AM) / 오후(PM) 2열 작업 보드 [REQ-09]
 */
@Composable
private fun AmPmSessionDockBoard(
    controller: StsmsController,
    tasksOnDate: List<TaskCapsule>,
    modifier: Modifier = Modifier
) {
    val amTasks = tasksOnDate.filter { it.assignedSession == SessionSlot.AM }.sortedBy { it.sessionOrder }
    val pmTasks = tasksOnDate.filter { it.assignedSession == SessionSlot.PM }.sortedBy { it.sessionOrder }

    Row(
        modifier = modifier,
        horizontalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        // 좌측: 오전(AM) 작업 세션 보드
        SessionColumnBoard(
            title = "🌅 오전 작업 세션 (AM DropZone) — ${amTasks.size}건",
            accentColor = StsmsColors.AccentCyan,
            tasks = amTasks,
            controller = controller,
            modifier = Modifier
                .weight(1f)
                .fillMaxHeight()
        )

        // 우측: 오후(PM) 작업 세션 보드
        SessionColumnBoard(
            title = "🌇 오후 작업 세션 (PM DropZone) — ${pmTasks.size}건",
            accentColor = StsmsColors.AccentAmber,
            tasks = pmTasks,
            controller = controller,
            modifier = Modifier
                .weight(1f)
                .fillMaxHeight()
        )
    }
}

@Composable
private fun SessionColumnBoard(
    title: String,
    accentColor: Color,
    tasks: List<TaskCapsule>,
    controller: StsmsController,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(12.dp))
            .background(StsmsColors.BgCard)
            .border(1.dp, StsmsColors.BorderSubtle, RoundedCornerShape(12.dp))
            .padding(10.dp)
    ) {
        Text(
            text = title,
            color = accentColor,
            fontSize = 13.sp,
            fontWeight = FontWeight.Bold
        )
        Spacer(modifier = Modifier.height(6.dp))

        if (tasks.isEmpty()) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Text("배정된 세션 과업이 없습니다.", color = StsmsColors.TextMuted, fontSize = 12.sp)
            }
        } else {
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                verticalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                items(tasks) { task ->
                    SessionTaskCard(controller = controller, task = task)
                }
            }
        }
    }
}

@Composable
private fun SessionTaskCard(
    controller: StsmsController,
    task: TaskCapsule
) {
    val isSelected = task.taskId == controller.selectedTaskId
    val isCompleted = task.status == TaskStatus.COMPLETED

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(8.dp))
            .background(if (isSelected) StsmsColors.BgCardHover else StsmsColors.BgSecondary)
            .border(
                1.dp,
                if (isSelected) StsmsColors.AccentCyan else StsmsColors.BorderSubtle,
                RoundedCornerShape(8.dp)
            )
            .clickable { controller.selectTask(task.taskId) }
            .padding(8.dp),
        verticalArrangement = Arrangement.spacedBy(5.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(6.dp),
                modifier = Modifier.weight(1f)
            ) {
                Text(
                    text = "[${task.taskId}] ${task.title}",
                    color = if (isCompleted) StsmsColors.AccentEmerald else StsmsColors.TextPrimary,
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Bold,
                    textDecoration = if (isCompleted) TextDecoration.LineThrough else TextDecoration.None,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
            StatusBadge(task.status)
        }

        // 동적 체크리스트 [REQ-15나]
        if (task.customCheckboxes.isNotEmpty()) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                task.customCheckboxes.forEach { chk ->
                    Row(
                        modifier = Modifier
                            .clip(RoundedCornerShape(4.dp))
                            .background(StsmsColors.BgPrimary.copy(alpha = 0.6f))
                            .clickable { controller.toggleChecklist(task.taskId, chk.sortOrder) }
                            .padding(horizontal = 6.dp, vertical = 2.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(4.dp)
                    ) {
                        Text(
                            text = if (chk.isChecked) "☑" else "☐",
                            color = if (chk.isChecked) StsmsColors.AccentEmerald else StsmsColors.TextSecondary,
                            fontSize = 11.sp
                        )
                        Text(
                            text = chk.label.take(22),
                            color = StsmsColors.TextSecondary,
                            fontSize = 10.sp
                        )
                    }
                }
            }
        }

        // 카드 하단 조작 버튼 (세션 전환, 사진 입력, 일정 연기, 완료 처리)
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                val targetSlot = if (task.assignedSession == SessionSlot.AM) SessionSlot.PM else SessionSlot.AM
                SmallActionChip(
                    label = if (task.assignedSession == SessionSlot.AM) "→ 오후(PM) 이동" else "← 오전(AM) 이동",
                    onClick = { controller.moveTaskSession(task.taskId, targetSlot) }
                )
                SmallActionChip(
                    label = "📷 사진(${task.attachments.size})",
                    accentColor = StsmsColors.AccentCyan,
                    onClick = {
                        controller.selectTask(task.taskId)
                        controller.activeModal = ActiveModal.AttachEvidence(task)
                    }
                )
                SmallActionChip(
                    label = "📅 연기",
                    accentColor = StsmsColors.AccentAmber,
                    onClick = {
                        controller.selectTask(task.taskId)
                        controller.activeModal = ActiveModal.RescheduleTask(task)
                    }
                )
                SmallActionChip(
                    label = "✅ 완료",
                    accentColor = StsmsColors.AccentEmerald,
                    onClick = {
                        controller.selectTask(task.taskId)
                        controller.activeModal = ActiveModal.CompleteTask(task)
                    }
                )
            }

            task.weatherSnapshot?.let { w ->
                Text(
                    text = "${"%.0f".format(w.temperatureCelsius)}°C·☔${w.precipitationProbPercent}%",
                    color = if (w.umbrellaRequired) StsmsColors.AccentCoral else StsmsColors.TextSecondary,
                    fontSize = 10.sp,
                    fontWeight = FontWeight.SemiBold
                )
            }
        }
    }
}

@Composable
private fun AuditLogCard(
    auditLogs: List<TaskAuditLog>,
    adjustments: List<TaskAdjustment>,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .background(StsmsColors.BgCard)
            .border(1.dp, StsmsColors.BorderSubtle, RoundedCornerShape(12.dp))
            .padding(10.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = "🛡️ 초 단위 불변 감사 로그 [REQ-08]",
                color = StsmsColors.TextPrimary,
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold
            )
            Text(
                text = "조정이력 ${adjustments.size}건 / 로그 ${auditLogs.size}건",
                color = StsmsColors.AccentCyan,
                fontSize = 10.sp
            )
        }
        Spacer(modifier = Modifier.height(6.dp))

        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            verticalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            items(auditLogs) { log ->
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(5.dp))
                        .background(StsmsColors.BgSecondary)
                        .padding(horizontal = 6.dp, vertical = 4.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column(modifier = Modifier.weight(1f)) {
                        Text(
                            text = "[${log.eventType}] ${log.taskId} (${log.sourceChannel})",
                            color = StsmsColors.AccentCyan,
                            fontSize = 10.sp,
                            fontWeight = FontWeight.Bold
                        )
                        Text(
                            text = log.newValue ?: "-",
                            color = StsmsColors.TextSecondary,
                            fontSize = 9.sp,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                    Text(
                        text = log.occurredAt.takeLast(8),
                        color = StsmsColors.TextMuted,
                        fontSize = 9.sp,
                        fontFamily = FontFamily.Monospace
                    )
                }
            }
        }
    }
}

@Composable
private fun StatusBadge(status: TaskStatus) {
    val color = when (status) {
        TaskStatus.COMPLETED -> StsmsColors.AccentEmerald
        TaskStatus.IN_SESSION -> StsmsColors.AccentCyan
        TaskStatus.RESCHEDULED_HOLD -> StsmsColors.AccentAmber
        TaskStatus.SCHEDULED -> StsmsColors.AccentBlue
        TaskStatus.DRAFT -> StsmsColors.TextMuted
    }
    BadgePill(text = status.shortLabel, color = color)
}

@Composable
private fun StsmsStatusFooter(controller: StsmsController) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(StsmsColors.BgSecondary)
            .border(1.dp, StsmsColors.BorderSubtle)
            .padding(horizontal = 12.dp, vertical = 5.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(
            text = controller.statusBanner,
            color = StsmsColors.AccentCyan,
            fontSize = 11.sp,
            fontWeight = FontWeight.Medium,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.weight(1f)
        )
        Text(
            text = "DB: ${controller.repo.dbFile.name} | Storage: ${controller.repo.storageRootDir.name}/",
            color = StsmsColors.TextMuted,
            fontSize = 10.sp,
            fontFamily = FontFamily.Monospace
        )
    }
}
