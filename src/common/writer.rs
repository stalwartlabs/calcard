/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::vcard::Jscomp;

use super::parser::Timestamp;
use mail_builder::encoders::base64::*;
use mail_parser::DateTime;
use std::fmt::{Display, Write};

pub(crate) fn write_value(
    out: &mut impl Write,
    line_len: &mut usize,
    value: &str,
) -> std::fmt::Result {
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

pub(crate) fn write_bytes(
    out: &mut impl Write,
    line_len: &mut usize,
    value: &[u8],
) -> std::fmt::Result {
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

pub(crate) fn write_param_value(
    out: &mut impl Write,
    line_len: &mut usize,
    value: &str,
) -> std::fmt::Result {
    let needs_quotes = value
        .as_bytes()
        .iter()
        .any(|&ch| matches!(ch, b',' | b':' | b'=' | b' ' | b';' | b'"'));

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
            0x22 => {
                write!(out, "\\\"")?;
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

pub(crate) fn write_param(
    out: &mut impl Write,
    line_len: &mut usize,
    name: &str,
    value: impl AsRef<str>,
) -> std::fmt::Result {
    write!(out, "{}=", name)?;
    *line_len += name.len() + 1;

    write_param_value(out, line_len, value.as_ref())
}

pub(crate) fn write_params(
    out: &mut impl Write,
    line_len: &mut usize,
    name: &str,
    values: &[impl AsRef<str>],
) -> std::fmt::Result {
    write!(out, "{}", name)?;
    *line_len += name.len();

    for (pos, v) in values.iter().enumerate() {
        if pos > 0 {
            write!(out, ",")?;
        } else {
            write!(out, "=")?;
        }
        *line_len += 1;
        write_param_value(out, line_len, v.as_ref())?;
    }

    Ok(())
}

pub(crate) fn write_jscomps(
    out: &mut impl Write,
    line_len: &mut usize,
    values: &[Jscomp],
) -> std::fmt::Result {
    for (pos, item) in values.iter().enumerate() {
        if pos > 0 {
            out.write_char(';')?;
            *line_len += 1;
        }
        match item {
            Jscomp::Entry { position, value } => {
                write!(out, "{position}")?;
                if *position > 9 {
                    *line_len += 2;
                } else {
                    *line_len += 1;
                }
                if *value > 0 {
                    write!(out, ",{value}")?;
                    out.write_char(',')?;
                    if *value > 9 {
                        *line_len += 3;
                    } else {
                        *line_len += 2;
                    }
                }
            }
            Jscomp::Separator(s) => {
                if !s.is_empty() {
                    out.write_str("s,")?;
                    *line_len += 2;

                    for ch in s.chars() {
                        if matches!(ch, ',' | ':' | '=' | ' ' | ';' | '"') {
                            out.write_char('\\')?;
                            *line_len += 1;
                        }
                        out.write_char(ch)?;
                        *line_len += 1;
                    }
                }
            }
        }
    }

    Ok(())
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dt = DateTime::from_timestamp(self.0);
        write!(
            f,
            "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
            dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second
        )
    }
}
