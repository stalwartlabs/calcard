/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::*;
use crate::{
    common::{
        ArchivedPartialDateTime,
        writer::{write_bytes, write_param_value, write_text},
    },
    icalendar::ValueSeparator,
};
use std::{
    fmt::{Display, Write},
    slice::Iter,
};

impl ArchivedICalendar {
    pub fn write_to(&self, out: &mut impl Write) -> std::fmt::Result {
        let _v = [0.into()];
        let mut component_iter: Iter<'_, rkyv::rend::u32_le> = _v.iter();
        let mut component_stack = Vec::with_capacity(4);

        loop {
            if let Some(component_id) = component_iter.next() {
                let component = self
                    .components
                    .get(component_id.to_native() as usize)
                    .unwrap();
                write!(out, "BEGIN:{}\r\n", component.component_type.as_str())?;

                for entry in component.entries.iter() {
                    if !matches!(
                        entry.name,
                        ArchivedICalendarProperty::Begin | ArchivedICalendarProperty::End
                    ) {
                        entry.write_to(out, true)?;
                    }
                }

                if !component.component_ids.is_empty() {
                    component_stack.push((component, component_iter));
                    component_iter = component.component_ids.iter();
                } else {
                    write!(out, "END:{}\r\n", component.component_type.as_str())?;
                }
            } else if let Some((component, iter)) = component_stack.pop() {
                write!(out, "END:{}\r\n", component.component_type.as_str())?;
                component_iter = iter;
            } else {
                break;
            }
        }

        Ok(())
    }
}

impl ArchivedICalendarEntry {
    pub fn write_to(&self, out: &mut impl Write, with_value: bool) -> std::fmt::Result {
        let mut line_len = 0;

        let entry_name = self.name.as_str();
        write!(out, "{}", entry_name)?;
        line_len += entry_name.len();

        if matches!(
            self.values.first().as_ref(),
            Some(ArchivedICalendarValue::Binary(_))
        ) {
            write!(out, ";ENCODING=BASE64")?;
            line_len += 18;
        }

        let mut types = None;
        let mut last_param: Option<&ArchivedICalendarParameterName> = None;

        for param in self.params.iter() {
            if last_param.is_some_and(|last_param| last_param == &param.name) {
                write!(out, ",")?;
                line_len += 1;

                if line_len + 1 > 75 {
                    write!(out, "\r\n ")?;
                    line_len = 1;
                }
            } else {
                write!(out, ";")?;
                line_len += 1;
                let name = param.name.as_str();
                let need_len = name.len() + 1;
                if line_len + need_len > 75 {
                    write!(out, "\r\n ")?;
                    line_len = 1;
                }
                if !matches!(param.value, ArchivedICalendarParameterValue::Null) {
                    write!(out, "{name}=")?;
                } else {
                    write!(out, "{name}")?;
                }
                line_len += need_len;
                last_param = Some(&param.name);
            }

            match &param.value {
                ArchivedICalendarParameterValue::Text(v) => {
                    write_param_value(out, &mut line_len, v)?;
                }
                ArchivedICalendarParameterValue::Integer(i) => {
                    write!(out, "{i}")?;
                    line_len += 2;
                }
                ArchivedICalendarParameterValue::Bool(v) => {
                    let v = if !matches!(param.name, ArchivedICalendarParameterName::Range) {
                        if *v { "TRUE" } else { "FALSE" }
                    } else {
                        "THISANDFUTURE"
                    };
                    line_len += v.len();
                    write!(out, "{v}")?;
                }
                ArchivedICalendarParameterValue::Uri(uri) => {
                    write!(out, "\"")?;
                    write_uri(out, &mut line_len, uri, false)?;
                    write!(out, "\"")?;
                }
                ArchivedICalendarParameterValue::Cutype(v) => {
                    write_param_value(out, &mut line_len, v.as_str())?;
                }
                ArchivedICalendarParameterValue::Fbtype(v) => {
                    write_param_value(out, &mut line_len, v.as_str())?;
                }
                ArchivedICalendarParameterValue::Partstat(v) => {
                    write_param_value(out, &mut line_len, v.as_str())?;
                }
                ArchivedICalendarParameterValue::Related(v) => {
                    write_param_value(out, &mut line_len, v.as_str())?;
                }
                ArchivedICalendarParameterValue::Reltype(v) => {
                    write_param_value(out, &mut line_len, v.as_str())?;
                }
                ArchivedICalendarParameterValue::Role(v) => {
                    write_param_value(out, &mut line_len, v.as_str())?;
                }
                ArchivedICalendarParameterValue::ScheduleAgent(v) => {
                    write_param_value(out, &mut line_len, v.as_str())?;
                }
                ArchivedICalendarParameterValue::ScheduleForceSend(v) => {
                    write_param_value(out, &mut line_len, v.as_str())?;
                }
                ArchivedICalendarParameterValue::Value(v) => {
                    types = Some(v);
                    write_param_value(out, &mut line_len, v.as_str())?;
                }
                ArchivedICalendarParameterValue::Display(v) => {
                    write_param_value(out, &mut line_len, v.as_str())?;
                }
                ArchivedICalendarParameterValue::Feature(v) => {
                    write_param_value(out, &mut line_len, v.as_str())?;
                }
                ArchivedICalendarParameterValue::Duration(v) => {
                    write!(out, "{v}")?;
                    line_len += 14;
                }
                ArchivedICalendarParameterValue::Linkrel(v) => {
                    write_param_value(out, &mut line_len, v.as_str())?;
                }
                ArchivedICalendarParameterValue::Null => {
                    last_param = None;
                }
            }
        }

        write!(out, ":")?;

        if with_value {
            let (default_type, separator) = self.name.default_types();
            let separator = if !matches!(separator, ValueSeparator::Comma) {
                ";"
            } else {
                ","
            };
            let default_type = default_type.unwrap_ical();

            for (pos, value) in self.values.iter().enumerate() {
                if pos > 0 {
                    write!(out, "{separator}")?;
                    line_len += 1;
                }

                if line_len + 1 > 75 {
                    write!(out, "\r\n ")?;
                    line_len = 1;
                }

                let text = match value {
                    ArchivedICalendarValue::Binary(v) => {
                        write_bytes(out, Some(&mut line_len), v)?;
                        continue;
                    }
                    ArchivedICalendarValue::Boolean(v) => {
                        let text = if *v { "TRUE" } else { "FALSE" };
                        write!(out, "{text}")?;
                        line_len += text.len();
                        continue;
                    }
                    ArchivedICalendarValue::Uri(v) => {
                        write_uri(out, &mut line_len, v, true)?;
                        continue;
                    }
                    ArchivedICalendarValue::PartialDateTime(v) => {
                        v.format_as_ical(out, types.unwrap_or(&default_type))?;
                        line_len += 6;
                        continue;
                    }
                    ArchivedICalendarValue::Duration(v) => {
                        write!(out, "{}", v)?;
                        line_len += 6;
                        continue;
                    }
                    ArchivedICalendarValue::RecurrenceRule(v) => {
                        write!(out, "{}", v)?;
                        line_len += 6;
                        continue;
                    }
                    ArchivedICalendarValue::Period(v) => {
                        write!(out, "{}", v)?;
                        line_len += 32;
                        continue;
                    }
                    ArchivedICalendarValue::Float(v) => {
                        write!(out, "{v}")?;
                        line_len += 4;
                        continue;
                    }
                    ArchivedICalendarValue::Integer(v) => {
                        write!(out, "{v}")?;
                        line_len += 4;
                        continue;
                    }
                    ArchivedICalendarValue::Text(v) => {
                        write_text(out, &mut line_len, v, true, true)?;
                        continue;
                    }
                    ArchivedICalendarValue::CalendarScale(v) => v.as_str(),
                    ArchivedICalendarValue::Method(v) => v.as_str(),
                    ArchivedICalendarValue::Classification(v) => v.as_str(),
                    ArchivedICalendarValue::Status(v) => v.as_str(),
                    ArchivedICalendarValue::Transparency(v) => v.as_str(),
                    ArchivedICalendarValue::Action(v) => v.as_str(),
                    ArchivedICalendarValue::BusyType(v) => v.as_str(),
                    ArchivedICalendarValue::ParticipantType(v) => v.as_str(),
                    ArchivedICalendarValue::ResourceType(v) => v.as_str(),
                    ArchivedICalendarValue::Proximity(v) => v.as_str(),
                };

                write!(out, "{text}")?;
                line_len += text.len();
            }
        }
        write!(out, "\r\n")
    }
}

/*pub(crate) fn write_uri_param(
    out: &mut impl Write,
    line_len: &mut usize,
    name: &str,
    value: &ArchivedUri,
) -> std::fmt::Result {
    write!(out, "{}=\"", name)?;
    *line_len += name.len() + 3;
    write_uri(out, line_len, value, false)?;
    write!(out, "\"")
}

pub(crate) fn write_uri_params(
    out: &mut impl Write,
    line_len: &mut usize,
    name: &str,
    values: &[ArchivedUri],
) -> std::fmt::Result {
    write!(out, "{}", name)?;
    *line_len += name.len() + 1;

    for (pos, v) in values.iter().enumerate() {
        if pos > 0 {
            write!(out, ",\"")?;
        } else {
            write!(out, "=\"")?;
        }
        *line_len += 3;
        write_uri(out, line_len, v, false)?;
        write!(out, "\"")?;
    }

    Ok(())
}*/

pub(crate) fn write_uri(
    out: &mut impl Write,
    line_len: &mut usize,
    value: &ArchivedUri,
    escape: bool,
) -> std::fmt::Result {
    match value {
        ArchivedUri::Data(v) => {
            write!(out, "data:")?;
            *line_len += 5;
            if let Some(ct) = v.content_type.as_ref() {
                write!(out, "{ct};")?;
                *line_len += ct.len() + 1;
            }
            if escape {
                write!(out, "base64\\,")?;
            } else {
                write!(out, "base64,")?;
            }
            *line_len += 8;
            write_bytes(out, Some(line_len), &v.data)
        }
        ArchivedUri::Location(v) => write_text(out, line_len, v, true, true),
    }
}

impl Display for ArchivedICalendarRecurrenceRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FREQ={}", self.freq.as_str())?;
        if let Some(until) = self.until.as_ref() {
            write!(f, ";UNTIL=")?;
            until.format_as_ical(f, &ArchivedICalendarValueType::DateTime)?;
        }
        if let Some(count) = self.count.as_ref().filter(|c| **c > 0) {
            write!(f, ";COUNT={}", count)?;
        }
        if let Some(interval) = self.interval.as_ref() {
            write!(f, ";INTERVAL={}", interval)?;
        }
        if !self.bysecond.is_empty() {
            write!(f, ";BYSECOND=")?;
            for (pos, item) in self.bysecond.iter().enumerate() {
                if pos > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}", item)?;
            }
        }
        if !self.byminute.is_empty() {
            write!(f, ";BYMINUTE=")?;
            for (pos, item) in self.byminute.iter().enumerate() {
                if pos > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}", item)?;
            }
        }
        if !self.byhour.is_empty() {
            write!(f, ";BYHOUR=")?;
            for (pos, item) in self.byhour.iter().enumerate() {
                if pos > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}", item)?;
            }
        }
        if !self.byday.is_empty() {
            write!(f, ";BYDAY=")?;
            for (pos, item) in self.byday.iter().enumerate() {
                if pos > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}", item)?;
            }
        }
        if !self.bymonthday.is_empty() {
            write!(f, ";BYMONTHDAY=")?;
            for (pos, item) in self.bymonthday.iter().enumerate() {
                if pos > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}", item)?;
            }
        }
        if !self.byyearday.is_empty() {
            write!(f, ";BYYEARDAY=")?;
            for (pos, item) in self.byyearday.iter().enumerate() {
                if pos > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}", item)?;
            }
        }
        if !self.byweekno.is_empty() {
            write!(f, ";BYWEEKNO=")?;
            for (pos, item) in self.byweekno.iter().enumerate() {
                if pos > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}", item)?;
            }
        }
        if !self.bymonth.is_empty() {
            write!(f, ";BYMONTH=")?;
            for (pos, item) in self.bymonth.iter().enumerate() {
                if pos > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}", item)?;
            }
        }
        if !self.bysetpos.is_empty() {
            write!(f, ";BYSETPOS=")?;
            for (pos, item) in self.bysetpos.iter().enumerate() {
                if pos > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}", item)?;
            }
        }
        if let Some(wkst) = self.wkst.as_ref() {
            write!(f, ";WKST={}", wkst.as_str())?;
        }
        if let Some(rscale) = self.rscale.as_ref() {
            write!(f, ";RSCALE={}", rscale.as_str())?;
        }
        if let Some(skip) = self.skip.as_ref() {
            write!(f, ";SKIP={}", skip.as_str())?;
        }

        Ok(())
    }
}

impl Display for ArchivedICalendarDay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ordwk) = self.ordwk.as_ref() {
            write!(f, "{}", ordwk)?;
        }
        write!(f, "{}", self.weekday.as_str())
    }
}

impl Display for ArchivedICalendarMonth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.is_leap() {
            write!(f, "{}", self.month())
        } else {
            write!(f, "{}L", self.month())
        }
    }
}

impl Display for ArchivedICalendarPeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchivedICalendarPeriod::Range { start, end } => {
                start.format_as_ical(f, &ArchivedICalendarValueType::DateTime)?;
                write!(f, "/")?;
                end.format_as_ical(f, &ArchivedICalendarValueType::DateTime)
            }
            ArchivedICalendarPeriod::Duration { start, duration } => {
                start.format_as_ical(f, &ArchivedICalendarValueType::DateTime)?;
                write!(f, "/{}", duration)
            }
        }
    }
}

impl Display for ArchivedICalendarDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.neg {
            write!(f, "-")?;
        }
        write!(f, "P")?;
        if self.weeks != 0 {
            write!(f, "{}W", self.weeks)?;
        }
        if self.days != 0 {
            write!(f, "{}D", self.days)?;
        }
        if self.hours != 0 || self.minutes != 0 || self.seconds != 0 {
            write!(f, "T")?;
            if self.hours != 0 {
                write!(f, "{}H", self.hours)?;
            }
            if self.minutes != 0 {
                write!(f, "{}M", self.minutes)?;
            }
            if self.seconds != 0 {
                write!(f, "{}S", self.seconds)?;
            }
        }

        Ok(())
    }
}

impl ArchivedPartialDateTime {
    pub fn format_as_ical(
        &self,
        out: &mut impl Write,
        fmt: &ArchivedICalendarValueType,
    ) -> std::fmt::Result {
        if matches!(
            fmt,
            ArchivedICalendarValueType::Date | ArchivedICalendarValueType::DateTime
        ) {
            write!(
                out,
                "{:04}{:02}{:02}",
                self.year
                    .as_ref()
                    .map(|n| n.to_native())
                    .unwrap_or_default(),
                self.month.as_ref().copied().unwrap_or_default(),
                self.day.as_ref().copied().unwrap_or_default(),
            )?;
        }

        if matches!(fmt, ArchivedICalendarValueType::DateTime) {
            write!(out, "T")?;
        }

        if matches!(
            fmt,
            ArchivedICalendarValueType::DateTime | ArchivedICalendarValueType::Time
        ) {
            write!(
                out,
                "{:02}{:02}{:02}",
                self.hour.as_ref().copied().unwrap_or_default(),
                self.minute.as_ref().copied().unwrap_or_default(),
                self.second.as_ref().copied().unwrap_or_default(),
            )?;

            if matches!(
                (self.tz_hour.as_ref(), self.tz_minute.as_ref()),
                (Some(0), Some(0))
            ) {
                write!(out, "Z")?;
            }
        }

        if matches!(fmt, ArchivedICalendarValueType::UtcOffset) {
            if self.tz_minus {
                write!(out, "-")?;
            } else {
                write!(out, "+")?;
            }

            write!(
                out,
                "{:02}{:02}",
                self.tz_hour.as_ref().copied().unwrap_or_default(),
                self.tz_minute.as_ref().copied().unwrap_or_default(),
            )?;
        }

        Ok(())
    }
}

impl Display for ArchivedICalendar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_to(f)
    }
}
