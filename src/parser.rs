use std::{
    iter::{Enumerate, Peekable},
    slice::Iter,
    str::FromStr,
};

use mail_parser::{
    decoders::{base64::base64_decode, hex::decode_hex},
    DateTime,
};

use crate::{
    tokenizer::{StopChar, Token},
    vcard::{VCard, VCardBinary, VCardPartialDateTime},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    VCard(VCard),
    InvalidLine(String),
    Eof,
}

pub struct Parser<'x> {
    pub(crate) input: &'x [u8],
    pub(crate) iter: Peekable<Enumerate<Iter<'x, u8>>>,
    pub(crate) strict: bool,
    pub(crate) stop_colon: bool,
    pub(crate) stop_semicolon: bool,
    pub(crate) stop_comma: bool,
    pub(crate) stop_equal: bool,
    pub(crate) stop_slash: bool,
    pub(crate) stop_dot: bool,
    pub(crate) unfold_qp: bool,
    pub(crate) unquote: bool,
    pub(crate) token_buf: Vec<Token<'x>>,
}

impl<'x> Parser<'x> {
    pub fn new(input: &'x [u8]) -> Self {
        Self {
            input,
            iter: input.iter().enumerate().peekable(),
            strict: false,
            stop_colon: true,
            stop_semicolon: true,
            stop_comma: true,
            stop_equal: true,
            stop_slash: false,
            stop_dot: false,
            unfold_qp: false,
            unquote: true,
            token_buf: Vec::with_capacity(10),
        }
    }

    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    pub fn entry(&mut self) -> Entry {
        self.expect_iana_token();

        loop {
            if let Some(token) = self.token() {
                if token.text.eq_ignore_ascii_case(b"BEGIN") && token.stop_char == StopChar::Colon {
                    if let Some(token) = self.token() {
                        if token.stop_char == StopChar::Lf {
                            hashify::fnc_map_ignore_case!(token.text.as_ref(),
                                b"VCARD" => {
                                    return Entry::VCard(self.vcard());
                                },
                                b"VCALENDAR" => {
                                    //self.vcalendar()
                                    todo!()
                                }
                                _ => {
                                    return Entry::InvalidLine(token.into_string());
                                }
                            )
                        }
                    } else {
                        return Entry::Eof;
                    }
                }

                let token_start = token.start;
                let mut token_end = token.end;

                if token.stop_char != StopChar::Lf {
                    self.expect_single_value();
                    while let Some(token) = self.token() {
                        token_end = token.end;
                        if token.stop_char == StopChar::Lf {
                            break;
                        }
                    }
                } else if token.text.is_empty() {
                    continue;
                }

                return Entry::InvalidLine(
                    std::str::from_utf8(self.input.get(token_start..token_end).unwrap_or_default())
                        .unwrap_or_default()
                        .to_string(),
                );
            } else {
                return Entry::Eof;
            }
        }
    }
}

impl Token<'_> {
    pub(crate) fn into_uri_bytes(self) -> std::result::Result<VCardBinary, String> {
        if self
            .text
            .as_ref()
            .get(0..5)
            .unwrap_or_default()
            .eq_ignore_ascii_case(b"data:")
        {
            let mut bin = VCardBinary::default();
            let text = self.text.as_ref().get(5..).unwrap_or_default();
            let mut offset_start = 0;
            let mut is_base64 = false;

            for (idx, ch) in text.iter().enumerate() {
                match ch {
                    b';' => {
                        if idx > 0 {
                            bin.content_type = Some(
                                std::str::from_utf8(&text[offset_start..idx])
                                    .unwrap_or_default()
                                    .to_string(),
                            );
                        }
                        offset_start = idx + 1;
                    }
                    b',' => {
                        if idx != offset_start {
                            let text = text.get(offset_start..idx).unwrap_or_default();
                            if text.eq_ignore_ascii_case(b"base64") {
                                is_base64 = true;
                            } else if bin.content_type.is_none() {
                                bin.content_type =
                                    Some(std::str::from_utf8(text).unwrap_or_default().to_string());
                            }
                        }

                        offset_start = idx + 1;
                        break;
                    }
                    _ => {}
                }
            }

            let text = text.get(offset_start..).unwrap_or_default();
            if !text.is_empty() {
                if is_base64 {
                    if let Some(bytes) = base64_decode(text) {
                        bin.data = bytes;
                        return Ok(bin);
                    }
                } else {
                    let (success, bytes) = decode_hex(text);
                    if success {
                        bin.data = bytes;
                        return Ok(bin);
                    }
                }
            }
        }

        Err(self.into_string())
    }

    pub(crate) fn into_date(self) -> std::result::Result<VCardPartialDateTime, String> {
        let mut dt = VCardPartialDateTime::default();
        dt.parse_date(&mut self.text.iter().peekable());
        if !dt.is_null() {
            Ok(dt)
        } else {
            Err(self.into_string())
        }
    }

    pub(crate) fn into_date_and_or_datetime(
        self,
    ) -> std::result::Result<VCardPartialDateTime, String> {
        let mut dt = VCardPartialDateTime::default();
        dt.parse_date_and_or_time(&mut self.text.iter().peekable());
        if !dt.is_null() {
            Ok(dt)
        } else {
            Err(self.into_string())
        }
    }

    pub(crate) fn into_date_time(self) -> std::result::Result<VCardPartialDateTime, String> {
        let mut dt = VCardPartialDateTime::default();
        dt.parse_date_time(&mut self.text.iter().peekable());
        if !dt.is_null() {
            Ok(dt)
        } else {
            Err(self.into_string())
        }
    }

    pub(crate) fn into_time(self) -> std::result::Result<VCardPartialDateTime, String> {
        let mut dt = VCardPartialDateTime::default();
        dt.parse_time(&mut self.text.iter().peekable(), false);
        if !dt.is_null() {
            Ok(dt)
        } else {
            Err(self.into_string())
        }
    }

    pub(crate) fn into_offset(self) -> std::result::Result<VCardPartialDateTime, String> {
        let mut dt = VCardPartialDateTime::default();
        dt.parse_zone(&mut self.text.iter().peekable());
        if !dt.is_null() {
            Ok(dt)
        } else {
            Err(self.into_string())
        }
    }

    pub(crate) fn into_float(self) -> std::result::Result<f64, String> {
        if let Ok(text) = std::str::from_utf8(self.text.as_ref()) {
            if let Ok(float) = text.parse::<f64>() {
                return Ok(float);
            }
        }

        Err(self.into_string())
    }

    pub(crate) fn into_integer(self) -> std::result::Result<i64, String> {
        if let Ok(text) = std::str::from_utf8(self.text.as_ref()) {
            if let Ok(float) = text.parse::<i64>() {
                return Ok(float);
            }
        }

        Err(self.into_string())
    }

    pub(crate) fn into_timestamp(self) -> std::result::Result<VCardPartialDateTime, String> {
        let mut dt = VCardPartialDateTime::default();
        if dt.parse_timestamp(&mut self.text.iter().peekable()) {
            Ok(dt)
        } else {
            Err(self.into_string())
        }
    }

    pub(crate) fn into_timestamp_or_legacy(
        self,
    ) -> std::result::Result<VCardPartialDateTime, String> {
        let mut dt = VCardPartialDateTime::default();
        if dt.parse_timestamp(&mut self.text.iter().peekable()) {
            Ok(dt)
        } else {
            let mut dt = VCardPartialDateTime::default();
            if dt.parse_date_legacy(&mut self.text.iter().peekable()) {
                Ok(dt)
            } else {
                Err(self.into_string())
            }
        }
    }

    pub(crate) fn into_datetime_or_legacy(
        self,
    ) -> std::result::Result<VCardPartialDateTime, String> {
        let mut dt = VCardPartialDateTime::default();
        if dt.parse_date_legacy(&mut self.text.iter().peekable()) {
            Ok(dt)
        } else {
            self.into_date_and_or_datetime()
        }
    }

    pub(crate) fn into_offset_or_legacy(self) -> std::result::Result<VCardPartialDateTime, String> {
        let mut dt = VCardPartialDateTime::default();
        if dt.parse_zone_legacy(&mut self.text.iter().peekable()) {
            Ok(dt)
        } else {
            self.into_offset()
        }
    }

    pub(crate) fn into_boolean(self) -> bool {
        self.text.as_ref().eq_ignore_ascii_case(b"true")
    }
}

impl VCardPartialDateTime {
    fn parse_timestamp(&mut self, iter: &mut Peekable<Iter<u8>>) -> bool {
        let mut idx = 0;
        for ch in iter {
            match ch {
                b'0'..=b'9' => {
                    let value = match idx {
                        0..=3 => &mut self.year,
                        4..=5 => &mut self.month,
                        6..=7 => &mut self.day,
                        9..=10 => &mut self.hour,
                        11..=12 => &mut self.minute,
                        13..=14 => &mut self.second,
                        16..=17 => &mut self.tz_hour,
                        18..=19 => &mut self.tz_minute,
                        _ => return false,
                    };

                    if let Some(value) = value {
                        *value = value.saturating_mul(10).saturating_add((ch - b'0') as u16);
                    } else {
                        *value = Some((ch - b'0') as u16);
                    }
                }
                b'T' | b't' if idx == 8 => {}
                b'+' if idx == 15 => {}
                b'Z' | b'z' if idx == 15 => {
                    self.tz_hour = Some(0);
                    self.tz_minute = Some(0);
                }
                b'-' if idx == 15 => {
                    self.tz_minus = true;
                }
                b' ' | b'\t' | b'\r' | b'\n' => {
                    continue;
                }
                _ => return false,
            }
            idx += 1;
        }

        true
    }

    fn parse_date_legacy(&mut self, iter: &mut Peekable<Iter<u8>>) -> bool {
        let mut idx = 0;

        for ch in iter {
            match ch {
                b'0'..=b'9' => {
                    let value = match idx {
                        0 => &mut self.year,
                        1 => &mut self.month,
                        2 => &mut self.day,
                        3 => &mut self.hour,
                        4 => &mut self.minute,
                        5 => &mut self.second,
                        6 => &mut self.tz_hour,
                        7 => &mut self.tz_minute,
                        _ => return false,
                    };

                    if let Some(value) = value {
                        *value = value.saturating_mul(10).saturating_add((ch - b'0') as u16);
                    } else {
                        *value = Some((ch - b'0') as u16);
                    }
                }
                b'T' | b't' if idx < 3 => {
                    idx = 3;
                }
                b'+' if idx <= 5 => {
                    idx = 6;
                }
                b'Z' | b'z' if idx == 5 => {
                    self.tz_hour = Some(0);
                    self.tz_minute = Some(0);
                    break;
                }
                b'-' if idx <= 2 => {
                    idx += 1;
                }
                b'-' if idx <= 5 => {
                    self.tz_minus = true;
                    idx = 6;
                }
                b':' if (3..=6).contains(&idx) => {
                    idx += 1;
                }
                b' ' | b'\t' | b'\r' | b'\n' => {
                    continue;
                }
                _ => return false,
            }
        }

        self.has_date() || self.has_zone()
    }

    fn parse_zone_legacy(&mut self, iter: &mut Peekable<Iter<u8>>) -> bool {
        let mut idx = 0;

        for ch in iter {
            match ch {
                b'0'..=b'9' => {
                    let value = match idx {
                        0 => &mut self.tz_hour,
                        1 => &mut self.tz_minute,
                        _ => return false,
                    };

                    if let Some(value) = value {
                        *value = value.saturating_mul(10).saturating_add((ch - b'0') as u16);
                    } else {
                        *value = Some((ch - b'0') as u16);
                    }
                }
                b'+' if self.tz_hour.is_none() => {}
                b'-' if self.tz_hour.is_none() => {
                    self.tz_minus = true;
                }
                b'Z' | b'z' if self.tz_hour.is_none() => {
                    self.tz_hour = Some(0);
                    self.tz_minute = Some(0);
                    break;
                }
                b':' => {
                    idx += 1;
                }
                b' ' | b'\t' | b'\r' | b'\n' => {
                    continue;
                }
                _ => return false,
            }
        }

        self.tz_hour.is_some() && self.tz_minute.is_some()
    }

    fn parse_date_time(&mut self, iter: &mut Peekable<Iter<u8>>) {
        self.parse_date_noreduc(iter);
        if matches!(iter.peek(), Some(&&b'T' | &&b't')) {
            iter.next();
            self.parse_time(iter, true);
        }
    }

    fn parse_date_and_or_time(&mut self, iter: &mut Peekable<Iter<u8>>) {
        self.parse_date(iter);
        if matches!(iter.peek(), Some(&&b'T' | &&b't')) {
            iter.next();
            self.parse_time(iter, false);
        }
    }

    fn parse_date(&mut self, iter: &mut Peekable<Iter<u8>>) {
        parse_digits(iter, &mut self.year, 4, true);
        if self.year.is_some() && iter.peek() == Some(&&b'-') {
            iter.next();
            parse_digits(iter, &mut self.month, 2, true);
        } else {
            parse_digits(iter, &mut self.month, 2, true);
            parse_digits(iter, &mut self.day, 2, false);
        }
    }

    fn parse_date_noreduc(&mut self, iter: &mut Peekable<Iter<u8>>) {
        parse_digits(iter, &mut self.year, 4, true);
        parse_digits(iter, &mut self.month, 2, true);
        parse_digits(iter, &mut self.day, 2, false);
    }

    fn parse_time(&mut self, iter: &mut Peekable<Iter<u8>>, mut notrunc: bool) {
        for part in [&mut self.hour, &mut self.minute, &mut self.second] {
            match iter.peek() {
                Some(b'0'..=b'9') => {
                    notrunc = true;
                    parse_digits(iter, part, 2, false);
                }
                Some(b'-') if !notrunc => {
                    iter.next();
                }
                _ => break,
            }
        }
        self.parse_zone(iter);
    }

    fn parse_zone(&mut self, iter: &mut Peekable<Iter<u8>>) -> bool {
        self.tz_minus = match iter.peek() {
            Some(b'-') => true,
            Some(b'+') => false,
            Some(b'Z') | Some(b'z') => {
                self.tz_hour = Some(0);
                self.tz_minute = Some(0);
                iter.next();
                return true;
            }
            _ => return false,
        };

        iter.next();
        let mut idx = 0;
        for ch in iter {
            match ch {
                b'0'..=b'9' => {
                    idx += 1;
                    let value = match idx {
                        1 | 2 => &mut self.tz_hour,
                        3 | 4 => &mut self.tz_minute,
                        _ => return false,
                    };

                    if let Some(value) = value {
                        *value = value.saturating_mul(10).saturating_add((ch - b'0') as u16);
                    } else {
                        *value = Some((ch - b'0') as u16);
                    }
                }
                _ => {
                    if !ch.is_ascii_whitespace() {
                        return false;
                    }
                }
            }
        }

        self.tz_hour.is_some()
    }

    pub fn is_null(&self) -> bool {
        self.year.is_none()
            && self.month.is_none()
            && self.day.is_none()
            && self.hour.is_none()
            && self.minute.is_none()
            && self.second.is_none()
            && self.tz_hour.is_none()
            && self.tz_minute.is_none()
    }

    pub fn has_date(&self) -> bool {
        self.year.is_some() && self.month.is_some() && self.day.is_some()
    }

    pub fn has_time(&self) -> bool {
        self.hour.is_some() && self.minute.is_some()
    }

    pub fn has_zone(&self) -> bool {
        self.tz_hour.is_some()
    }

    pub fn to_timestamp(&self) -> Option<i64> {
        if self.has_date() && self.has_time() {
            DateTime {
                year: self.year.unwrap(),
                month: self.month.unwrap() as u8,
                day: self.day.unwrap() as u8,
                hour: self.hour.unwrap() as u8,
                minute: self.minute.unwrap() as u8,
                second: self.second.unwrap_or_default() as u8,
                tz_before_gmt: self.tz_minus,
                tz_hour: self.tz_hour.unwrap_or_default() as u8,
                tz_minute: self.tz_minute.unwrap_or_default() as u8,
            }
            .to_timestamp()
            .into()
        } else {
            None
        }
    }
}

fn parse_digits(
    iter: &mut Peekable<Iter<u8>>,
    target: &mut Option<u16>,
    num: usize,
    nullable: bool,
) {
    let mut idx = 0;
    while let Some(ch) = iter.peek() {
        match ch {
            b'0'..=b'9' => {
                let ch = (*ch - b'0') as u16;
                idx += 1;
                iter.next();

                if let Some(target) = target {
                    *target = target.saturating_mul(10).saturating_add(ch);

                    if idx == num {
                        return;
                    }
                } else {
                    *target = Some(ch);
                }
            }
            b'-' if nullable => {
                idx += 1;
                iter.next();
                if idx == num / 2 {
                    return;
                }
            }
            _ => {
                if !ch.is_ascii_whitespace() {
                    return;
                } else {
                    iter.next();
                }
            }
        };
    }
}

#[derive(Default)]
pub(crate) struct Timestamp(pub i64);

impl FromStr for Timestamp {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut dt = VCardPartialDateTime::default();
        dt.parse_timestamp(&mut s.as_bytes().iter().peekable());
        dt.to_timestamp().map(Timestamp).ok_or(())
    }
}
