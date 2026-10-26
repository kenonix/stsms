package com.nix.stsms.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.nix.stsms.calendar.CalendarDayCell
import com.nix.stsms.calendar.DualCalendarEngine
import com.nix.stsms.domain.CalendarLayer
import com.nix.stsms.domain.HolidayAndLeave
import com.nix.stsms.domain.TaskCapsule
import com.nix.stsms.domain.TaskStatus
import com.nix.stsms.ui.theme.StsmsColors
import java.time.LocalDate
import java.time.YearMonth

@Composable
fun DualCalendarCard(
    selectedDate: LocalDate,
    today: LocalDate,
    showLunar: Boolean,
    onToggleLunar: () -> Unit,
    onSelectDate: (LocalDate) -> Unit,
    allTasks: List<TaskCapsule>,
    holidays: List<HolidayAndLeave>,
    modifier: Modifier = Modifier,
    compact: Boolean = false
) {
    val yearMonth = YearMonth.from(selectedDate)
    val cells = DualCalendarEngine.buildMonthGrid(
        yearMonth = yearMonth,
        selectedDate = selectedDate,
        today = today,
        allTasks = allTasks,
        holidays = holidays
    )
    val selectedLunar = DualCalendarEngine.solarToLunar(selectedDate)

    Column(
        modifier = modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .background(StsmsColors.BgCard)
            .border(1.dp, StsmsColors.BorderSubtle, RoundedCornerShape(12.dp))
            .padding(if (compact) 8.dp else 10.dp)
    ) {
        // 상단 헤더: 월 이동 및 음력 토글
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                SmallActionChip("◀") { onSelectDate(selectedDate.minusMonths(1)) }
                Text(
                    text = "${yearMonth.year}년 ${yearMonth.monthValue}월",
                    color = StsmsColors.TextPrimary,
                    fontSize = if (compact) 13.sp else 14.sp,
                    fontWeight = FontWeight.Bold
                )
                SmallActionChip("▶") { onSelectDate(selectedDate.plusMonths(1)) }
                SmallActionChip("오늘") { onSelectDate(today) }
            }

            SmallActionChip(
                label = if (showLunar) "음력 ON" else "음력 OFF",
                active = showLunar,
                onClick = onToggleLunar
            )
        }

        Spacer(modifier = Modifier.height(4.dp))

        // 선택된 날짜의 양력/음력 상세 배지
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(6.dp))
                .background(StsmsColors.BgSecondary)
                .padding(horizontal = 8.dp, vertical = 4.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = "선택일: $selectedDate (${selectedLunar?.formatShort() ?: "음력 -"})",
                color = StsmsColors.AccentCyan,
                fontSize = 11.sp,
                fontWeight = FontWeight.Medium
            )
            val dayHolidays = holidays.filter { it.targetDate == selectedDate }
            if (dayHolidays.isNotEmpty()) {
                Text(
                    text = dayHolidays.joinToString(", ") { "🎌 ${it.title}" },
                    color = StsmsColors.AccentCoral,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Bold
                )
            }
        }

        Spacer(modifier = Modifier.height(6.dp))

        // 요일 헤더 (일 ~ 토)
        val dayLabels = listOf("일", "월", "화", "수", "목", "금", "토")
        Row(modifier = Modifier.fillMaxWidth()) {
            dayLabels.forEachIndexed { idx, label ->
                val color = when (idx) {
                    0 -> StsmsColors.AccentCoral
                    6 -> StsmsColors.AccentBlue
                    else -> StsmsColors.TextSecondary
                }
                Text(
                    text = label,
                    color = color,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Bold,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.weight(1f)
                )
            }
        }

        Spacer(modifier = Modifier.height(4.dp))

        // 6주 x 7일 달력 그리드
        for (week in 0 until 6) {
            Row(
                modifier = Modifier.fillMaxWidth().padding(vertical = 1.dp),
                horizontalArrangement = Arrangement.spacedBy(2.dp)
            ) {
                for (dow in 0 until 7) {
                    val cell = cells[week * 7 + dow]
                    CalendarCellItem(
                        cell = cell,
                        showLunar = showLunar,
                        compact = compact,
                        onClick = { onSelectDate(cell.solarDate) },
                        modifier = Modifier.weight(1f)
                    )
                }
            }
        }
    }
}

@Composable
private fun CalendarCellItem(
    cell: CalendarDayCell,
    showLunar: Boolean,
    compact: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    val bgColor = when {
        cell.isSelected -> StsmsColors.AccentCyan.copy(alpha = 0.22f)
        cell.isToday -> StsmsColors.AccentBlue.copy(alpha = 0.14f)
        !cell.isCurrentMonth -> StsmsColors.BgPrimary.copy(alpha = 0.4f)
        else -> StsmsColors.BgSecondary
    }
    val borderColor = when {
        cell.isSelected -> StsmsColors.AccentCyan
        cell.isToday -> StsmsColors.AccentBlue.copy(alpha = 0.7f)
        else -> Color.Transparent
    }

    val dayColor = when {
        !cell.isCurrentMonth -> StsmsColors.TextMuted.copy(alpha = 0.5f)
        cell.isSundayOrHoliday -> StsmsColors.AccentCoral
        cell.isSaturday -> StsmsColors.AccentBlue
        else -> StsmsColors.TextPrimary
    }

    Column(
        modifier = modifier
            .height(if (compact) 40.dp else 46.dp)
            .clip(RoundedCornerShape(6.dp))
            .background(bgColor)
            .border(1.dp, borderColor, RoundedCornerShape(6.dp))
            .clickable(onClick = onClick)
            .padding(horizontal = 3.dp, vertical = 2.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.SpaceBetween
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = cell.solarDate.dayOfMonth.toString(),
                color = dayColor,
                fontSize = if (compact) 10.sp else 11.sp,
                fontWeight = if (cell.isSelected || cell.isToday) FontWeight.ExtraBold else FontWeight.SemiBold
            )
            // 과업 도트 표시
            if (cell.tasks.isNotEmpty()) {
                Row(horizontalArrangement = Arrangement.spacedBy(2.dp)) {
                    val hasWork = cell.tasks.any { it.calendarLayer == CalendarLayer.WORK }
                    val hasPersonal = cell.tasks.any { it.calendarLayer == CalendarLayer.PERSONAL }
                    val allDone = cell.tasks.all { it.status == TaskStatus.COMPLETED }
                    Box(
                        modifier = Modifier
                            .size(5.dp)
                            .clip(CircleShape)
                            .background(
                                when {
                                    allDone -> StsmsColors.AccentEmerald
                                    hasWork -> StsmsColors.AccentCyan
                                    hasPersonal -> StsmsColors.AccentPurple
                                    else -> StsmsColors.AccentAmber
                                }
                            )
                    )
                }
            }
        }

        val holidayTitle = cell.holidays.firstOrNull()?.title
        if (holidayTitle != null) {
            Text(
                text = holidayTitle.take(4),
                color = if (cell.holidays.first().itemType == "PUBLIC_HOLIDAY") StsmsColors.AccentCoral else StsmsColors.AccentAmber,
                fontSize = 8.sp,
                maxLines = 1,
                overflow = TextOverflow.Clip
            )
        } else if (showLunar && cell.lunarDate != null) {
            // 매월 1일, 15일 또는 선택된 날짜는 음력 강조
            val l = cell.lunarDate
            val lText = if (l.isLeapMonth) "윤${l.month}.${l.day}" else "${l.month}.${l.day}"
            Text(
                text = lText,
                color = if (l.day == 1 || l.day == 15) StsmsColors.AccentCyan.copy(alpha = 0.85f) else StsmsColors.TextMuted,
                fontSize = 8.sp,
                maxLines = 1
            )
        }
    }
}

@Composable
fun SmallActionChip(
    label: String,
    active: Boolean = false,
    accentColor: Color = StsmsColors.AccentCyan,
    onClick: () -> Unit
) {
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(6.dp))
            .background(if (active) accentColor.copy(alpha = 0.22f) else StsmsColors.BgElevated)
            .border(
                width = 1.dp,
                color = if (active) accentColor else StsmsColors.BorderSubtle,
                shape = RoundedCornerShape(6.dp)
            )
            .clickable(onClick = onClick)
            .padding(horizontal = 7.dp, vertical = 3.dp),
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = label,
            color = if (active) accentColor else StsmsColors.TextPrimary,
            fontSize = 11.sp,
            fontWeight = if (active) FontWeight.Bold else FontWeight.Medium
        )
    }
}
