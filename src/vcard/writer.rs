/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::{PartialDateTime, VCard, VCardEntry, VCardValueType, VCardVersion};
use crate::{
    common::{
        IanaString,
        parser::Timestamp,
        writer::{write_bytes, write_jscomps, write_param_value, write_text},
    },
    vcard::{VCardParameterName, VCardParameterValue, VCardProperty, VCardValue, ValueSeparator},
};
use std::fmt::{Display, Write};

impl VCard {
    pub fn write_to(&self, out: &mut impl Write, version: VCardVersion) -> std::fmt::Result {
        write!(out, "BEGIN:VCARD\r\n")?;
        write!(out, "VERSION:{version}\r\n")?;
        let is_v4 = matches!(version, VCardVersion::V4_0);
        for entry in &self.entries {
            if !matches!(
                entry.name,
                VCardProperty::Begin | VCardProperty::End | VCardProperty::Version
            ) {
                entry.write_to(out, is_v4)?;
            }
        }
        write!(out, "END:VCARD\r\n")
    }
}

impl VCardEntry {
    pub fn write_to(&self, out: &mut impl Write, is_v4: bool) -> std::fmt::Result {
        let mut line_len = 0;

        if let Some(group_name) = &self.group {
            write!(out, "{group_name}.")?;
            line_len += group_name.len() + 1;
        }

        let entry_name = self.name.as_str();
        write!(out, "{}", entry_name)?;
        line_len += entry_name.len();
        let mut types = None;
        let mut last_param: Option<&VCardParameterName> = None;

        for param in &self.params {
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
                if !matches!(param.value, VCardParameterValue::Null) {
                    write!(out, "{name}=")?;
                } else {
                    write!(out, "{name}")?;
                }
                line_len += need_len;
                last_param = Some(&param.name);
            }

            match &param.value {
                VCardParameterValue::Text(v) => {
                    write_param_value(out, &mut line_len, v)?;
                }
                VCardParameterValue::Integer(i) => {
                    write!(out, "{i}")?;
                    line_len += 2;
                }
                VCardParameterValue::Timestamp(v) => {
                    write!(out, "{}", Timestamp(*v))?;
                    line_len += 17;
                }
                VCardParameterValue::Bool(v) => {
                    let v = if *v { "TRUE" } else { "FALSE" };
                    line_len += v.len();
                    write!(out, "{v}")?;
                }
                VCardParameterValue::ValueType(v) => {
                    if types.is_none() {
                        types = Some(v);
                    }
                    write_param_value(out, &mut line_len, v.as_str())?;
                }
                VCardParameterValue::Type(v) => {
                    write_param_value(out, &mut line_len, v.as_str())?;
                }
                VCardParameterValue::Calscale(v) => {
                    write_param_value(out, &mut line_len, v.as_str())?;
                }
                VCardParameterValue::Level(v) => {
                    write_param_value(out, &mut line_len, v.as_str())?;
                }
                VCardParameterValue::Phonetic(v) => {
                    write_param_value(out, &mut line_len, v.as_str())?;
                }
                VCardParameterValue::Jscomps(v) => {
                    out.write_str("\"")?;
                    line_len += 1;
                    write_jscomps(out, &mut line_len, v)?;
                    out.write_str("\"")?;
                    last_param = None;
                }
                VCardParameterValue::Null => {
                    last_param = None;
                }
            }
        }

        if !is_v4
            && self
                .values
                .iter()
                .any(|v| matches!(v, VCardValue::Binary(_)))
        {
            write!(out, ";ENCODING=b")?;
            line_len += 11;
        }

        write!(out, ":")?;

        let (default_type, value_separator) = self.name.default_types();
        let default_type = default_type.unwrap_vcard();

        let mut separator = ";";
        let mut escape_semicolon = matches!(types.unwrap_or(&default_type), VCardValueType::Text);
        let mut escape_comma = escape_semicolon;

        match value_separator {
            ValueSeparator::Comma => {
                escape_comma = true;
                separator = ",";
            }
            ValueSeparator::Semicolon => escape_semicolon = true,
            ValueSeparator::SemicolonAndComma => {
                escape_semicolon = true;
                escape_comma = true;
            }
            _ => {}
        }

        for (pos, value) in self.values.iter().enumerate() {
            if pos > 0 {
                write!(out, "{separator}")?;
                line_len += 1;
            }

            if line_len + 1 > 75 {
                write!(out, "\r\n ")?;
                line_len = 1;
            }

            match value {
                VCardValue::Text(v) => {
                    write_text(out, &mut line_len, v, escape_semicolon, escape_comma)?;
                }
                VCardValue::Component(v) => {
                    for (pos, item) in v.iter().enumerate() {
                        if pos > 0 {
                            write!(out, ",")?;
                            line_len += 1;
                        }
                        write_text(out, &mut line_len, item, true, true)?;
                    }
                }
                VCardValue::Integer(v) => {
                    write!(out, "{v}")?;
                    line_len += 4;
                }
                VCardValue::Float(v) => {
                    write!(out, "{v}")?;
                    line_len += 4;
                }
                VCardValue::Boolean(v) => {
                    let text = if *v { "TRUE" } else { "FALSE" };
                    write!(out, "{text}")?;
                    line_len += text.len();
                }
                VCardValue::PartialDateTime(v) => {
                    let typ = if pos == 0 {
                        types
                    } else {
                        self.parameters(&VCardParameterName::Value)
                            .nth(pos)
                            .and_then(|v| v.as_value_type())
                            .and_then(|v| v.iana().copied())
                    }
                    .unwrap_or(&default_type);
                    if is_v4 {
                        v.format_as_vcard(out, typ)?;
                    } else {
                        v.format_as_legacy_vcard(out, typ)?;
                    }
                    line_len += 16;
                }
                VCardValue::Binary(v) => {
                    if is_v4 {
                        write!(out, "data:")?;
                        line_len += 5;
                        if let Some(ct) = &v.content_type {
                            write!(out, "{ct};")?;
                            line_len += ct.len() + 1;
                        }
                        write!(out, "base64\\,")?;
                        line_len += 8;
                    }
                    write_bytes(out, &mut line_len, &v.data)?;
                }
                VCardValue::Sex(v) => {
                    let text = v.as_str();
                    write!(out, "{text}")?;
                    line_len += text.len();
                }
                VCardValue::GramGender(v) => {
                    let text = v.as_str();
                    write!(out, "{text}")?;
                    line_len += text.len();
                }
                VCardValue::Kind(v) => {
                    let text = v.as_str();
                    write!(out, "{text}")?;
                    line_len += text.len();
                }
            }
        }

        write!(out, "\r\n")
    }
}

impl PartialDateTime {
    pub fn format_as_vcard(&self, out: &mut impl Write, fmt: &VCardValueType) -> std::fmt::Result {
        if matches!(fmt, VCardValueType::Timestamp) {
            write!(
                out,
                "{:04}{:02}{:02}T{:02}{:02}{:02}",
                self.year.unwrap_or_default(),
                self.month.unwrap_or_default(),
                self.day.unwrap_or_default(),
                self.hour.unwrap_or_default(),
                self.minute.unwrap_or_default(),
                self.second.unwrap_or_default()
            )?;

            if let Some(tz_hour) = self.tz_hour {
                let tz_minute = self.tz_minute.unwrap_or_default();
                if tz_hour == 0 && tz_minute == 0 {
                    write!(out, "Z")?;
                } else {
                    write!(
                        out,
                        "{}{:02}",
                        if self.tz_minus { "-" } else { "+" },
                        tz_hour,
                    )?;

                    if let Some(tz_minute) = self.tz_minute {
                        write!(out, "{:02}", tz_minute)?;
                    }
                }
            }
            Ok(())
        } else {
            let missing_time =
                self.hour.is_none() && self.minute.is_none() && self.second.is_none();
            let missing_tz = self.tz_hour.is_none();

            if matches!(
                fmt,
                VCardValueType::Date | VCardValueType::DateAndOrTime | VCardValueType::DateTime
            ) {
                match (self.year, self.month, self.day) {
                    (Some(year), Some(month), Some(day)) => {
                        write!(out, "{:04}{:02}{:02}", year, month, day)?;
                    }
                    (Some(year), Some(month), None) => {
                        if missing_time && missing_tz {
                            write!(out, "{:04}-{:02}", year, month)?;
                        } else {
                            write!(out, "{:04}{:02}", year, month)?;
                        }
                    }
                    (None, Some(month), Some(day)) => {
                        write!(out, "--{:02}{:02}", month, day)?;
                    }
                    (None, None, Some(day)) => {
                        write!(out, "---{:02}", day)?;
                    }
                    (Some(year), None, None) => {
                        write!(out, "{:04}", year)?;
                    }
                    (None, Some(month), None) => {
                        write!(out, "--{month}")?;
                    }
                    _ => {}
                }
            }

            if matches!(
                fmt,
                VCardValueType::DateAndOrTime | VCardValueType::DateTime | VCardValueType::Time
            ) && !missing_time
            {
                if matches!(
                    fmt,
                    VCardValueType::DateAndOrTime | VCardValueType::DateTime
                ) {
                    write!(out, "T")?;
                }
                let mut last_is_some = false;
                for value in [&self.hour, &self.minute, &self.second].iter() {
                    if let Some(value) = value {
                        write!(out, "{:02}", value)?;
                        last_is_some = true;
                    } else if !last_is_some {
                        write!(out, "-")?;
                    }
                }
            }

            if matches!(
                fmt,
                VCardValueType::DateAndOrTime
                    | VCardValueType::DateTime
                    | VCardValueType::Time
                    | VCardValueType::UtcOffset
            ) {
                match (self.tz_hour, self.tz_minute) {
                    (Some(0), Some(0)) | (Some(0), _) => {
                        write!(out, "Z")?;
                    }
                    (Some(hour), Some(minute)) => {
                        if self.tz_minus {
                            write!(out, "-")?;
                        } else {
                            write!(out, "+")?;
                        }
                        write!(out, "{hour:02}{minute:02}")?;
                    }
                    (Some(hour), None) => {
                        if self.tz_minus {
                            write!(out, "-")?;
                        } else {
                            write!(out, "+")?;
                        }
                        write!(out, "{hour:02}")?;
                    }
                    _ => {}
                }
            }

            Ok(())
        }
    }

    pub fn format_as_legacy_vcard(
        &self,
        out: &mut impl Write,
        fmt: &VCardValueType,
    ) -> std::fmt::Result {
        if matches!(fmt, VCardValueType::Timestamp) {
            write!(
                out,
                "{:04}{:02}{:02}T{:02}{:02}{:02}",
                self.year.unwrap_or_default(),
                self.month.unwrap_or_default(),
                self.day.unwrap_or_default(),
                self.hour.unwrap_or_default(),
                self.minute.unwrap_or_default(),
                self.second.unwrap_or_default()
            )?;

            if let Some(tz_hour) = self.tz_hour {
                let tz_minute = self.tz_minute.unwrap_or_default();
                if tz_hour == 0 && tz_minute == 0 {
                    write!(out, "Z")?;
                } else {
                    write!(
                        out,
                        "{}{:02}",
                        if self.tz_minus { "-" } else { "+" },
                        tz_hour,
                    )?;

                    if let Some(tz_minute) = self.tz_minute {
                        write!(out, "{:02}", tz_minute)?;
                    }
                }
            }
            Ok(())
        } else {
            let missing_time =
                self.hour.is_none() && self.minute.is_none() && self.second.is_none();

            if matches!(
                fmt,
                VCardValueType::Date | VCardValueType::DateAndOrTime | VCardValueType::DateTime
            ) {
                match (self.year, self.month, self.day) {
                    (Some(year), Some(month), Some(day)) => {
                        write!(out, "{:04}-{:02}-{:02}", year, month, day)?;
                    }
                    (Some(year), Some(month), None) => {
                        write!(out, "{:04}-{:02}", year, month)?;
                    }
                    (None, Some(month), Some(day)) => {
                        write!(out, "--{:02}-{:02}", month, day)?;
                    }
                    (None, None, Some(day)) => {
                        write!(out, "---{:02}", day)?;
                    }
                    (Some(year), None, None) => {
                        write!(out, "{:04}", year)?;
                    }
                    (None, Some(month), None) => {
                        write!(out, "--{month}")?;
                    }
                    _ => {}
                }
            }

            if matches!(
                fmt,
                VCardValueType::DateAndOrTime | VCardValueType::DateTime | VCardValueType::Time
            ) && !missing_time
            {
                if matches!(
                    fmt,
                    VCardValueType::DateAndOrTime | VCardValueType::DateTime
                ) {
                    write!(out, "T")?;
                }
                let mut last_is_some = false;
                for value in [&self.hour, &self.minute, &self.second].iter() {
                    if let Some(value) = value {
                        if last_is_some {
                            write!(out, ":")?;
                        }
                        write!(out, "{:02}", value)?;
                        last_is_some = true;
                    } else if !last_is_some {
                        write!(out, "-")?;
                    }
                }
            }

            if matches!(
                fmt,
                VCardValueType::DateAndOrTime
                    | VCardValueType::DateTime
                    | VCardValueType::Time
                    | VCardValueType::UtcOffset
            ) {
                match (self.tz_hour, self.tz_minute) {
                    (Some(0), Some(0)) | (Some(0), _) => {
                        write!(out, "Z")?;
                    }
                    (Some(hour), Some(minute)) => {
                        if self.tz_minus {
                            write!(out, "-")?;
                        } else {
                            write!(out, "+")?;
                        }
                        write!(out, "{hour:02}:{minute:02}")?;
                    }
                    (Some(hour), None) => {
                        if self.tz_minus {
                            write!(out, "-")?;
                        } else {
                            write!(out, "+")?;
                        }
                        write!(out, "{hour:02}")?;
                    }
                    _ => {}
                }
            }

            Ok(())
        }
    }
}

impl Display for VCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_to(f, self.version().unwrap_or_default())
    }
}

impl Display for VCardVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VCardVersion::V2_0 => write!(f, "2.0"),
            VCardVersion::V2_1 => write!(f, "2.1"),
            VCardVersion::V3_0 => write!(f, "3.0"),
            VCardVersion::V4_0 => write!(f, "4.0"),
        }
    }
}
