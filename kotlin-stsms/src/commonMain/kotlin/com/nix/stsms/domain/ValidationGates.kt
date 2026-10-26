package com.nix.stsms.domain

import java.security.MessageDigest
import java.time.LocalDate

object ValidationGates {

    /**
     * [Gate 4] 기상 데이터 기반 복장 & 우산 자동 판단 매트릭스 (CORE-02)
     */
    fun evaluateWeatherAssist(
        tempC: Double,
        popPct: Int,
        windSpeedMs: Double = 0.0
    ): WeatherSnapshot {
        val clothing = when {
            tempC <= 4.0 -> "한파/영하권(<=4°C): 패딩, 헤비코트, 방한내의, 장갑·목도리 착용 필수"
            tempC in 4.0..9.9 -> "쌀쌀함(5~9°C): 울코트, 경량패딩, 플리스, 니트 레이어링 권장"
            tempC in 10.0..16.9 -> "선선함(10~16°C): 자켓, 가디건, 맨투맨, 긴팔 셔츠 및 외투 권장"
            tempC in 17.0..22.9 -> "쾌적함(17~22°C): 얇은 가디건, 긴팔 셔츠, 면바지 최적 활동복"
            tempC in 23.0..27.9 -> "더움(23~27°C): 반팔 셔츠, 얇은 면의류 및 통기성 복장 권장"
            else -> "폭염/무더위(>=28°C): 쿨링 기능성 반팔, 모자, 수분 섭취 안내"
        }

        val (umbrellaReq, umbrellaMsg) = if (popPct >= 60 || windSpeedMs >= 9.0) {
            if (windSpeedMs >= 9.0) {
                true to "🔴 [방수우비 권장] 강풍 동반 우천: 우산 파손 주의, 방수자켓/우비 권장"
            } else {
                true to "🔴 [우산 필수] 강수확률 60% 이상: 우산 및 장비 방수 커버 필수 지참"
            }
        } else if (popPct in 30..59) {
            true to "🟡 [우산 권장] 강수확률 30~59%: 휴대용 접이식 우산 구비 권장"
        } else {
            false to "🟢 [우산 불필요] 강수확률 30% 미만: 야외 이동 원활"
        }

        val skyCode = when {
            popPct >= 60 -> "RAINY"
            popPct >= 30 -> "CLOUDY"
            else -> "CLEAR"
        }

        return WeatherSnapshot(
            temperatureCelsius = tempC,
            precipitationProbPercent = popPct,
            weatherSkyCode = skyCode,
            clothingAdvice = clothing,
            umbrellaRequired = umbrellaReq,
            umbrellaAlert = umbrellaMsg
        )
    }

    /**
     * [Gate 1 & Gate 2] 업무 완료 유효성 검증 (REQ-15가, REQ-15다)
     */
    fun validateAndCompleteTask(
        task: TaskCapsule,
        actualCount: Int,
        reportSummary: String?,
        overrideNotes: String? = null,
        nowIso: String,
        sourceChannel: String = "KOTLIN_UI"
    ): Result<Pair<TaskCapsule, TaskAuditLog>> {
        // Gate 1: 보고필요 업무 결과 입력 강제 검증
        if (task.requiresReport) {
            if (actualCount < 1) {
                return Result.failure(
                    IllegalArgumentException("[Gate 1 차단] 보고필요 업무는 실적 횟수(1 이상) 입력이 필수입니다.")
                )
            }
            if (reportSummary == null || reportSummary.trim().length < 2) {
                return Result.failure(
                    IllegalArgumentException("[Gate 1 차단] 보고 결과 요약 텍스트(2자 이상)를 반드시 기입해야 완료할 수 있습니다.")
                )
            }
        }

        // Gate 2: 필수 사진 증빙 첨부 검증
        if (task.requiresPhotoEvidence) {
            val hasPhoto = task.attachments.any { att ->
                att.attachmentType == "MESSENGER_PHOTO" ||
                    (att.mimeType?.startsWith("image/") == true) ||
                    att.fileName.lowercase().endsWith(".jpg") ||
                    att.fileName.lowercase().endsWith(".jpeg") ||
                    att.fileName.lowercase().endsWith(".png") ||
                    att.fileName.lowercase().endsWith(".webp")
            }
            if (!hasPhoto) {
                return Result.failure(
                    IllegalArgumentException("[Gate 2 차단] 사진 보고 필수 업무입니다. 완료 처리 전 현장 증빙 사진을 1장 이상 첨부하십시오.")
                )
            }
        }

        val updatedTask = task.copy(
            status = TaskStatus.COMPLETED,
            reportActualCount = if (task.requiresReport) actualCount else task.reportActualCount,
            reportResultSummary = reportSummary?.trim()?.takeIf { it.isNotEmpty() } ?: task.reportResultSummary,
            specialNotes = overrideNotes?.trim()?.takeIf { it.isNotEmpty() } ?: task.specialNotes,
            lastModifiedAt = nowIso
        )

        val auditLog = TaskAuditLog(
            taskId = task.taskId,
            eventType = "COMPLETED",
            fieldName = "status",
            oldValue = task.status.dbValue,
            newValue = "COMPLETED (실적:${updatedTask.reportActualCount}/${updatedTask.reportTargetCount}, 요약:${updatedTask.reportResultSummary ?: "-"})",
            sourceChannel = sourceChannel,
            occurredAt = nowIso
        )

        return Result.success(updatedTask to auditLog)
    }

    /**
     * [Gate 3] 마스터 스케줄 보존형 일정 조정 (REQ-07, REQ-08)
     * - baselineDate는 절대 변경하지 않음!
     * - executionDate만 변경하고 masterBaselineAffected = false 보장
     */
    fun validateAndRescheduleTask(
        task: TaskCapsule,
        newDate: LocalDate,
        reason: String,
        nowIso: String,
        sourceChannel: String = "KOTLIN_UI"
    ): Result<Triple<TaskCapsule, TaskAdjustment, TaskAuditLog>> {
        val trimmedReason = reason.trim()
        if (trimmedReason.length < 2) {
            return Result.failure(
                IllegalArgumentException("[Gate 3 차단] 일정 조정 사유(2글자 이상)를 반드시 입력해야 합니다.")
            )
        }

        val prevDate = task.executionDate
        val updatedTask = task.copy(
            // baselineDate는 절대 변경하지 않음!
            baselineDate = task.baselineDate,
            executionDate = newDate,
            status = TaskStatus.RESCHEDULED_HOLD,
            masterBaselineAffected = false,
            lastModifiedAt = nowIso
        )

        val adjustment = TaskAdjustment(
            taskId = task.taskId,
            previousDate = prevDate,
            adjustedDate = newDate,
            reason = trimmedReason,
            masterBaselineAffected = false,
            recordedAt = nowIso
        )

        val auditLog = TaskAuditLog(
            taskId = task.taskId,
            eventType = "RESCHEDULED",
            fieldName = "execution_date",
            oldValue = prevDate.toString(),
            newValue = "$newDate (사유: $trimmedReason, 마스터기준일 ${task.baselineDate} 보존)",
            sourceChannel = sourceChannel,
            occurredAt = nowIso
        )

        return Result.success(Triple(updatedTask, adjustment, auditLog))
    }

    /**
     * [REQ-10, 6.2] 메신저 사진 Webhook 자동 매핑 3단계 알고리즘
     * 1순위: 메시지 단축 태그 매칭 (#TASK-001 또는 #001 등)
     * 2순위: 현재 세션(AM/PM)에 IN_SESSION 이면서 requiresPhotoEvidence == true 인 단일 업무 자동 매칭
     * 3순위: 후보가 여러 개이거나 없을 때 대화형 선택 후보 리스트 반환
     */
    sealed class WebhookMatchResult {
        data class MatchedSingle(val task: TaskCapsule, val matchStage: String) : WebhookMatchResult()
        data class RequiresInteractiveChoice(val candidates: List<TaskCapsule>, val promptMessage: String) : WebhookMatchResult()
        data class NoCandidateFound(val reason: String) : WebhookMatchResult()
    }

    fun matchMessengerWebhookTask(
        tasksOnDate: List<TaskCapsule>,
        currentSession: SessionSlot,
        messageText: String
    ): WebhookMatchResult {
        // 1순위: #태그 매칭 (예: #TASK-002, #002, #2)
        val tagRegex = Regex("#([A-Za-z0-9_-]+)")
        val tagMatch = tagRegex.find(messageText)
        if (tagMatch != null) {
            val rawTag = tagMatch.groupValues[1].uppercase()
            val matchedByTag = tasksOnDate.find { t ->
                t.taskId.uppercase() == rawTag ||
                    t.taskId.uppercase().endsWith("-$rawTag") ||
                    t.taskId.uppercase().endsWith(rawTag.padStart(3, '0'))
            }
            if (matchedByTag != null) {
                return WebhookMatchResult.MatchedSingle(
                    matchedByTag,
                    "1순위 단축태그 매칭 (#$rawTag -> ${matchedByTag.taskId})"
                )
            }
        }

        // 2순위: 현재 세션(AM/PM)의 활성(IN_SESSION) + 사진 필수(requiresPhotoEvidence) 업무
        val activePhotoTasks = tasksOnDate.filter { t ->
            t.assignedSession == currentSession &&
                t.status != TaskStatus.COMPLETED &&
                t.requiresPhotoEvidence
        }
        if (activePhotoTasks.size == 1) {
            val single = activePhotoTasks.first()
            return WebhookMatchResult.MatchedSingle(
                single,
                "2순위 현재 세션(${currentSession.name}) 사진필수 단일 업무 자동 매칭 (${single.taskId})"
            )
        }

        // 3순위: 후보가 2개 이상이면 대화형 버튼 선택 요청 반환
        if (activePhotoTasks.size >= 2) {
            return WebhookMatchResult.RequiresInteractiveChoice(
                candidates = activePhotoTasks,
                promptMessage = "당일 ${currentSession.label} 사진보고 대상 업무가 ${activePhotoTasks.size}건 있습니다. 연결할 업무를 선택하세요."
            )
        }

        // 사진 필수가 아니더라도 현재 날짜 미완료 업무들을 대화형 후보로 제시
        val fallbackCandidates = tasksOnDate.filter { it.status != TaskStatus.COMPLETED }
        return if (fallbackCandidates.isNotEmpty()) {
            WebhookMatchResult.RequiresInteractiveChoice(
                candidates = fallbackCandidates,
                promptMessage = "단축 태그가 없어 당일 진행 중인 업무 후보(${fallbackCandidates.size}건) 중 선택을 요청합니다."
            )
        } else {
            WebhookMatchResult.NoCandidateFound("당일 매핑 가능한 미완료 업무가 없습니다.")
        }
    }

    /**
     * [EXT-03] 표준 SHA-256 해시 계산 (FIPS 180-4)
     */
    fun computeSha256Hex(bytes: ByteArray): String {
        val digest = MessageDigest.getInstance("SHA-256").digest(bytes)
        return digest.joinToString("") { "%02x".format(it) }
    }
}
