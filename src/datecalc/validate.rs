/*
 * SPDX-FileCopyrightText: 2021 Fredrik Meringdal, Ralph Bisschops <https://github.com/fmeringdal/rust-rrule>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::{
    error::ValidationError,
    rrule::{NWeekday, RRule},
};
use crate::{common::timezone::Tz, icalendar::ICalendarFrequency};
use std::ops::RangeInclusive;

pub(crate) static MONTH_RANGE: RangeInclusive<u8> = 1..=12;
pub(crate) static YEAR_RANGE: RangeInclusive<i32> = -10_000..=10_000;

type Validator = &'static dyn Fn(&RRule, &chrono::DateTime<Tz>) -> Result<(), ValidationError>;

const VALIDATION_PIPELINE: [Validator; 11] = [
    &validate_until,
    &validate_by_set_pos,
    &validate_by_month,
    &validate_by_month_day,
    &validate_by_year_day,
    &validate_by_week_number,
    &validate_by_weekday,
    &validate_by_hour,
    &validate_by_minute,
    &validate_by_second,
    &validate_by_easter,
];

/// Check if rules are valid as defined by the RFC and crate limitations.
/// It checks all values in the [`RRule`] and makes sure that they are in
/// the accepted ranges. If the function returns `Ok`, no errors where found.
///
/// This check should always be done and just enforces limits set by the standard.
/// Validation will always be enforced and can not be disabled using feature flags.
pub(crate) fn validate_rrule_forced(
    rrule: &RRule,
    dt_start: &chrono::DateTime<Tz>,
) -> Result<(), ValidationError> {
    VALIDATION_PIPELINE
        .into_iter()
        .try_for_each(|validator| validator(rrule, dt_start))
}

// Until:
// - Timezones are correctly synced as specified in the RFC
// - Value should be later than `dt_start`.
fn validate_until(rrule: &RRule, dt_start: &chrono::DateTime<Tz>) -> Result<(), ValidationError> {
    match rrule.until {
        Some(until) => {
            match dt_start.timezone() {
                Tz::Tz(_) => {
                    if until.timezone() != Tz::UTC {
                        return Err(ValidationError::DtStartUntilMismatchTimezone {
                            dt_start_tz: dt_start.timezone().name().into(),
                            until_tz: until.timezone().name().into(),
                            expected: vec!["UTC".into()],
                        });
                    }
                }
                v => {
                    let allowed_timezones = vec![v, Tz::UTC];
                    if !allowed_timezones.contains(&until.timezone()) {
                        return Err(ValidationError::DtStartUntilMismatchTimezone {
                            dt_start_tz: dt_start.timezone().name().into(),
                            until_tz: until.timezone().name().into(),
                            expected: allowed_timezones
                                .into_iter()
                                .map(|tz| tz.name().into())
                                .collect(),
                        });
                    }
                }
            }

            if until < *dt_start {
                return Err(ValidationError::UntilBeforeStart {
                    until: until.to_rfc3339(),
                    dt_start: dt_start.to_rfc3339(),
                });
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

// By_set_pos:
// - Can be a value from -366 to -1 and 1 to 366 depending on `freq`
fn validate_by_set_pos(
    rrule: &RRule,
    _dt_start: &chrono::DateTime<Tz>,
) -> Result<(), ValidationError> {
    validate_not_equal_for_vec(&0, &rrule.by_set_pos, "BYSETPOS")?;
    let range = match rrule.freq {
        ICalendarFrequency::Yearly | ICalendarFrequency::Daily => -366..=366, // TODO is the daily range correct?
        ICalendarFrequency::Monthly => -31..=31,
        ICalendarFrequency::Weekly => -53..=53,
        ICalendarFrequency::Hourly => -24..=24,
        ICalendarFrequency::Minutely | ICalendarFrequency::Secondly => -60..=60,
    };
    if let Err(value) = validate_range_for_vec_error(&range, &rrule.by_set_pos) {
        return Err(ValidationError::InvalidFieldValueRangeWithFreq {
            field: "BYSETPOS".into(),
            value: value.to_string(),
            freq: rrule.freq,
            start_idx: range.start().to_string(),
            end_idx: range.end().to_string(),
        });
    }
    // - It MUST only be used in conjunction with another BYxxx rule part.
    if !rrule.by_set_pos.is_empty()
        && rrule.by_easter.is_none()
        && rrule.by_hour.is_empty()
        && rrule.by_minute.is_empty()
        && rrule.by_second.is_empty()
        && rrule.by_month_day.is_empty()
        && rrule.by_month.is_empty()
        && rrule.by_year_day.is_empty()
        && rrule.by_week_no.is_empty()
        && rrule.by_weekday.is_empty()
    {
        return Err(ValidationError::BySetPosWithoutByRule);
    }
    Ok(())
}

// By_month:
// - Can be a value from 1 to 12.
fn validate_by_month(
    rrule: &RRule,
    _dt_start: &chrono::DateTime<Tz>,
) -> Result<(), ValidationError> {
    validate_range_for_vec(&MONTH_RANGE, &rrule.by_month, "BYMONTH")
}

// By_month_day:
// - Can be a value from -31 to -1 and 1 to 31.
fn validate_by_month_day(
    rrule: &RRule,
    _dt_start: &chrono::DateTime<Tz>,
) -> Result<(), ValidationError> {
    validate_not_equal_for_vec(&0, &rrule.by_month_day, "BYMONTHDAY")?;
    validate_range_for_vec(&(-31..=31), &rrule.by_month_day, "BYMONTHDAY")?;
    // - MUST NOT be specified when the FREQ rule part is set to WEEKLY.
    //   Validated below
    if !rrule.by_month_day.is_empty() {
        let valid = rrule.freq != ICalendarFrequency::Weekly;
        if !valid {
            return Err(ValidationError::InvalidByRuleAndFrequency {
                by_rule: "BYMONTHDAY".into(),
                freq: rrule.freq,
            });
        }
    }
    Ok(())
}

// By_year_day:
// - Can be a value from -366 to -1 and 1 to 366.
fn validate_by_year_day(
    rrule: &RRule,
    _dt_start: &chrono::DateTime<Tz>,
) -> Result<(), ValidationError> {
    validate_not_equal_for_vec(&0, &rrule.by_year_day, "BYYEARDAY")?;
    validate_range_for_vec(&(-366..=366), &rrule.by_year_day, "BYYEARDAY")?;
    // - MUST NOT be specified when the FREQ rule part is set to DAILY, WEEKLY, or MONTHLY.
    //   Validated below
    if !rrule.by_year_day.is_empty() {
        let valid = !matches!(
            rrule.freq,
            ICalendarFrequency::Monthly | ICalendarFrequency::Weekly | ICalendarFrequency::Daily
        );
        if !valid {
            return Err(ValidationError::InvalidByRuleAndFrequency {
                by_rule: "BYYEARDAY".into(),
                freq: rrule.freq,
            });
        }
    }
    Ok(())
}
// By_week_no:
// - Can be a value from -53 to -1 and 1 to 53.
fn validate_by_week_number(
    rrule: &RRule,
    _dt_start: &chrono::DateTime<Tz>,
) -> Result<(), ValidationError> {
    validate_not_equal_for_vec(&0, &rrule.by_week_no, "BYWEEKNO")?;
    validate_range_for_vec(&(-53..=53), &rrule.by_week_no, "BYWEEKNO")?;
    // - MUST NOT be used when the FREQ rule part is set to anything other than YEARLY.
    //   Validated below
    if !rrule.by_week_no.is_empty() {
        let valid = rrule.freq == ICalendarFrequency::Yearly;
        if !valid {
            return Err(ValidationError::InvalidByRuleAndFrequency {
                by_rule: "BYWEEKNO".into(),
                freq: rrule.freq,
            });
        }
    }
    Ok(())
}

// By_weekday:
// - Check if value for `Nth` is within range.
//   The Range depends on frequency and can only happen weekly, so `/7` from normal count.
fn validate_by_weekday(
    rrule: &RRule,
    _dt_start: &chrono::DateTime<Tz>,
) -> Result<(), ValidationError> {
    let range = match rrule.freq {
        ICalendarFrequency::Yearly | ICalendarFrequency::Daily => (-366 / 7)..=(366 / 7 + 1), // TODO is the daily range correct?
        ICalendarFrequency::Monthly => (-31 / 7)..=(31 / 7 + 1),
        ICalendarFrequency::Weekly => (-53 / 7)..=(53 / 7 + 1),
        ICalendarFrequency::Hourly => (-24 / 7)..=(24 / 7 + 1),
        ICalendarFrequency::Minutely | ICalendarFrequency::Secondly => (-60 / 7)..=(60 / 7 + 1),
    };
    for item in &rrule.by_weekday {
        if let NWeekday::Nth(number, _weekday) = item {
            // If value not in range = error
            if !range.contains(number) {
                return Err(ValidationError::InvalidFieldValueRangeWithFreq {
                    field: "BYDAY".into(),
                    value: number.to_string(),
                    freq: rrule.freq,
                    start_idx: range.start().to_string(),
                    end_idx: range.end().to_string(),
                });
            }
        }
    }
    Ok(())
}

// By_hour:
// - Can be a value from 0 to 23.
fn validate_by_hour(
    rrule: &RRule,
    _dt_start: &chrono::DateTime<Tz>,
) -> Result<(), ValidationError> {
    validate_range_for_vec(&(0..=23), &rrule.by_hour, "BYHOUR")
}

// By_minute:
// - Can be a value from 0 to 59.
fn validate_by_minute(
    rrule: &RRule,
    _dt_start: &chrono::DateTime<Tz>,
) -> Result<(), ValidationError> {
    validate_range_for_vec(&(0..=59), &rrule.by_minute, "BYMINUTE")
}

// By_second:
// - Can be a value from 0 to 59.
fn validate_by_second(
    rrule: &RRule,
    _dt_start: &chrono::DateTime<Tz>,
) -> Result<(), ValidationError> {
    validate_range_for_vec(&(0..=59), &rrule.by_second, "BYSECOND")
}

fn validate_by_easter(
    rrule: &RRule,
    _dt_start: &chrono::DateTime<Tz>,
) -> Result<(), ValidationError> {
    // By_easter:
    // - Can be a value from -366 to 366.
    //   Validated below
    if let Some(by_easter) = &rrule.by_easter {
        validate_range_for_vec(&(-366..=366), &[*by_easter], "BYEASTER")?;
    }
    // - Can only be used on frequency: Yearly, Monthly, Daily
    //   Validated below
    if rrule.by_easter.is_some() {
        let valid = matches!(
            rrule.freq,
            ICalendarFrequency::Yearly | ICalendarFrequency::Monthly | ICalendarFrequency::Daily
        );
        if !valid {
            return Err(ValidationError::InvalidByRuleAndFrequency {
                by_rule: "BYEASTER".into(),
                freq: rrule.freq,
            });
        }
    }
    // - Can only be used when `by_hour`, `by_minute` and `by_second` are used.
    //   TODO don't know why this is true, but seems to loop forever otherwise.
    if rrule.by_easter.is_some()
        && (rrule.by_hour.is_empty() || rrule.by_minute.is_empty() || rrule.by_second.is_empty())
    {
        return Err(ValidationError::InvalidByRuleWithByEaster);
    }

    Ok(())
}

fn validate_range_for_vec_error<'a, T: PartialOrd>(
    range: &RangeInclusive<T>,
    list: &'a [T],
) -> Result<(), &'a T> {
    for item in list {
        // If value not in range = error
        if !range.contains(item) {
            return Err(item);
        }
    }
    Ok(())
}

fn validate_range_for_vec<T: PartialOrd + ToString>(
    range: &RangeInclusive<T>,
    list: &[T],
    field: &str,
) -> Result<(), ValidationError> {
    for item in list {
        // If value not in range = error
        if !range.contains(item) {
            return Err(ValidationError::InvalidFieldValueRange {
                field: field.to_string(),
                value: item.to_string(),
                start_idx: range.start().to_string(),
                end_idx: range.end().to_string(),
            });
        }
    }
    Ok(())
}

fn validate_not_equal_for_vec<T: PartialEq<T> + ToString>(
    value: &T,
    list: &[T],
    field: &str,
) -> Result<(), ValidationError> {
    if list.iter().any(|item| item == value) {
        return Err(ValidationError::InvalidFieldValue {
            field: field.to_string(),
            value: value.to_string(),
        });
    }
    Ok(())
}
