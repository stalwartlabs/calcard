use super::{
    error::{RRuleError, ValidationError},
    get_day, get_hour, get_minute, get_month, get_second,
    validate::validate_rrule_forced,
};
use crate::{
    common::timezone::Tz,
    icalendar::{ICalendarFrequency, ICalendarRecurrenceRule},
};
use chrono::{DateTime, Datelike, Weekday};
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RRule {
    pub(crate) freq: ICalendarFrequency,
    pub(crate) interval: u16,
    pub(crate) count: Option<u32>,
    pub(crate) until: Option<DateTime<Tz>>,
    pub(crate) week_start: Weekday,
    pub(crate) by_set_pos: Vec<i32>,
    pub(crate) by_month: Vec<u8>,
    pub(crate) by_month_day: Vec<i8>,
    pub(crate) by_n_month_day: Vec<i8>,
    pub(crate) by_year_day: Vec<i16>,
    pub(crate) by_week_no: Vec<i8>,
    pub(crate) by_weekday: Vec<NWeekday>,
    pub(crate) by_hour: Vec<u8>,
    pub(crate) by_minute: Vec<u8>,
    pub(crate) by_second: Vec<u8>,
    pub(crate) by_easter: Option<i16>,
}

impl RRule {
    pub fn from_floating_ical(ical: &ICalendarRecurrenceRule) -> Option<Self> {
        Some(RRule {
            freq: ical.freq,
            interval: ical.interval.unwrap_or(1),
            count: ical.count,
            until: if let Some(until) = &ical.until {
                until.to_date_time_with_tz(Tz::Floating)?.into()
            } else {
                None
            },
            week_start: ical.wkst.map(Into::into).unwrap_or(Weekday::Mon),
            by_set_pos: ical.bysetpos.clone(),
            by_month: ical.bymonth.clone(),
            by_month_day: ical
                .bymonthday
                .iter()
                .filter(|v| **v > 0)
                .copied()
                .collect(),
            by_n_month_day: ical
                .bymonthday
                .iter()
                .filter(|v| **v < 0)
                .copied()
                .collect(),
            by_year_day: ical.byyearday.clone(),
            by_week_no: ical.byweekno.clone(),
            by_weekday: ical
                .byday
                .iter()
                .map(|wday| NWeekday::new(wday.ordwk, wday.weekday.into()))
                .collect(),
            by_hour: ical.byhour.clone(),
            by_minute: ical.byminute.clone(),
            by_second: ical.bysecond.clone(),
            by_easter: None,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NWeekday {
    Every(Weekday),
    Nth(i16, Weekday),
}

impl RRule {
    pub(crate) fn finalize_parsed_rrule(mut self, dt_start: &DateTime<Tz>) -> Self {
        // Can only be set to true if the feature flag is set.
        let by_easter_is_some = self.by_easter.is_some();

        // Add some freq-specific additional properties
        if !(!self.by_week_no.is_empty()
            || !self.by_year_day.is_empty()
            || !self.by_month_day.is_empty()
            || !self.by_n_month_day.is_empty()
            || !self.by_weekday.is_empty()
            || by_easter_is_some)
        {
            match self.freq {
                ICalendarFrequency::Yearly => {
                    if self.by_month.is_empty() {
                        let month = get_month(dt_start);
                        self.by_month = vec![month];
                    }
                    let day = get_day(dt_start);
                    self.by_month_day = vec![day];
                }
                ICalendarFrequency::Monthly => {
                    let day = get_day(dt_start);
                    self.by_month_day = vec![day];
                }
                ICalendarFrequency::Weekly => {
                    self.by_weekday = vec![NWeekday::Every(dt_start.weekday())];
                }
                _ => (),
            };
        }

        // by_hour
        if self.by_hour.is_empty() && self.freq < ICalendarFrequency::Hourly {
            let hour = get_hour(dt_start);
            self.by_hour = vec![hour];
        }

        // by_minute
        if self.by_minute.is_empty() && self.freq < ICalendarFrequency::Minutely {
            let minute = get_minute(dt_start);
            self.by_minute = vec![minute];
        }

        // by_second
        if self.by_second.is_empty() && self.freq < ICalendarFrequency::Secondly {
            let second = get_second(dt_start);
            self.by_second = vec![second];
        }

        // make sure all BYXXX are unique and sorted
        self.by_hour.sort_unstable();
        self.by_hour.dedup();

        self.by_minute.sort_unstable();
        self.by_minute.dedup();

        self.by_second.sort_unstable();
        self.by_second.dedup();

        self.by_month.sort_unstable();
        self.by_month.dedup();

        self.by_month_day.sort_unstable();
        self.by_month_day.dedup();

        self.by_n_month_day.sort_unstable();
        self.by_n_month_day.dedup();

        self.by_year_day.sort_unstable();
        self.by_year_day.dedup();

        self.by_week_no.sort_unstable();
        self.by_week_no.dedup();

        self.by_set_pos.sort_unstable();
        self.by_set_pos.dedup();

        self.by_weekday.sort_unstable();
        self.by_weekday.dedup();

        self
    }

    pub fn validate(self, dt_start: DateTime<Tz>) -> Result<RRule, RRuleError> {
        let rrule = self.finalize_parsed_rrule(&dt_start);

        // Validate required checks (defined by RFC 5545)
        validate_rrule_forced(&rrule, &dt_start)?;

        // Check if it is possible to generate a timeset
        match rrule.freq {
            ICalendarFrequency::Hourly => {
                if rrule.by_minute.is_empty() && rrule.by_second.is_empty() {
                    return Err(ValidationError::UnableToGenerateTimeset.into());
                }
            }
            ICalendarFrequency::Minutely => {
                if rrule.by_second.is_empty() {
                    return Err(ValidationError::UnableToGenerateTimeset.into());
                }
            }
            ICalendarFrequency::Secondly => {}
            _ => {
                if rrule.by_hour.is_empty()
                    && rrule.by_minute.is_empty()
                    && rrule.by_second.is_empty()
                {
                    return Err(ValidationError::UnableToGenerateTimeset.into());
                }
            }
        }

        Ok(rrule)
    }
}

impl Default for RRule {
    fn default() -> Self {
        Self {
            freq: ICalendarFrequency::Yearly,
            interval: 1,
            count: None,
            until: None,
            week_start: Weekday::Mon,
            by_set_pos: Vec::new(),
            by_month: Vec::new(),
            by_month_day: Vec::new(),
            by_n_month_day: Vec::new(),
            by_year_day: Vec::new(),
            by_week_no: Vec::new(),
            by_weekday: Vec::new(),
            by_hour: Vec::new(),
            by_minute: Vec::new(),
            by_second: Vec::new(),
            by_easter: None,
        }
    }
}

impl PartialOrd for NWeekday {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NWeekday {
    fn cmp(&self, other: &Self) -> Ordering {
        n_weekday_cmp(*self, *other)
    }
}

// The ordering here doesn't really matter as it is only used to sort for display purposes
fn n_weekday_cmp(val1: NWeekday, val2: NWeekday) -> Ordering {
    match val1 {
        NWeekday::Every(wday) => match val2 {
            NWeekday::Every(other_wday) => wday
                .num_days_from_monday()
                .cmp(&other_wday.num_days_from_monday()),
            NWeekday::Nth(_n, _other_wday) => Ordering::Less,
        },
        NWeekday::Nth(n, wday) => match val2 {
            NWeekday::Every(_) => Ordering::Greater,
            NWeekday::Nth(other_n, other_wday) => match n.cmp(&other_n) {
                Ordering::Equal => wday
                    .num_days_from_monday()
                    .cmp(&other_wday.num_days_from_monday()),
                less_or_greater => less_or_greater,
            },
        },
    }
}

impl NWeekday {
    pub fn new(number: Option<i16>, weekday: Weekday) -> Self {
        match number {
            Some(number) => Self::Nth(number, weekday),
            None => Self::Every(weekday),
        }
    }
}
