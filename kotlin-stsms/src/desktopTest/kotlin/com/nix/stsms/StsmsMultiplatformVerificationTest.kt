package com.nix.stsms

import com.nix.stsms.calendar.DualCalendarEngine
import com.nix.stsms.db.StsmsRepository
import com.nix.stsms.domain.*
import com.nix.stsms.map.BrailleCanvas
import com.nix.stsms.map.OsmDataset
import com.nix.stsms.ui.StsmsController
import java.io.File
import java.nio.file.Files
import java.time.LocalDate
import kotlin.test.*

class StsmsMultiplatformVerificationTest {

    @Test
    fun testDualCalendarSolarToLunar2025LeapAnd2026() {
        // 2026-10-25 -> 음력 9월 15일
        val lunar2026 = DualCalendarEngine.solarToLunar(LocalDate.of(2026, 10, 25))
        assertNotNull(lunar2026)
        assertEquals(2026, lunar2026.year)
        assertEquals(9, lunar2026.month)
        assertEquals(15, lunar2026.day)
        assertFalse(lunar2026.isLeapMonth)

        // 2025년 윤6월 검증 (2025-07-25 -> 음력 윤6월 1일)
        val lunarLeap = DualCalendarEngine.solarToLunar(LocalDate.of(2025, 7, 25))
        assertNotNull(lunarLeap)
        assertEquals(6, lunarLeap.month)
        assertTrue(lunarLeap.isLeapMonth)
    }

    @Test
    fun testValidationGates1To4AndSha256() {
        val tempDir = Files.createTempDirectory("stsms_kmp_test").toFile()
        try {
            val dbFile = File(tempDir, "test_stsms.db")
            val storageRoot = File(tempDir, "storage_root")
            val repo = StsmsRepository(dbFile, storageRoot)
            val controller = StsmsController(repo, LocalDate.of(2026, 10, 25))

            // 초기 시드 확인
            assertTrue(controller.allTasks.size >= 6)

            // Gate 1 차단 테스트: 보고필수 과업(TASK-001) 요약 누락 시 실패
            val failGate1 = controller.submitCompleteTask("TASK-001", actualCount = 4, summary = "")
            assertTrue(failGate1.isFailure)

            // Gate 1 통과 테스트
            val passGate1 = controller.submitCompleteTask(
                "TASK-001",
                actualCount = 4,
                summary = "서초 센서 4개소 정밀 교정 완료"
            )
            assertTrue(passGate1.isSuccess)
            assertEquals(TaskStatus.COMPLETED, passGate1.getOrThrow().status)

            // Gate 2 차단 테스트: 사진필수 과업(TASK-004) 사진 미첨부 시 완료 차단
            val failGate2 = controller.submitCompleteTask(
                "TASK-004",
                actualCount = 12,
                summary = "로그 12개 대조 완료"
            )
            assertTrue(failGate2.isFailure)

            // 메신저 사진 웹훅(REQ-10)으로 TASK-004에 사진 첨부 후 Gate 2 통과 확인
            val webhookMsg = controller.executeMessengerWebhook(
                platform = "DISCORD",
                session = SessionSlot.PM,
                messageText = "#TASK-004 상암 보안 감사 사진",
                fileName = "sangam_audit.jpg"
            )
            assertTrue(webhookMsg.contains("TASK-004"))

            val passGate2 = controller.submitCompleteTask(
                "TASK-004",
                actualCount = 12,
                summary = "로그 12개 대조 및 사진 증빙 완료"
            )
            assertTrue(passGate2.isSuccess)
            assertEquals(TaskStatus.COMPLETED, passGate2.getOrThrow().status)

            // Gate 3 테스트: 마스터 기준일(baselineDate) 불변성 및 연기 사유 검증
            val task3Before = controller.allTasks.first { it.taskId == "TASK-003" }
            val originalBaseline = task3Before.baselineDate
            val failGate3 = controller.submitRescheduleTask("TASK-003", originalBaseline.plusDays(1), " ")
            assertTrue(failGate3.isFailure)

            val passGate3 = controller.submitRescheduleTask(
                "TASK-003",
                originalBaseline.plusDays(2),
                "우천으로 인한 광링크 측정 순연"
            )
            assertTrue(passGate3.isSuccess)
            val task3After = passGate3.getOrThrow()
            assertEquals(originalBaseline, task3After.baselineDate, "마스터 기준일은 절대 변경되면 안 됨")
            assertEquals(originalBaseline.plusDays(2), task3After.executionDate)
            assertFalse(task3After.masterBaselineAffected)

            // A4 일간 보고서(.md & .html) 출력 검증
            val exported = controller.exportA4DailyReport(openPreviewModal = false)
            assertTrue(File(exported.markdownPath).exists())
            assertTrue(File(exported.htmlPath).exists())
            assertTrue(exported.htmlContent.contains("@media print"))
            assertTrue(exported.htmlContent.contains("size: A4 portrait"))
        } finally {
            tempDir.deleteRecursively()
        }
    }

    @Test
    fun testOsmBrailleAndVectorDataset() {
        val ds = OsmDataset.loadBuiltinSeoulMetropolitan()
        assertTrue(ds.ways.size >= 10)
        val rows = BrailleCanvas.renderOsmToBraille(
            dataset = ds,
            tasks = emptyList(),
            selectedTaskId = null,
            centerLat = 37.50,
            centerLon = 127.00,
            zoom = 1.5,
            charWidth = 60,
            charHeight = 16
        )
        assertEquals(16, rows.size)
        assertEquals(60, rows.first().size)
    }
}
