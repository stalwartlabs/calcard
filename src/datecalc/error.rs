use crate::icalendar::ICalendarFrequency;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RRuleError {
    ValidationError(ValidationError),
    IterError(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Create a new iterator error with the given message.
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
