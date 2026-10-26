package com.nix.stsms.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

object StsmsColors {
    val BgPrimary = Color(0xFF0B1120)
    val BgSecondary = Color(0xFF111827)
    val BgCard = Color(0xFF162032)
    val BgCardHover = Color(0xFF1E2D45)
    val BgElevated = Color(0xFF22304A)

    val TextPrimary = Color(0xFFF8FAFC)
    val TextSecondary = Color(0xFF94A3B8)
    val TextMuted = Color(0xFF64748B)

    val AccentCyan = Color(0xFF38BDF8)
    val AccentBlue = Color(0xFF3B82F6)
    val AccentPurple = Color(0xFFA855F7)
    val AccentEmerald = Color(0xFF10B981)
    val AccentAmber = Color(0xFFF59E0B)
    val AccentCoral = Color(0xFFEF4444)

    val BorderSubtle = Color(0xFF283548)
    val BorderFocus = Color(0xFF38BDF8)

    // 지도 전용 컬러
    val MapBg = Color(0xFF070D19)
    val MapWater = Color(0xFF0EA5E9)
    val MapMotorway = Color(0xFFF59E0B)
    val MapPrimaryRoad = Color(0xFF64748B)
    val MapRouteLine = Color(0xFF22D3EE)
    val MapPinActive = Color(0xFFF43F5E)
    val MapPinNormal = Color(0xFF3B82F6)
}

private val StsmsDarkColorScheme = darkColorScheme(
    primary = StsmsColors.AccentCyan,
    onPrimary = Color(0xFF04131D),
    primaryContainer = Color(0xFF0C4A6E),
    onPrimaryContainer = Color(0xFFE0F2FE),
    secondary = StsmsColors.AccentBlue,
    onSecondary = Color.White,
    tertiary = StsmsColors.AccentEmerald,
    background = StsmsColors.BgPrimary,
    onBackground = StsmsColors.TextPrimary,
    surface = StsmsColors.BgCard,
    onSurface = StsmsColors.TextPrimary,
    surfaceVariant = StsmsColors.BgElevated,
    onSurfaceVariant = StsmsColors.TextSecondary,
    error = StsmsColors.AccentCoral,
    outline = StsmsColors.BorderSubtle
)

@Composable
fun StsmsTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = StsmsDarkColorScheme,
        content = content
    )
}
