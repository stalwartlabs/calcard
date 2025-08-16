/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::{
    PartialDateTime, VCard, VCardEntry, VCardParameter, VCardParameterName, VCardType, VCardValue,
    VCardValueType, VCardVersion, ValueSeparator, ValueType,
};
use crate::{
    common::{
        parser::{parse_digits, parse_small_digits, Timestamp},
        tokenizer::StopChar,
        Data, Encoding,
    },
    vcard::{Jscomp, VCardProperty},
    Entry, Parser, Token,
};
use mail_parser::decoders::{
    base64::base64_decode, charsets::map::charset_decoder,
    quoted_printable::quoted_printable_decode,
};
use std::{borrow::Cow, iter::Peekable, slice::Iter};

struct Params {
    params: Vec<VCardParameter>,
    stop_char: StopChar,
    data_types: Vec<VCardValueType>,
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
            let mut token = match self.token() {
                Some(token) => token,
                None => break,
            };

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
            let name = match VCardProperty::try_from(name.as_ref()) {
                Ok(name) => name,
                Err(_) => {
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
                match multi_value {
                    ValueSeparator::None => {
                        self.expect_single_value();
                    }
                    ValueSeparator::Comma => {
                        self.expect_multi_value_comma();
                    }
                    ValueSeparator::Semicolon => {
                        self.expect_multi_value_semicolon();
                    }
                    ValueSeparator::Skip => {
                        is_valid = entry.name == VCardProperty::End;
                        self.expect_single_value();
                        self.token();
                        break 'outer;
                    }
                }
                match params.encoding {
                    Some(Encoding::Base64) if multi_value != ValueSeparator::None => {
                        self.expect_single_value();
                    }
                    Some(Encoding::QuotedPrintable) => {
                        self.unfold_qp = true;
                    }
                    _ => {}
                }

                let mut data_types = params.data_types.iter();
                let mut token_idx = 0;
                while let Some(mut token) = self.token() {
                    let eol = token.stop_char == StopChar::Lf;

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
                                if eol {
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
                            if let Ok(gram_gender) = token.text.as_ref().try_into() {
                                entry.values.push(VCardValue::Kind(gram_gender));
                                if eol {
                                    break;
                                } else {
                                    continue;
                                }
                            }
                            &VCardValueType::Text
                        }
                        ValueType::Sex if token_idx == 0 => {
                            if let Ok(gram_gender) = token.text.as_ref().try_into() {
                                entry.values.push(VCardValue::Sex(gram_gender));
                                if eol {
                                    break;
                                } else {
                                    continue;
                                }
                            }
                            &VCardValueType::Text
                        }
                        ValueType::GramGender if token_idx == 0 => {
                            if let Ok(gram_gender) = token.text.as_ref().try_into() {
                                entry.values.push(VCardValue::GramGender(gram_gender));
                                if eol {
                                    break;
                                } else {
                                    continue;
                                }
                            }
                            &VCardValueType::Text
                        }
                        _ => &VCardValueType::Text,
                    };

                    let value = match data_types.next().unwrap_or(default_type) {
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
                        VCardValueType::Other(_) => VCardValue::Text(token.into_string()),
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
                    };

                    entry.values.push(value);

                    if eol {
                        break;
                    }

                    token_idx += 1;
                }
            }

            // Add types
            if !params.data_types.is_empty() {
                entry.params.push(VCardParameter::Value(params.data_types));
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

            hashify::fnc_map_ignore_case!(param_name.as_ref(),
                b"LANGUAGE" => {
                    param_values.push(VCardParameter::Language(self.buf_to_string()));
                },
                b"VALUE" => {
                    params.data_types.extend(
                        self.token_buf
                            .drain(..)
                            .map(Into::into),
                    );
                },
                b"PREF" => {
                    param_values.push(VCardParameter::Pref(self.buf_to_other().unwrap_or_default()));
                },
                b"ALTID" => {
                    param_values.push(VCardParameter::Altid(self.buf_to_string()));
                },
                b"PID" => {
                    param_values.push(VCardParameter::Pid(
                        self.token_buf
                            .drain(..)
                            .map(|token| token.into_string())
                            .collect(),
                    ));
                },
                b"TYPE" => {
                    let mut types = self.buf_parse_many();

                    // RFC6350 has many mistakes, this is a workaround for the "TYPE" values
                    // which in the examples sometimes appears between quotes.
                    match types.first() {
                        Some(VCardType::Other(text)) if types.len() == 1 && text.contains(",") => {
                            let mut types_ = Vec::with_capacity(2);
                            for text in text.split(',') {
                                if let Ok(typ) = VCardType::try_from(text.as_bytes()) {
                                    types_.push(typ);
                                }
                            }
                            types = types_;
                        }
                        _ => {}
                    }

                    if let Some(types_) = param_values.iter_mut().find_map(|param| {
                        if let VCardParameter::Type(types) = param {
                            Some(types)
                        } else {
                            None
                        }
                    }) {
                        types_.extend(types);
                    } else {
                        param_values.push(VCardParameter::Type(types));
                    }
                },
                b"MEDIATYPE" => {
                    param_values.push(VCardParameter::Mediatype(self.buf_to_string()));
                },
                b"CALSCALE" => {
                    if let Some(value) = self.buf_parse_one() {
                        param_values.push(VCardParameter::Calscale(value));
                    }
                },
                b"SORT-AS" => {
                    param_values.push(VCardParameter::SortAs(self.buf_to_string()));
                },
                b"GEO" => {
                    param_values.push(VCardParameter::Geo(self.buf_to_string()));
                },
                b"TZ" => {
                    param_values.push(VCardParameter::Tz(self.buf_to_string()));
                },
                b"INDEX" => {
                    param_values.push(VCardParameter::Index(self.buf_to_other().unwrap_or_default()));
                },
                b"LEVEL" => {
                    if let Some(value) = self.buf_try_parse_one() {
                        param_values.push(VCardParameter::Level(value));
                    }
                },
                b"GROUP" => {
                    param_values.push(VCardParameter::Group(self.buf_to_string()));
                },
                b"CC" => {
                    param_values.push(VCardParameter::Cc(self.buf_to_string()));
                },
                b"AUTHOR" => {
                    param_values.push(VCardParameter::Author(self.buf_to_string()));
                },
                b"AUTHOR-NAME" => {
                    param_values.push(VCardParameter::AuthorName(self.buf_to_string()));
                },
                b"CREATED" => {
                    param_values.push(VCardParameter::Created(self.buf_to_other::<Timestamp>().unwrap_or_default().0));
                },
                b"DERIVED" => {
                    param_values.push(VCardParameter::Derived(self.buf_to_bool()));
                },
                b"LABEL" => {
                    param_values.push(VCardParameter::Label(self.buf_to_string()));
                },
                b"PHONETIC" => {
                    if let Some(value) = self.buf_parse_one() {
                        param_values.push(VCardParameter::Phonetic(value));
                    }
                },
                b"PROP-ID" => {
                    param_values.push(VCardParameter::PropId(self.buf_to_string()));
                },
                b"SCRIPT" => {
                    param_values.push(VCardParameter::Script(self.buf_to_string()));
                },
                b"SERVICE-TYPE" => {
                    param_values.push(VCardParameter::ServiceType(self.buf_to_string()));
                },
                b"USERNAME" => {
                    param_values.push(VCardParameter::Username(self.buf_to_string()));
                },
                b"JSPTR" => {
                    param_values.push(VCardParameter::Jsptr(self.buf_to_string()));
                },
                b"JSCOMPS" => {
                    if let Some(text) = self.raw_token() {
                        let mut jscomps = Vec::with_capacity(4);
                        for item in text.split(';') {
                            if let Some(item) = item.strip_prefix("s,") {
                                let mut sep = String::with_capacity(item.len());
                                let mut last_is_escape = false;

                                for ch in item.chars() {
                                    if ch == '\\' {
                                        if last_is_escape {
                                            last_is_escape = false;
                                        } else {
                                            last_is_escape = true;
                                            continue;
                                        }
                                    }
                                    sep.push(ch);
                                }

                                jscomps.push(Jscomp::Separator(sep));
                            } else {
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
                                    jscomps.push(Jscomp::Entry { position, value: value.unwrap_or_default() });
                                 }
                            }
                        }

                        param_values.push(VCardParameter::Jscomps(jscomps));
                    }
                    self.token_buf.clear();
                },
                b"CHARSET" => {
                    for token in self.token_buf.drain(..) {
                        params.charset = token.into_string().into();
                    }
                },
                b"ENCODING" => {
                    for token in self.token_buf.drain(..) {
                        params.encoding = Encoding::parse(token.text.as_ref());
                    }
                },
                _ => {
                    match VCardType::try_from(param_name.as_ref()) {
                        Ok(typ) if self.token_buf.is_empty() => {
                            if let Some(types) = param_values.iter_mut().find_map(|param| {
                                if let VCardParameter::Type(types) = param {
                                    Some(types)
                                } else {
                                    None
                                }
                            }) {
                                types.push(typ);
                            } else {
                                param_values.push(VCardParameter::Type(vec![typ]));
                            }
                        },
                        _ => {
                            if !param_name.is_empty() {
                                if params.encoding.is_none() && param_name.eq_ignore_ascii_case(b"base64") {
                                    params.encoding = Some(Encoding::Base64);
                                } else {
                                    param_values.push(VCardParameter::Other(
                                        [Token::new(param_name).into_string()]
                                            .into_iter()
                                            .chain(self.token_buf.drain(..).map(|token| token.into_string()))
                                            .collect(),
                                    ));
                                }
                            }
                        }
                    }
                }
            );
        }
    }
}

impl VCardParameterName {
    pub fn try_parse(input: &str) -> Option<Self> {
        hashify::tiny_map_ignore_case!(input.as_bytes(),
            b"LANGUAGE" => VCardParameterName::Language,
            b"VALUE" => VCardParameterName::Value,
            b"PREF" => VCardParameterName::Pref,
            b"ALTID" => VCardParameterName::Altid,
            b"PID" => VCardParameterName::Pid,
            b"TYPE" => VCardParameterName::Type,
            b"MEDIATYPE" => VCardParameterName::Mediatype,
            b"CALSCALE" => VCardParameterName::Calscale,
            b"SORT-AS" => VCardParameterName::SortAs,
            b"GEO" => VCardParameterName::Geo,
            b"TZ" => VCardParameterName::Tz,
            b"INDEX" => VCardParameterName::Index,
            b"LEVEL" => VCardParameterName::Level,
            b"GROUP" => VCardParameterName::Group,
            b"CC" => VCardParameterName::Cc,
            b"AUTHOR" => VCardParameterName::Author,
            b"AUTHOR-NAME" => VCardParameterName::AuthorName,
            b"CREATED" => VCardParameterName::Created,
            b"DERIVED" => VCardParameterName::Derived,
            b"LABEL" => VCardParameterName::Label,
            b"PHONETIC" => VCardParameterName::Phonetic,
            b"PROP-ID" => VCardParameterName::PropId,
            b"SCRIPT" => VCardParameterName::Script,
            b"SERVICE-TYPE" => VCardParameterName::ServiceType,
            b"USERNAME" => VCardParameterName::Username,
            b"JSPTR" => VCardParameterName::Jsptr,
        )
    }

    pub fn parse(input: &str) -> Self {
        Self::try_parse(input).unwrap_or_else(|| VCardParameterName::Other(input.into()))
    }

    pub fn as_str(&self) -> &str {
        match self {
            VCardParameterName::Language => "LANGUAGE",
            VCardParameterName::Value => "VALUE",
            VCardParameterName::Pref => "PREF",
            VCardParameterName::Altid => "ALTID",
            VCardParameterName::Pid => "PID",
            VCardParameterName::Type => "TYPE",
            VCardParameterName::Mediatype => "MEDIATYPE",
            VCardParameterName::Calscale => "CALSCALE",
            VCardParameterName::SortAs => "SORT-AS",
            VCardParameterName::Geo => "GEO",
            VCardParameterName::Tz => "TZ",
            VCardParameterName::Index => "INDEX",
            VCardParameterName::Level => "LEVEL",
            VCardParameterName::Group => "GROUP",
            VCardParameterName::Cc => "CC",
            VCardParameterName::Author => "AUTHOR",
            VCardParameterName::AuthorName => "AUTHOR-NAME",
            VCardParameterName::Created => "CREATED",
            VCardParameterName::Derived => "DERIVED",
            VCardParameterName::Label => "LABEL",
            VCardParameterName::Phonetic => "PHONETIC",
            VCardParameterName::PropId => "PROP-ID",
            VCardParameterName::Script => "SCRIPT",
            VCardParameterName::ServiceType => "SERVICE-TYPE",
            VCardParameterName::Username => "USERNAME",
            VCardParameterName::Jsptr => "JSPTR",
            VCardParameterName::Other(name) => name,
            VCardParameterName::Jscomps => "JSCOMPS",
        }
    }

    pub fn into_string(self) -> Cow<'static, str> {
        match self {
            VCardParameterName::Language => "LANGUAGE".into(),
            VCardParameterName::Value => "VALUE".into(),
            VCardParameterName::Pref => "PREF".into(),
            VCardParameterName::Altid => "ALTID".into(),
            VCardParameterName::Pid => "PID".into(),
            VCardParameterName::Type => "TYPE".into(),
            VCardParameterName::Mediatype => "MEDIATYPE".into(),
            VCardParameterName::Calscale => "CALSCALE".into(),
            VCardParameterName::SortAs => "SORT-AS".into(),
            VCardParameterName::Geo => "GEO".into(),
            VCardParameterName::Tz => "TZ".into(),
            VCardParameterName::Index => "INDEX".into(),
            VCardParameterName::Level => "LEVEL".into(),
            VCardParameterName::Group => "GROUP".into(),
            VCardParameterName::Cc => "CC".into(),
            VCardParameterName::Author => "AUTHOR".into(),
            VCardParameterName::AuthorName => "AUTHOR-NAME".into(),
            VCardParameterName::Created => "CREATED".into(),
            VCardParameterName::Derived => "DERIVED".into(),
            VCardParameterName::Label => "LABEL".into(),
            VCardParameterName::Phonetic => "PHONETIC".into(),
            VCardParameterName::PropId => "PROP-ID".into(),
            VCardParameterName::Script => "SCRIPT".into(),
            VCardParameterName::ServiceType => "SERVICE-TYPE".into(),
            VCardParameterName::Username => "USERNAME".into(),
            VCardParameterName::Jsptr => "JSPTR".into(),
            VCardParameterName::Other(name) => name.into(),
            VCardParameterName::Jscomps => "JSCOMPS".into(),
        }
    }
}

impl VCardVersion {
    pub fn try_parse(input: &str) -> Option<Self> {
        hashify::tiny_map!(input.as_bytes(),
            b"4.0" => VCardVersion::V4_0,
            b"3.0" => VCardVersion::V3_0,
            b"2.1" => VCardVersion::V2_1,
            b"2.0" => VCardVersion::V2_0,
        )
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
        if dt.parse_timestamp(&mut self.text.iter().peekable(), true) {
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
