/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::parser::Timestamp;
use crate::vcard::Jscomp;
use mail_builder::encoders::base64::*;
use mail_parser::DateTime;
use std::fmt::{Display, Write};

pub(crate) const MAX_LINE_LEN: usize = 75;

pub(crate) trait LineWriter: Write {
    fn write_atomic(&mut self, text: &str) -> std::fmt::Result;
}

pub(crate) struct FoldingWriter<'x, W: Write> {
    out: &'x mut W,
    line_len: usize,
}

impl<'x, W: Write> FoldingWriter<'x, W> {
    pub(crate) fn new(out: &'x mut W) -> Self {
        Self { out, line_len: 0 }
    }

    pub(crate) fn end_line(&mut self) -> std::fmt::Result {
        self.line_len = 0;
        self.out.write_str("\r\n")
    }
}

impl<W: Write> LineWriter for FoldingWriter<'_, W> {
    fn write_atomic(&mut self, text: &str) -> std::fmt::Result {
        let len = text.len();
        if len > MAX_LINE_LEN - 1 {
            return self.write_str(text);
        }
        if self.line_len + len > MAX_LINE_LEN {
            self.out.write_str("\r\n ")?;
            self.line_len = 1;
        }
        self.out.write_str(text)?;
        self.line_len += len;
        Ok(())
    }
}

impl LineWriter for String {
    fn write_atomic(&mut self, text: &str) -> std::fmt::Result {
        self.push_str(text);
        Ok(())
    }
}

impl<W: Write> Write for FoldingWriter<'_, W> {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        let mut rest = text;

        loop {
            let available = MAX_LINE_LEN.saturating_sub(self.line_len);

            if rest.len() <= available {
                self.line_len += rest.len();
                return self.out.write_str(rest);
            }

            let mut split = available;
            while split > 0 && !rest.is_char_boundary(split) {
                split -= 1;
            }

            if split == 0 {
                self.out.write_str("\r\n ")?;
                self.line_len = 1;
                continue;
            }

            self.out.write_str(&rest[..split])?;
            self.line_len += split;
            rest = &rest[split..];
        }
    }

    fn write_char(&mut self, ch: char) -> std::fmt::Result {
        let ch_len = ch.len_utf8();
        if self.line_len + ch_len > MAX_LINE_LEN {
            self.out.write_str("\r\n ")?;
            self.line_len = 1;
        }
        self.line_len += ch_len;
        self.out.write_char(ch)
    }
}

pub(crate) fn write_text(
    out: &mut impl LineWriter,
    value: &str,
    escape_semicolon: bool,
    escape_comma: bool,
) -> std::fmt::Result {
    for ch in value.chars() {
        match ch {
            '\r' => out.write_atomic("\\r")?,
            '\n' => out.write_atomic("\\n")?,
            '\\' => out.write_atomic("\\\\")?,
            ';' if escape_semicolon => out.write_atomic("\\;")?,
            ',' if escape_comma => out.write_atomic("\\,")?,
            _ => out.write_char(ch)?,
        }
    }

    Ok(())
}

pub(crate) fn write_bytes(out: &mut impl Write, value: &[u8]) -> std::fmt::Result {
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
                out.write_char(char::from(ch))?;
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
            out.write_char(char::from(*ch))?;
        }
    }

    Ok(())
}

pub(crate) trait NeedsQuotes {
    fn needs_quotes(&self) -> bool;
}

impl<T: AsRef<[u8]>> NeedsQuotes for T {
    fn needs_quotes(&self) -> bool {
        self.as_ref()
            .iter()
            .any(|&ch| matches!(ch, b',' | b':' | b'=' | b' ' | b';' | b'"'))
    }
}

pub(crate) fn write_param_value(out: &mut impl LineWriter, value: &str) -> std::fmt::Result {
    let needs_quotes = value.needs_quotes();

    if needs_quotes {
        out.write_atomic("\"")?;
    }

    for ch in value.chars() {
        match ch as u32 {
            0x0A => out.write_atomic("\\n")?,
            0x0D => out.write_atomic("\\r")?,
            0x5C => out.write_atomic("\\\\")?,
            0x22 => out.write_atomic("\\\"")?,
            0x20 | 0x09 | 0x21 | 0x23..=0x7E | 0x80.. => out.write_char(ch)?,
            _ => {}
        }
    }

    if needs_quotes {
        out.write_atomic("\"")?;
    }

    Ok(())
}

pub(crate) fn write_jscomps(out: &mut impl LineWriter, values: &[Jscomp]) -> std::fmt::Result {
    for (pos, item) in values.iter().enumerate() {
        if pos > 0 {
            out.write_atomic(";")?;
        }
        match item {
            Jscomp::Entry { position, value } => {
                write!(out, "{position}")?;
                if *value > 0 {
                    write!(out, ",{value}")?;
                }
            }
            Jscomp::Separator(s) => {
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
pub(crate) fn assert_fold_width(text: &str, context: &str) {
    let mut lines = text.split("\r\n").peekable();

    while let Some(line) = lines.next() {
        assert!(
            line.len() <= MAX_LINE_LEN,
            "physical line of {} octets exceeds the {MAX_LINE_LEN} octet fold width in {context}: {line:?}",
            line.len()
        );
        assert!(
            !line.is_empty() || lines.peek().is_none(),
            "empty physical line in {context}: {text:?}"
        );
        assert!(
            line != " ",
            "continuation line holding nothing but the fold character in {context}: {text:?}"
        );
    }
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
