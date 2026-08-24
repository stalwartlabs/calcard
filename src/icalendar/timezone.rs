/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::{
    ICalendar, ICalendarComponent, ICalendarComponentType, ICalendarDay, ICalendarEntry,
    ICalendarFrequency, ICalendarMonth, ICalendarPeriod, ICalendarProperty,
    ICalendarRecurrenceRule, ICalendarValue, ICalendarWeekday,
};
use crate::{
    common::{PartialDateTime, timezone::Tz},
    icalendar::ICalendarParameterName,
};
use chrono::{Datelike, NaiveDate, NaiveDateTime, Offset, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::{OffsetComponents, OffsetName};
use std::{collections::HashMap, ops::Range, str::FromStr};

pub struct TzResolver<T> {
    tzs: HashMap<T, Tz>,
    default: Tz,
}

impl<T> TzResolver<T>
where
    T: std::borrow::Borrow<str> + std::hash::Hash + Eq,
{
    pub fn resolve(&self, tz_name: &str) -> Option<Tz> {
        self.tzs
            .get(tz_name)
            .copied()
            .or_else(|| Tz::from_str(tz_name).ok())
    }

    pub fn resolve_or_default(&self, tz_name: Option<&str>) -> Tz {
        tz_name
            .and_then(|tz_name| {
                self.tzs
                    .get(tz_name)
                    .copied()
                    .or_else(|| Tz::from_str(tz_name).ok())
            })
            .unwrap_or(self.default)
    }

    pub fn with_default(mut self, default: impl Into<Tz>) -> Self {
        self.default = default.into();
        self
    }
}

impl ICalendar {
    pub fn timezones(&self) -> impl Iterator<Item = &ICalendarComponent> {
        self.components
            .iter()
            .filter(|comp| matches!(comp.component_type, ICalendarComponentType::VTimezone))
    }

    pub fn is_timezone(&self) -> bool {
        self.timezones().count() == 1
    }

    pub fn build_tz_resolver(&self) -> TzResolver<&'_ str> {
        TzResolver {
            tzs: self.timezones().filter_map(|tz| tz.timezone()).collect(),
            default: Tz::Floating,
        }
    }

    pub fn build_owned_tz_resolver(&self) -> TzResolver<String> {
        TzResolver {
            tzs: self
                .timezones()
                .filter_map(|tz| tz.timezone())
                .map(|(name, tz)| (name.to_string(), tz))
                .collect(),
            default: Tz::Floating,
        }
    }
}

impl ICalendarComponent {
    pub fn timezone(&self) -> Option<(&str, Tz)> {
        let mut tz_name = None;
        let mut tz_id = None;
        let mut tz_lic = None;
        let mut tz_cdo_id = None;

        for entry in &self.entries {
            match (&entry.name, entry.values.first()) {
                (ICalendarProperty::Tzid, Some(ICalendarValue::Text(id))) => {
                    tz_id = Tz::from_str(id).ok();
                    tz_name = Some(id.as_str());
                }
                (ICalendarProperty::Other(value), Some(ICalendarValue::Text(id))) => {
                    hashify::fnc_map!(value.as_bytes(),
                        "X-LIC-LOCATION" => {
                            tz_lic = Tz::from_str(id.strip_prefix("SystemV/").unwrap_or(id.as_str())).ok();
                        },
                        "X-MICROSOFT-CDO-TZID" => {
                            tz_cdo_id = Tz::from_ms_cdo_zone_id(id);
                        },
                        _ => {}
                    );
                }
                _ => (),
            }
        }

        tz_name.zip(tz_id.or(tz_lic).or(tz_cdo_id))
    }
}

impl ICalendarEntry {
    pub fn tz_id(&self) -> Option<&str> {
        self.parameters(&ICalendarParameterName::Tzid)
            .find_map(|v| v.as_text())
    }
}

const TZ_MIN_YEAR: i32 = 1900;
const TZ_MAX_FUTURE_YEARS: i32 = 100;
const TZ_RANGE_MARGIN: i64 = 86400;
const TZ_SCAN_STEP: i64 = 3 * 86400;
const TZ_RULE_ACTIVE_SECONDS: i64 = 366 * 86400;

impl ICalendar {
    pub fn add_timezone(&mut self, tz_id: &str, from: i64, to: i64) -> Option<u32> {
        if self
            .components
            .first()
            .is_none_or(|root| root.component_type != ICalendarComponentType::VCalendar)
        {
            return None;
        }

        let observances = match Tz::from_str(tz_id).ok()? {
            Tz::Tz(tz) => build_observances(tz, from, to),
            Tz::Fixed(offset) => {
                let offset = round_to_minute(offset.local_minus_utc());
                vec![TzObservance {
                    at: from,
                    from_offset: offset,
                    to_offset: offset,
                    is_dst: false,
                    name: None,
                    rrule: None,
                }]
            }
            Tz::Floating => return None,
        };

        let insert_at = self.components[0]
            .component_ids
            .iter()
            .position(|component_id| {
                self.components
                    .get(*component_id as usize)
                    .is_none_or(|comp| comp.component_type != ICalendarComponentType::VTimezone)
            })
            .unwrap_or(self.components[0].component_ids.len());

        let tz_component_id = self.components.len() as u32;
        self.components.reserve(observances.len() + 1);
        self.components.push(ICalendarComponent {
            component_type: ICalendarComponentType::VTimezone,
            entries: vec![
                ICalendarEntry::new(ICalendarProperty::Tzid).with_value(tz_id.to_string()),
            ],
            component_ids: Vec::with_capacity(observances.len()),
        });

        let first_component_id = self.components.len() as u32;
        for observance in observances {
            self.components.push(observance.into_component());
        }
        let last_component_id = self.components.len() as u32;
        self.components[tz_component_id as usize]
            .component_ids
            .extend(first_component_id..last_component_id);

        self.components[0]
            .component_ids
            .insert(insert_at, tz_component_id);

        Some(tz_component_id)
    }

    pub fn add_missing_timezones(&mut self) -> usize {
        let mut defined: Vec<&str> = Vec::new();
        for component in self.timezones() {
            for entry in &component.entries {
                if entry.name == ICalendarProperty::Tzid
                    && let Some(ICalendarValue::Text(tz_id)) = entry.values.first()
                {
                    defined.push(tz_id.as_str());
                }
            }
        }

        let mut referenced: Vec<String> = Vec::new();
        let mut range: Option<(i64, i64)> = None;
        let mut recurrence_span = 0;
        let mut is_unbounded = false;

        for component in &self.components {
            if matches!(
                component.component_type,
                ICalendarComponentType::VTimezone
                    | ICalendarComponentType::Standard
                    | ICalendarComponentType::Daylight
            ) {
                continue;
            }

            for entry in &component.entries {
                for param in &entry.params {
                    if param.name == ICalendarParameterName::Tzid
                        && let Some(tz_id) = param.value.as_text()
                        && !defined.contains(&tz_id)
                        && !referenced.iter().any(|id| id == tz_id)
                    {
                        referenced.push(tz_id.to_string());
                    }
                }

                match &entry.name {
                    ICalendarProperty::Rrule => {
                        for value in &entry.values {
                            if let ICalendarValue::RecurrenceRule(rule) = value {
                                match (&rule.until, &rule.count) {
                                    (Some(until), _) => expand_range(&mut range, until),
                                    (None, Some(count)) => {
                                        recurrence_span = recurrence_span.max(recurrence_span_of(
                                            &rule.freq,
                                            *count,
                                            rule.interval,
                                        ));
                                    }
                                    (None, None) => is_unbounded = true,
                                }
                            }
                        }
                    }
                    ICalendarProperty::Dtstart
                    | ICalendarProperty::Dtend
                    | ICalendarProperty::Due
                    | ICalendarProperty::RecurrenceId
                    | ICalendarProperty::Exdate
                    | ICalendarProperty::Rdate => {
                        for value in &entry.values {
                            match value {
                                ICalendarValue::PartialDateTime(dt) => {
                                    expand_range(&mut range, dt);
                                }
                                ICalendarValue::Period(ICalendarPeriod::Range { start, end }) => {
                                    expand_range(&mut range, start);
                                    expand_range(&mut range, end);
                                }
                                ICalendarValue::Period(ICalendarPeriod::Duration {
                                    start, ..
                                }) => {
                                    expand_range(&mut range, start);
                                }
                                _ => (),
                            }
                        }
                    }
                    _ => (),
                }
            }
        }

        if referenced.is_empty() {
            return 0;
        }

        let now = Utc::now().timestamp();
        let (min, max) = range.unwrap_or((now, now));
        let max = max.saturating_add(recurrence_span);
        let current_year = year_of(now);
        let max_year = current_year.saturating_add(TZ_MAX_FUTURE_YEARS);
        let from_year = year_of(min).clamp(TZ_MIN_YEAR, max_year);
        let to_year = if is_unbounded {
            year_of(max).max(current_year)
        } else {
            year_of(max)
        }
        .saturating_add(2)
        .clamp(from_year.saturating_add(1), max_year.saturating_add(2));
        let from = start_of_year(from_year).saturating_sub(TZ_RANGE_MARGIN);
        let to = start_of_year(to_year);

        let mut added = 0;
        for tz_id in referenced {
            if self.add_timezone(&tz_id, from, to).is_some() {
                added += 1;
            }
        }
        added
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TzState {
    offset: i32,
    is_dst: bool,
}

#[derive(PartialEq, Eq)]
struct TzRuleKey {
    month: u8,
    ordwk: i16,
    weekday: ICalendarWeekday,
    hour: u8,
    minute: u8,
    second: u8,
    from_offset: i32,
    to_offset: i32,
}

struct TzObservance {
    at: i64,
    from_offset: i32,
    to_offset: i32,
    is_dst: bool,
    name: Option<String>,
    rrule: Option<ICalendarRecurrenceRule>,
}

impl TzObservance {
    fn onset(&self) -> NaiveDateTime {
        naive_utc(self.at.saturating_add(self.from_offset as i64))
    }

    fn rule_key(&self) -> Option<(i32, TzRuleKey)> {
        let onset = self.onset();
        let day = onset.day();
        let ordwk = if day.saturating_add(7) > days_in_month(onset.year(), onset.month()) {
            -1
        } else {
            ((day - 1) / 7 + 1) as i16
        };

        Some((
            onset.year(),
            TzRuleKey {
                month: u8::try_from(onset.month()).ok()?,
                ordwk,
                weekday: weekday_of(onset.weekday()),
                hour: u8::try_from(onset.hour()).ok()?,
                minute: u8::try_from(onset.minute()).ok()?,
                second: u8::try_from(onset.second()).ok()?,
                from_offset: self.from_offset,
                to_offset: self.to_offset,
            },
        ))
    }

    fn into_component(self) -> ICalendarComponent {
        let mut component = ICalendarComponent::new(if self.is_dst {
            ICalendarComponentType::Daylight
        } else {
            ICalendarComponentType::Standard
        });

        component
            .entries
            .push(ICalendarEntry::new(ICalendarProperty::Dtstart).with_value(
                PartialDateTime::from_naive_timestamp(self.onset().and_utc().timestamp()),
            ));
        component.entries.push(
            ICalendarEntry::new(ICalendarProperty::Tzoffsetfrom)
                .with_value(offset_to_partial(self.from_offset)),
        );
        component.entries.push(
            ICalendarEntry::new(ICalendarProperty::Tzoffsetto)
                .with_value(offset_to_partial(self.to_offset)),
        );
        if let Some(name) = self.name {
            component
                .entries
                .push(ICalendarEntry::new(ICalendarProperty::Tzname).with_value(name));
        }
        if let Some(rrule) = self.rrule {
            component
                .entries
                .push(ICalendarEntry::new(ICalendarProperty::Rrule).with_value(rrule));
        }

        component
    }
}

fn build_observances(tz: chrono_tz::Tz, from: i64, to: i64) -> Vec<TzObservance> {
    let (initial, initial_name) = tz_observance_at(tz, from);
    let mut observances = Vec::with_capacity(estimated_observances(from, to));
    observances.push(TzObservance {
        at: from,
        from_offset: round_to_minute(initial.offset),
        to_offset: round_to_minute(initial.offset),
        is_dst: initial.is_dst,
        name: initial_name,
        rrule: None,
    });

    let mut previous_at = from;
    let mut previous = initial;
    let mut at = from;

    while at < to {
        at = at.saturating_add(TZ_SCAN_STEP).min(to);
        let state = tz_state_at(tz, at);
        if state != previous {
            let onset = find_transition(tz, previous_at, at, previous);
            observances.push(TzObservance {
                at: onset,
                from_offset: round_to_minute(previous.offset),
                to_offset: round_to_minute(state.offset),
                is_dst: state.is_dst,
                name: tz_name_at(tz, onset),
                rrule: None,
            });
            previous = state;
        }
        previous_at = at;
    }

    label_observances(&mut observances);

    collapse_observances(observances, to)
}

fn label_observances(observances: &mut [TzObservance]) {
    let mut daylight_offset = None;
    let mut standard_offset = None;

    for observance in observances.iter() {
        let slot = if observance.is_dst {
            &mut daylight_offset
        } else {
            &mut standard_offset
        };
        *slot = Some(slot.map_or(observance.to_offset, |offset: i32| {
            offset.max(observance.to_offset)
        }));
    }

    if let (Some(daylight_offset), Some(standard_offset)) = (daylight_offset, standard_offset)
        && daylight_offset < standard_offset
    {
        for observance in observances.iter_mut() {
            observance.is_dst = !observance.is_dst;
        }
    }
}

fn collapse_observances(mut observances: Vec<TzObservance>, to: i64) -> Vec<TzObservance> {
    let mut discard = vec![false; observances.len()];
    let mut unbounded: [Option<(usize, i64)>; 2] = [None, None];
    let mut trailing: [Option<(usize, u8, i16, ICalendarWeekday)>; 2] = [None, None];

    let mut onsets: Vec<(usize, i32, TzRuleKey)> = Vec::new();
    let mut runs: Vec<Range<usize>> = Vec::new();

    for is_dst in [false, true] {
        onsets.clear();
        runs.clear();

        for (index, observance) in observances.iter().enumerate() {
            if index == 0 || observance.is_dst != is_dst {
                continue;
            }
            let Some((year, key)) = observance.rule_key() else {
                continue;
            };
            let extends = onsets
                .last()
                .is_some_and(|(last_index, last_year, last_key)| {
                    last_year.saturating_add(1) == year
                        && last_key == &key
                        && observances[*last_index].name == observance.name
                });

            onsets.push((index, year, key));
            match runs.last_mut() {
                Some(run) if extends => run.end = onsets.len(),
                _ => runs.push(onsets.len() - 1..onsets.len()),
            }
        }

        for (position, run) in runs.iter().enumerate() {
            let (Some((first, _, key)), Some((last, _, _))) =
                (onsets.get(run.start), onsets.get(run.end - 1))
            else {
                continue;
            };
            if run.len() < 2 {
                continue;
            }

            let last_at = observances[*last].at;
            let is_expired = last_at.saturating_add(TZ_RULE_ACTIVE_SECONDS) < to;
            let is_final = position + 1 == runs.len() && !is_expired;
            observances[*first].rrule = Some(ICalendarRecurrenceRule {
                freq: ICalendarFrequency::Yearly,
                until: (!is_final).then(|| PartialDateTime::from_utc_timestamp(last_at)),
                bymonth: vec![ICalendarMonth::new(key.month, false)],
                byday: vec![ICalendarDay {
                    ordwk: Some(key.ordwk),
                    weekday: key.weekday,
                }],
                ..Default::default()
            });
            if is_final {
                unbounded[usize::from(is_dst)] = Some((*first, last_at));
            }

            for (index, _, _) in &onsets[run.start + 1..run.end] {
                discard[*index] = true;
            }
        }

        if let Some(run) = runs.last()
            && run.len() == 1
            && let Some((index, _, key)) = onsets.get(run.start)
        {
            trailing[usize::from(is_dst)] = Some((*index, key.month, key.ordwk, key.weekday));
        }
    }

    for is_dst in [false, true] {
        let class = usize::from(is_dst);
        let (None, Some((other_index, other_at))) =
            (unbounded[class], unbounded[usize::from(!is_dst)])
        else {
            continue;
        };

        match trailing[class] {
            Some((index, month, ordwk, weekday))
                if observances[index].at.saturating_add(TZ_RULE_ACTIVE_SECONDS) >= to =>
            {
                observances[index].rrule = Some(ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Yearly,
                    bymonth: vec![ICalendarMonth::new(month, false)],
                    byday: vec![ICalendarDay {
                        ordwk: Some(ordwk),
                        weekday,
                    }],
                    ..Default::default()
                });
                unbounded[class] = Some((index, observances[index].at));
            }
            _ => {
                if let Some(rrule) = observances[other_index].rrule.as_mut() {
                    rrule.until = Some(PartialDateTime::from_utc_timestamp(other_at));
                    unbounded[usize::from(!is_dst)] = None;
                }
            }
        }
    }

    let mut index = 0;
    observances.retain(|_| {
        let keep = !discard[index];
        index += 1;
        keep
    });

    observances
}

fn find_transition(tz: chrono_tz::Tz, mut low: i64, mut high: i64, low_state: TzState) -> i64 {
    while high - low > 1 {
        let middle = low + (high - low) / 2;
        if tz_state_at(tz, middle) == low_state {
            low = middle;
        } else {
            high = middle;
        }
    }
    high
}

fn tz_state_at(tz: chrono_tz::Tz, timestamp: i64) -> TzState {
    let offset = tz.offset_from_utc_datetime(&naive_utc(timestamp));

    TzState {
        offset: offset.fix().local_minus_utc(),
        is_dst: offset.dst_offset().num_seconds() != 0,
    }
}

fn tz_observance_at(tz: chrono_tz::Tz, timestamp: i64) -> (TzState, Option<String>) {
    let offset = tz.offset_from_utc_datetime(&naive_utc(timestamp));

    (
        TzState {
            offset: offset.fix().local_minus_utc(),
            is_dst: offset.dst_offset().num_seconds() != 0,
        },
        offset.abbreviation().map(|name| name.to_string()),
    )
}

fn tz_name_at(tz: chrono_tz::Tz, timestamp: i64) -> Option<String> {
    tz.offset_from_utc_datetime(&naive_utc(timestamp))
        .abbreviation()
        .map(|name| name.to_string())
}

fn estimated_observances(from: i64, to: i64) -> usize {
    let years = to.saturating_sub(from) / TZ_RULE_ACTIVE_SECONDS;

    (years.clamp(0, 1024) as usize) * 2 + 2
}

fn round_to_minute(offset: i32) -> i32 {
    let minutes = (offset.unsigned_abs() + 30) / 60 * 60;

    if offset < 0 {
        -(minutes as i32)
    } else {
        minutes as i32
    }
}

fn offset_to_partial(offset: i32) -> PartialDateTime {
    let seconds = offset.unsigned_abs();

    PartialDateTime {
        tz_hour: u8::try_from(seconds / 3600).ok(),
        tz_minute: u8::try_from((seconds % 3600) / 60).ok(),
        tz_minus: offset < 0,
        ..Default::default()
    }
}

fn recurrence_span_of(freq: &ICalendarFrequency, count: u32, interval: Option<u16>) -> i64 {
    let unit = match freq {
        ICalendarFrequency::Yearly => 366 * 86400,
        ICalendarFrequency::Monthly => 31 * 86400,
        ICalendarFrequency::Weekly => 7 * 86400,
        ICalendarFrequency::Daily => 86400,
        ICalendarFrequency::Hourly => 3600,
        ICalendarFrequency::Minutely => 60,
        ICalendarFrequency::Secondly => 1,
    };

    i64::from(count)
        .saturating_mul(i64::from(interval.unwrap_or(1)))
        .saturating_mul(unit)
}

fn expand_range(range: &mut Option<(i64, i64)>, value: &PartialDateTime) {
    let Some(timestamp) = value.to_date_time().map(|result| {
        result.date_time.and_utc().timestamp()
            - result
                .offset
                .map_or(0, |offset| offset.local_minus_utc() as i64)
    }) else {
        return;
    };

    match range {
        Some((min, max)) => {
            *min = (*min).min(timestamp);
            *max = (*max).max(timestamp);
        }
        None => {
            *range = Some((timestamp, timestamp));
        }
    }
}

fn naive_utc(timestamp: i64) -> NaiveDateTime {
    chrono::DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_default()
        .naive_utc()
}

fn year_of(timestamp: i64) -> i32 {
    naive_utc(timestamp).year()
}

fn start_of_year(year: i32) -> i64 {
    NaiveDate::from_ymd_opt(year, 1, 1)
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map_or(0, |date_time| date_time.and_utc().timestamp())
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (year, month) = if month == 12 {
        (year.saturating_add(1), 1)
    } else {
        (year, month + 1)
    };

    NaiveDate::from_ymd_opt(year, month, 1)
        .and_then(|date| date.pred_opt())
        .map_or(31, |date| date.day())
}

fn weekday_of(weekday: Weekday) -> ICalendarWeekday {
    match weekday {
        Weekday::Mon => ICalendarWeekday::Monday,
        Weekday::Tue => ICalendarWeekday::Tuesday,
        Weekday::Wed => ICalendarWeekday::Wednesday,
        Weekday::Thu => ICalendarWeekday::Thursday,
        Weekday::Fri => ICalendarWeekday::Friday,
        Weekday::Sat => ICalendarWeekday::Saturday,
        Weekday::Sun => ICalendarWeekday::Sunday,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icalendar::{ICalendar, ICalendarComponent, ICalendarComponentType};

    const ORACLE_STEP: i64 = 3600;
    const HORIZON_YEARS: i32 = 20;
    const MAX_DIVERGENCE: f64 = 0.15;

    fn add_timezones(ical: &str) -> String {
        let mut ical = ICalendar::parse(ical).expect("failed to parse iCalendar object");
        ical.add_missing_timezones();
        let result = ical.to_string();

        let reparsed = ICalendar::parse(&result).expect("generated object failed to re-parse");
        assert_eq!(
            reparsed.to_string(),
            result,
            "generated object is not stable"
        );

        result.replace("\r\n", "\n")
    }

    fn event(properties: &str) -> String {
        format!(
            "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:test\r\n{properties}END:VEVENT\r\nEND:VCALENDAR\r\n"
        )
    }

    #[test]
    fn test_add_missing_timezones() {
        for (properties, expected) in [
            (
                "DTSTART;TZID=Europe/Berlin:20260822T163000\r\nDTEND;TZID=Europe/Berlin:20260822T173000\r\n",
                concat!(
                    "BEGIN:VTIMEZONE\n",
                    "TZID:Europe/Berlin\n",
                    "BEGIN:STANDARD\n",
                    "DTSTART:20251231T010000\nTZOFFSETFROM:+0100\nTZOFFSETTO:+0100\nTZNAME:CET\n",
                    "END:STANDARD\n",
                    "BEGIN:DAYLIGHT\n",
                    "DTSTART:20260329T020000\nTZOFFSETFROM:+0100\nTZOFFSETTO:+0200\nTZNAME:CEST\n",
                    "RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=3\n",
                    "END:DAYLIGHT\n",
                    "BEGIN:STANDARD\n",
                    "DTSTART:20261025T030000\nTZOFFSETFROM:+0200\nTZOFFSETTO:+0100\nTZNAME:CET\n",
                    "RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10\n",
                    "END:STANDARD\n",
                    "END:VTIMEZONE\n",
                ),
            ),
            (
                "DTSTART;TZID=America/New_York:20260822T163000\r\n",
                concat!(
                    "BEGIN:VTIMEZONE\n",
                    "TZID:America/New_York\n",
                    "BEGIN:STANDARD\n",
                    "DTSTART:20251230T190000\nTZOFFSETFROM:-0500\nTZOFFSETTO:-0500\nTZNAME:EST\n",
                    "END:STANDARD\n",
                    "BEGIN:DAYLIGHT\n",
                    "DTSTART:20260308T020000\nTZOFFSETFROM:-0500\nTZOFFSETTO:-0400\nTZNAME:EDT\n",
                    "RRULE:FREQ=YEARLY;BYDAY=2SU;BYMONTH=3\n",
                    "END:DAYLIGHT\n",
                    "BEGIN:STANDARD\n",
                    "DTSTART:20261101T020000\nTZOFFSETFROM:-0400\nTZOFFSETTO:-0500\nTZNAME:EST\n",
                    "RRULE:FREQ=YEARLY;BYDAY=1SU;BYMONTH=11\n",
                    "END:STANDARD\n",
                    "END:VTIMEZONE\n",
                ),
            ),
            (
                "DTSTART;TZID=Australia/Sydney:20260822T163000\r\n",
                concat!(
                    "BEGIN:VTIMEZONE\n",
                    "TZID:Australia/Sydney\n",
                    "BEGIN:DAYLIGHT\n",
                    "DTSTART:20251231T110000\nTZOFFSETFROM:+1100\nTZOFFSETTO:+1100\nTZNAME:AEDT\n",
                    "END:DAYLIGHT\n",
                    "BEGIN:STANDARD\n",
                    "DTSTART:20260405T030000\nTZOFFSETFROM:+1100\nTZOFFSETTO:+1000\nTZNAME:AEST\n",
                    "RRULE:FREQ=YEARLY;BYDAY=1SU;BYMONTH=4\n",
                    "END:STANDARD\n",
                    "BEGIN:DAYLIGHT\n",
                    "DTSTART:20261004T020000\nTZOFFSETFROM:+1000\nTZOFFSETTO:+1100\nTZNAME:AEDT\n",
                    "RRULE:FREQ=YEARLY;BYDAY=1SU;BYMONTH=10\n",
                    "END:DAYLIGHT\n",
                    "END:VTIMEZONE\n",
                ),
            ),
            (
                "DTSTART;TZID=Asia/Kolkata:20260822T163000\r\n",
                concat!(
                    "BEGIN:VTIMEZONE\n",
                    "TZID:Asia/Kolkata\n",
                    "BEGIN:STANDARD\n",
                    "DTSTART:20251231T053000\nTZOFFSETFROM:+0530\nTZOFFSETTO:+0530\nTZNAME:IST\n",
                    "END:STANDARD\n",
                    "END:VTIMEZONE\n",
                ),
            ),
            (
                "DTSTART;TZID=America/Sao_Paulo:20180301T163000\r\nRRULE:FREQ=MONTHLY;UNTIL=20190401T000000Z\r\n",
                concat!(
                    "BEGIN:VTIMEZONE\n",
                    "TZID:America/Sao_Paulo\n",
                    "BEGIN:DAYLIGHT\n",
                    "DTSTART:20171230T220000\nTZOFFSETFROM:-0200\nTZOFFSETTO:-0200\n",
                    "END:DAYLIGHT\n",
                    "BEGIN:STANDARD\n",
                    "DTSTART:20180218T000000\nTZOFFSETFROM:-0200\nTZOFFSETTO:-0300\n",
                    "RRULE:FREQ=YEARLY;UNTIL=20190217T020000Z;BYDAY=3SU;BYMONTH=2\n",
                    "END:STANDARD\n",
                    "BEGIN:DAYLIGHT\n",
                    "DTSTART:20181104T000000\nTZOFFSETFROM:-0300\nTZOFFSETTO:-0200\n",
                    "END:DAYLIGHT\n",
                    "END:VTIMEZONE\n",
                ),
            ),
            (
                // Negative daylight saving: the lower offset is the STANDARD observance,
                // matching what other implementations publish for Europe/Dublin
                "DTSTART;TZID=Europe/Dublin:20260822T163000\r\n",
                concat!(
                    "BEGIN:VTIMEZONE\n",
                    "TZID:Europe/Dublin\n",
                    "BEGIN:STANDARD\n",
                    "DTSTART:20251231T000000\nTZOFFSETFROM:+0000\nTZOFFSETTO:+0000\nTZNAME:GMT\n",
                    "END:STANDARD\n",
                    "BEGIN:DAYLIGHT\n",
                    "DTSTART:20260329T010000\nTZOFFSETFROM:+0000\nTZOFFSETTO:+0100\nTZNAME:IST\n",
                    "RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=3\n",
                    "END:DAYLIGHT\n",
                    "BEGIN:STANDARD\n",
                    "DTSTART:20261025T020000\nTZOFFSETFROM:+0100\nTZOFFSETTO:+0000\nTZNAME:GMT\n",
                    "RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10\n",
                    "END:STANDARD\n",
                    "END:VTIMEZONE\n",
                ),
            ),
        ] {
            let result = add_timezones(&event(properties));
            assert!(
                result.contains(expected),
                "expected:\n{expected}\ngot:\n{result}"
            );
            assert!(
                result.contains("BEGIN:VTIMEZONE"),
                "no definition generated:\n{result}"
            );
            assert!(
                result.find("BEGIN:VTIMEZONE") < result.find("BEGIN:VEVENT"),
                "VTIMEZONE must precede VEVENT:\n{result}"
            );
        }
    }

    #[test]
    fn test_add_missing_timezones_multiple() {
        // One definition per referenced identifier, all of them ahead of the event
        let result = add_timezones(&event(concat!(
            "DTSTART;TZID=Europe/Berlin:20260822T163000\r\n",
            "DTEND;TZID=America/New_York:20260822T173000\r\n",
        )));
        assert!(result.contains("TZID:Europe/Berlin"), "{result}");
        assert!(result.contains("TZID:America/New_York"), "{result}");
        assert_eq!(result.matches("BEGIN:VTIMEZONE").count(), 2, "{result}");
        assert!(
            result.find("BEGIN:VEVENT") > result.rfind("END:VTIMEZONE"),
            "every VTIMEZONE must precede VEVENT:\n{result}"
        );

        // A definition added next to an existing one keeps both ahead of the event
        let result = add_timezones(concat!(
            "BEGIN:VCALENDAR\r\n",
            "BEGIN:VTIMEZONE\r\nTZID:Europe/Berlin\r\n",
            "BEGIN:STANDARD\r\nDTSTART:19961027T030000\r\nTZOFFSETFROM:+0200\r\nTZOFFSETTO:+0100\r\n",
            "END:STANDARD\r\nEND:VTIMEZONE\r\n",
            "BEGIN:VEVENT\r\nUID:test\r\n",
            "DTSTART;TZID=Europe/Berlin:20260822T163000\r\n",
            "DTEND;TZID=Asia/Kolkata:20260822T173000\r\n",
            "END:VEVENT\r\nEND:VCALENDAR\r\n"
        ));
        assert_eq!(result.matches("BEGIN:VTIMEZONE").count(), 2, "{result}");
        assert!(
            result.find("BEGIN:VEVENT") > result.rfind("END:VTIMEZONE"),
            "every VTIMEZONE must precede VEVENT:\n{result}"
        );

        // Windows style identifiers resolve and are preserved verbatim
        for tz_id in [
            "Pacific Standard Time",
            "(UTC-08:00) Pacific Time (US & Canada)",
        ] {
            let result = add_timezones(&event(&format!(
                "DTSTART;TZID=\"{tz_id}\":20260822T163000\r\n"
            )));
            assert!(result.contains(&format!("TZID:{tz_id}\n")), "{result}");
            assert!(result.contains("TZOFFSETTO:-0700"), "{result}");
        }

        // Fixed offset identifiers yield a single observance
        let result = add_timezones(&event("DTSTART;TZID=Etc/GMT+5:20260822T163000\r\n"));
        assert!(
            result.contains("TZOFFSETFROM:-0500\nTZOFFSETTO:-0500"),
            "{result}"
        );
        assert_eq!(result.matches("BEGIN:STANDARD").count(), 1, "{result}");
    }

    #[test]
    fn test_add_missing_timezones_noop() {
        // Time zones that are already defined are left untouched
        let defined = concat!(
            "BEGIN:VCALENDAR\r\n",
            "BEGIN:VTIMEZONE\r\nTZID:Europe/Berlin\r\n",
            "BEGIN:STANDARD\r\nDTSTART:19961027T030000\r\nTZOFFSETFROM:+0200\r\nTZOFFSETTO:+0100\r\n",
            "END:STANDARD\r\nEND:VTIMEZONE\r\n",
            "BEGIN:VEVENT\r\nUID:test\r\nDTSTART;TZID=Europe/Berlin:20260822T163000\r\n",
            "END:VEVENT\r\nEND:VCALENDAR\r\n"
        );
        let mut ical = ICalendar::parse(defined).unwrap();
        assert_eq!(ical.add_missing_timezones(), 0);
        assert_eq!(ical.timezones().count(), 1);

        // Unresolvable time zone identifiers are skipped
        let mut ical =
            ICalendar::parse(event("DTSTART;TZID=Nowhere/Atlantis:20260822T163000\r\n")).unwrap();
        assert_eq!(ical.add_missing_timezones(), 0);
        assert_eq!(ical.timezones().count(), 0);

        // Objects without TZID references need no definitions
        let mut ical = ICalendar::parse(event("DTSTART:20260822T163000Z\r\n")).unwrap();
        assert_eq!(ical.add_missing_timezones(), 0);
        assert_eq!(ical.timezones().count(), 0);

        // A VTIMEZONE cannot be nested within another component
        let mut ical = ICalendar {
            components: vec![ICalendarComponent::new(ICalendarComponentType::VEvent)],
        };
        assert_eq!(ical.add_timezone("Europe/Berlin", 0, TZ_RANGE_MARGIN), None);
        assert_eq!(
            ICalendar::default().add_timezone("Europe/Berlin", 0, TZ_RANGE_MARGIN),
            None
        );
    }

    #[test]
    fn test_add_missing_timezones_covers_recurrence_range() {
        // The window must reach the last instance of a counted recurrence
        let mut ical = ICalendar::parse(event(concat!(
            "DTSTART;TZID=Europe/Berlin:20260822T163000\r\n",
            "RRULE:FREQ=YEARLY;COUNT=10\r\n"
        )))
        .unwrap();
        assert_eq!(ical.add_missing_timezones(), 1);
        assert!(
            ical.to_string()
                .contains("RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=3\r\n"),
            "{ical}"
        );

        let tz_component_id = ical
            .components
            .iter()
            .position(|comp| comp.component_type == ICalendarComponentType::VTimezone)
            .expect("no definition generated") as u32;
        let onsets = implied_onsets(&ical, tz_component_id);
        let last = onsets.last().expect("no onsets").0;
        assert!(
            last >= start_of_year(2035),
            "definition ends at {last}, before the last instance"
        );
    }

    #[test]
    fn test_generated_definitions_match_tzdb() {
        for (tz_id, from_year, to_year) in [
            ("Europe/Berlin", 2024, 2028),
            ("Europe/Berlin", 1975, 2000),
            ("Europe/Dublin", 2024, 2028),
            ("America/New_York", 2024, 2028),
            ("America/New_York", 1965, 1990),
            ("Australia/Sydney", 2024, 2028),
            ("Pacific/Chatham", 2024, 2028),
            ("Australia/Lord_Howe", 2024, 2028),
            ("America/Santiago", 2015, 2025),
            ("America/Sao_Paulo", 2015, 2025),
            ("Asia/Tehran", 2015, 2025),
            ("Asia/Gaza", 2024, 2028),
            ("Africa/Casablanca", 2018, 2025),
            ("Africa/Monrovia", 1970, 1973),
            ("Asia/Kolkata", 2024, 2028),
            ("UTC", 2024, 2028),
        ] {
            let (from, to) = (start_of_year(from_year), start_of_year(to_year));
            let (tz, onsets) = generate(tz_id, from, to);

            // Inside the covered range the definition must reproduce the database exactly
            let mut at = from;
            while at < to {
                let implied = offset_at(&onsets, at)
                    .unwrap_or_else(|| panic!("{tz_id}: no observance covers {at}"));
                let actual = tz_state_at(tz, at).offset;
                assert!(
                    implied.abs_diff(actual) <= 30,
                    "{tz_id}: offset {implied} does not match {actual} at {at}"
                );
                at += ORACLE_STEP;
            }
        }
    }

    #[test]
    fn test_generated_definitions_do_not_drift() {
        // Rules left unbounded are an approximation of the future, but they must never
        // strand the zone on the wrong side of a daylight saving transition
        for (tz_id, from_year, to_year) in [
            ("Europe/Berlin", 2024, 2028),
            ("Europe/Dublin", 2024, 2028),
            ("America/New_York", 2024, 2028),
            ("Australia/Sydney", 2024, 2028),
            ("Australia/Lord_Howe", 2024, 2028),
            ("Asia/Gaza", 2024, 2028),
            ("Asia/Hebron", 2024, 2028),
            ("Asia/Jerusalem", 2024, 2028),
            ("America/Santiago", 2024, 2028),
            ("America/Nuuk", 2024, 2028),
            ("Africa/Cairo", 2024, 2028),
            ("Africa/Casablanca", 2024, 2028),
            ("America/Sao_Paulo", 2024, 2028),
            ("Asia/Kolkata", 2024, 2028),
        ] {
            let (from, to) = (start_of_year(from_year), start_of_year(to_year));
            let (tz, onsets) = generate(tz_id, from, to);

            let horizon = start_of_year(to_year + HORIZON_YEARS);
            let (mut diverged, mut samples) = (0u32, 0u32);
            let mut at = to;
            while at < horizon {
                if offset_at(&onsets, at) != Some(tz_state_at(tz, at).offset) {
                    diverged += 1;
                }
                samples += 1;
                at += ORACLE_STEP;
            }

            let ratio = f64::from(diverged) / f64::from(samples);
            assert!(
                ratio < MAX_DIVERGENCE,
                "{tz_id}: definition drifts for {:.1}% of the {HORIZON_YEARS} years after the covered range",
                ratio * 100.0
            );
        }
    }

    fn generate(tz_id: &str, from: i64, to: i64) -> (chrono_tz::Tz, Vec<(i64, i32)>) {
        let Ok(Tz::Tz(tz)) = Tz::from_str(tz_id) else {
            panic!("{tz_id} did not resolve to an IANA time zone");
        };

        let mut ical = ICalendar {
            components: vec![ICalendarComponent::new(ICalendarComponentType::VCalendar)],
        };
        let tz_component_id = ical
            .add_timezone(tz_id, from, to)
            .unwrap_or_else(|| panic!("no definition generated for {tz_id}"));

        (tz, implied_onsets(&ical, tz_component_id))
    }

    /// Expands the definition into the onsets it describes, exactly as a client would,
    /// without reusing anything the generator relies on to find transitions.
    fn implied_onsets(ical: &ICalendar, tz_component_id: u32) -> Vec<(i64, i32)> {
        let mut onsets = Vec::new();
        let timezone = &ical.components[tz_component_id as usize];
        assert_eq!(timezone.component_type, ICalendarComponentType::VTimezone);

        for component_id in &timezone.component_ids {
            let component = &ical.components[*component_id as usize];
            assert!(matches!(
                component.component_type,
                ICalendarComponentType::Standard | ICalendarComponentType::Daylight
            ));

            let date_time = observance_date_time(component);
            let from_offset = observance_offset(component, &ICalendarProperty::Tzoffsetfrom);
            let to_offset = observance_offset(component, &ICalendarProperty::Tzoffsetto);
            onsets.push((
                date_time.and_utc().timestamp() - i64::from(from_offset),
                to_offset,
            ));

            let Some(rule) = component
                .entries
                .iter()
                .find(|entry| entry.name == ICalendarProperty::Rrule)
                .and_then(|entry| match entry.values.first() {
                    Some(ICalendarValue::RecurrenceRule(rule)) => Some(rule),
                    _ => None,
                })
            else {
                continue;
            };

            assert_eq!(rule.freq, ICalendarFrequency::Yearly);
            assert_eq!(rule.byday.len(), 1);
            assert_eq!(rule.bymonth.len(), 1);
            assert!(rule.count.is_none());

            let day = rule.byday[0];
            let month = rule.bymonth[0].month();
            let until = rule
                .until
                .as_ref()
                .and_then(|until| until.to_date_time())
                .map(|until| until.date_time.and_utc().timestamp());

            for year in date_time.year() + 1..=date_time.year() + 200 {
                let Some(date) =
                    nth_weekday(year, month, day.ordwk.unwrap_or_default(), day.weekday)
                else {
                    continue;
                };
                let onset =
                    date.and_time(date_time.time()).and_utc().timestamp() - i64::from(from_offset);
                if until.is_some_and(|until| onset > until) {
                    break;
                }
                onsets.push((onset, to_offset));
            }
        }

        onsets.sort_unstable();
        onsets
    }

    /// RFC 5545, Section 3.6.5: the offset in effect is the one of the observance with the
    /// last onset before the time in question.
    fn offset_at(onsets: &[(i64, i32)], at: i64) -> Option<i32> {
        onsets
            .partition_point(|(onset, _)| *onset <= at)
            .checked_sub(1)
            .map(|position| onsets[position].1)
    }

    fn observance_value<'x>(
        component: &'x ICalendarComponent,
        name: &ICalendarProperty,
    ) -> &'x ICalendarValue {
        component
            .entries
            .iter()
            .find(|entry| &entry.name == name)
            .and_then(|entry| entry.values.first())
            .unwrap_or_else(|| panic!("missing {name:?}"))
    }

    fn observance_offset(component: &ICalendarComponent, name: &ICalendarProperty) -> i32 {
        let ICalendarValue::PartialDateTime(offset) = observance_value(component, name) else {
            panic!("{name:?} is not a UTC offset");
        };
        let seconds = i32::from(offset.tz_hour.unwrap_or_default()) * 3600
            + i32::from(offset.tz_minute.unwrap_or_default()) * 60;

        if offset.tz_minus { -seconds } else { seconds }
    }

    fn observance_date_time(component: &ICalendarComponent) -> NaiveDateTime {
        let ICalendarValue::PartialDateTime(dtstart) =
            observance_value(component, &ICalendarProperty::Dtstart)
        else {
            panic!("DTSTART is not a date-time");
        };
        assert!(
            dtstart.tz_hour.is_none() && dtstart.tz_minute.is_none(),
            "DTSTART must be a local time value"
        );

        dtstart.to_date_time().expect("invalid DTSTART").date_time
    }

    fn nth_weekday(
        year: i32,
        month: u8,
        ordwk: i16,
        weekday: ICalendarWeekday,
    ) -> Option<NaiveDate> {
        let matches = (1..=days_in_month(year, u32::from(month)))
            .filter_map(|day| NaiveDate::from_ymd_opt(year, u32::from(month), day))
            .filter(|date| weekday_of(date.weekday()) == weekday)
            .collect::<Vec<_>>();

        if ordwk < 0 {
            matches
                .len()
                .checked_sub(usize::from(ordwk.unsigned_abs()))
                .and_then(|index| matches.get(index))
                .copied()
        } else {
            matches.get((ordwk as usize).saturating_sub(1)).copied()
        }
    }
}
