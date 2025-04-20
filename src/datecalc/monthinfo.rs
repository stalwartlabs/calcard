use crate::icalendar::ICalendarFrequency;

use super::{
    rrule::{NWeekday, RRule},
    utils::pymod,
    yearinfo::YearInfo,
};

#[derive(Debug, Clone)]
pub(crate) struct MonthInfo {
    pub last_year: i32,
    pub last_month: u8,
    pub neg_weekday_mask: Vec<u8>,
}

impl MonthInfo {
    pub fn new(year_info: &YearInfo, month: u8, rrule: &RRule) -> Self {
        let neg_weekday_mask = Self::get_neg_weekday_mask(year_info, month, rrule);
        Self {
            last_year: year_info.year,
            last_month: month,
            neg_weekday_mask,
        }
    }

    fn get_neg_weekday_mask(year_info: &YearInfo, month: u8, rrule: &RRule) -> Vec<u8> {
        let YearInfo {
            year_len,
            month_range,
            weekday_mask,
            ..
        } = year_info;

        // Build up `ranges`
        let mut ranges = vec![];
        if rrule.freq == ICalendarFrequency::Yearly {
            if rrule.by_month.is_empty() {
                ranges.push((0, u32::from(*year_len) - 1));
            } else {
                for month in &rrule.by_month {
                    let month = usize::from(*month);
                    let first = u32::from(month_range[month - 1]);
                    let last = u32::from(month_range[month]);
                    ranges.push((first, last - 1));
                }
            }
        } else if rrule.freq == ICalendarFrequency::Monthly {
            let month = usize::from(month);
            let first = u32::from(month_range[month - 1]);
            let last = u32::from(month_range[month]);
            ranges.push((first, last - 1));
        }

        if ranges.is_empty() {
            return vec![];
        }

        // Weekly frequency won't get here, so we may not
        // care about cross-year weekly periods.
        let mut neg_weekday_mask = vec![0; usize::from(*year_len)];

        // Loop over `ranges`
        for (first, last) in ranges {
            for by_weekday in &rrule.by_weekday {
                // Only check Nth occurrences here
                if let NWeekday::Nth(number, weekday) = by_weekday {
                    let weekday = i16::try_from(weekday.num_days_from_monday())
                        .expect("num_days_from_monday is between 0 and 6 which is covered by i16");
                    let nth_weekday = if *number < 0 {
                        let number = match u32::try_from(-number) {
                            Ok(num) => num,
                            _ => continue,
                        };
                        let nth_last_week = match last.checked_sub((number - 1) * 7) {
                            Some(val) => val,
                            None => continue,
                        };
                        let index = match usize::try_from(nth_last_week) {
                            Ok(i) => i,
                            _ => continue,
                        };
                        let nth_last_weekday = match weekday_mask.get(index) {
                            Some(val) => i16::try_from(*val).expect("values in weekday mask are all between 0-6 which is covered by i32"),
                            None => continue,
                        };

                        // Adjust to get the correct weekday
                        let modulo = u32::try_from(pymod(nth_last_weekday - weekday, 7))
                            .expect("pymod is positive because 7 is the modulus");
                        match nth_last_week.checked_sub(modulo) {
                            Some(val) => val,
                            None => continue,
                        }
                    } else {
                        let number = match u32::try_from(number - 1) {
                            Ok(num) => num,
                            _ => continue,
                        };
                        let nth_first_day = first + number * 7;
                        let index = match usize::try_from(nth_first_day) {
                            Ok(i) => i,
                            _ => continue,
                        };
                        let nth_first_day_weekday = match weekday_mask.get(index) {
                            Some(val) => i16::try_from(*val).expect("values in weekday mask are all between 0-6 which is covered by i32"),
                            None => continue,
                        };

                        // Adjust to get the correct weekday
                        let a = u32::try_from(7 - nth_first_day_weekday + weekday)
                            .expect("to be positive because nth_first_day_weekday is at most 6");
                        nth_first_day + pymod(a, 7)
                    };
                    if first <= nth_weekday && nth_weekday <= last {
                        if let Ok(nth_weekday) = usize::try_from(nth_weekday) {
                            neg_weekday_mask[nth_weekday] = 1;
                        }
                    }
                }
            }
        }

        neg_weekday_mask
    }
}
