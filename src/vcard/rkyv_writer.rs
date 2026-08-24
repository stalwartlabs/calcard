/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{
        parser::Timestamp,
        writer::{FoldingWriter, LineWriter, write_bytes, write_param_value, write_text},
    },
    vcard::{media_type::legacy_media_type, *},
};
use std::fmt::{Display, Write};

impl ArchivedVCard {
    pub fn write_to(&self, out: &mut impl Write, version: VCardVersion) -> std::fmt::Result {
        write!(out, "BEGIN:VCARD\r\n")?;
        write!(out, "VERSION:{version}\r\n")?;
        let is_v4 = matches!(version, VCardVersion::V4_0);
        for entry in self.entries.iter() {
            if !matches!(
                entry.name,
                ArchivedVCardProperty::Version
                    | ArchivedVCardProperty::Begin
                    | ArchivedVCardProperty::End
            ) {
                entry.write_to(out, true, is_v4)?;
            }
        }

        write!(out, "END:VCARD\r\n")
    }
}

impl ArchivedVCardEntry {
    pub fn write_to(
        &self,
        out: &mut impl Write,
        with_value: bool,
        is_v4: bool,
    ) -> std::fmt::Result {
        let mut folded = FoldingWriter::new(out);
        let out = &mut folded;

        if let Some(group_name) = self.group.as_ref() {
            out.write_atomic(group_name)?;
            out.write_atomic(".")?;
        }

        out.write_atomic(self.name.as_str())?;
        let mut types = None;
        let mut last_param: Option<&ArchivedVCardParameterName> = None;

        for param in self.params.iter() {
            if last_param.is_some_and(|last_param| last_param == &param.name) {
                out.write_atomic(",")?;
            } else {
                out.write_atomic(";")?;
                out.write_atomic(param.name.as_str())?;
                if !matches!(param.value, ArchivedVCardParameterValue::Null) {
                    out.write_atomic("=")?;
                }
                last_param = Some(&param.name);
            }

            match &param.value {
                ArchivedVCardParameterValue::Text(v) => {
                    write_param_value(out, v)?;
                }
                ArchivedVCardParameterValue::Integer(i) => {
                    write!(out, "{}", i)?;
                }
                ArchivedVCardParameterValue::Timestamp(v) => {
                    write!(out, "{}", Timestamp(v.to_native()))?;
                }
                ArchivedVCardParameterValue::Bool(v) => {
                    out.write_atomic(if *v { "TRUE" } else { "FALSE" })?;
                }
                ArchivedVCardParameterValue::ValueType(v) => {
                    if types.is_none() {
                        types = Some(v);
                    }
                    write_param_value(out, v.as_str())?;
                }
                ArchivedVCardParameterValue::Type(v) => {
                    write_param_value(out, v.as_str())?;
                }
                ArchivedVCardParameterValue::Calscale(v) => {
                    write_param_value(out, v.as_str())?;
                }
                ArchivedVCardParameterValue::Level(v) => {
                    write_param_value(out, v.as_str())?;
                }
                ArchivedVCardParameterValue::Phonetic(v) => {
                    write_param_value(out, v.as_str())?;
                }
                ArchivedVCardParameterValue::Jscomps(v) => {
                    out.write_atomic("\"")?;
                    write_jscomps(out, v)?;
                    out.write_atomic("\"")?;
                    last_param = None;
                }
                ArchivedVCardParameterValue::Null => {
                    last_param = None;
                }
            }
        }

        if !is_v4 {
            if let Some(data) = self.values.iter().find_map(|v| match v {
                ArchivedVCardValue::Binary(data) => Some(data),
                _ => None,
            }) {
                out.write_atomic(";ENCODING=b")?;

                if let Some(media_type) = data.content_type.as_deref()
                    && !self
                        .params
                        .iter()
                        .any(|param| param.name == VCardParameterName::Type)
                    && let Some(token) = legacy_media_type(self.name.as_str(), media_type)
                {
                    out.write_atomic(";TYPE=")?;
                    out.write_atomic(&token)?;
                }
            }

            if self.values.iter().any(|v| match v {
                ArchivedVCardValue::Text(s) => !s.is_ascii(),
                ArchivedVCardValue::Component(items) => items.iter().any(|s| !s.is_ascii()),
                _ => false,
            }) {
                out.write_atomic(";CHARSET=UTF-8")?;
            }
        }

        out.write_atomic(":")?;

        if with_value {
            let (default_type, value_separator) = self.name.default_types();
            let default_type = default_type.unwrap_vcard();

            let mut separator = ";";
            let mut escape_semicolon =
                matches!(types.unwrap_or(&default_type), ArchivedVCardValueType::Text);
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
                    out.write_atomic(separator)?;
                }

                match value {
                    ArchivedVCardValue::Text(v) => {
                        write_text(out, v, escape_semicolon, escape_comma)?;
                    }
                    ArchivedVCardValue::Component(v) => {
                        for (pos, item) in v.iter().enumerate() {
                            if pos > 0 {
                                out.write_atomic(",")?;
                            }
                            write_text(out, item, true, true)?;
                        }
                    }
                    ArchivedVCardValue::Integer(v) => {
                        write!(out, "{v}")?;
                    }
                    ArchivedVCardValue::Float(v) => {
                        write!(out, "{v}")?;
                    }
                    ArchivedVCardValue::Boolean(v) => {
                        out.write_atomic(if *v { "TRUE" } else { "FALSE" })?;
                    }
                    ArchivedVCardValue::PartialDateTime(v) => {
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
                    }
                    ArchivedVCardValue::Binary(v) => {
                        if is_v4 {
                            let media_type = v.content_type.as_deref().unwrap_or_default();
                            out.write_str("data:")?;
                            out.write_str(media_type)?;
                            out.write_str(";")?;
                            out.write_atomic("base64\\,")?;
                        }
                        write_bytes(out, &v.data)?;
                    }
                    ArchivedVCardValue::Sex(v) => {
                        out.write_atomic(v.as_str())?;
                    }
                    ArchivedVCardValue::GramGender(v) => {
                        out.write_atomic(v.as_str())?;
                    }
                    ArchivedVCardValue::Kind(v) => {
                        out.write_atomic(v.as_str())?;
                    }
                }
            }
        }
        out.end_line()
    }
}

impl crate::common::ArchivedPartialDateTime {
    pub fn format_as_vcard(
        &self,
        out: &mut impl Write,
        fmt: &ArchivedVCardValueType,
    ) -> std::fmt::Result {
        use ArchivedVCardValueType;
        use rkyv::option::ArchivedOption;

        if matches!(fmt, ArchivedVCardValueType::Timestamp) {
            write!(
                out,
                "{:04}{:02}{:02}T{:02}{:02}{:02}",
                self.year.as_ref().map(u16::from).unwrap_or_default(),
                self.month.as_ref().copied().unwrap_or_default(),
                self.day.as_ref().copied().unwrap_or_default(),
                self.hour.as_ref().copied().unwrap_or_default(),
                self.minute.as_ref().copied().unwrap_or_default(),
                self.second.as_ref().copied().unwrap_or_default()
            )?;

            if let Some(tz_hour) = self.tz_hour.as_ref().copied() {
                let tz_minute = self.tz_minute.as_ref().copied().unwrap_or_default();
                if tz_hour == 0 && tz_minute == 0 {
                    write!(out, "Z")?;
                } else {
                    write!(
                        out,
                        "{}{:02}",
                        if self.tz_minus { "-" } else { "+" },
                        tz_hour,
                    )?;

                    if let Some(tz_minute) = self.tz_minute.as_ref() {
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
                ArchivedVCardValueType::Date
                    | ArchivedVCardValueType::DateAndOrTime
                    | ArchivedVCardValueType::DateTime
            ) {
                match (self.year, self.month, self.day) {
                    (
                        ArchivedOption::Some(year),
                        ArchivedOption::Some(month),
                        ArchivedOption::Some(day),
                    ) => {
                        write!(out, "{:04}{:02}{:02}", year, month, day)?;
                    }
                    (
                        ArchivedOption::Some(year),
                        ArchivedOption::Some(month),
                        ArchivedOption::None,
                    ) => {
                        if missing_time && missing_tz {
                            write!(out, "{:04}-{:02}", year, month)?;
                        } else {
                            write!(out, "{:04}{:02}", year, month)?;
                        }
                    }
                    (
                        ArchivedOption::None,
                        ArchivedOption::Some(month),
                        ArchivedOption::Some(day),
                    ) => {
                        write!(out, "--{:02}{:02}", month, day)?;
                    }
                    (ArchivedOption::None, ArchivedOption::None, ArchivedOption::Some(day)) => {
                        write!(out, "---{:02}", day)?;
                    }
                    (ArchivedOption::Some(year), ArchivedOption::None, ArchivedOption::None) => {
                        write!(out, "{:04}", year)?;
                    }
                    (ArchivedOption::None, ArchivedOption::Some(month), ArchivedOption::None) => {
                        write!(out, "--{month}")?;
                    }
                    _ => {}
                }
            }

            if matches!(
                fmt,
                ArchivedVCardValueType::DateAndOrTime
                    | ArchivedVCardValueType::DateTime
                    | ArchivedVCardValueType::Time
            ) && !missing_time
            {
                if matches!(
                    fmt,
                    ArchivedVCardValueType::DateAndOrTime | ArchivedVCardValueType::DateTime
                ) {
                    write!(out, "T")?;
                }
                let mut last_is_some = false;
                for value in [&self.hour, &self.minute, &self.second].iter() {
                    if let ArchivedOption::Some(value) = value {
                        write!(out, "{:02}", value)?;
                        last_is_some = true;
                    } else if !last_is_some {
                        write!(out, "-")?;
                    }
                }
            }

            if matches!(
                fmt,
                ArchivedVCardValueType::DateAndOrTime
                    | ArchivedVCardValueType::DateTime
                    | ArchivedVCardValueType::Time
                    | ArchivedVCardValueType::UtcOffset
            ) {
                match (self.tz_hour.as_ref(), self.tz_minute.as_ref()) {
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
        fmt: &ArchivedVCardValueType,
    ) -> std::fmt::Result {
        if matches!(fmt, ArchivedVCardValueType::Timestamp) {
            write!(
                out,
                "{:04}{:02}{:02}",
                self.year.as_ref().map(u16::from).unwrap_or_default(),
                self.month.as_ref().copied().unwrap_or_default(),
                self.day.as_ref().copied().unwrap_or_default(),
            )?;

            if self.hour.is_some() {
                write!(
                    out,
                    "T{:02}{:02}{:02}",
                    self.hour.as_ref().copied().unwrap_or_default(),
                    self.minute.as_ref().copied().unwrap_or_default(),
                    self.second.as_ref().copied().unwrap_or_default()
                )?;
            }

            if let Some(tz_hour) = self.tz_hour.as_ref().copied() {
                let tz_minute = self.tz_minute.as_ref().copied().unwrap_or_default();
                if tz_hour == 0 && tz_minute == 0 {
                    write!(out, "Z")?;
                } else {
                    write!(
                        out,
                        "{}{:02}",
                        if self.tz_minus { "-" } else { "+" },
                        tz_hour,
                    )?;

                    if let Some(tz_minute) = self.tz_minute.as_ref().copied() {
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
                ArchivedVCardValueType::Date
                    | ArchivedVCardValueType::DateAndOrTime
                    | ArchivedVCardValueType::DateTime
            ) {
                match (self.year.as_ref(), self.month.as_ref(), self.day.as_ref()) {
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
                ArchivedVCardValueType::DateAndOrTime
                    | ArchivedVCardValueType::DateTime
                    | ArchivedVCardValueType::Time
            ) && !missing_time
            {
                if matches!(
                    fmt,
                    ArchivedVCardValueType::DateAndOrTime | ArchivedVCardValueType::DateTime
                ) {
                    write!(out, "T")?;
                }
                let mut last_is_some = false;
                for value in [&self.hour, &self.minute, &self.second].iter() {
                    if let Some(value) = value.as_ref() {
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
                ArchivedVCardValueType::DateAndOrTime
                    | ArchivedVCardValueType::DateTime
                    | ArchivedVCardValueType::Time
                    | ArchivedVCardValueType::UtcOffset
            ) {
                match (self.tz_hour.as_ref(), self.tz_minute.as_ref()) {
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

impl Display for ArchivedVCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_to(f, self.version().unwrap_or_default())
    }
}

pub(crate) fn write_jscomps(
    out: &mut impl LineWriter,
    values: &[ArchivedJscomp],
) -> std::fmt::Result {
    for (pos, item) in values.iter().enumerate() {
        if pos > 0 {
            out.write_atomic(";")?;
        }
        match item {
            ArchivedJscomp::Entry { position, value } => {
                write!(out, "{position}")?;
                if *value > 0 {
                    write!(out, ",{value}")?;
                }
            }
            ArchivedJscomp::Separator(s) => {
                if !s.is_empty() {
                    out.write_atomic("s,")?;

                    for ch in s.chars() {
                        match ch {
                            '\\' => out.write_atomic("\\\\")?,
                            ',' => out.write_atomic("\\,")?,
                            ':' => out.write_atomic("\\:")?,
                            '=' => out.write_atomic("\\=")?,
                            ';' => out.write_atomic("\\;")?,
                            '"' => out.write_atomic("\\\"")?,
                            '\r' | '\n' => {}
                            _ => out.write_char(ch)?,
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        Entry, Parser,
        vcard::{ArchivedVCard, VCardVersion},
    };

    #[test]
    fn archived_v3_emits_charset_for_non_ascii() {
        let input = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:José\r\nEND:VCARD\r\n".to_string();
        let mut parser = Parser::new(&input);
        let Entry::VCard(vcard) = parser.entry() else {
            panic!("expected vcard");
        };

        let mut owned = String::new();
        vcard.write_to(&mut owned, VCardVersion::V3_0).unwrap();

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&vcard).unwrap();
        let archived = rkyv::access::<ArchivedVCard, rkyv::rancor::Error>(&bytes).unwrap();
        let mut archived_out = String::new();
        archived
            .write_to(&mut archived_out, VCardVersion::V3_0)
            .unwrap();

        assert!(
            owned.contains(";CHARSET=UTF-8"),
            "owned missing charset: {owned}"
        );
        assert!(
            archived_out.contains(";CHARSET=UTF-8"),
            "archived missing charset: {archived_out}"
        );
        assert_eq!(owned, archived_out);
    }
}
