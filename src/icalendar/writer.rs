use std::{
    fmt::{Display, Write},
    slice::Iter,
};

use crate::{
    common::{
        writer::{write_bytes, write_param, write_param_value, write_params, write_value},
        PartialDateTime,
    },
    icalendar::{ICalendarValue, Uri, ValueSeparator},
};

use super::{
    ICalendar, ICalendarDay, ICalendarDuration, ICalendarEntry, ICalendarParameter,
    ICalendarPeriod, ICalendarRecurrenceRule, ICalendarValueType,
};

impl ICalendar {
    pub fn write_to(&self, out: &mut impl Write) -> std::fmt::Result {
        let mut component_iter: Iter<'_, u16> = [0].iter();
        let mut component_stack = Vec::with_capacity(4);

        loop {
            if let Some(component_id) = component_iter.next() {
                let component = self.components.get(*component_id as usize).unwrap();
                write!(out, "BEGIN:{}\r\n", component.component_type.as_str())?;

                for entry in &component.entries {
                    entry.write_to(out)?;
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

impl ICalendarEntry {
    pub fn write_to(&self, out: &mut impl Write) -> std::fmt::Result {
        let mut line_len = 0;

        let entry_name = self.name.as_str();
        write!(out, "{}", entry_name)?;
        line_len += entry_name.len();

        if matches!(self.values.first(), Some(ICalendarValue::Binary(_))) {
            write!(out, ";ENCODING=BASE64")?;
            line_len += 18;
        }

        let mut value_type = None;
        for param in &self.params {
            write!(out, ";")?;
            line_len += 1;

            if line_len + 1 > 75 {
                write!(out, "\r\n ")?;
                line_len = 1;
            }

            match param {
                ICalendarParameter::Altrep(v) => {
                    write_uri_param(out, &mut line_len, "ALTREP", v)?;
                }
                ICalendarParameter::Cn(v) => {
                    write_param(out, &mut line_len, "CN", v)?;
                }
                ICalendarParameter::Cutype(v) => {
                    write_param(out, &mut line_len, "CUTYPE", v)?;
                }
                ICalendarParameter::DelegatedFrom(v) => {
                    write_uri_params(out, &mut line_len, "DELEGATED-FROM", v)?;
                }
                ICalendarParameter::DelegatedTo(v) => {
                    write_uri_params(out, &mut line_len, "DELEGATED-TO", v)?;
                }
                ICalendarParameter::Dir(v) => {
                    write_uri_param(out, &mut line_len, "DIR", v)?;
                }
                ICalendarParameter::Fmttype(v) => {
                    write_param(out, &mut line_len, "FMTTYPE", v)?;
                }
                ICalendarParameter::Fbtype(v) => {
                    write_param(out, &mut line_len, "FBTYPE", v)?;
                }
                ICalendarParameter::Language(v) => {
                    write_param(out, &mut line_len, "LANGUAGE", v)?;
                }
                ICalendarParameter::Member(v) => {
                    write_uri_params(out, &mut line_len, "MEMBER", v)?;
                }
                ICalendarParameter::Partstat(v) => {
                    write_param(out, &mut line_len, "PARTSTAT", v)?;
                }
                ICalendarParameter::Range => {
                    write!(out, "RANGE=THISANDFUTURE")?;
                    line_len += 18;
                }
                ICalendarParameter::Related(v) => {
                    write_param(out, &mut line_len, "RELATED", v)?;
                }
                ICalendarParameter::Reltype(v) => {
                    write_param(out, &mut line_len, "RELTYPE", v)?;
                }
                ICalendarParameter::Role(v) => {
                    write_param(out, &mut line_len, "ROLE", v)?;
                }
                ICalendarParameter::Rsvp(v) => {
                    write_param(
                        out,
                        &mut line_len,
                        "RSVP",
                        if *v { "TRUE" } else { "FALSE" },
                    )?;
                }
                ICalendarParameter::ScheduleAgent(v) => {
                    write_param(out, &mut line_len, "SCHEDULE-AGENT", v)?;
                }
                ICalendarParameter::ScheduleForceSend(v) => {
                    write_param(out, &mut line_len, "SCHEDULE-FORCE-SEND", v)?;
                }
                ICalendarParameter::ScheduleStatus(v) => {
                    write_param(out, &mut line_len, "SCHEDULE-STATUS", v)?;
                }
                ICalendarParameter::SentBy(v) => {
                    write_uri_param(out, &mut line_len, "SENT-BY", v)?;
                }
                ICalendarParameter::Tzid(v) => {
                    write_param(out, &mut line_len, "TZID", v)?;
                }
                ICalendarParameter::Value(v) => {
                    write_param(out, &mut line_len, "VALUE", v)?;
                    value_type = Some(v);
                }
                ICalendarParameter::Display(v) => {
                    write_params(out, &mut line_len, "DISPLAY", v)?;
                }
                ICalendarParameter::Email(v) => {
                    write_param(out, &mut line_len, "EMAIL", v)?;
                }
                ICalendarParameter::Feature(v) => {
                    write_params(out, &mut line_len, "FEATURE", v)?;
                }
                ICalendarParameter::Label(v) => {
                    write_param(out, &mut line_len, "LABEL", v)?;
                }
                ICalendarParameter::Size(v) => {
                    write!(out, "SIZE={}", v)?;
                    line_len += 8;
                }
                ICalendarParameter::Filename(v) => {
                    write_param(out, &mut line_len, "FILENAME", v)?;
                }
                ICalendarParameter::ManagedId(v) => {
                    write_param(out, &mut line_len, "MANAGED-ID", v)?;
                }
                ICalendarParameter::Order(v) => {
                    write!(out, "ORDER={}", v)?;
                    line_len += 8;
                }
                ICalendarParameter::Schema(v) => {
                    write_uri_param(out, &mut line_len, "SCHEMA", v)?;
                }
                ICalendarParameter::Derived(v) => {
                    write_param(
                        out,
                        &mut line_len,
                        "DERIVED",
                        if *v { "TRUE" } else { "FALSE" },
                    )?;
                }
                ICalendarParameter::Gap(v) => {
                    write!(out, "GAP={}", v)?;
                    line_len += 14;
                }
                ICalendarParameter::Linkrel(v) => {
                    write_uri_param(out, &mut line_len, "LINKREL", v)?;
                }
                ICalendarParameter::Other(v) => {
                    for (pos, item) in v.iter().enumerate() {
                        if pos == 0 {
                            write!(out, "{item}")?;
                            line_len += item.len() + 1;
                        } else {
                            if pos == 1 {
                                write!(out, "=")?;
                            } else {
                                write!(out, ",")?;
                            }
                            line_len += 1;

                            write_param_value(out, &mut line_len, item)?;
                        }
                    }
                }
            }
        }

        write!(out, ":")?;

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
                ICalendarValue::Binary(v) => {
                    write_bytes(out, &mut line_len, v)?;
                    continue;
                }
                ICalendarValue::Boolean(v) => {
                    let text = if *v { "TRUE" } else { "FALSE" };
                    write!(out, "{text}")?;
                    line_len += text.len();
                    continue;
                }
                ICalendarValue::Uri(v) => {
                    write_uri(out, &mut line_len, v, true)?;
                    continue;
                }
                ICalendarValue::PartialDateTime(v) => {
                    v.format_as_ical(out, value_type.unwrap_or(&default_type))?;
                    line_len += 6;
                    continue;
                }
                ICalendarValue::Duration(v) => {
                    write!(out, "{}", v)?;
                    line_len += 6;
                    continue;
                }
                ICalendarValue::RecurrenceRule(v) => {
                    write!(out, "{}", v)?;
                    line_len += 6;
                    continue;
                }
                ICalendarValue::Period(v) => {
                    write!(out, "{}", v)?;
                    line_len += 32;
                    continue;
                }
                ICalendarValue::Float(v) => {
                    write!(out, "{v}")?;
                    line_len += 4;
                    continue;
                }
                ICalendarValue::Integer(v) => {
                    write!(out, "{v}")?;
                    line_len += 4;
                    continue;
                }
                ICalendarValue::Text(v) => {
                    write_value(out, &mut line_len, v)?;
                    continue;
                }
                ICalendarValue::CalendarScale(v) => v.as_str(),
                ICalendarValue::Method(v) => v.as_str(),
                ICalendarValue::Classification(v) => v.as_str(),
                ICalendarValue::Status(v) => v.as_str(),
                ICalendarValue::Transparency(v) => v.as_str(),
                ICalendarValue::Action(v) => v.as_str(),
                ICalendarValue::BusyType(v) => v.as_str(),
                ICalendarValue::ParticipantType(v) => v.as_str(),
                ICalendarValue::ResourceType(v) => v.as_str(),
                ICalendarValue::Proximity(v) => v.as_str(),
            };

            write!(out, "{text}")?;
            line_len += text.len();
        }

        write!(out, "\r\n")
    }
}

pub(crate) fn write_uri_param(
    out: &mut impl Write,
    line_len: &mut usize,
    name: &str,
    value: &Uri,
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
    values: &[Uri],
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
}

pub(crate) fn write_uri(
    out: &mut impl Write,
    line_len: &mut usize,
    value: &Uri,
    escape: bool,
) -> std::fmt::Result {
    match value {
        Uri::Data(v) => {
            write!(out, "data:")?;
            *line_len += 5;
            if let Some(ct) = &v.content_type {
                write!(out, "{ct};")?;
                *line_len += ct.len() + 1;
            }
            if escape {
                write!(out, "base64\\,")?;
            } else {
                write!(out, "base64,")?;
            }
            *line_len += 8;
            write_bytes(out, line_len, &v.data)
        }
        Uri::Location(v) => write_value(out, line_len, v),
    }
}

impl Display for ICalendarRecurrenceRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FREQ={}", self.freq.as_str())?;
        if let Some(until) = &self.until {
            write!(f, ";UNTIL=")?;
            until.format_as_ical(f, &ICalendarValueType::DateTime)?;
        }
        if let Some(count) = self.count.filter(|c| *c > 0) {
            write!(f, ";COUNT={}", count)?;
        }
        if let Some(interval) = self.interval {
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
        if let Some(wkst) = self.wkst {
            write!(f, ";WKST={}", wkst.as_str())?;
        }

        Ok(())
    }
}

impl Display for ICalendarDay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ordwk) = self.ordwk {
            write!(f, "{}", ordwk)?;
        }
        write!(f, "{}", self.weekday.as_str())
    }
}

impl Display for ICalendarPeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ICalendarPeriod::Range { start, end } => {
                start.format_as_ical(f, &ICalendarValueType::DateTime)?;
                write!(f, "/")?;
                end.format_as_ical(f, &ICalendarValueType::DateTime)
            }
            ICalendarPeriod::Duration { start, duration } => {
                start.format_as_ical(f, &ICalendarValueType::DateTime)?;
                write!(f, "/{}", duration)
            }
        }
    }
}

impl Display for ICalendarDuration {
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

impl PartialDateTime {
    pub fn format_as_ical(
        &self,
        out: &mut impl Write,
        fmt: &ICalendarValueType,
    ) -> std::fmt::Result {
        if matches!(fmt, ICalendarValueType::Date | ICalendarValueType::DateTime) {
            write!(
                out,
                "{:04}{:02}{:02}",
                self.year.unwrap_or_default(),
                self.month.unwrap_or_default(),
                self.day.unwrap_or_default()
            )?;
        }

        if matches!(fmt, ICalendarValueType::DateTime) {
            write!(out, "T")?;
        }

        if matches!(fmt, ICalendarValueType::DateTime | ICalendarValueType::Time) {
            write!(
                out,
                "{:02}{:02}{:02}",
                self.hour.unwrap_or_default(),
                self.minute.unwrap_or_default(),
                self.second.unwrap_or_default()
            )?;

            if matches!((self.tz_hour, self.tz_minute), (Some(0), Some(0))) {
                write!(out, "Z")?;
            }
        }

        if matches!(fmt, ICalendarValueType::UtcOffset) {
            if self.tz_minus {
                write!(out, "-")?;
            } else {
                write!(out, "+")?;
            }

            write!(
                out,
                "{:02}{:02}",
                self.tz_hour.unwrap_or_default(),
                self.tz_minute.unwrap_or_default(),
            )?;
        }

        Ok(())
    }
}

impl Display for ICalendar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_to(f)
    }
}
