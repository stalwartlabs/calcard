use std::{borrow::Cow, iter::Peekable, slice::Iter, str::FromStr};

use mail_parser::{
    decoders::{base64::base64_decode, hex::decode_hex},
    DateTime,
};

use crate::Parser;

use super::{tokenizer::Token, Data, PartialDateTime};

impl<'x> Parser<'x> {
    pub(crate) fn buf_to_string(&mut self) -> String {
        match self.token_buf.len() {
            0 => String::new(),
            1 => self.token_buf.pop().unwrap().into_string(),
            _ => {
                let from_offset = self.token_buf.first().unwrap().start;
                let to_offset = self.token_buf.last().unwrap().end;

                if self
                    .token_buf
                    .iter()
                    .all(|t| matches!(t.text, Cow::Borrowed(_)))
                {
                    self.token_buf.clear();
                    self.input
                        .get(from_offset..=to_offset)
                        .map(|slice| String::from_utf8_lossy(slice).into_owned())
                        .unwrap_or_default()
                } else {
                    let mut string = String::with_capacity(to_offset - from_offset);
                    for token in self.token_buf.drain(..) {
                        string
                            .push_str(std::str::from_utf8(token.text.as_ref()).unwrap_or_default());
                    }
                    string
                }
            }
        }
    }

    pub(crate) fn buf_to_other<T: FromStr>(&mut self) -> Option<T> {
        let result = self.token_buf.first().and_then(|token| {
            std::str::from_utf8(token.text.as_ref())
                .ok()
                .and_then(|s| s.parse().ok())
        });
        self.token_buf.clear();
        result
    }

    pub(crate) fn buf_to_bool(&mut self) -> bool {
        let result = self
            .token_buf
            .pop()
            .is_some_and(|token| token.text.as_ref().eq_ignore_ascii_case(b"TRUE"));
        self.token_buf.clear();
        result
    }

    pub(crate) fn buf_parse_many<T: From<Token<'x>>>(&mut self) -> Vec<T> {
        self.token_buf.drain(..).map(T::from).collect()
    }

    pub(crate) fn buf_parse_one<T: From<Token<'x>>>(&mut self) -> Option<T> {
        let result = self.token_buf.pop().map(T::from);
        self.token_buf.clear();
        result
    }

    pub(crate) fn buf_try_parse_one<T: for<'y> TryFrom<&'y [u8]>>(&mut self) -> Option<T> {
        let result = self
            .token_buf
            .first()
            .and_then(|t| T::try_from(t.text.as_ref()).ok());
        self.token_buf.clear();
        result
    }
}

impl Token<'_> {
    pub(crate) fn into_uri_bytes(self) -> std::result::Result<Data, String> {
        if self
            .text
            .as_ref()
            .get(0..5)
            .unwrap_or_default()
            .eq_ignore_ascii_case(b"data:")
        {
            let mut bin = Data::default();
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

    pub(crate) fn into_timestamp(self) -> std::result::Result<PartialDateTime, String> {
        let mut dt = PartialDateTime::default();
        if dt.parse_timestamp(&mut self.text.iter().peekable()) {
            Ok(dt)
        } else {
            Err(self.into_string())
        }
    }

    pub(crate) fn into_offset(self) -> std::result::Result<PartialDateTime, String> {
        let mut dt = PartialDateTime::default();
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

    pub(crate) fn into_boolean(self) -> bool {
        self.text.as_ref().eq_ignore_ascii_case(b"true")
    }
}

impl PartialDateTime {
    pub fn parse_timestamp(&mut self, iter: &mut Peekable<Iter<u8>>) -> bool {
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
                _ => break,
            }
            idx += 1;
        }

        self.has_date() && self.has_time()
    }

    pub(crate) fn parse_zone(&mut self, iter: &mut Peekable<Iter<u8>>) -> bool {
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

pub(crate) fn parse_digits(
    iter: &mut Peekable<Iter<u8>>,
    target: &mut Option<u16>,
    num: usize,
    nullable: bool,
) -> bool {
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
                        return true;
                    }
                } else {
                    *target = Some(ch);
                }
            }
            b'-' if nullable => {
                idx += 1;
                iter.next();
                if idx == num / 2 {
                    return true;
                }
            }
            _ => {
                if !ch.is_ascii_whitespace() {
                    return false;
                } else {
                    iter.next();
                }
            }
        };
    }

    false
}

#[derive(Default)]
pub(crate) struct Timestamp(pub i64);

impl FromStr for Timestamp {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut dt = PartialDateTime::default();
        dt.parse_timestamp(&mut s.as_bytes().iter().peekable());
        dt.to_timestamp().map(Timestamp).ok_or(())
    }
}

impl TryFrom<&[u8]> for Timestamp {
    type Error = ();

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        let mut dt = PartialDateTime::default();
        dt.parse_timestamp(&mut value.iter().peekable());
        dt.to_timestamp().map(Timestamp).ok_or(())
    }
}

pub(crate) struct Integer(pub i64);

impl TryFrom<&[u8]> for Integer {
    type Error = ();

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        let mut num: i64 = 0;
        let mut is_negative = false;

        for (pos, ch) in value.iter().enumerate() {
            match ch {
                b'-' if pos == 0 => {
                    is_negative = true;
                }
                b'+' if pos == 0 => {}
                b'0'..=b'9' => {
                    num = num.saturating_mul(10).saturating_add((*ch - b'0') as i64);
                }
                _ => {
                    if !ch.is_ascii_whitespace() {
                        return Err(());
                    }
                }
            }
        }

        Ok(Integer(if is_negative { -num } else { num }))
    }
}
