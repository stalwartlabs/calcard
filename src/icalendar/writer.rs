/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::{
    ICalendar, ICalendarDay, ICalendarDuration, ICalendarEntry, ICalendarPeriod,
    ICalendarRecurrenceRule, ICalendarValueType,
};
use crate::{
    common::{
        CalendarScale, IanaString, PartialDateTime,
        writer::{FoldingWriter, LineWriter, write_bytes, write_param_value, write_text},
    },
    icalendar::{
        ICalendarMonth, ICalendarParameterName, ICalendarParameterValue, ICalendarValue, Uri,
        ValueSeparator,
    },
};
use std::{
    fmt::{Display, Write},
    slice::Iter,
};

impl ICalendar {
    pub fn write_to(&self, out: &mut impl Write) -> std::fmt::Result {
        let mut component_iter: Iter<'_, u32> = [0].iter();
        let mut component_stack = Vec::with_capacity(4);

        loop {
            if let Some(component_id) = component_iter.next() {
                let component = self.components.get(*component_id as usize).unwrap();
                write_boundary(out, "BEGIN:", component.component_type.as_str())?;

                for entry in &component.entries {
                    entry.write_to(out)?;
                }

                if !component.component_ids.is_empty() {
                    component_stack.push((component, component_iter));
                    component_iter = component.component_ids.iter();
                } else {
                    write_boundary(out, "END:", component.component_type.as_str())?;
                }
            } else if let Some((component, iter)) = component_stack.pop() {
                write_boundary(out, "END:", component.component_type.as_str())?;
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
        let mut folded = FoldingWriter::new(out);
        let out = &mut folded;

        out.write_atomic(self.name.as_str())?;

        if matches!(self.values.first(), Some(ICalendarValue::Binary(_))) {
            out.write_atomic(";ENCODING=BASE64")?;
        }

        let mut types = None;
        let mut last_param: Option<&ICalendarParameterName> = None;

        for param in &self.params {
            if last_param.is_some_and(|last_param| last_param == &param.name) {
                out.write_atomic(",")?;
            } else {
                out.write_atomic(";")?;
                out.write_atomic(param.name.as_str())?;
                if !matches!(param.value, ICalendarParameterValue::Null) {
                    out.write_atomic("=")?;
                }
                last_param = Some(&param.name);
            }

            match &param.value {
                ICalendarParameterValue::Text(v) => {
                    write_param_value(out, v)?;
                }
                ICalendarParameterValue::Integer(i) => {
                    write!(out, "{i}")?;
                }
                ICalendarParameterValue::Bool(v) => {
                    let v = if !matches!(param.name, ICalendarParameterName::Range) {
                        if *v { "TRUE" } else { "FALSE" }
                    } else {
                        "THISANDFUTURE"
                    };
                    out.write_atomic(v)?;
                }
                ICalendarParameterValue::Uri(uri) => {
                    out.write_atomic("\"")?;
                    write_uri(out, uri, false)?;
                    out.write_atomic("\"")?;
                }
                ICalendarParameterValue::Cutype(v) => {
                    write_param_value(out, v.as_str())?;
                }
                ICalendarParameterValue::Fbtype(v) => {
                    write_param_value(out, v.as_str())?;
                }
                ICalendarParameterValue::Partstat(v) => {
                    write_param_value(out, v.as_str())?;
                }
                ICalendarParameterValue::Related(v) => {
                    write_param_value(out, v.as_str())?;
                }
                ICalendarParameterValue::Reltype(v) => {
                    write_param_value(out, v.as_str())?;
                }
                ICalendarParameterValue::Role(v) => {
                    write_param_value(out, v.as_str())?;
                }
                ICalendarParameterValue::ScheduleAgent(v) => {
                    write_param_value(out, v.as_str())?;
                }
                ICalendarParameterValue::ScheduleForceSend(v) => {
                    write_param_value(out, v.as_str())?;
                }
                ICalendarParameterValue::Value(v) => {
                    types = Some(v);
                    write_param_value(out, v.as_str())?;
                }
                ICalendarParameterValue::Display(v) => {
                    write_param_value(out, v.as_str())?;
                }
                ICalendarParameterValue::Feature(v) => {
                    write_param_value(out, v.as_str())?;
                }
                ICalendarParameterValue::Duration(v) => {
                    write!(out, "{v}")?;
                }
                ICalendarParameterValue::Linkrel(v) => {
                    write_param_value(out, v.as_str())?;
                }
                ICalendarParameterValue::Null => {
                    last_param = None;
                }
            }
        }

        out.write_atomic(":")?;

        let (default_type, separator) = self.name.default_types();
        let separator = if !matches!(separator, ValueSeparator::Comma) {
            ";"
        } else {
            ","
        };
        let default_type = default_type.unwrap_ical();

        for (pos, value) in self.values.iter().enumerate() {
            if pos > 0 {
                out.write_atomic(separator)?;
            }

            let text = match value {
                ICalendarValue::Binary(v) => {
                    write_bytes(out, v)?;
                    continue;
                }
                ICalendarValue::Boolean(v) => {
                    out.write_atomic(if *v { "TRUE" } else { "FALSE" })?;
                    continue;
                }
                ICalendarValue::Uri(v) => {
                    write_uri(out, v, true)?;
                    continue;
                }
                ICalendarValue::PartialDateTime(v) => {
                    v.format_as_ical(out, types.unwrap_or(&default_type))?;
                    continue;
                }
                ICalendarValue::Duration(v) => {
                    write!(out, "{}", v)?;
                    continue;
                }
                ICalendarValue::RecurrenceRule(v) => {
                    write!(out, "{}", v)?;
                    continue;
                }
                ICalendarValue::Period(v) => {
                    write!(out, "{}", v)?;
                    continue;
                }
                ICalendarValue::Float(v) => {
                    write!(out, "{v}")?;
                    continue;
                }
                ICalendarValue::Integer(v) => {
                    write!(out, "{v}")?;
                    continue;
                }
                ICalendarValue::Text(v) => {
                    let escape =
                        !matches!(types.unwrap_or(&default_type), ICalendarValueType::Recur);
                    write_text(out, v, escape, escape)?;
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

            out.write_atomic(text)?;
        }

        out.end_line()
    }
}

pub(crate) fn write_uri<W: Write>(
    out: &mut FoldingWriter<'_, W>,
    value: &Uri,
    escape: bool,
) -> std::fmt::Result {
    match value {
        Uri::Data(v) => {
            let media_type = v.content_type.as_deref().unwrap_or_default();
            out.write_str("data:")?;
            out.write_str(media_type)?;
            out.write_str(";")?;
            if escape {
                out.write_atomic("base64\\,")?;
            } else {
                out.write_atomic("base64,")?;
            }
            write_bytes(out, &v.data)
        }
        Uri::Location(v) => write_text(out, v, escape, escape),
    }
}

fn write_boundary(out: &mut impl Write, keyword: &str, name: &str) -> std::fmt::Result {
    let mut folded = FoldingWriter::new(out);
    folded.write_atomic(keyword)?;
    folded.write_atomic(name)?;
    folded.end_line()
}

#[cfg(feature = "rkyv")]
pub(crate) fn write_component_begin(out: &mut impl Write, name: &str) -> std::fmt::Result {
    write_boundary(out, "BEGIN:", name)
}

#[cfg(feature = "rkyv")]
pub(crate) fn write_component_end(out: &mut impl Write, name: &str) -> std::fmt::Result {
    write_boundary(out, "END:", name)
}

impl Uri {
    pub fn to_unwrapped_string(&self) -> String {
        match self {
            Uri::Data(v) => v.to_unwrapped_string(),
            Uri::Location(v) => v.to_string(),
        }
    }

    pub fn into_unwrapped_string(self) -> String {
        match self {
            Uri::Data(v) => v.to_unwrapped_string(),
            Uri::Location(v) => v,
        }
    }
}

impl Display for ICalendarRecurrenceRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FREQ={}", self.freq.as_str())?;
        if let Some(until) = &self.until {
            write!(f, ";UNTIL=")?;
            until.format_as_ical(
                f,
                if until.has_date_and_time() {
                    &ICalendarValueType::DateTime
                } else {
                    &ICalendarValueType::Date
                },
            )?;
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
        if let Some(rscale) = &self.rscale {
            write!(f, ";RSCALE={}", rscale.as_str())?;
        } else if self.skip.is_some() {
            write!(f, ";RSCALE={}", CalendarScale::Gregorian.as_str())?;
        }
        if let Some(skip) = &self.skip {
            write!(f, ";SKIP={}", skip.as_str())?;
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

impl Display for ICalendarMonth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.is_leap() {
            write!(f, "{}", self.month())
        } else {
            write!(f, "{}L", self.month())
        }
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
        if self.is_empty() {
            return write!(f, "T0S");
        }
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

#[cfg(test)]
mod tests {
    use crate::{Entry, Parser, icalendar::ICalendar};

    fn parse(input: &str) -> ICalendar {
        let mut parser = Parser::new(input);
        let Entry::ICalendar(ical) = parser.entry() else {
            panic!("expected iCalendar for {input}");
        };
        ical
    }

    fn write(ical: &ICalendar) -> String {
        let out = ical.to_string();

        #[cfg(feature = "rkyv")]
        {
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(ical).unwrap();
            let archived =
                rkyv::access::<crate::icalendar::ArchivedICalendar, rkyv::rancor::Error>(&bytes)
                    .unwrap();
            assert_eq!(out, archived.to_string(), "archived writer diverged");
        }

        out
    }

    #[test]
    fn test_write_fold_width() {
        for input in [
            "RRULE:FREQ=WEEKLY;UNTIL=20210309T080000Z;INTERVAL=1;BYDAY=MO,TU,WE,TH,FR,SA;WKST=MO",
            "RDATE;VALUE=PERIOD:19970101T180000Z/19970102T070000Z,19970109T180000Z/PT5H30M",
            "FREEBUSY;FBTYPE=BUSY:20120103T091500Z/20120103T101500Z,20120113T130000Z/20120113T150000Z",
            "EXDATE;TZID=US/Central:20170706T090000,20170713T090000,20170720T090000,20170803T090000",
            "DTSTART;TZID=/softwarestudio.org/Olson_20011030_5/America/Chicago:20030515T183000",
            "REQUEST-STATUS:EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE;;",
        ] {
            let ical = parse(&format!(
                "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//x//EN\r\n\
                 BEGIN:VEVENT\r\nUID:1\r\n{input}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
            ));

            let out = write(&ical);
            crate::common::writer::assert_fold_width(&out, input);

            assert_eq!(write(&parse(&out)), out, "not idempotent for {input}");
        }
    }

    #[test]
    fn test_write_component_boundary_fold_width() {
        let name = "X".repeat(90);
        let ical = parse(&format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//x//EN\r\n\
             BEGIN:{name}\r\nEND:{name}\r\nEND:VCALENDAR\r\n"
        ));

        let out = write(&ical);
        crate::common::writer::assert_fold_width(&out, &name);

        assert_eq!(write(&parse(&out)), out, "not idempotent for {name}");
    }
}
