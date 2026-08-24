/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::{Data, PartialDateTime, tokenizer::Token};
use crate::{
    Parser,
    common::{IanaParse, IanaString, IanaType},
    icalendar::Uri,
};
use mail_parser::{
    DateTime,
    decoders::{base64::base64_decode, hex::decode_hex},
};
use std::{borrow::Cow, iter::Peekable, slice::Iter, str::FromStr};

pub(crate) fn unfold(text: &str) -> Cow<'_, str> {
    if !text.as_bytes().contains(&b'\n') {
        return Cow::Borrowed(text);
    }

    let mut unfolded = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(pos) = rest.find('\n') {
        let after = &rest[pos + 1..];

        if let Some(continuation) = after.strip_prefix([' ', '\t']) {
            let end = if rest[..pos].ends_with('\r') {
                pos - 1
            } else {
                pos
            };
            unfolded.push_str(&rest[..end]);
            rest = continuation;
        } else {
            unfolded.push_str(&rest[..=pos]);
            rest = after;
        }
    }

    unfolded.push_str(rest);

    Cow::Owned(unfolded)
}

impl<'x> Parser<'x> {
    pub(crate) fn raw_token(&mut self) -> Option<Cow<'x, str>> {
        self.token_buf
            .first()
            .and_then(|first| {
                self.input
                    .get(first.start..=self.token_buf.last().unwrap().end)
            })
            .and_then(|v| std::str::from_utf8(v).ok())
            .map(unfold)
    }

    pub(crate) fn buf_parse_many<T: From<Token<'x>>>(&mut self) -> Vec<T> {
        self.token_buf.drain(..).map(T::from).collect()
    }

    pub(crate) fn buf_parse_one<T: From<Token<'x>>>(&mut self) -> Option<T> {
        let result = self.token_buf.pop().map(T::from);
        self.token_buf.clear();
        result
    }
}

impl Token<'_> {
    pub(crate) fn into_uri_bytes(self) -> std::result::Result<Data, String> {
        match Data::try_parse(self.text.as_ref()) {
            Some(data) => Ok(data),
            None => Err(self.into_string()),
        }
    }

    pub(crate) fn into_timestamp(
        self,
        require_time: bool,
    ) -> std::result::Result<PartialDateTime, String> {
        let mut dt = PartialDateTime::default();
        if dt.parse_timestamp(&mut self.text.iter().peekable(), require_time) {
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
        if let Ok(text) = std::str::from_utf8(self.text.as_ref())
            && let Ok(float) = text.parse::<f64>()
        {
            return Ok(float);
        }

        Err(self.into_string())
    }

    pub(crate) fn into_integer(self) -> std::result::Result<i64, String> {
        if let Ok(text) = std::str::from_utf8(self.text.as_ref())
            && let Ok(float) = text.parse::<i64>()
        {
            return Ok(float);
        }

        Err(self.into_string())
    }

    pub(crate) fn into_boolean(self) -> bool {
        self.text.as_ref().eq_ignore_ascii_case(b"true")
    }
}

impl Data {
    pub fn try_parse(text: &[u8]) -> Option<Self> {
        if text
            .as_ref()
            .get(0..5)
            .unwrap_or_default()
            .eq_ignore_ascii_case(b"data:")
        {
            let mut bin = Data::default();
            let text = text.as_ref().get(5..).unwrap_or_default();
            let mut offset_start = 0;
            let mut is_base64 = false;

            for (idx, ch) in text.iter().enumerate() {
                match ch {
                    b';' => {
                        if idx > offset_start && bin.content_type.is_none() {
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
                            static B64_LEN: usize = "base64".len();

                            let text = text.get(offset_start..idx).unwrap_or_default();
                            if text.len() == B64_LEN && text.eq_ignore_ascii_case(b"base64")
                                || text.len() == B64_LEN + 1
                                    && text[..B64_LEN].eq_ignore_ascii_case(b"base64")
                            {
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
                        return Some(bin);
                    }
                } else {
                    let (success, bytes) = decode_hex(text);
                    if success {
                        bin.data = bytes;
                        return Some(bin);
                    }
                }
            }
        }

        None
    }
}

impl PartialDateTime {
    pub fn parse_timestamp(
        &mut self,
        iter: &mut Peekable<Iter<'_, u8>>,
        require_time: bool,
    ) -> bool {
        let mut idx = 0;
        for ch in iter {
            match ch {
                b'0'..=b'9' => {
                    let value = match idx {
                        0..=3 => {
                            if let Some(value) = &mut self.year {
                                *value =
                                    value.saturating_mul(10).saturating_add((ch - b'0') as u16);
                            } else {
                                self.year = Some((ch - b'0') as u16);
                            }
                            idx += 1;
                            continue;
                        }
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
                        *value = value.saturating_mul(10).saturating_add(ch - b'0');
                    } else {
                        *value = Some(ch - b'0');
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

        if self.hour.is_some() {
            self.minute.get_or_insert(0);
            self.second.get_or_insert(0);
        }

        self.has_date() && (!require_time || self.has_time())
    }

    pub(crate) fn parse_zone(&mut self, iter: &mut Peekable<Iter<'_, u8>>) -> bool {
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
                        *value = value.saturating_mul(10).saturating_add(ch - b'0');
                    } else {
                        *value = Some(ch - b'0');
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

    #[inline(always)]
    pub fn has_date(&self) -> bool {
        self.year.is_some() && self.month.is_some() && self.day.is_some()
    }

    #[inline(always)]
    pub fn has_time(&self) -> bool {
        self.hour.is_some() && self.minute.is_some()
    }

    #[inline(always)]
    pub fn has_zone(&self) -> bool {
        self.tz_hour.is_some()
    }

    #[inline(always)]
    pub fn has_date_and_time(&self) -> bool {
        self.has_date() && self.has_time()
    }

    fn to_datetime(&self) -> Option<DateTime> {
        if self.has_date() && self.has_time() {
            DateTime {
                year: self.year.unwrap(),
                month: self.month.unwrap(),
                day: self.day.unwrap(),
                hour: self.hour.unwrap(),
                minute: self.minute.unwrap(),
                second: self.second.unwrap_or_default(),
                tz_before_gmt: self.tz_minus,
                tz_hour: self.tz_hour.unwrap_or_default(),
                tz_minute: self.tz_minute.unwrap_or_default(),
            }
            .into()
        } else {
            None
        }
    }

    pub fn to_rfc3339(&self) -> Option<String> {
        self.to_datetime().map(|dt| dt.to_rfc3339())
    }

    pub fn to_timestamp(&self) -> Option<i64> {
        self.to_datetime().map(|dt| dt.to_timestamp())
    }
}

pub(crate) fn parse_digits(
    iter: &mut Peekable<Iter<'_, u8>>,
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

pub(crate) fn parse_small_digits(
    iter: &mut Peekable<Iter<'_, u8>>,
    target: &mut Option<u8>,
    num: usize,
    nullable: bool,
) -> bool {
    let mut idx = 0;
    while let Some(ch) = iter.peek() {
        match ch {
            b'0'..=b'9' => {
                let ch = *ch - b'0';
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
        dt.parse_timestamp(&mut s.as_bytes().iter().peekable(), true);
        dt.to_timestamp().map(Timestamp).ok_or(())
    }
}

impl IanaParse for Timestamp {
    fn parse(value: &[u8]) -> Option<Self> {
        let mut dt = PartialDateTime::default();
        dt.parse_timestamp(&mut value.iter().peekable(), true);
        dt.to_timestamp().map(Timestamp)
    }
}

pub(crate) struct Integer(pub i64);

impl IanaParse for Integer {
    fn parse(value: &[u8]) -> Option<Self> {
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
                        return None;
                    }
                }
            }
        }

        Some(Integer(if is_negative { -num } else { num }))
    }
}

pub(crate) struct Boolean(pub bool);

impl IanaParse for Boolean {
    fn parse(value: &[u8]) -> Option<Self> {
        if value.eq_ignore_ascii_case(b"true") {
            Some(Boolean(true))
        } else if value.eq_ignore_ascii_case(b"false") {
            Some(Boolean(false))
        } else {
            None
        }
    }
}

impl<I> From<Token<'_>> for IanaType<I, String>
where
    I: IanaParse,
{
    fn from(value: Token<'_>) -> Self {
        I::parse(value.text.as_ref())
            .map(IanaType::Iana)
            .unwrap_or_else(|| IanaType::Other(value.into_string()))
    }
}

impl<I> From<Token<'_>> for IanaType<I, Uri>
where
    I: IanaString
        + IanaParse
        + std::fmt::Debug
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + std::hash::Hash,
{
    fn from(value: Token<'_>) -> Self {
        I::parse(value.text.as_ref())
            .map(IanaType::Iana)
            .unwrap_or_else(|| {
                IanaType::Other(
                    value
                        .into_uri_bytes()
                        .map(Uri::Data)
                        .unwrap_or_else(Uri::Location),
                )
            })
    }
}

#[cfg(test)]
mod tests {

    use crate::common::tokenizer::StopChar;

    use super::*;

    #[test]
    fn test_parse_uri() {
        for (uri, expected) in [
            (
                concat!(
                    "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAsAAAALCAQAAAADpb+\n",
                    "tAAAAQklEQVQI122PQQ4AMAjCKv//Mzs4M0zmRYKkamEwWQVoRJogk4PuRoOoMC/EK8nYb+",
                    "l08WGvSxKlNHO5kxnp/WXrAzsSERN1N6q5AAAAAElFTkSuQmCC"
                ),
                Data {
                    content_type: Some("image/png".to_string()),
                    data: vec![
                        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 11,
                        0, 0, 0, 11, 8, 4, 0, 0, 0, 3, 165, 191, 173, 0, 0, 0, 66, 73, 68, 65, 84,
                        8, 215, 109, 143, 65, 14, 0, 48, 8, 194, 42, 255, 255, 51, 59, 56, 51, 76,
                        230, 69, 130, 164, 106, 97, 48, 89, 5, 104, 68, 154, 32, 147, 131, 238, 70,
                        131, 168, 48, 47, 196, 43, 201, 216, 111, 233, 116, 241, 97, 175, 75, 18,
                        165, 52, 115, 185, 147, 25, 233, 253, 101, 235, 3, 59, 18, 17, 19, 117, 55,
                        170, 185, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
                    ],
                },
            ),
            (
                concat!(
                    "data:text/html,%3Cbody%3E%3Cp%3EToller%20%3Cstron",
                    "g%20class%3D%22text-bold%22%3ETermin%3C%2Fstrong%3E%20f%C3%BCr%3C%2Fp",
                    "%3E%3Cblockquote%3E%3Cp%3Emal%20%3Cem%20class%3D%22text-italic%22%3Ez",
                    "u%22gucken%22%3C%2Fem%3E%3C%2Fp%3E%3C%2Fblockquote%3E%3Cp%3E%3Cu%20cl",
                    "ass%3D%22text-underline%22%3Eund%3C%2Fu%3E%20%3Cdel%3Eso%3C%2Fdel%3E%",
                    "3Cbr%2F%3E%3C%2Fp%3E%3C%2Fbody%3E"
                ),
                Data {
                    content_type: Some("text/html".to_string()),
                    data: vec![
                        60, 98, 111, 100, 121, 62, 60, 112, 62, 84, 111, 108, 108, 101, 114, 32,
                        60, 115, 116, 114, 111, 110, 103, 32, 99, 108, 97, 115, 115, 61, 34, 116,
                        101, 120, 116, 45, 98, 111, 108, 100, 34, 62, 84, 101, 114, 109, 105, 110,
                        60, 47, 115, 116, 114, 111, 110, 103, 62, 32, 102, 195, 188, 114, 60, 47,
                        112, 62, 60, 98, 108, 111, 99, 107, 113, 117, 111, 116, 101, 62, 60, 112,
                        62, 109, 97, 108, 32, 60, 101, 109, 32, 99, 108, 97, 115, 115, 61, 34, 116,
                        101, 120, 116, 45, 105, 116, 97, 108, 105, 99, 34, 62, 122, 117, 34, 103,
                        117, 99, 107, 101, 110, 34, 60, 47, 101, 109, 62, 60, 47, 112, 62, 60, 47,
                        98, 108, 111, 99, 107, 113, 117, 111, 116, 101, 62, 60, 112, 62, 60, 117,
                        32, 99, 108, 97, 115, 115, 61, 34, 116, 101, 120, 116, 45, 117, 110, 100,
                        101, 114, 108, 105, 110, 101, 34, 62, 117, 110, 100, 60, 47, 117, 62, 32,
                        60, 100, 101, 108, 62, 115, 111, 60, 47, 100, 101, 108, 62, 60, 98, 114,
                        47, 62, 60, 47, 112, 62, 60, 47, 98, 111, 100, 121, 62,
                    ],
                },
            ),
        ] {
            assert_eq!(
                Token {
                    text: uri.as_bytes().into(),
                    start: 0,
                    end: 0,
                    stop_char: StopChar::Lf
                }
                .into_uri_bytes(),
                Ok(expected)
            );
        }
    }
}
