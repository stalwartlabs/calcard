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
        writer::{
            FoldingWriter, LineWriter, write_bytes, write_jscomps, write_param_value, write_text,
        },
    },
    vcard::{
        VCardParameterName, VCardParameterValue, VCardProperty, VCardValue, ValueSeparator,
        media_type::legacy_media_type,
    },
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
        let mut folded = FoldingWriter::new(out);
        let out = &mut folded;

        if let Some(group_name) = &self.group {
            out.write_atomic(group_name)?;
            out.write_atomic(".")?;
        }

        out.write_atomic(self.name.as_str())?;
        let mut types = None;
        let mut last_param: Option<&VCardParameterName> = None;

        for param in &self.params {
            if last_param.is_some_and(|last_param| last_param == &param.name) {
                out.write_atomic(",")?;
            } else {
                out.write_atomic(";")?;
                out.write_atomic(param.name.as_str())?;
                if !matches!(param.value, VCardParameterValue::Null) {
                    out.write_atomic("=")?;
                }
                last_param = Some(&param.name);
            }

            match &param.value {
                VCardParameterValue::Text(v) => {
                    write_param_value(out, v)?;
                }
                VCardParameterValue::Integer(i) => {
                    write!(out, "{i}")?;
                }
                VCardParameterValue::Timestamp(v) => {
                    write!(out, "{}", Timestamp(*v))?;
                }
                VCardParameterValue::Bool(v) => {
                    out.write_atomic(if *v { "TRUE" } else { "FALSE" })?;
                }
                VCardParameterValue::ValueType(v) => {
                    if types.is_none() {
                        types = Some(v);
                    }
                    write_param_value(out, v.as_str())?;
                }
                VCardParameterValue::Type(v) => {
                    write_param_value(out, v.as_str())?;
                }
                VCardParameterValue::Calscale(v) => {
                    write_param_value(out, v.as_str())?;
                }
                VCardParameterValue::Level(v) => {
                    write_param_value(out, v.as_str())?;
                }
                VCardParameterValue::Phonetic(v) => {
                    write_param_value(out, v.as_str())?;
                }
                VCardParameterValue::Jscomps(v) => {
                    out.write_atomic("\"")?;
                    write_jscomps(out, v)?;
                    out.write_atomic("\"")?;
                    last_param = None;
                }
                VCardParameterValue::Null => {
                    last_param = None;
                }
            }
        }

        if !is_v4 {
            if let Some(data) = self.values.iter().find_map(|v| match v {
                VCardValue::Binary(data) => Some(data),
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
                VCardValue::Text(s) => !s.is_ascii(),
                VCardValue::Component(items) => items.iter().any(|s| !s.is_ascii()),
                _ => false,
            }) {
                out.write_atomic(";CHARSET=UTF-8")?;
            }
        }

        out.write_atomic(":")?;

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
                out.write_atomic(separator)?;
            }

            match value {
                VCardValue::Text(v) => {
                    write_text(out, v, escape_semicolon, escape_comma)?;
                }
                VCardValue::Component(v) => {
                    for (pos, item) in v.iter().enumerate() {
                        if pos > 0 {
                            out.write_atomic(",")?;
                        }
                        write_text(out, item, true, true)?;
                    }
                }
                VCardValue::Integer(v) => {
                    write!(out, "{v}")?;
                }
                VCardValue::Float(v) => {
                    write!(out, "{v}")?;
                }
                VCardValue::Boolean(v) => {
                    out.write_atomic(if *v { "TRUE" } else { "FALSE" })?;
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
                }
                VCardValue::Binary(v) => {
                    if is_v4 {
                        let media_type = v.content_type.as_deref().unwrap_or_default();
                        out.write_str("data:")?;
                        out.write_str(media_type)?;
                        out.write_str(";")?;
                        out.write_atomic("base64\\,")?;
                    }
                    write_bytes(out, &v.data)?;
                }
                VCardValue::Sex(v) => {
                    out.write_atomic(v.as_str())?;
                }
                VCardValue::GramGender(v) => {
                    out.write_atomic(v.as_str())?;
                }
                VCardValue::Kind(v) => {
                    out.write_atomic(v.as_str())?;
                }
            }
        }

        out.end_line()
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
                "{:04}{:02}{:02}",
                self.year.unwrap_or_default(),
                self.month.unwrap_or_default(),
                self.day.unwrap_or_default(),
            )?;

            if self.hour.is_some() {
                write!(
                    out,
                    "T{:02}{:02}{:02}",
                    self.hour.unwrap_or_default(),
                    self.minute.unwrap_or_default(),
                    self.second.unwrap_or_default()
                )?;
            }

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

#[cfg(test)]
mod tests {
    use crate::{
        Entry, Parser,
        vcard::{VCard, VCardVersion},
    };

    fn parse(input: &str) -> VCard {
        let mut parser = Parser::new(input);
        let Entry::VCard(vcard) = parser.entry() else {
            panic!("expected vcard for {input}");
        };
        vcard
    }

    fn write(vcard: &VCard, version: VCardVersion) -> String {
        let mut out = String::new();
        vcard.write_to(&mut out, version).unwrap();

        #[cfg(feature = "rkyv")]
        {
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(vcard).unwrap();
            let archived =
                rkyv::access::<crate::vcard::ArchivedVCard, rkyv::rancor::Error>(&bytes).unwrap();
            let mut archived_out = String::new();
            archived.write_to(&mut archived_out, version).unwrap();
            assert_eq!(out, archived_out, "archived writer diverged at {version}");
        }

        out
    }

    #[test]
    fn test_write_binary_data_uri() {
        let photo = "iVBORw0KGgoAAAANSUhEUgAAAAsAAAALCAQAAAADpb+tAAAAQklEQVQI122PQQ4AMAj\
                     CKv//Mzs4M0zmRYKkamEwWQVoRJogk4PuRoOoMC/EK8nYb+l08WGvSxKlNHO5kxnp/\
                     WXrAzsSERN1N6q5AAAAAElFTkSuQmCC";

        for (input, version, expect) in [
            (
                format!("PHOTO;TYPE=JPEG;ENCODING=b:{photo}"),
                VCardVersion::V4_0,
                format!("PHOTO;TYPE=JPEG:data:image/jpeg;base64\\,{photo}"),
            ),
            (
                format!("PHOTO;TYPE=JPEG;ENCODING=b:{photo}"),
                VCardVersion::V3_0,
                format!("PHOTO;TYPE=JPEG;ENCODING=b:{photo}"),
            ),
            (
                format!("PHOTO;ENCODING=b:{photo}"),
                VCardVersion::V4_0,
                format!("PHOTO:data:;base64\\,{photo}"),
            ),
            (
                format!("SOUND;TYPE=WAVE;ENCODING=b:{photo}"),
                VCardVersion::V4_0,
                format!("SOUND;TYPE=WAVE:data:audio/wav;base64\\,{photo}"),
            ),
            (
                format!("PHOTO;TYPE=WORK;ENCODING=b:{photo}"),
                VCardVersion::V4_0,
                format!("PHOTO;TYPE=WORK:data:;base64\\,{photo}"),
            ),
        ] {
            let vcard = parse(&format!(
                "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:T\r\n{input}\r\nEND:VCARD\r\n"
            ));
            let out = write(&vcard, version).replace("\r\n ", "");

            assert!(
                out.contains(expect.as_str()),
                "expected {expect} in {out} for {input}"
            );
        }
    }

    #[test]
    fn test_write_binary_data_uri_v4_to_v3() {
        let vcard = parse(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:T\r\n\
             PHOTO:data:image/png;base64\\,iVBORw0KGgo=\r\nEND:VCARD\r\n",
        );

        assert!(
            write(&vcard, VCardVersion::V3_0).contains("PHOTO;ENCODING=b;TYPE=PNG:iVBORw0KGgo="),
            "{}",
            write(&vcard, VCardVersion::V3_0)
        );
    }

    #[test]
    fn test_write_fold_width() {
        let filler = "A".repeat(73);
        let long_param = "B".repeat(64);
        let component = "C".repeat(71);

        for input in [
            format!("N:{filler};B;C;D;E"),
            format!("N:{filler};;;;"),
            format!("ADR:{component};;;;;;"),
            format!("N:{component},,,,,;;;;"),
            format!("NOTE;X-FOO={long_param}:hello"),
            format!("NOTE:{}", "D".repeat(500)),
        ] {
            let vcard = parse(&format!(
                "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:T\r\n{input}\r\nEND:VCARD\r\n"
            ));

            for version in [VCardVersion::V3_0, VCardVersion::V4_0] {
                let out = write(&vcard, version);
                crate::common::writer::assert_fold_width(&out, &input);

                let reparsed = parse(&out);
                assert_eq!(
                    write(&reparsed, version),
                    out,
                    "not idempotent for {input} at version {version}"
                );
            }
        }
    }

    #[test]
    fn test_write_jscomps() {
        let filler = "-".repeat(55);
        let folded_separator = format!("{}\r\n {}", "-".repeat(60), "=".repeat(10));

        for input in [
            format!("N;JSCOMPS=\"s,{filler};1;2;3;0\":a;b;c;d;e"),
            format!("N;JSCOMPS=\"s,{folded_separator};1;2\":a;b;c;d;e"),
            format!("N;JSCOMPS=\"s,{};12;34,5\":a;b;c;d;e", "-".repeat(54)),
            "N;JSCOMPS=\"s,==========;1;2\":a;b;c;d;e".to_string(),
            "N;JSCOMPS=\"s,\\\\;1;2\":a;b;c;d;e".to_string(),
        ] {
            let vcard = parse(&format!(
                "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:T\r\n{input}\r\nEND:VCARD\r\n"
            ));
            let out = write(&vcard, VCardVersion::V4_0);
            crate::common::writer::assert_fold_width(&out, &input);

            assert_eq!(
                parse(&out).entries,
                vcard.entries,
                "JSCOMPS did not survive a round trip for {input}\n{out}"
            );
        }
    }
}
