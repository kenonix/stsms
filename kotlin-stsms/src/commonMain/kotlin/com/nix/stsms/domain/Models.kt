package com.nix.stsms.domain

import kotlinx.serialization.Serializable
import java.time.LocalDate

@Serializable
enum class OperationMode(val dbValue: String, val label: String) {
    SOLO_SPLIT("SOLO_SPLIT", "1인용 [업무/개인 분리]"),
    TEAM_MULTI("TEAM_MULTI", "다수용 [팀 통합 협업]");

    companion object {
        fun fromDb(v: String?): OperationMode =
            entries.find { it.dbValue == v } ?: SOLO_SPLIT
    }
}

@Serializable
enum class CalendarLayer(val dbValue: String, val label: String) {
    WORK("WORK", "업무"),
    PERSONAL("PERSONAL", "개인");

    companion object {
        fun fromDb(v: String?): CalendarLayer =
            entries.find { it.dbValue == v } ?: WORK
    }
}

@Serializable
enum class TaskOrigin(val dbValue: String, val label: String) {
    MANUAL("MANUAL", "수동 등록"),
    TEMPLATE_SCHEDULED("TEMPLATE_SCHEDULED", "D-Day 템플릿"),
    AD_HOC("AD_HOC", "돌발 추가");

    companion object {
        fun fromDb(v: String?): TaskOrigin =
            entries.find { it.dbValue == v } ?: MANUAL
    }
}

@Serializable
enum class TaskStatus(val dbValue: String, val label: String, val shortLabel: String) {
    DRAFT("DRAFT", "임시(DRAFT)", "임시"),
    SCHEDULED("SCHEDULED", "예정(SCHEDULED)", "예정"),
    IN_SESSION("IN_SESSION", "진행중(IN_SESSION)", "진행중"),
    COMPLETED("COMPLETED", "완료(COMPLETED)", "완료"),
    RESCHEDULED_HOLD("RESCHEDULED_HOLD", "연기보류(HOLD)", "연기보류");

    companion object {
        fun fromDb(v: String?): TaskStatus =
            entries.find { it.name.lowercase() == v?.lowercase() || it.dbValue == v } ?: when (v) {
                "InSession" -> IN_SESSION
                "Completed" -> COMPLETED
                "RescheduledHold", "DELAYED" -> RESCHEDULED_HOLD
                "Draft" -> DRAFT
                else -> SCHEDULED
            }
    }
}

@Serializable
enum class SessionSlot(val dbValue: String, val label: String) {
    UNASSIGNED("UNASSIGNED", "미배정"),
    AM("AM", "오전 (AM)"),
    PM("PM", "오후 (PM)");

    companion object {
        fun fromDb(v: String?): SessionSlot = when (v?.uppercase()) {
            "AM" -> AM
            "PM" -> PM
            else -> UNASSIGNED
        }
    }
}

@Serializable
enum class RecurrenceType(val dbValue: String, val label: String) {
    NONE("NONE", "반복 없음"),
    DAILY("DAILY", "매일"),
    MONTHLY("MONTHLY", "매월"),
    YEARLY("YEARLY", "매년");

    companion object {
        fun fromDb(v: String?): RecurrenceType =
            entries.find { it.dbValue == v?.uppercase() } ?: NONE
    }
}

@Serializable
data class RecurrenceConfig(
    val recurrenceType: RecurrenceType = RecurrenceType.NONE,
    val month: Int? = null,
    val day: Int? = null,
    val calendarBase: String = "SOLAR"
)

@Serializable
data class CheckItem(
    val checklistId: Long? = null,
    val label: String,
    val isChecked: Boolean = false,
    val sortOrder: Int = 0
)

@Serializable
data class WeatherSnapshot(
    val temperatureCelsius: Double,
    val precipitationProbPercent: Int,
    val weatherSkyCode: String,
    val clothingAdvice: String,
    val umbrellaRequired: Boolean,
    val umbrellaAlert: String
)

@Serializable
data class TaskAttachment(
    val attachmentId: Long? = null,
    val taskId: String,
    val attachmentType: String, // "EVIDENCE_DOC" | "MESSENGER_PHOTO"
    val sourceChannel: String,  // "LOCAL_FS" | "DISCORD" | "KAKAO"
    val fileName: String,
    val filePath: String,
    val fileSizeBytes: Long? = null,
    val mimeType: String? = null,
    val sha256Hash: String? = null,
    val attachedAt: String
)

data class TaskCapsule(
    val taskId: String,
    val subjectId: String? = null,
    val userId: String = "USR-001",
    val masterTemplateId: String? = null,
    val title: String,
    val description: String? = null,
    val calendarLayer: CalendarLayer = CalendarLayer.WORK,
    val taskOrigin: TaskOrigin = TaskOrigin.MANUAL,
    val status: TaskStatus = TaskStatus.SCHEDULED,
    val visualDdayTag: String? = null,
    val baselineDate: LocalDate,
    val executionDate: LocalDate,
    val assignedSession: SessionSlot = SessionSlot.UNASSIGNED,
    val sessionOrder: Int = 0,
    val masterBaselineAffected: Boolean = false,
    val recurrence: RecurrenceConfig = RecurrenceConfig(),
    val requiresReport: Boolean = false,
    val reportItemName: String? = null,
    val reportTargetCount: Int = 0,
    val reportActualCount: Int = 0,
    val reportResultSummary: String? = null,
    val requiresPhotoEvidence: Boolean = false,
    val customCheckboxes: List<CheckItem> = emptyList(),
    val specialNotes: String? = null,
    val placeName: String? = null,
    val address: String? = null,
    val latitude: Double? = null,
    val longitude: Double? = null,
    val sequenceInSubject: Int = 0,
    val distanceFromPrevKm: Double = 0.0,
    val travelTimeFromPrevMin: Int = 0,
    val weatherSnapshot: WeatherSnapshot? = null,
    val attachments: List<TaskAttachment> = emptyList(),
    val scheduledFixedAt: String,
    val createdAt: String,
    val lastModifiedAt: String
)

data class Subject(
    val subjectId: String,
    val userId: String,
    val subjectName: String,
    val colorHex: String = "#3B82F6",
    val totalDistanceKm: Double = 0.0,
    val estimatedDurationMin: Int = 0,
    val createdAt: String,
    val updatedAt: String
)

data class ScheduleTemplate(
    val templateId: String,
    val userId: String,
    val templateName: String,
    val description: String? = null,
    val createdAt: String
)

data class TaskAdjustment(
    val adjustmentId: Long? = null,
    val taskId: String,
    val previousDate: LocalDate,
    val adjustedDate: LocalDate,
    val reason: String,
    val masterBaselineAffected: Boolean = false,
    val recordedAt: String
)

data class TaskAuditLog(
    val logId: Long? = null,
    val taskId: String,
    val eventType: String, // CREATED, RESCHEDULED, SESSION_MOVED, COMPLETED, EVIDENCE_ATTACHED, CHECKLIST_TOGGLED
    val fieldName: String? = null,
    val oldValue: String? = null,
    val newValue: String? = null,
    val sourceChannel: String = "KOTLIN_UI",
    val occurredAt: String
)

data class HolidayAndLeave(
    val leaveId: Long? = null,
    val userId: String = "USR-001",
    val targetDate: LocalDate,
    val itemType: String, // PUBLIC_HOLIDAY, REGULAR_OFF, ANNUAL_LEAVE, HALF_LEAVE_AM, HALF_LEAVE_PM, SICK_LEAVE
    val title: String,
    val isWorkDisabled: Boolean = true
)

enum class ViewportLayoutMode(val label: String) {
    AUTO_ADAPTIVE("자동 반응형"),
    DESKTOP_4SPLIT("데스크탑 4분할"),
    MOBILE_COMPACT("모바일 터치뷰")
}

enum class MapRenderStyle(val label: String) {
    VECTOR_GIS("벡터 GIS 지도"),
    BRAILLE_MAPSCII("점자(Braille) 지도")
}
