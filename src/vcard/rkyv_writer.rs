use std::fmt::{Display, Write};

use crate::{
    common::{
        parser::Timestamp,
        writer::{write_bytes, write_param, write_param_value, write_params, write_value},
    },
    vcard::*,
};

impl ArchivedVCard {
    pub fn write_to(&self, out: &mut impl Write) -> std::fmt::Result {
        write!(out, "BEGIN:VCARD\r\n")?;

        for entry in self.entries.iter() {
            if !matches!(
                entry.name,
                ArchivedVCardProperty::Begin | ArchivedVCardProperty::End
            ) {
                entry.write_to(out, true)?;
            }
        }

        write!(out, "END:VCARD\r\n")
    }
}

impl ArchivedVCardEntry {
    pub fn write_to(&self, out: &mut impl Write, with_value: bool) -> std::fmt::Result {
        let mut line_len = 0;

        if let Some(group_name) = self.group.as_ref() {
            write!(out, "{group_name}.")?;
            line_len += group_name.len() + 1;
        }

        let entry_name = self.name.as_str();
        write!(out, "{}", entry_name)?;
        line_len += entry_name.len();
        let mut types = &[][..];

        for param in self.params.iter() {
            write!(out, ";")?;
            line_len += 1;

            if line_len + 1 > 75 {
                write!(out, "\r\n ")?;
                line_len = 1;
            }

            match param {
                ArchivedVCardParameter::Language(v) => {
                    write_param(out, &mut line_len, "LANGUAGE", v)?;
                }
                ArchivedVCardParameter::Value(v) => {
                    write_params(out, &mut line_len, "VALUE", v)?;
                    types = v.as_slice();
                }
                ArchivedVCardParameter::Pref(v) => {
                    write!(out, "PREF={v}")?;
                    line_len += 7;
                }
                ArchivedVCardParameter::Altid(v) => {
                    write_param(out, &mut line_len, "ALTID", v)?;
                }
                ArchivedVCardParameter::Pid(v) => {
                    write_params(out, &mut line_len, "PID", v)?;
                }
                ArchivedVCardParameter::Type(v) => {
                    write_params(out, &mut line_len, "TYPE", v.as_slice())?;
                }
                ArchivedVCardParameter::Mediatype(v) => {
                    write_param(out, &mut line_len, "MEDIATYPE", v)?;
                }
                ArchivedVCardParameter::Calscale(v) => {
                    write_param(out, &mut line_len, "CALSCALE", v)?;
                }
                ArchivedVCardParameter::SortAs(v) => {
                    write_param(out, &mut line_len, "SORT-AS", v)?;
                }
                ArchivedVCardParameter::Geo(v) => {
                    write_param(out, &mut line_len, "GEO", v)?;
                }
                ArchivedVCardParameter::Tz(v) => {
                    write_param(out, &mut line_len, "TZ", v)?;
                }
                ArchivedVCardParameter::Index(v) => {
                    write!(out, "INDEX={v}")?;
                    line_len += 8;
                }
                ArchivedVCardParameter::Level(v) => {
                    write_param(out, &mut line_len, "LEVEL", v)?;
                }
                ArchivedVCardParameter::Group(v) => {
                    write_param(out, &mut line_len, "GROUP", v)?;
                }
                ArchivedVCardParameter::Cc(v) => {
                    write_param(out, &mut line_len, "CC", v)?;
                }
                ArchivedVCardParameter::Author(v) => {
                    write_param(out, &mut line_len, "AUTHOR", v)?;
                }
                ArchivedVCardParameter::AuthorName(v) => {
                    write_param(out, &mut line_len, "AUTHOR-NAME", v)?;
                }
                ArchivedVCardParameter::Created(v) => {
                    write!(out, "CREATED={}", Timestamp(v.to_native()))?;
                    line_len += 17;
                }
                ArchivedVCardParameter::Derived(v) => {
                    write_param(
                        out,
                        &mut line_len,
                        "DERIVED",
                        if *v { "TRUE" } else { "FALSE" },
                    )?;
                }
                ArchivedVCardParameter::Label(v) => {
                    write_param(out, &mut line_len, "LABEL", v)?;
                }
                ArchivedVCardParameter::Phonetic(v) => {
                    write_param(out, &mut line_len, "PHONETIC", v)?;
                }
                ArchivedVCardParameter::PropId(v) => {
                    write_param(out, &mut line_len, "PROP-ID", v)?;
                }
                ArchivedVCardParameter::Script(v) => {
                    write_param(out, &mut line_len, "SCRIPT", v)?;
                }
                ArchivedVCardParameter::ServiceType(v) => {
                    write_param(out, &mut line_len, "SERVICE-TYPE", v)?;
                }
                ArchivedVCardParameter::Username(v) => {
                    write_param(out, &mut line_len, "USERNAME", v)?;
                }
                ArchivedVCardParameter::Jsptr(v) => {
                    write_param(out, &mut line_len, "JSPTR", v)?;
                }
                ArchivedVCardParameter::Other(v) => {
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

        if with_value {
            let (default_type, separator) = self.name.default_types();
            let separator = if !matches!(separator, ValueSeparator::Comma) {
                ";"
            } else {
                ","
            };
            let default_type = default_type.unwrap_vcard();

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
                    ArchivedVCardValue::Text(v) => {
                        write_value(out, &mut line_len, v)?;
                    }
                    ArchivedVCardValue::Integer(v) => {
                        write!(out, "{v}")?;
                        line_len += 4;
                    }
                    ArchivedVCardValue::Float(v) => {
                        write!(out, "{v}")?;
                        line_len += 4;
                    }
                    ArchivedVCardValue::Boolean(v) => {
                        let text = if *v { "TRUE" } else { "FALSE" };
                        write!(out, "{text}")?;
                        line_len += text.len();
                    }
                    ArchivedVCardValue::PartialDateTime(v) => {
                        v.format_as_vcard(out, types.get(pos).unwrap_or(&default_type))?;
                        line_len += 16;
                    }
                    ArchivedVCardValue::Binary(v) => {
                        write!(out, "data:")?;
                        line_len += 5;
                        if let Some(ct) = v.content_type.as_ref() {
                            write!(out, "{ct};")?;
                            line_len += ct.len() + 1;
                        }
                        write!(out, "base64\\,")?;
                        line_len += 8;
                        write_bytes(out, &mut line_len, &v.data)?;
                    }
                    ArchivedVCardValue::Sex(v) => {
                        let text = v.as_str();
                        write!(out, "{text}")?;
                        line_len += text.len();
                    }
                    ArchivedVCardValue::GramGender(v) => {
                        let text = v.as_str();
                        write!(out, "{text}")?;
                        line_len += text.len();
                    }
                    ArchivedVCardValue::Kind(v) => {
                        let text = v.as_str();
                        write!(out, "{text}")?;
                        line_len += text.len();
                    }
                }
            }
        }
        write!(out, "\r\n")
    }
}

impl crate::common::ArchivedPartialDateTime {
    pub fn format_as_vcard(
        &self,
        out: &mut impl Write,
        fmt: &ArchivedVCardValueType,
    ) -> std::fmt::Result {
        use rkyv::option::ArchivedOption;
        use ArchivedVCardValueType;

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
}

impl Display for ArchivedVCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_to(f)
    }
}
