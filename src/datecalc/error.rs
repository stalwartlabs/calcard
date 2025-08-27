/*
 * SPDX-FileCopyrightText: 2021 Fredrik Meringdal, Ralph Bisschops <https://github.com/fmeringdal/rust-rrule>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{common::IanaString, icalendar::ICalendarFrequency};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum RRuleError {
    ValidationError(ValidationError),
    IterError(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ValidationError {
    BySetPosWithoutByRule,
    InvalidFieldValue {
        field: String,
        value: String,
    },
    InvalidFieldValueRange {
        field: String,
        value: String,
        start_idx: String,
        end_idx: String,
    },
    InvalidFieldValueRangeWithFreq {
        field: String,
        value: String,
        freq: ICalendarFrequency,
        start_idx: String,
        end_idx: String,
    },
    InvalidByRuleAndFrequency {
        by_rule: String,
        freq: ICalendarFrequency,
    },
    UntilBeforeStart {
        until: String,
        dt_start: String,
    },
    TooBigInterval(u16),
    StartYearOutOfRange(i32),
    UnableToGenerateTimeset,
    InvalidByRuleWithByEaster,
    DtStartUntilMismatchTimezone {
        dt_start_tz: String,
        until_tz: String,
        expected: Vec<String>,
    },
}

impl From<ValidationError> for RRuleError {
    fn from(err: ValidationError) -> Self {
        Self::ValidationError(err)
    }
}

impl From<String> for RRuleError {
    fn from(err: String) -> Self {
        Self::IterError(err)
    }
}

impl RRuleError {
    pub fn new_iter_err<S: AsRef<str>>(msg: S) -> Self {
        Self::IterError(msg.as_ref().to_owned())
    }
}

pub(crate) fn checked_mul_u32(v1: u32, v2: u32, hint: Option<&str>) -> Result<u32, RRuleError> {
    v1.checked_mul(v2).ok_or_else(|| match hint {
        Some(hint) => RRuleError::new_iter_err(format!(
            "Could not multiply number, would overflow (`{} * {}`), {}.",
            v1, v2, hint
        )),
        None => RRuleError::new_iter_err(format!(
            "Could not multiply number, would overflow (`{} * {}`).",
            v1, v2,
        )),
    })
}

pub(crate) fn checked_add_u32(v1: u32, v2: u32, hint: Option<&str>) -> Result<u32, RRuleError> {
    v1.checked_add(v2).ok_or_else(|| match hint {
        Some(hint) => RRuleError::new_iter_err(format!(
            "Could not add numbers, would overflow (`{} + {}`), {}.",
            v1, v2, hint
        )),
        None => RRuleError::new_iter_err(format!(
            "Could not add numbers, would overflow (`{} + {}`).",
            v1, v2,
        )),
    })
}

impl Display for RRuleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RRuleError::ValidationError(err) => write!(f, "{}", err),
            RRuleError::IterError(err) => write!(f, "Iteration error: {}", err),
        }
    }
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::BySetPosWithoutByRule => {
                write!(f, "BYSETPOS without BYxxx rule is not allowed")
            }
            ValidationError::InvalidFieldValue { field, value } => {
                write!(f, "Invalid value `{value}` for field `{field}`")
            }
            ValidationError::InvalidFieldValueRange {
                field,
                value,
                start_idx,
                end_idx,
            } => write!(
                f,
                "Invalid value `{value}` for field `{field}`, start index `{start_idx}` and end index `{end_idx}`"
            ),
            ValidationError::InvalidFieldValueRangeWithFreq {
                field,
                value,
                freq,
                start_idx,
                end_idx,
            } => write!(
                f,
                "Invalid value `{value}` for field `{field}`, frequency `{}`, start index `{start_idx}` and end index `{end_idx}`",
                freq.as_str()
            ),
            ValidationError::InvalidByRuleAndFrequency { by_rule, freq } => write!(
                f,
                "Invalid BY rule `{by_rule}` with frequency `{}`",
                freq.as_str()
            ),
            ValidationError::UntilBeforeStart { until, dt_start } => write!(
                f,
                "Until date `{until}` is before the start date `{dt_start}`"
            ),
            ValidationError::TooBigInterval(interval) => write!(
                f,
                "Interval of {interval} is too big. The maximum interval is 32767."
            ),
            ValidationError::StartYearOutOfRange(year) => write!(
                f,
                "Start year {year} is out of range. The valid range is 1970 to 2038."
            ),
            ValidationError::UnableToGenerateTimeset => {
                write!(f, "Unable to generate timeset")
            }
            ValidationError::InvalidByRuleWithByEaster => {
                write!(f, "BYEASTER cannot be used with BYxxx rules")
            }
            ValidationError::DtStartUntilMismatchTimezone {
                dt_start_tz,
                until_tz,
                expected,
            } => write!(
                f,
                "DTSTART timezone `{dt_start_tz}` does not match UNTIL timezone `{until_tz}`, expected timezones: {:?}",
                expected
            ),
        }
    }
}
