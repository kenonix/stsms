package com.nix.stsms.calendar

import com.nix.stsms.domain.HolidayAndLeave
import com.nix.stsms.domain.TaskCapsule
import java.time.DayOfWeek
import java.time.LocalDate
import java.time.YearMonth
import java.time.temporal.ChronoUnit

data class LunarDate(
    val year: Int,
    val month: Int,
    val day: Int,
    val isLeapMonth: Boolean
) {
    fun formatShort(): String =
        if (isLeapMonth) "음(윤)$month.$day" else "음 $month.$day"
}

data class CalendarDayCell(
    val solarDate: LocalDate,
    val lunarDate: LunarDate?,
    val isCurrentMonth: Boolean,
    val isToday: Boolean,
    val isSelected: Boolean,
    val holidays: List<HolidayAndLeave>,
    val tasks: List<TaskCapsule>
) {
    val isSundayOrHoliday: Boolean
        get() = solarDate.dayOfWeek == DayOfWeek.SUNDAY ||
            holidays.any { it.itemType == "PUBLIC_HOLIDAY" }

    val isSaturday: Boolean
        get() = solarDate.dayOfWeek == DayOfWeek.SATURDAY
}

object DualCalendarEngine {

    private data class LunarYearData(
        val year: Int,
        val lunarNewYear: LocalDate,
        val monthDays: IntArray,
        val leapMonth: Int // 0 = 윤달 없음
    )

    private val LUNAR_TABLE = listOf(
        LunarYearData(
            year = 2024,
            lunarNewYear = LocalDate.of(2024, 2, 10),
            monthDays = intArrayOf(30, 29, 30, 30, 29, 30, 29, 30, 29, 30, 29, 30),
            leapMonth = 0
        ),
        LunarYearData(
            year = 2025,
            lunarNewYear = LocalDate.of(2025, 1, 29),
            monthDays = intArrayOf(29, 30, 29, 30, 29, 30, 29, 30, 30, 29, 30, 29, 30), // 윤6월
            leapMonth = 6
        ),
        LunarYearData(
            year = 2026,
            lunarNewYear = LocalDate.of(2026, 2, 17),
            monthDays = intArrayOf(30, 29, 30, 29, 30, 29, 30, 29, 30, 30, 29, 30),
            leapMonth = 0
        ),
        LunarYearData(
            year = 2027,
            lunarNewYear = LocalDate.of(2027, 2, 6),
            monthDays = intArrayOf(30, 29, 30, 29, 30, 29, 29, 30, 29, 30, 30, 29),
            leapMonth = 0
        )
    )

    /**
     * 양력 날짜를 한국 전통 음력(윤달 포함)으로 변환
     */
    fun solarToLunar(solar: LocalDate): LunarDate? {
        val currentData = LUNAR_TABLE.find { it.year == solar.year } ?: return null
        val targetData = if (solar.isBefore(currentData.lunarNewYear)) {
            LUNAR_TABLE.find { it.year == solar.year - 1 } ?: return null
        } else {
            currentData
        }

        var daysDiff = ChronoUnit.DAYS.between(targetData.lunarNewYear, solar).toInt()
        if (daysDiff < 0) return null

        var monthIdx = 0
        var currentLunarMonth = 1
        var isCurrentLeap = false

        while (monthIdx < targetData.monthDays.size) {
            val monthLen = targetData.monthDays[monthIdx]
            if (daysDiff < monthLen) {
                return LunarDate(
                    year = targetData.year,
                    month = currentLunarMonth,
                    day = daysDiff + 1,
                    isLeapMonth = isCurrentLeap
                )
            }
            daysDiff -= monthLen

            if (targetData.leapMonth > 0 && currentLunarMonth == targetData.leapMonth && !isCurrentLeap) {
                isCurrentLeap = true
            } else {
                isCurrentLeap = false
                currentLunarMonth += 1
            }
            monthIdx += 1
        }
        return null
    }

    /**
     * 월간 6주(42일) 달력 그리드 생성
     */
    fun buildMonthGrid(
        yearMonth: YearMonth,
        selectedDate: LocalDate,
        today: LocalDate,
        allTasks: List<TaskCapsule>,
        holidays: List<HolidayAndLeave>
    ): List<CalendarDayCell> {
        val firstOfMonth = yearMonth.atDay(1)
        // 일요일(0) ~ 토요일(6) 기준 오프셋
        val startOffset = firstOfMonth.dayOfWeek.value % 7
        val gridStart = firstOfMonth.minusDays(startOffset.toLong())

        return (0 until 42).map { idx ->
            val date = gridStart.plusDays(idx.toLong())
            CalendarDayCell(
                solarDate = date,
                lunarDate = solarToLunar(date),
                isCurrentMonth = YearMonth.from(date) == yearMonth,
                isToday = date == today,
                isSelected = date == selectedDate,
                holidays = holidays.filter { it.targetDate == date },
                tasks = allTasks.filter { it.executionDate == date }
            )
        }
    }
}
