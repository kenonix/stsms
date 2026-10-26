package com.nix.stsms

import androidx.compose.ui.ImageComposeScene
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import androidx.compose.ui.window.rememberWindowState
import com.nix.stsms.db.StsmsRepository
import com.nix.stsms.domain.MapRenderStyle
import com.nix.stsms.domain.SessionSlot
import com.nix.stsms.domain.ViewportLayoutMode
import com.nix.stsms.ui.StsmsApp
import com.nix.stsms.ui.StsmsController
import org.jetbrains.skia.EncodedImageFormat
import java.io.File

fun resolveWorkspaceRoot(): File {
    val cwd = File(System.getProperty("user.dir") ?: ".")
    return if (File(cwd, "stsms_data.db").exists() || File(cwd, "Cargo.toml").exists()) {
        cwd
    } else if (cwd.parentFile != null && (File(cwd.parentFile, "stsms_data.db").exists() || File(cwd.parentFile, "Cargo.toml").exists())) {
        cwd.parentFile
    } else {
        cwd
    }
}

fun main(args: Array<String>) {
    val workspaceRoot = resolveWorkspaceRoot()
    val dbFile = File(workspaceRoot, "stsms_data.db")
    val storageRoot = File(workspaceRoot, "storage_root")
    val repo = StsmsRepository(dbFile = dbFile, storageRootDir = storageRoot)
    val controller = StsmsController(repo = repo)

    if (args.contains("--verify-and-screenshot")) {
        runHeadlessUiVerificationAndScreenshots(controller, storageRoot)
        return
    }

    application {
        val windowState = rememberWindowState(size = DpSize(1520.dp, 940.dp))
        Window(
            onCloseRequest = ::exitApplication,
            state = windowState,
            title = "STSMS v1.0 — Smart Task & Schedule Management System (Kotlin Desktop & Mobile)"
        ) {
            StsmsApp(controller = controller)
        }
    }
}

private fun runHeadlessUiVerificationAndScreenshots(
    controller: StsmsController,
    storageRoot: File
) {
    val screenshotsDir = File(storageRoot, "ui_previews")
    screenshotsDir.mkdirs()

    fun captureScenePng(width: Int, height: Int, density: Float, fileName: String) {
        val scene = ImageComposeScene(
            width = width,
            height = height,
            density = Density(density)
        ) {
            StsmsApp(controller = controller)
        }
        try {
            val img = scene.render(nanoTime = System.nanoTime())
            val encoded = img.encodeToData(EncodedImageFormat.PNG)
                ?: error("PNG encoding failed for $fileName")
            val outFile = File(screenshotsDir, fileName)
            outFile.writeBytes(encoded.bytes)
            println("[UI Screenshot Saved] ${outFile.absolutePath} (${encoded.size} bytes, ${width}x${height})")
        } finally {
            scene.close()
        }
    }

    // 1. 데스크탑 4분할 대시보드 (벡터 GIS 지도) 렌더링 캡처
    controller.viewportMode = ViewportLayoutMode.DESKTOP_4SPLIT
    controller.mapRenderStyle = MapRenderStyle.VECTOR_GIS
    captureScenePng(1600, 980, 1.0f, "desktop_4split_vector_map.png")

    // 2. 데스크탑 4분할 대시보드 (MapSCII 점자 Braille 지도 모드) 렌더링 캡처
    controller.mapRenderStyle = MapRenderStyle.BRAILLE_MAPSCII
    controller.mapZoom = 1.8
    captureScenePng(1600, 980, 1.0f, "desktop_4split_braille_map.png")

    // 3. 실제 기능 조작: 카카오톡 웹훅 사진 첨부 + Gate 1 & 2 완료 처리 + A4 보고서 출력
    val webhookLog = controller.executeMessengerWebhook(
        platform = "KAKAO",
        session = SessionSlot.PM,
        messageText = "#TASK-004 상암 IDS 보안로그 대조 증빙 사진",
        fileName = "kakao_sangam_ids_proof.jpg"
    )
    println("[Webhook Executed] $webhookLog")

    controller.submitCompleteTask(
        taskId = "TASK-004",
        actualCount = 12,
        summary = "3분기 IDS 보안 로그 12건 SHA-256 무결성 검증 완료"
    )

    val reportPaths = controller.exportA4DailyReport(openPreviewModal = false)
    println("[A4 Report Exported] MD=${reportPaths.markdownPath} | HTML=${reportPaths.htmlPath}")

    // 4. 모바일 터치 뷰 (보드 탭) 렌더링 캡처
    controller.viewportMode = ViewportLayoutMode.MOBILE_COMPACT
    controller.mobileTabIndex = 0
    controller.mapRenderStyle = MapRenderStyle.VECTOR_GIS
    captureScenePng(430, 920, 1.0f, "mobile_touch_board.png")

    // 5. 모바일 터치 뷰 (동선 지도 탭) 렌더링 캡처
    controller.mobileTabIndex = 2
    controller.mapZoom = 1.4
    captureScenePng(430, 920, 1.0f, "mobile_touch_map.png")
}
