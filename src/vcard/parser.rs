/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::{
    PartialDateTime, VCard, VCardEntry, VCardParameter, VCardParameterName, VCardType, VCardValue,
    VCardValueType, ValueSeparator, ValueType,
};
use crate::{
    Entry, Parser, Token,
    common::{
        CalendarScale, Data, Encoding, IanaParse, IanaType,
        parser::{Boolean, Integer, Timestamp, parse_digits, parse_small_digits},
        tokenizer::StopChar,
    },
    vcard::{
        Jscomp, VCardGramGender, VCardKind, VCardLevel, VCardParameterValue, VCardPhonetic,
        VCardProperty, VCardSex,
    },
};
use mail_parser::decoders::{
    base64::base64_decode, charsets::map::charset_decoder,
    quoted_printable::quoted_printable_decode,
};
use std::{borrow::Cow, iter::Peekable, slice::Iter};

struct Params {
    params: Vec<VCardParameter>,
    stop_char: StopChar,
    data_types: Vec<IanaType<VCardValueType, String>>,
    charset: Option<String>,
    encoding: Option<Encoding>,
    group_name: Option<String>,
}

impl Parser<'_> {
    pub fn vcard(&mut self) -> Entry {
        let mut vcard = VCard::default();
        let mut is_v4 = true;
        let mut is_valid = false;

        'outer: loop {
            // Fetch property name
            self.expect_iana_token();
            self.stop_dot = true;
            let mut token = match self.token() {
                Some(token) => token,
                None => break,
            };
            self.stop_dot = false;

            let mut params = Params {
                params: Vec::new(),
                stop_char: token.stop_char,
                data_types: Vec::new(),
                group_name: None,
                encoding: None,
                charset: None,
            };

            // Parse group name
            if matches!(token.stop_char, StopChar::Dot) {
                params.group_name = token.into_string().into();
                token = match self.token() {
                    Some(token) => token,
                    None => break,
                };
                params.stop_char = token.stop_char;
            }

            // Parse parameters
            let name = token.text;
            match params.stop_char {
                StopChar::Semicolon => {
                    self.vcard_parameters(&mut params);
                }
                StopChar::Colon => {}
                StopChar::Lf => {
                    // Invalid line
                    if name.is_empty() || !self.strict {
                        continue;
                    } else {
                        return Entry::InvalidLine(Token::new(name).into_string());
                    }
                }
                _ => {}
            }

            // Invalid stop char, try seeking colon
            if !matches!(params.stop_char, StopChar::Colon | StopChar::Lf) {
                params.stop_char = self.seek_value_or_eol();
            }

            // Parse property
            let name = match VCardProperty::parse(name.as_ref()) {
                Some(name) => name,
                None => {
                    if !name.is_empty() {
                        VCardProperty::Other(Token::new(name).into_string())
                    } else {
                        // Invalid line, skip
                        if params.stop_char != StopChar::Lf {
                            self.seek_lf();
                        }
                        continue;
                    }
                }
            };
            let mut entry = VCardEntry {
                group: params.group_name,
                name,
                params: params.params,
                values: Vec::new(),
            };

            // Parse value
            if params.stop_char != StopChar::Lf {
                let (default_type, multi_value) = entry.name.default_types();

                let is_structured = match multi_value {
                    ValueSeparator::None => {
                        self.expect_single_value();
                        false
                    }
                    ValueSeparator::Comma => {
                        self.expect_multi_value_comma();
                        false
                    }
                    ValueSeparator::Semicolon => {
                        self.expect_multi_value_semicolon();
                        false
                    }
                    ValueSeparator::SemicolonAndComma => {
                        self.expect_multi_value_semicolon_and_comma();
                        true
                    }
                    ValueSeparator::Skip => {
                        is_valid = entry.name == VCardProperty::End;
                        self.expect_single_value();
                        self.token();
                        break 'outer;
                    }
                };
                match params.encoding {
                    Some(Encoding::Base64) => {
                        if multi_value != ValueSeparator::None {
                            self.expect_single_value();
                        }
                        self.unfold_b64 = true;
                    }
                    Some(Encoding::QuotedPrintable) => {
                        self.unfold_qp = true;
                    }
                    _ => {}
                }

                let mut data_types = params.data_types.iter();
                let mut token_idx = 0;
                let mut last_is_comma = false;

                while let Some(mut token) = self.token() {
                    let (is_eol, is_comma) = match token.stop_char {
                        StopChar::Lf => (true, false),
                        StopChar::Comma => (false, true),
                        _ => (false, false),
                    };

                    // Decode old vCard
                    if let Some(encoding) = params.encoding {
                        let (bytes, default_encoding) = match encoding {
                            Encoding::Base64 => (base64_decode(&token.text), None),
                            Encoding::QuotedPrintable => {
                                (quoted_printable_decode(&token.text), "iso-8859-1".into())
                            }
                        };
                        if let Some(bytes) = bytes {
                            if let Some(decoded) = params
                                .charset
                                .as_deref()
                                .or(default_encoding)
                                .and_then(|charset| {
                                    charset_decoder(charset.as_bytes())
                                        .map(|decoder| decoder(&bytes))
                                })
                            {
                                token.text = Cow::Owned(decoded.into_bytes());
                            } else if std::str::from_utf8(&bytes).is_ok() {
                                token.text = Cow::Owned(bytes);
                            } else {
                                entry.values.push(VCardValue::Binary(Data {
                                    data: bytes,
                                    content_type: None,
                                }));
                                if is_eol {
                                    break;
                                } else {
                                    continue;
                                }
                            }
                        }
                    }

                    let default_type = match &default_type {
                        ValueType::Vcard(default_type) => default_type,
                        ValueType::Kind if token_idx == 0 => {
                            if let Some(value) = VCardKind::parse(token.text.as_ref()) {
                                entry.values.push(VCardValue::Kind(value));
                                if is_eol {
                                    break;
                                } else {
                                    continue;
                                }
                            }
                            &VCardValueType::Text
                        }
                        ValueType::Sex if token_idx == 0 => {
                            if let Some(value) = VCardSex::parse(token.text.as_ref()) {
                                entry.values.push(VCardValue::Sex(value));
                                if is_eol {
                                    break;
                                } else {
                                    continue;
                                }
                            }
                            &VCardValueType::Text
                        }
                        ValueType::GramGender if token_idx == 0 => {
                            if let Some(value) = VCardGramGender::parse(token.text.as_ref()) {
                                entry.values.push(VCardValue::GramGender(value));
                                if is_eol {
                                    break;
                                } else {
                                    continue;
                                }
                            }
                            &VCardValueType::Text
                        }
                        _ => &VCardValueType::Text,
                    };

                    let value = match data_types.next().unwrap_or(&IanaType::Iana(*default_type)) {
                        IanaType::Iana(value) => match value {
                            VCardValueType::Date if is_v4 => token
                                .into_vcard_date()
                                .map(VCardValue::PartialDateTime)
                                .unwrap_or_else(VCardValue::Text),
                            VCardValueType::DateAndOrTime if is_v4 => token
                                .into_vcard_date_and_or_datetime()
                                .map(VCardValue::PartialDateTime)
                                .unwrap_or_else(VCardValue::Text),
                            VCardValueType::DateTime if is_v4 => token
                                .into_vcard_date_time()
                                .map(VCardValue::PartialDateTime)
                                .unwrap_or_else(VCardValue::Text),
                            VCardValueType::Time if is_v4 => token
                                .into_vcard_time()
                                .map(VCardValue::PartialDateTime)
                                .unwrap_or_else(VCardValue::Text),
                            VCardValueType::Timestamp if is_v4 => token
                                .into_timestamp(true)
                                .map(VCardValue::PartialDateTime)
                                .unwrap_or_else(VCardValue::Text),
                            VCardValueType::UtcOffset if is_v4 => token
                                .into_offset()
                                .map(VCardValue::PartialDateTime)
                                .unwrap_or_else(VCardValue::Text),
                            VCardValueType::Boolean => VCardValue::Boolean(token.into_boolean()),
                            VCardValueType::Float => token
                                .into_float()
                                .map(VCardValue::Float)
                                .unwrap_or_else(VCardValue::Text),
                            VCardValueType::Integer => token
                                .into_integer()
                                .map(VCardValue::Integer)
                                .unwrap_or_else(VCardValue::Text),
                            VCardValueType::LanguageTag => VCardValue::Text(token.into_string()),
                            VCardValueType::Text => {
                                if is_v4
                                    && matches!(
                                        (&entry.name, token.text.first()),
                                        (VCardProperty::Version, Some(b'1'..=b'3'))
                                    )
                                {
                                    is_v4 = false;
                                }

                                VCardValue::Text(token.into_string())
                            }
                            VCardValueType::Uri => token
                                .into_uri_bytes()
                                .map(VCardValue::Binary)
                                .unwrap_or_else(VCardValue::Text),
                            // VCard 3.0 and older
                            VCardValueType::Date
                            | VCardValueType::DateAndOrTime
                            | VCardValueType::DateTime
                            | VCardValueType::Time => token
                                .into_vcard_datetime_or_legacy()
                                .map(VCardValue::PartialDateTime)
                                .unwrap_or_else(VCardValue::Text),
                            VCardValueType::Timestamp => token
                                .into_vcard_timestamp_or_legacy()
                                .map(VCardValue::PartialDateTime)
                                .unwrap_or_else(VCardValue::Text),
                            VCardValueType::UtcOffset => token
                                .into_vcard_offset_or_legacy()
                                .map(VCardValue::PartialDateTime)
                                .unwrap_or_else(VCardValue::Text),
                        },
                        IanaType::Other(_) => VCardValue::Text(token.into_string()),
                    };

                    if is_structured {
                        match (last_is_comma, entry.values.last_mut(), value) {
                            (
                                true,
                                Some(VCardValue::Component(structured)),
                                VCardValue::Text(value),
                            ) => {
                                structured.push(value);
                            }
                            (true, Some(VCardValue::Text(prev_value)), VCardValue::Text(value)) => {
                                *entry.values.last_mut().unwrap() =
                                    VCardValue::Component(vec![std::mem::take(prev_value), value]);
                            }
                            (_, _, value) => {
                                entry.values.push(value);
                            }
                        }

                        last_is_comma = is_comma;
                    } else {
                        entry.values.push(value);
                    }

                    if is_eol {
                        break;
                    }

                    token_idx += 1;
                }
            } else {
                entry.values.push(VCardValue::Text(String::new()));
            }

            // Add types
            if !params.data_types.is_empty() {
                entry
                    .params
                    .extend(params.data_types.into_iter().map(|dt| VCardParameter {
                        name: VCardParameterName::Value,
                        value: match dt {
                            IanaType::Iana(v) => VCardParameterValue::ValueType(v),
                            IanaType::Other(v) => VCardParameterValue::Text(v),
                        },
                    }));
            }

            if !is_v4 {
                entry.normalize_legacy_media_type();
            }

            vcard.entries.push(entry);
        }

        if is_valid || !self.strict {
            Entry::VCard(vcard)
        } else {
            Entry::UnterminatedComponent("BEGIN".into())
        }
    }

    fn vcard_parameters(&mut self, params: &mut Params) {
        while params.stop_char == StopChar::Semicolon {
            self.expect_iana_token();
            let token = match self.token() {
                Some(token) => token,
                None => {
                    params.stop_char = StopChar::Lf;
                    break;
                }
            };

            // Obtain parameter values
            let param_name = token.text;
            params.stop_char = token.stop_char;
            if !matches!(
                params.stop_char,
                StopChar::Lf | StopChar::Colon | StopChar::Semicolon
            ) {
                if params.stop_char != StopChar::Equal {
                    params.stop_char = self.seek_param_value_or_eol();
                }
                if params.stop_char == StopChar::Equal {
                    self.expect_param_value();
                    while !matches!(
                        params.stop_char,
                        StopChar::Lf | StopChar::Colon | StopChar::Semicolon
                    ) {
                        match self.token() {
                            Some(token) => {
                                params.stop_char = token.stop_char;
                                self.token_buf.push(token);
                            }
                            None => {
                                params.stop_char = StopChar::Lf;
                                break;
                            }
                        }
                    }
                }
            }

            let param_values = &mut params.params;
            if let Some(param_name) = VCardParameterName::try_parse(param_name.as_ref()) {
                if self.token_buf.is_empty() {
                    param_values.push(VCardParameter::new(param_name, VCardParameterValue::Null));
                    continue;
                }

                match param_name {
                    VCardParameterName::Value => {
                        params
                            .data_types
                            .extend(self.token_buf.drain(..).map(Into::into));
                    }
                    VCardParameterName::Pref | VCardParameterName::Index => {
                        if let Some(value) = self.buf_parse_one::<IanaType<Integer, String>>() {
                            param_values.push(VCardParameter {
                                name: param_name,
                                value: match value {
                                    IanaType::Iana(value) => {
                                        VCardParameterValue::Integer(value.0 as u32)
                                    }
                                    IanaType::Other(value) => VCardParameterValue::Text(value),
                                },
                            });
                        }
                    }
                    VCardParameterName::Calscale => {
                        if let Some(value) = self.buf_parse_one::<IanaType<CalendarScale, String>>()
                        {
                            param_values.push(VCardParameter::new(param_name, value));
                        }
                    }
                    VCardParameterName::Phonetic => {
                        if let Some(value) = self.buf_parse_one::<IanaType<VCardPhonetic, String>>()
                        {
                            param_values.push(VCardParameter::new(param_name, value));
                        }
                    }
                    VCardParameterName::Level => {
                        if let Some(value) = self.buf_parse_one::<IanaType<VCardLevel, String>>() {
                            param_values.push(VCardParameter::new(param_name, value));
                        }
                    }
                    VCardParameterName::Created => {
                        if let Some(value) = self.buf_parse_one::<IanaType<Timestamp, String>>() {
                            param_values.push(VCardParameter::new(param_name, value));
                        }
                    }
                    VCardParameterName::Derived => {
                        if let Some(value) = self.buf_parse_one::<IanaType<Boolean, String>>() {
                            param_values.push(VCardParameter::new(param_name, value));
                        }
                    }
                    VCardParameterName::Type => {
                        let mut types: Vec<IanaType<VCardType, String>> = self.buf_parse_many();

                        // RFC6350 has many mistakes, this is a workaround for the "TYPE" values
                        // which in the examples sometimes appears between quotes.
                        match types.first() {
                            Some(IanaType::Other(text))
                                if types.len() == 1 && text.contains(",") =>
                            {
                                let mut types_ = Vec::with_capacity(2);
                                for text in text.split(',') {
                                    if let Some(typ) = VCardType::parse(text.as_bytes()) {
                                        types_.push(IanaType::Iana(typ));
                                    }
                                }
                                types = types_;
                            }
                            _ => {}
                        }

                        param_values.extend(types.into_iter().map(VCardParameter::typ));
                    }
                    VCardParameterName::Jscomps => {
                        if let Some(text) = self.raw_token() {
                            let mut jscomps = Vec::with_capacity(4);
                            for (item_pos, item) in text.split(';').enumerate() {
                                if let Some(item) = item.strip_prefix("s,") {
                                    let mut sep = String::with_capacity(item.len());
                                    let mut last_is_escape = false;

                                    for ch in item.chars() {
                                        if ch == '\\' && !last_is_escape {
                                            last_is_escape = true;
                                            continue;
                                        }
                                        last_is_escape = false;
                                        sep.push(ch);
                                    }

                                    jscomps.push(Jscomp::Separator(sep));
                                } else if item_pos != 0 {
                                    let mut position = None;
                                    let mut value = None;

                                    for (pos, item) in item.split(',').enumerate() {
                                        if pos == 0 {
                                            position = item.parse::<u32>().ok();
                                        } else if pos == 1 {
                                            value = item.parse::<u32>().ok();
                                        }
                                    }

                                    if let Some(position) = position {
                                        jscomps.push(Jscomp::Entry {
                                            position,
                                            value: value.unwrap_or_default(),
                                        });
                                    }
                                } else {
                                    jscomps.push(Jscomp::Separator(item.to_string()));
                                }
                            }

                            param_values.push(VCardParameter::jscomps(
                                VCardParameterValue::Jscomps(jscomps),
                            ));
                        }
                        self.token_buf.clear();
                    }
                    _ => {
                        param_values.extend(self.token_buf.drain(..).map(|token| {
                            VCardParameter::new(
                                param_name.clone(),
                                VCardParameterValue::Text(token.into_string()),
                            )
                        }));
                    }
                }
            } else if !param_name.is_empty() {
                match VCardType::parse(param_name.as_ref()) {
                    Some(typ) if self.token_buf.is_empty() => {
                        param_values.push(VCardParameter::typ(VCardParameterValue::Type(typ)));
                    }
                    _ => match param_name.first() {
                        Some(b'c' | b'C')
                            if param_name.as_ref().eq_ignore_ascii_case(b"charset") =>
                        {
                            for token in self.token_buf.drain(..) {
                                params.charset = token.into_string().into();
                            }
                        }
                        Some(b'e' | b'E')
                            if param_name.as_ref().eq_ignore_ascii_case(b"encoding") =>
                        {
                            for token in self.token_buf.drain(..) {
                                params.encoding = Encoding::parse(token.text.as_ref());
                            }
                        }
                        _ => {
                            if params.encoding.is_none()
                                && param_name.eq_ignore_ascii_case(b"base64")
                            {
                                params.encoding = Some(Encoding::Base64);
                            } else {
                                let name =
                                    VCardParameterName::Other(Token::new(param_name).into_string())
                                        .clone();
                                if !self.token_buf.is_empty() {
                                    param_values.extend(self.token_buf.drain(..).map(|token| {
                                        VCardParameter::new(
                                            name.clone(),
                                            VCardParameterValue::Text(token.into_string()),
                                        )
                                    }));
                                } else {
                                    param_values
                                        .push(VCardParameter::new(name, VCardParameterValue::Null));
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}

impl Token<'_> {
    pub(crate) fn into_vcard_date(self) -> std::result::Result<PartialDateTime, String> {
        let mut dt = PartialDateTime::default();
        dt.parse_vcard_date(&mut self.text.iter().peekable());
        if !dt.is_null() {
            Ok(dt)
        } else {
            Err(self.into_string())
        }
    }

    pub(crate) fn into_vcard_date_and_or_datetime(
        self,
    ) -> std::result::Result<PartialDateTime, String> {
        let mut dt = PartialDateTime::default();
        dt.parse_vcard_date_and_or_time(&mut self.text.iter().peekable());
        if !dt.is_null() {
            Ok(dt)
        } else {
            Err(self.into_string())
        }
    }

    pub(crate) fn into_vcard_date_time(self) -> std::result::Result<PartialDateTime, String> {
        let mut dt = PartialDateTime::default();
        dt.parse_vcard_date_time(&mut self.text.iter().peekable());
        if !dt.is_null() {
            Ok(dt)
        } else {
            Err(self.into_string())
        }
    }

    pub(crate) fn into_vcard_time(self) -> std::result::Result<PartialDateTime, String> {
        let mut dt = PartialDateTime::default();
        dt.parse_vcard_time(&mut self.text.iter().peekable(), false);
        if !dt.is_null() {
            Ok(dt)
        } else {
            Err(self.into_string())
        }
    }

    pub(crate) fn into_vcard_timestamp_or_legacy(
        self,
    ) -> std::result::Result<PartialDateTime, String> {
        let mut dt = PartialDateTime::default();
        if dt.parse_timestamp(&mut self.text.iter().peekable(), false) {
            Ok(dt)
        } else {
            let mut dt = PartialDateTime::default();
            if dt.parse_vcard_date_legacy(&mut self.text.iter().peekable()) {
                Ok(dt)
            } else {
                Err(self.into_string())
            }
        }
    }

    pub(crate) fn into_vcard_datetime_or_legacy(
        self,
    ) -> std::result::Result<PartialDateTime, String> {
        let mut dt = PartialDateTime::default();
        if dt.parse_vcard_date_legacy(&mut self.text.iter().peekable()) {
            Ok(dt)
        } else {
            self.into_vcard_date_and_or_datetime()
        }
    }

    pub(crate) fn into_vcard_offset_or_legacy(
        self,
    ) -> std::result::Result<PartialDateTime, String> {
        let mut dt = PartialDateTime::default();
        if dt.parse_vcard_zone_legacy(&mut self.text.iter().peekable()) {
            Ok(dt)
        } else {
            self.into_offset()
        }
    }
}

impl PartialDateTime {
    pub fn parse_vcard_date_legacy(&mut self, iter: &mut Peekable<Iter<'_, u8>>) -> bool {
        let mut idx = 0;

        for ch in iter {
            match ch {
                b'0'..=b'9' => {
                    let value = match idx {
                        0 => {
                            if let Some(value) = &mut self.year {
                                *value =
                                    value.saturating_mul(10).saturating_add((ch - b'0') as u16);
                            } else {
                                self.year = Some((ch - b'0') as u16);
                            }
                            continue;
                        }
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
                        *value = value.saturating_mul(10).saturating_add(ch - b'0');
                    } else {
                        *value = Some(ch - b'0');
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

    pub fn parse_vcard_zone_legacy(&mut self, iter: &mut Peekable<Iter<'_, u8>>) -> bool {
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
                        *value = value.saturating_mul(10).saturating_add(ch - b'0');
                    } else {
                        *value = Some(ch - b'0');
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

    pub fn parse_vcard_date_time(&mut self, iter: &mut Peekable<Iter<'_, u8>>) {
        self.parse_vcard_date_noreduc(iter);
        if matches!(iter.peek(), Some(&&b'T' | &&b't')) {
            iter.next();
            self.parse_vcard_time(iter, true);
        }
    }

    pub fn parse_vcard_date_and_or_time(&mut self, iter: &mut Peekable<Iter<'_, u8>>) {
        self.parse_vcard_date(iter);
        if matches!(iter.peek(), Some(&&b'T' | &&b't')) {
            iter.next();
            self.parse_vcard_time(iter, false);
        }
    }

    pub fn parse_vcard_date(&mut self, iter: &mut Peekable<Iter<'_, u8>>) {
        parse_digits(iter, &mut self.year, 4, true);
        if self.year.is_some() && iter.peek() == Some(&&b'-') {
            iter.next();
            parse_small_digits(iter, &mut self.month, 2, true);
        } else {
            parse_small_digits(iter, &mut self.month, 2, true);
            parse_small_digits(iter, &mut self.day, 2, false);
        }
    }

    pub fn parse_vcard_date_noreduc(&mut self, iter: &mut Peekable<Iter<'_, u8>>) {
        parse_digits(iter, &mut self.year, 4, true);
        parse_small_digits(iter, &mut self.month, 2, true);
        parse_small_digits(iter, &mut self.day, 2, false);
    }

    pub fn parse_vcard_time(&mut self, iter: &mut Peekable<Iter<'_, u8>>, mut notrunc: bool) {
        for part in [&mut self.hour, &mut self.minute, &mut self.second] {
            match iter.peek() {
                Some(b'0'..=b'9') => {
                    notrunc = true;
                    parse_small_digits(iter, part, 2, false);
                }
                Some(b'-') if !notrunc => {
                    iter.next();
                }
                _ => break,
            }
        }
        self.parse_zone(iter);
    }
}

#[cfg(test)]
mod tests {
    use crate::Entry;

    use super::*;
    use std::io::Write;

    #[test]
    fn parse_vcard() {
        // Read all .vcf files in the test directory
        for entry in std::fs::read_dir("resources/vcard").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "vcf") {
                let input = match String::from_utf8(std::fs::read(&path).unwrap()) {
                    Ok(input) => input,
                    Err(err) => {
                        // ISO-8859-1
                        err.as_bytes()
                            .iter()
                            .map(|&b| b as char)
                            .collect::<String>()
                    }
                };
                let mut parser = Parser::new(&input);
                let mut output = std::fs::File::create(path.with_extension("vcf.out")).unwrap();
                let file_name = path.as_path().to_str().unwrap();

                loop {
                    match parser.entry() {
                        Entry::VCard(mut vcard) => {
                            /*for item in &mut vcard.entries {
                                if item.name == VCardProperty::Version {
                                    item.values = vec![VCardValue::Text("4.0".into())];
                                }
                            }*/
                            let vcard_text = vcard.to_string();
                            crate::common::writer::assert_fold_width(&vcard_text, file_name);
                            writeln!(output, "{}", vcard_text).unwrap();
                            let _vcard_orig = vcard.clone();

                            // Roundtrip parsing
                            let mut parser = Parser::new(&vcard_text);
                            match parser.entry() {
                                Entry::VCard(mut vcard_) => {
                                    vcard.entries.retain(|entry| {
                                        !matches!(entry.name, VCardProperty::Version)
                                    });
                                    vcard_.entries.retain(|entry| {
                                        !matches!(entry.name, VCardProperty::Version)
                                    });
                                    assert_eq!(vcard.entries.len(), vcard_.entries.len());

                                    if !file_name.contains("003.vcf") {
                                        for (entry, entry_) in
                                            vcard.entries.iter().zip(vcard_.entries.iter())
                                        {
                                            if entry != entry_
                                                && matches!(
                                                    (entry.values.first(), entry_.values.first()),
                                                    (
                                                        Some(VCardValue::Binary(_),),
                                                        Some(VCardValue::Text(_))
                                                    )
                                                )
                                            {
                                                continue;
                                            }
                                            assert_eq!(entry, entry_, "failed for {file_name}");
                                        }
                                    }
                                }
                                other => panic!("Expected VCard, got {other:?} for {file_name}"),
                            }

                            // Rkyv archiving tests
                            #[cfg(feature = "rkyv")]
                            {
                                let vcard_bytes =
                                    rkyv::to_bytes::<rkyv::rancor::Error>(&_vcard_orig).unwrap();
                                let vcard_unarchived = rkyv::access::<
                                    crate::vcard::ArchivedVCard,
                                    rkyv::rancor::Error,
                                >(
                                    &vcard_bytes
                                )
                                .unwrap();
                                assert_eq!(vcard_text, vcard_unarchived.to_string());

                                for version in [
                                    crate::vcard::VCardVersion::V3_0,
                                    crate::vcard::VCardVersion::V4_0,
                                ] {
                                    let mut native = String::new();
                                    let mut archived = String::new();
                                    _vcard_orig.write_to(&mut native, version).unwrap();
                                    vcard_unarchived.write_to(&mut archived, version).unwrap();
                                    assert_eq!(
                                        native, archived,
                                        "archived writer diverged at {version} for {file_name}"
                                    );
                                    crate::common::writer::assert_fold_width(&native, file_name);
                                }
                            }
                        }
                        Entry::InvalidLine(text) => {
                            println!("Invalid line in {file_name}: {text}");
                        }
                        Entry::Eof => break,
                        other => {
                            panic!("Expected VCard, got {other:?} for {file_name}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_dates() {
        for (input, typ, expected) in [
            (
                "19850412",
                VCardValueType::Date,
                PartialDateTime {
                    year: Some(1985),
                    month: Some(4),
                    day: Some(12),
                    ..Default::default()
                },
            ),
            (
                "1985-04",
                VCardValueType::Date,
                PartialDateTime {
                    year: Some(1985),
                    month: Some(4),
                    ..Default::default()
                },
            ),
            (
                "1985",
                VCardValueType::Date,
                PartialDateTime {
                    year: Some(1985),
                    ..Default::default()
                },
            ),
            (
                "--0412",
                VCardValueType::Date,
                PartialDateTime {
                    month: Some(4),
                    day: Some(12),
                    ..Default::default()
                },
            ),
            (
                "---12",
                VCardValueType::Date,
                PartialDateTime {
                    day: Some(12),
                    ..Default::default()
                },
            ),
            (
                "102200",
                VCardValueType::Time,
                PartialDateTime {
                    hour: Some(10),
                    minute: Some(22),
                    second: Some(0),
                    ..Default::default()
                },
            ),
            (
                "1022",
                VCardValueType::Time,
                PartialDateTime {
                    hour: Some(10),
                    minute: Some(22),
                    ..Default::default()
                },
            ),
            (
                "10",
                VCardValueType::Time,
                PartialDateTime {
                    hour: Some(10),
                    ..Default::default()
                },
            ),
            (
                "-2200",
                VCardValueType::Time,
                PartialDateTime {
                    minute: Some(22),
                    second: Some(0),
                    ..Default::default()
                },
            ),
            (
                "--00",
                VCardValueType::Time,
                PartialDateTime {
                    second: Some(0),
                    ..Default::default()
                },
            ),
            (
                "102200Z",
                VCardValueType::Time,
                PartialDateTime {
                    hour: Some(10),
                    minute: Some(22),
                    second: Some(0),
                    tz_hour: Some(0),
                    tz_minute: Some(0),
                    ..Default::default()
                },
            ),
            (
                "102200-0800",
                VCardValueType::Time,
                PartialDateTime {
                    hour: Some(10),
                    minute: Some(22),
                    second: Some(0),
                    tz_hour: Some(8),
                    tz_minute: Some(0),
                    tz_minus: true,
                    ..Default::default()
                },
            ),
            (
                "19961022T140000",
                VCardValueType::DateTime,
                PartialDateTime {
                    year: Some(1996),
                    month: Some(10),
                    day: Some(22),
                    hour: Some(14),
                    minute: Some(0),
                    second: Some(0),
                    ..Default::default()
                },
            ),
            (
                "--1022T1400",
                VCardValueType::DateTime,
                PartialDateTime {
                    month: Some(10),
                    day: Some(22),
                    hour: Some(14),
                    minute: Some(0),
                    ..Default::default()
                },
            ),
            (
                "---22T14",
                VCardValueType::DateTime,
                PartialDateTime {
                    day: Some(22),
                    hour: Some(14),
                    ..Default::default()
                },
            ),
            (
                "19961022T140000",
                VCardValueType::DateAndOrTime,
                PartialDateTime {
                    year: Some(1996),
                    month: Some(10),
                    day: Some(22),
                    hour: Some(14),
                    minute: Some(0),
                    second: Some(0),
                    ..Default::default()
                },
            ),
            (
                "--1022T1400",
                VCardValueType::DateAndOrTime,
                PartialDateTime {
                    month: Some(10),
                    day: Some(22),
                    hour: Some(14),
                    minute: Some(0),
                    ..Default::default()
                },
            ),
            (
                "---22T14",
                VCardValueType::DateAndOrTime,
                PartialDateTime {
                    day: Some(22),
                    hour: Some(14),
                    ..Default::default()
                },
            ),
            (
                "19850412",
                VCardValueType::DateAndOrTime,
                PartialDateTime {
                    year: Some(1985),
                    month: Some(4),
                    day: Some(12),
                    ..Default::default()
                },
            ),
            (
                "1985-04",
                VCardValueType::DateAndOrTime,
                PartialDateTime {
                    year: Some(1985),
                    month: Some(4),
                    ..Default::default()
                },
            ),
            (
                "1985",
                VCardValueType::DateAndOrTime,
                PartialDateTime {
                    year: Some(1985),
                    ..Default::default()
                },
            ),
            (
                "--0412",
                VCardValueType::DateAndOrTime,
                PartialDateTime {
                    month: Some(4),
                    day: Some(12),
                    ..Default::default()
                },
            ),
            (
                "---12",
                VCardValueType::DateAndOrTime,
                PartialDateTime {
                    day: Some(12),
                    ..Default::default()
                },
            ),
            (
                "T102200",
                VCardValueType::DateAndOrTime,
                PartialDateTime {
                    hour: Some(10),
                    minute: Some(22),
                    second: Some(0),
                    ..Default::default()
                },
            ),
            (
                "T1022",
                VCardValueType::DateAndOrTime,
                PartialDateTime {
                    hour: Some(10),
                    minute: Some(22),
                    ..Default::default()
                },
            ),
            (
                "T10",
                VCardValueType::DateAndOrTime,
                PartialDateTime {
                    hour: Some(10),
                    ..Default::default()
                },
            ),
            (
                "T-2200",
                VCardValueType::DateAndOrTime,
                PartialDateTime {
                    minute: Some(22),
                    second: Some(0),
                    ..Default::default()
                },
            ),
            (
                "T--00",
                VCardValueType::DateAndOrTime,
                PartialDateTime {
                    second: Some(0),
                    ..Default::default()
                },
            ),
            (
                "T102200Z",
                VCardValueType::DateAndOrTime,
                PartialDateTime {
                    hour: Some(10),
                    minute: Some(22),
                    second: Some(0),
                    tz_hour: Some(0),
                    tz_minute: Some(0),
                    ..Default::default()
                },
            ),
            (
                "T102200-0800",
                VCardValueType::DateAndOrTime,
                PartialDateTime {
                    hour: Some(10),
                    minute: Some(22),
                    second: Some(0),
                    tz_hour: Some(8),
                    tz_minute: Some(0),
                    tz_minus: true,
                    ..Default::default()
                },
            ),
            (
                "19961022T140000",
                VCardValueType::Timestamp,
                PartialDateTime {
                    year: Some(1996),
                    month: Some(10),
                    day: Some(22),
                    hour: Some(14),
                    minute: Some(0),
                    second: Some(0),
                    ..Default::default()
                },
            ),
            (
                "19961022T140000Z",
                VCardValueType::Timestamp,
                PartialDateTime {
                    year: Some(1996),
                    month: Some(10),
                    day: Some(22),
                    hour: Some(14),
                    minute: Some(0),
                    second: Some(0),
                    tz_hour: Some(0),
                    tz_minute: Some(0),
                    ..Default::default()
                },
            ),
            (
                "19961022T140000-05",
                VCardValueType::Timestamp,
                PartialDateTime {
                    year: Some(1996),
                    month: Some(10),
                    day: Some(22),
                    hour: Some(14),
                    minute: Some(0),
                    second: Some(0),
                    tz_hour: Some(5),
                    tz_minus: true,
                    ..Default::default()
                },
            ),
            (
                "19961022T140000-0500",
                VCardValueType::Timestamp,
                PartialDateTime {
                    year: Some(1996),
                    month: Some(10),
                    day: Some(22),
                    hour: Some(14),
                    minute: Some(0),
                    second: Some(0),
                    tz_hour: Some(5),
                    tz_minute: Some(0),
                    tz_minus: true,
                },
            ),
            (
                "-0500",
                VCardValueType::UtcOffset,
                PartialDateTime {
                    tz_hour: Some(5),
                    tz_minute: Some(0),
                    tz_minus: true,
                    ..Default::default()
                },
            ),
        ] {
            let mut iter = input.as_bytes().iter().peekable();
            let mut dt = PartialDateTime::default();

            match typ {
                VCardValueType::Date => dt.parse_vcard_date(&mut iter),
                VCardValueType::DateAndOrTime => dt.parse_vcard_date_and_or_time(&mut iter),
                VCardValueType::DateTime => dt.parse_vcard_date_time(&mut iter),
                VCardValueType::Time => dt.parse_vcard_time(&mut iter, false),
                VCardValueType::Timestamp => {
                    dt.parse_timestamp(&mut iter, true);
                }
                VCardValueType::UtcOffset => {
                    dt.parse_zone(&mut iter);
                }
                _ => unreachable!(),
            }

            assert_eq!(dt, expected, "failed for {input:?} with type {typ:?}");
            let mut dt_str = String::new();
            dt.format_as_vcard(&mut dt_str, &typ).unwrap();

            assert_eq!(
                input, dt_str,
                "roundtrip failed for {input} with type {typ:?} {dt:?}"
            );
        }
    }
}
