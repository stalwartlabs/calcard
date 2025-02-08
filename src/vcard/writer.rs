use std::fmt::{Display, Write};

use mail_builder::encoders::base64::*;
use mail_parser::DateTime;

use crate::vcard::{VCardParameter, VCardProperty, VCardValue};

use super::{VCard, VCardPartialDateTime};

impl VCard {
    pub fn write_to(&self, out: &mut impl Write) -> std::fmt::Result {
        write!(out, "BEGIN:VCARD\r\n")?;
        write!(out, "VERSION:4.0\r\n")?;

        for entry in &self.entries {
            if matches!(
                entry.name,
                VCardProperty::Begin | VCardProperty::End | VCardProperty::Version
            ) {
                continue;
            }

            let mut line_len = 0;

            if let Some(group_name) = &entry.group {
                write!(out, "{group_name}.")?;
                line_len += group_name.len() + 1;
            }

            let entry_name = entry.name.as_str();
            write!(out, "{}", entry_name)?;
            line_len += entry_name.len();

            for param in &entry.params {
                write!(out, ";")?;
                line_len += 1;

                match param {
                    VCardParameter::Language(v) => {
                        write!(out, "LANGUAGE=")?;
                        line_len += 9;
                        write_param_value(out, &mut line_len, v)?;
                    }
                    VCardParameter::Value(v) => {
                        if !v.is_empty() {
                            write!(out, "VALUE=")?;
                            line_len += 6;
                            for (pos, v) in v.iter().enumerate() {
                                if pos > 0 {
                                    write!(out, ",")?;
                                    line_len += 1;
                                }
                                write_param_value(out, &mut line_len, v.as_str())?;
                            }
                        }
                    }
                    VCardParameter::Pref(v) => {
                        write!(out, "PREF={v}")?;
                        line_len += 7;
                    }
                    VCardParameter::Altid(v) => {
                        write!(out, "ALTID=")?;
                        line_len += 6;
                        write_param_value(out, &mut line_len, v)?;
                    }
                    VCardParameter::Pid(v) => {
                        if !v.is_empty() {
                            write!(out, "PID=")?;
                            line_len += 4;
                            for (pos, v) in v.iter().enumerate() {
                                if pos > 0 {
                                    write!(out, ",")?;
                                    line_len += 1;
                                }
                                write_param_value(out, &mut line_len, v)?;
                            }
                        }
                    }
                    VCardParameter::Type(v) => {
                        if !v.is_empty() {
                            write!(out, "TYPE=")?;
                            line_len += 5;
                            for (pos, v) in v.iter().enumerate() {
                                if pos > 0 {
                                    write!(out, ",")?;
                                    line_len += 1;
                                }
                                write_param_value(out, &mut line_len, v.as_str())?;
                            }
                        }
                    }
                    VCardParameter::Mediatype(v) => {
                        write!(out, "MEDIATYPE=")?;
                        line_len += 10;
                        write_param_value(out, &mut line_len, v)?;
                    }
                    VCardParameter::Calscale(v) => {
                        let text = v.as_str();
                        write!(out, "CALSCALE={text}")?;
                        line_len += 9 + text.len();
                    }
                    VCardParameter::SortAs(v) => {
                        write!(out, "SORT-AS=")?;
                        line_len += 8;
                        write_param_value(out, &mut line_len, v)?;
                    }
                    VCardParameter::Geo(v) => {
                        write!(out, "GEO=")?;
                        line_len += 4;
                        write_param_value(out, &mut line_len, v)?;
                    }
                    VCardParameter::Tz(v) => {
                        write!(out, "TZ=")?;
                        line_len += 3;
                        write_param_value(out, &mut line_len, v)?;
                    }
                    VCardParameter::Index(v) => {
                        write!(out, "INDEX={v}")?;
                        line_len += 8;
                    }
                    VCardParameter::Level(v) => {
                        let text = v.as_str();
                        write!(out, "LEVEL={text}")?;
                        line_len += 6 + text.len();
                    }
                    VCardParameter::Group(v) => {
                        write!(out, "GROUP=")?;
                        line_len += 6;
                        write_param_value(out, &mut line_len, v)?;
                    }
                    VCardParameter::Cc(v) => {
                        write!(out, "CC=")?;
                        line_len += 3;
                        write_param_value(out, &mut line_len, v)?;
                    }
                    VCardParameter::Author(v) => {
                        write!(out, "AUTHOR=")?;
                        line_len += 7;
                        write_param_value(out, &mut line_len, v)?;
                    }
                    VCardParameter::AuthorName(v) => {
                        write!(out, "AUTHOR-NAME=")?;
                        line_len += 12;
                        write_param_value(out, &mut line_len, v)?;
                    }
                    VCardParameter::Created(v) => {
                        let dt = DateTime::from_timestamp(*v);
                        write!(
                            out,
                            "CREATED={:04}{:02}{:02}T{:02}{:02}{:02}Z",
                            dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second
                        )?;
                        line_len += 17;
                    }
                    VCardParameter::Derived(v) => {
                        write!(out, "DERIVED=")?;
                        line_len += 8;
                        if *v {
                            write!(out, "TRUE")?;
                            line_len += 4;
                        } else {
                            write!(out, "FALSE")?;
                            line_len += 5;
                        }
                    }
                    VCardParameter::Label(v) => {
                        write!(out, "LABEL=")?;
                        line_len += 6;
                        write_param_value(out, &mut line_len, v)?;
                    }
                    VCardParameter::Phonetic(v) => {
                        let text = v.as_str();
                        write!(out, "PHONETIC={text}")?;
                        line_len += 9 + text.len();
                    }
                    VCardParameter::PropId(v) => {
                        write!(out, "PROP-ID=")?;
                        line_len += 8;
                        write_param_value(out, &mut line_len, v)?;
                    }
                    VCardParameter::Script(v) => {
                        write!(out, "SCRIPT=")?;
                        line_len += 7;
                        write_param_value(out, &mut line_len, v)?;
                    }
                    VCardParameter::ServiceType(v) => {
                        write!(out, "SERVICE-TYPE=")?;
                        line_len += 12;
                        write_param_value(out, &mut line_len, v)?;
                    }
                    VCardParameter::Username(v) => {
                        write!(out, "USERNAME=")?;
                        line_len += 9;
                        write_param_value(out, &mut line_len, v)?;
                    }
                    VCardParameter::Jsptr(v) => {
                        write!(out, "JSPTR=")?;
                        line_len += 6;
                        write_param_value(out, &mut line_len, v)?;
                    }
                    VCardParameter::Other(v) => {
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

            let separator = if !matches!(
                entry.name,
                VCardProperty::Categories | VCardProperty::Nickname
            ) {
                ";"
            } else {
                ","
            };

            for (pos, value) in entry.values.iter().enumerate() {
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
                        write_value(out, &mut line_len, v)?;
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
                        write!(out, "{v}")?;
                        line_len += 16;
                    }
                    VCardValue::Binary(v) => {
                        write!(out, "data:")?;
                        line_len += 5;
                        if let Some(ct) = &v.content_type {
                            write!(out, "{ct};")?;
                            line_len += ct.len() + 1;
                        }
                        write!(out, "base64\\,")?;
                        line_len += 8;
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
            write!(out, "\r\n")?;
        }

        write!(out, "END:VCARD\r\n")
    }
}

fn write_value(out: &mut impl Write, line_len: &mut usize, value: &str) -> std::fmt::Result {
    for ch in value.chars() {
        let ch_len = ch.len_utf8();
        if *line_len + ch_len > 75 {
            write!(out, "\r\n ")?;
            *line_len = 1;
        }

        match ch {
            '\r' => {
                write!(out, "\\r")?;
                *line_len += 2;
                continue;
            }
            '\n' => {
                write!(out, "\\n")?;
                *line_len += 2;
                continue;
            }
            '\\' | ',' | ';' => {
                write!(out, "\\")?;
                *line_len += 2;
            }
            _ => {
                *line_len += ch.len_utf8();
            }
        }

        write!(out, "{ch}")?;
    }

    Ok(())
}

fn write_bytes(out: &mut impl Write, line_len: &mut usize, value: &[u8]) -> std::fmt::Result {
    const CHARPAD: u8 = b'=';

    let mut i = 0;
    let mut t1;
    let mut t2;
    let mut t3;

    if value.len() > 2 {
        while i < value.len() - 2 {
            t1 = value[i];
            t2 = value[i + 1];
            t3 = value[i + 2];

            for ch in [
                E0[t1 as usize],
                E1[(((t1 & 0x03) << 4) | ((t2 >> 4) & 0x0F)) as usize],
                E1[(((t2 & 0x0F) << 2) | ((t3 >> 6) & 0x03)) as usize],
                E2[t3 as usize],
            ] {
                if *line_len + 1 > 75 {
                    write!(out, "\r\n ")?;
                    *line_len = 1;
                }

                write!(out, "{}", char::from(ch))?;
                *line_len += 1;
            }

            i += 3;
        }
    }

    let remaining = value.len() - i;
    if remaining > 0 {
        t1 = value[i];
        let chs = if remaining == 1 {
            [
                E0[t1 as usize],
                E1[((t1 & 0x03) << 4) as usize],
                CHARPAD,
                CHARPAD,
            ]
        } else {
            t2 = value[i + 1];
            [
                E0[t1 as usize],
                E1[(((t1 & 0x03) << 4) | ((t2 >> 4) & 0x0F)) as usize],
                E2[((t2 & 0x0F) << 2) as usize],
                CHARPAD,
            ]
        };

        for ch in chs.iter() {
            if *line_len + 1 > 75 {
                write!(out, "\r\n ")?;
                *line_len = 1;
            }

            write!(out, "{}", char::from(*ch))?;
            *line_len += 1;
        }
    }

    Ok(())
}

fn write_param_value(out: &mut impl Write, line_len: &mut usize, value: &str) -> std::fmt::Result {
    let needs_quotes = value.as_bytes().iter().any(|&ch| matches!(ch, b',' | b';'));

    if needs_quotes {
        write!(out, "\"")?;
        *line_len += 1;
    }

    for ch in value.chars() {
        match ch as u32 {
            0x0A => {
                write!(out, "\\n")?;
                *line_len += 2;
            }
            0x0D => {
                write!(out, "\\r")?;
                *line_len += 2;
            }
            0x5C => {
                write!(out, "\\\\")?;
                *line_len += 2;
            }
            0x20 | 0x09 | 0x21 | 0x23..=0x7E | 0x80.. => {
                let ch_len = ch.len_utf8();
                if *line_len + ch_len > 75 {
                    write!(out, "\r\n ")?;
                    *line_len = 1;
                }
                write!(out, "{ch}")?;
                *line_len += ch_len;
            }
            _ => {}
        }
    }

    if needs_quotes {
        write!(out, "\"")?;
        *line_len += 1;
    }

    Ok(())
}

impl Display for VCardPartialDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let missing_time = self.hour.is_none() && self.minute.is_none() && self.second.is_none();
        let missing_tz = self.tz_hour.is_none();
        match (self.year, self.month, self.day) {
            (Some(year), Some(month), Some(day)) => {
                write!(f, "{:04}{:02}{:02}", year, month, day)?;
            }
            (Some(year), Some(month), None) => {
                if missing_time && missing_tz {
                    write!(f, "{:04}-{:02}", year, month)?;
                } else {
                    write!(f, "{:04}{:02}", year, month)?;
                }
            }
            (None, Some(month), Some(day)) => {
                write!(f, "--{:02}{:02}", month, day)?;
            }
            (None, None, Some(day)) => {
                write!(f, "---{:02}", day)?;
            }
            (Some(year), None, None) => {
                write!(f, "{:04}", year)?;
            }
            (None, Some(month), None) => {
                write!(f, "--{month}")?;
            }
            _ => {}
        }

        if !missing_time {
            if self.year.is_some() || self.month.is_some() || self.day.is_some() {
                write!(f, "T")?;
            }
            let mut last_is_some = false;
            for value in [&self.hour, &self.minute, &self.second].iter() {
                if let Some(value) = value {
                    write!(f, "{:02}", value)?;
                    last_is_some = true;
                } else if !last_is_some {
                    write!(f, "-")?;
                }
            }
        }

        if !missing_tz {
            match (self.tz_hour, self.tz_minute) {
                (Some(0), Some(0)) | (Some(0), _) => {
                    write!(f, "Z")?;
                }
                (Some(hour), Some(minute)) => {
                    if self.tz_minus {
                        write!(f, "-")?;
                    } else {
                        write!(f, "+")?;
                    }
                    write!(f, "{hour:02}{minute:02}")?;
                }
                (Some(hour), None) => {
                    if self.tz_minus {
                        write!(f, "-")?;
                    } else {
                        write!(f, "+")?;
                    }
                    write!(f, "{hour:02}")?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

impl Display for VCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_to(f)
    }
}
