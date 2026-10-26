use chrono::{Datelike, NaiveDate};

/// 한국 전통 음력 변환기 (2024~2030)
/// 각 연도별 음력 1월 1일(설날)의 양력 날짜 및 음력 월별 일수(대월 30, 소월 29) + 윤달 정보
struct LunarYearData {
    year: i32,
    lunar_new_year: NaiveDate,
    /// 12개 월 (또는 윤달 포함 13개)의 일수 (29 or 30)
    month_days: &'static [u32],
    /// 윤달이 위치한 월 (0이면 윤달 없음, 예: 6이면 윤6월)
    leap_month: u32,
}

const LUNAR_TABLE: &[LunarYearData] = &[
    LunarYearData {
        year: 2024,
        lunar_new_year: match NaiveDate::from_ymd_opt(2024, 2, 10) {
            Some(d) => d,
            None => unreachable!(),
        },
        month_days: &[30, 29, 30, 30, 29, 30, 29, 30, 29, 30, 29, 30],
        leap_month: 0,
    },
    LunarYearData {
        year: 2025,
        lunar_new_year: match NaiveDate::from_ymd_opt(2025, 1, 29) {
            Some(d) => d,
            None => unreachable!(),
        },
        month_days: &[29, 30, 29, 30, 29, 30, 29, 30, 30, 29, 30, 29, 30], // 윤6월
        leap_month: 6,
    },
    LunarYearData {
        year: 2026,
        lunar_new_year: match NaiveDate::from_ymd_opt(2026, 2, 17) {
            Some(d) => d,
            None => unreachable!(),
        },
        month_days: &[30, 29, 30, 29, 30, 29, 30, 29, 30, 30, 29, 30],
        leap_month: 0,
    },
    LunarYearData {
        year: 2027,
        lunar_new_year: match NaiveDate::from_ymd_opt(2027, 2, 6) {
            Some(d) => d,
            None => unreachable!(),
        },
        month_days: &[30, 29, 30, 29, 30, 29, 29, 30, 29, 30, 30, 29],
        leap_month: 0,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LunarDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub is_leap_month: bool,
}

impl LunarDate {
    pub fn format_short(&self) -> String {
        if self.is_leap_month {
            format!("(윤){}.{}", self.month, self.day)
        } else {
            format!("{}.{}", self.month, self.day)
        }
    }
}

pub fn solar_to_lunar(solar: NaiveDate) -> Option<LunarDate> {
    let year = solar.year();
    let current_data = LUNAR_TABLE.iter().find(|d| d.year == year)?;

    let target_data = if solar < current_data.lunar_new_year {
        LUNAR_TABLE.iter().find(|d| d.year == year - 1)?
    } else {
        current_data
    };

    let mut days_diff = (solar - target_data.lunar_new_year).num_days();
    if days_diff < 0 {
        return None;
    }

    let mut month_idx = 0;
    let mut current_lunar_month = 1;
    let mut is_current_leap = false;

    while month_idx < target_data.month_days.len() {
        let month_len = target_data.month_days[month_idx] as i64;
        if days_diff < month_len {
            return Some(LunarDate {
                year: target_data.year,
                month: current_lunar_month,
                day: (days_diff + 1) as u32,
                is_leap_month: is_current_leap,
            });
        }
        days_diff -= month_len;

        if target_data.leap_month > 0 && current_lunar_month == target_data.leap_month && !is_current_leap {
            is_current_leap = true;
        } else {
            is_current_leap = false;
            current_lunar_month += 1;
        }
        month_idx += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solar_to_lunar_2026() {
        // 2026-10-25 -> lunar around 9.15
        let solar = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();
        let lunar = solar_to_lunar(solar);
        assert!(lunar.is_some());
        let l = lunar.unwrap();
        assert_eq!(l.year, 2026);
        assert_eq!(l.month, 9);
        assert_eq!(l.day, 15);
    }
}
