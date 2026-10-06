/*
 * SPDX-FileCopyrightText: 2025 Andrew Gallant <jamslam@gmail.com> (https://github.com/BurntSushi/bttf)
 *
 * SPDX-License-Identifier: Unlicense OR MIT
 */

//! Week dates under an arbitrary week start.
//!
//! RFC 5545 lets a rule choose the day a week starts on (WKST), so BYWEEKNO
//! cannot use [`jiff::civil::ISOWeekDate`], which always starts on Monday.
//!
//! Ported from `bttf`, with the `anyhow` errors replaced by `Option`: every
//! failure here means a date outside the supported range, which a recurrence
//! rule skips rather than reports.

use jiff::{
    ToSpan,
    civil::{Date, Weekday},
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct WeekDate {
    // The weekday on which this week date calendar starts weeks. This is the
    // key difference from `jiff::civil::ISOWeekDate`, which always starts on
    // Monday.
    start: Weekday,
    year: i16,
    week: i8,
    weekday: Weekday,
}

impl WeekDate {
    /// Create a new week date.
    ///
    /// `year` must be in the range `-9999..=9999`. `week` must be in the range
    /// `1..=53`, although `53` is only valid for "long" years.
    ///
    /// `start` corresponds to how the week numbering scheme determines the
    /// start of a week.
    pub(crate) fn new(start: Weekday, year: i16, week: i8, weekday: Weekday) -> Option<WeekDate> {
        if !(1..=53).contains(&week) || (week == 53 && !is_long_year(start, year)) {
            return None;
        }
        if year == -9999 || year == 9999 {
            let start_of_year = week_start_of_year(start, year)?;
            let mut days = i32::from(week - 1) * 7;
            days += i32::from(weekday.since(start));
            if start_of_year.checked_add(days.days()).is_err() {
                return None;
            }
        }
        Some(WeekDate {
            start,
            year,
            week,
            weekday,
        })
    }

    /// Returns a new week date for the given Gregorian date.
    ///
    /// The week date uses a week numbering scheme where the given weekday
    /// is the first day in the week. That is, the first week date of a year
    /// starts on the given weekday and is the first week whose majority of
    /// days (>= 4) falls in the same Gregorian year.
    #[cfg(test)]
    pub(crate) fn from_date(start: Weekday, date: Date) -> Option<WeekDate> {
        let mut start_of_year = week_start_of_year(start, date.year())?;
        if date < start_of_year {
            start_of_year = week_start_of_year(start, date.year() - 1)?;
        } else {
            let next = date.year() + 1;
            // This fails when `date` is in year 9999, which is Jiff's maximum.
            // But by the same token, in that case, `date` will never be
            // greater than or equal to the start of that year. So we can just
            // ignore it.
            if let Some(next_start_of_year) = week_start_of_year(start, next)
                && date >= next_start_of_year
            {
                start_of_year = next_start_of_year;
            }
        }

        debug_assert!(date >= start_of_year);
        let diff_days = start_of_year.until(date).ok()?.get_days();
        // +1 because weeks are one-indexed. `date` cannot be more than 53
        // weeks after `start_of_year`, so the conversion always fits.
        let week = i8::try_from(diff_days / 7).ok()? + 1;
        // The week date year is guaranteed to match the Gregorian year 4
        // days after the start of the first day of the week date year.
        let year = start_of_year.checked_add(4.days()).ok()?.year();
        let weekday = date.weekday();
        Some(WeekDate {
            start,
            year,
            week,
            weekday,
        })
    }

    /// Converts this week date to its corresponding Gregorian date.
    pub(crate) fn date(self) -> Option<Date> {
        let start_of_year = week_start_of_year(self.start, self.year)?;
        let mut days = i32::from(self.week - 1) * 7;
        days += i32::from(self.weekday.since(self.start));
        start_of_year.checked_add(days.days()).ok()
    }
}

pub(crate) fn weeks_in_year(start: Weekday, year: i16) -> i8 {
    if is_long_year(start, year) { 53 } else { 52 }
}

/// Returns the start of the week that the given date resides in.
///
/// The starting point of the week is determined by `start`.
pub(crate) fn first_of_week(start: Weekday, date: Date) -> Option<Date> {
    let wd = date.weekday();
    if start == wd {
        Some(date)
    } else {
        date.nth_weekday(-1, start).ok()
    }
}

/// Returns the end of the week that the given date resides in.
///
/// The starting point of the week is determined by `start`.
pub(crate) fn last_of_week(start: Weekday, date: Date) -> Option<Date> {
    let last = start.wrapping_sub(1);
    let wd = date.weekday();
    if last == wd {
        Some(date)
    } else {
        date.nth_weekday(1, last).ok()
    }
}

/// Returns true if the given week year (with weeks starting on `start`) is a
/// "long" year or not.
///
/// A "long" year is a year with 53 weeks. Otherwise, it's a "short" year with
/// 52 weeks.
fn is_long_year(start: Weekday, year: i16) -> bool {
    // Inspired by: https://en.wikipedia.org/wiki/ISO_week_date#Weeks_per_year
    let Ok(last) = Date::new(year, 12, 31) else {
        return false;
    };
    let weekday = last.weekday();
    weekday == start.wrapping_add(3) || (last.in_leap_year() && weekday == start.wrapping_add(4))
}

/// Returns the first date in the first week of the given year.
///
/// The date returned is guaranteed to have a weekday equivalent to `start`.
fn week_start_of_year(start: Weekday, year: i16) -> Option<Date> {
    // RFC 5545 says:
    //
    // > A week is defined as a seven day period, starting on the day of the
    // > week defined to be the week start (see WKST). Week number one of the
    // > calendar year is the first week that contains at least four (4) days
    // > in that calendar year.
    //
    // Which means that Jan 4 *must* be in the first week of the year.
    let date_in_first_week = Date::new(year, 1, 4).ok()?;
    // Now find the number of days since the start of the week from a date that
    // we know is in the first week of `year`.
    let diff_from_start = date_in_first_week.weekday().since(start);
    date_in_first_week.checked_sub(diff_from_start.days()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::{ISOWeekDate, Weekday::*};

    /// Just some sanity tests around the boundaries of a year for a weekday
    /// that isn't Sunday/Monday.
    #[test]
    fn week_date_start_of_year() {
        let date = jiff::civil::date(2025, 1, 4);
        let wd = WeekDate::from_date(Saturday, date).unwrap();
        assert_eq!((wd.year, wd.week, wd.weekday), (2025, 1, Saturday));
        assert_eq!(Some(date), wd.date());

        let date = jiff::civil::date(2025, 1, 3);
        let wd = WeekDate::from_date(Saturday, date).unwrap();
        assert_eq!((wd.year, wd.week, wd.weekday), (2024, 53, Friday));
        assert_eq!(Some(date), wd.date());

        let date = jiff::civil::date(2025, 1, 5);
        let wd = WeekDate::from_date(Saturday, date).unwrap();
        assert_eq!((wd.year, wd.week, wd.weekday), (2025, 1, Sunday));
        assert_eq!(Some(date), wd.date());
    }

    /// Tests that for the case of ISO weeks (weeks starting on Monday), the
    /// `WeekDate` gets the same result as Jiff's `ISOWeekDate`.
    #[test]
    fn week_date_start_of_year_consistent_jiff() {
        let years = &[-9999..=-9900, -100..=100, 1800..=2300, 9900..=9999];
        let month_days: &[(i8, i8)] = &[
            (1, 1),
            (1, 2),
            (1, 3),
            (1, 4),
            (1, 5),
            (1, 6),
            (1, 7),
            (1, 8),
            (1, 9),
            (1, 10),
            (7, 1),
            (12, 22),
            (12, 23),
            (12, 24),
            (12, 25),
            (12, 26),
            (12, 27),
            (12, 28),
            (12, 29),
            (12, 30),
            (12, 31),
        ];
        let mkiso = |wd: WeekDate| {
            // This conversion only applies to week dates with
            // Monday as the start of the week.
            assert_eq!(wd.start, Monday);
            ISOWeekDate::new(wd.year, wd.week, wd.weekday).unwrap()
        };
        for range in years.iter().cloned() {
            for year in range {
                for &(month, day) in month_days {
                    let date = jiff::civil::date(year, month, day);
                    let expected = date.iso_week_date();
                    let wd = WeekDate::from_date(Monday, date).unwrap();
                    let got = mkiso(wd);
                    assert_eq!(
                        expected, got,
                        "given {year:04}-{month:02}-{day:02}, expected ISO \
                         week year to be {expected:?}, but got {got:?}",
                    );

                    // While we're here, test that going back to Gregorian
                    // works.
                    assert_eq!(Some(date), wd.date());
                }
            }
        }
    }

    #[test]
    fn boundaries_different_week_starts() {
        assert!(WeekDate::new(Monday, 9999, 52, Friday).is_some());
        assert!(WeekDate::new(Monday, 9999, 52, Saturday).is_none());
        assert!(WeekDate::new(Monday, 9999, 52, Sunday).is_none());
        assert!(WeekDate::new(Monday, 9999, 53, Friday).is_none());

        assert!(WeekDate::new(Monday, -9999, 1, Monday).is_some());
        assert!(WeekDate::new(Tuesday, -9999, 1, Tuesday).is_some());
        assert!(WeekDate::new(Wednesday, -9999, 1, Wednesday).is_some());
        assert!(WeekDate::new(Thursday, -9999, 1, Thursday).is_some());
        assert!(WeekDate::new(Friday, -9999, 1, Friday).is_none());
        assert!(WeekDate::new(Saturday, -9999, 1, Saturday).is_none());
        assert!(WeekDate::new(Sunday, -9999, 1, Sunday).is_none());
    }

    #[test]
    fn boundaries_from_jiff() {
        let (min, max) = (Date::MIN, Date::MAX);

        assert!(WeekDate::from_date(Monday, min).is_some());
        assert!(WeekDate::from_date(Monday, max).is_some());

        assert!(WeekDate::from_date(Tuesday, min).is_none());
        assert!(WeekDate::from_date(Tuesday, max).is_some());

        assert!(WeekDate::from_date(Wednesday, min).is_none());
        assert!(WeekDate::from_date(Wednesday, max).is_some());

        assert!(WeekDate::from_date(Thursday, min).is_none());
        assert!(WeekDate::from_date(Thursday, max).is_some());

        assert!(WeekDate::from_date(Friday, min).is_none());
        assert!(WeekDate::from_date(Friday, max).is_some());

        assert!(WeekDate::from_date(Saturday, min).is_none());
        assert!(WeekDate::from_date(Saturday, max).is_some());

        assert!(WeekDate::from_date(Sunday, min).is_none());
        assert!(WeekDate::from_date(Sunday, max).is_some());
    }
}
