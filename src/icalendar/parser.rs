/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::*;
use crate::{
    Entry, Parser, StopChar, Token,
    common::{
        CalendarScale, Encoding, PartialDateTime,
        parser::{Boolean, Integer, parse_digits, parse_small_digits},
    },
    icalendar::{ICalendarDay, ICalendarWeekday},
};
use mail_parser::decoders::{
    base64::base64_decode, charsets::map::charset_decoder,
    quoted_printable::quoted_printable_decode,
};
use std::{borrow::Cow, iter::Peekable, slice::Iter};

struct Params {
    params: Vec<ICalendarParameter>,
    stop_char: StopChar,
    data_type: Option<IanaType<ICalendarValueType, String>>,
    charset: Option<String>,
    encoding: Option<Encoding>,
}

impl Parser<'_> {
    pub fn icalendar(&mut self, component_type: ICalendarComponentType) -> Entry {
        let mut ical_stack: Vec<usize> = Vec::new();
        let mut ical_components = vec![ICalendarComponent {
            component_type,
            ..Default::default()
        }];
        let mut next_component_id: u16 = 1;
        let mut ical_idx = 0;
        let mut ical = ical_components.last_mut().unwrap();

        loop {
            // Fetch property name
            self.expect_iana_token();
            let token = match self.token() {
                Some(token) => token,
                None => break,
            };

            let mut params = Params {
                params: Vec::new(),
                stop_char: token.stop_char,
                data_type: None,
                encoding: None,
                charset: None,
            };

            // Parse parameters
            let name = token.text;
            match params.stop_char {
                StopChar::Semicolon => {
                    self.ical_parameters(&mut params);
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
            let name = match ICalendarProperty::parse(name.as_ref()) {
                Some(ICalendarProperty::Begin) => {
                    if params.stop_char == StopChar::Colon {
                        self.expect_single_value();
                        if let Some(token) = self.token() {
                            let component_type = ICalendarComponentType::parse(token.text.as_ref())
                                .unwrap_or_else(|| {
                                    ICalendarComponentType::Other(token.into_string())
                                });
                            ical_stack.push(ical_idx);
                            ical.component_ids.push(next_component_id);
                            ical_components.push(ICalendarComponent {
                                component_type,
                                ..Default::default()
                            });
                            ical_idx = next_component_id as usize;
                            if let Some(id) = next_component_id.checked_add(1) {
                                next_component_id = id;
                            } else {
                                return Entry::TooManyComponents;
                            }
                            ical = ical_components.last_mut().unwrap();
                        }
                    }

                    if !self.strict {
                        continue;
                    } else {
                        return Entry::InvalidLine("BEGIN".to_string());
                    }
                }
                Some(ICalendarProperty::End) => {
                    if params.stop_char == StopChar::Colon {
                        self.expect_single_value();
                        if let Some(token) = self.token() {
                            let component_type = ICalendarComponentType::parse(token.text.as_ref())
                                .unwrap_or_else(|| {
                                    ICalendarComponentType::Other(token.into_string())
                                });
                            if ical.component_type == component_type || !self.strict {
                                if let Some(parent_ical_idx) = ical_stack.pop() {
                                    ical_idx = parent_ical_idx;
                                    ical = ical_components.get_mut(ical_idx).unwrap();
                                } else {
                                    break;
                                }
                            } else {
                                return Entry::UnexpectedComponentEnd {
                                    expected: ical.component_type.clone(),
                                    found: component_type,
                                };
                            }
                        }
                    }

                    if !self.strict {
                        continue;
                    } else {
                        return Entry::InvalidLine("END".to_string());
                    }
                }
                Some(name) => name,
                None => {
                    if !name.is_empty() {
                        ICalendarProperty::Other(Token::new(name).into_string())
                    } else {
                        // Invalid line, skip
                        if params.stop_char != StopChar::Lf {
                            self.seek_lf();
                        }
                        continue;
                    }
                }
            };
            let mut entry = ICalendarEntry {
                name,
                params: params.params,
                values: Vec::new(),
            };

            // Parse value
            if params.stop_char != StopChar::Lf {
                // Obtain default type and separator
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
                }

                // Decode
                match params.encoding {
                    Some(Encoding::Base64) if multi_value != ValueSeparator::None => {
                        self.expect_single_value();
                    }
                    Some(Encoding::QuotedPrintable) => {
                        self.unfold_qp = true;
                    }
                    _ => {}
                }

                if matches!(
                    (&params.data_type, &default_type),
                    (Some(IanaType::Iana(ICalendarValueType::Recur)), _)
                        | (None, ValueType::Ical(ICalendarValueType::Recur))
                ) {
                    match self.rrule() {
                        Ok(rrule) => {
                            entry
                                .values
                                .push(ICalendarValue::RecurrenceRule(Box::new(rrule)));
                        }
                        Err(other) => {
                            entry.values.push(ICalendarValue::Text(other));
                        }
                    }
                } else {
                    while let Some(mut token) = self.token() {
                        let eol = token.stop_char == StopChar::Lf;

                        if token.text.is_empty()
                            && (matches!(multi_value, ValueSeparator::None)
                                || matches!(entry.name, ICalendarProperty::Other(_))
                                    && entry.values.is_empty())
                        {
                            if eol {
                                break;
                            } else {
                                continue;
                            }
                        }

                        // Decode binary parts
                        if let Some(encoding) = params.encoding {
                            let (bytes, default_encoding) = match encoding {
                                Encoding::Base64 => (base64_decode(&token.text), None),
                                Encoding::QuotedPrintable => {
                                    (quoted_printable_decode(&token.text), "iso-8859-1".into())
                                }
                            };
                            if let Some(bytes) = bytes {
                                if let Some(decoded) =
                                    params.charset.as_deref().or(default_encoding).and_then(
                                        |charset| {
                                            charset_decoder(charset.as_bytes())
                                                .map(|decoder| decoder(&bytes))
                                        },
                                    )
                                {
                                    token.text = Cow::Owned(decoded.into_bytes());
                                } else if std::str::from_utf8(&bytes).is_ok() {
                                    token.text = Cow::Owned(bytes);
                                } else {
                                    entry.values.push(ICalendarValue::Binary(bytes));
                                    if eol {
                                        break;
                                    } else {
                                        continue;
                                    }
                                }
                            }
                        }

                        match &default_type {
                            ValueType::Ical(default_type) => {
                                let value = match params
                                    .data_type
                                    .as_ref()
                                    .map(|v| v.iana().unwrap_or(&ICalendarValueType::Text))
                                    .unwrap_or(default_type)
                                {
                                    ICalendarValueType::Date => token
                                        .into_ical_date()
                                        .map(|data| ICalendarValue::PartialDateTime(Box::new(data)))
                                        .unwrap_or_else(ICalendarValue::Text),
                                    ICalendarValueType::DateTime => {
                                        match token.into_timestamp(false) {
                                            Ok(timestamp) => {
                                                if !timestamp.has_time() {
                                                    params.data_type = Some(IanaType::Iana(
                                                        ICalendarValueType::Date,
                                                    ));
                                                }
                                                ICalendarValue::PartialDateTime(Box::new(timestamp))
                                            }
                                            Err(other) => ICalendarValue::Text(other),
                                        }
                                    }
                                    ICalendarValueType::Time => token
                                        .into_ical_time()
                                        .map(|data| ICalendarValue::PartialDateTime(Box::new(data)))
                                        .unwrap_or_else(ICalendarValue::Text),
                                    ICalendarValueType::UtcOffset => token
                                        .into_offset()
                                        .map(|data| ICalendarValue::PartialDateTime(Box::new(data)))
                                        .unwrap_or_else(ICalendarValue::Text),
                                    ICalendarValueType::Boolean => {
                                        ICalendarValue::Boolean(token.into_boolean())
                                    }
                                    ICalendarValueType::Float => token
                                        .into_float()
                                        .map(ICalendarValue::Float)
                                        .unwrap_or_else(ICalendarValue::Text),
                                    ICalendarValueType::Integer => token
                                        .into_integer()
                                        .map(ICalendarValue::Integer)
                                        .unwrap_or_else(ICalendarValue::Text),
                                    ICalendarValueType::Text
                                    | ICalendarValueType::Binary
                                    | ICalendarValueType::Unknown
                                    | ICalendarValueType::XmlReference
                                    | ICalendarValueType::Uid => {
                                        ICalendarValue::Text(token.into_string())
                                    }
                                    ICalendarValueType::Uri | ICalendarValueType::CalAddress => {
                                        token
                                            .into_uri_bytes()
                                            .map(|data| ICalendarValue::Uri(Uri::Data(data)))
                                            .unwrap_or_else(|uri| {
                                                ICalendarValue::Uri(Uri::Location(uri))
                                            })
                                    }
                                    ICalendarValueType::Duration => {
                                        if let Some(duration) =
                                            ICalendarDuration::parse(token.text.as_ref())
                                        {
                                            ICalendarValue::Duration(duration)
                                        } else {
                                            ICalendarValue::Text(token.into_string())
                                        }
                                    }
                                    ICalendarValueType::Period => {
                                        if let Some(period) =
                                            ICalendarPeriod::parse(token.text.as_ref())
                                        {
                                            ICalendarValue::Period(period)
                                        } else {
                                            ICalendarValue::Text(token.into_string())
                                        }
                                    }
                                    ICalendarValueType::Recur => unreachable!(),
                                };

                                entry.values.push(value);
                            }
                            ValueType::CalendarScale => {
                                entry.values.push(match IanaType::from(token) {
                                    IanaType::Iana(value) => ICalendarValue::CalendarScale(value),
                                    IanaType::Other(value) => ICalendarValue::Text(value),
                                });
                            }
                            ValueType::Method => {
                                entry.values.push(match IanaType::from(token) {
                                    IanaType::Iana(value) => ICalendarValue::Method(value),
                                    IanaType::Other(value) => ICalendarValue::Text(value),
                                });
                            }
                            ValueType::Classification => {
                                entry.values.push(match IanaType::from(token) {
                                    IanaType::Iana(value) => ICalendarValue::Classification(value),
                                    IanaType::Other(value) => ICalendarValue::Text(value),
                                });
                            }
                            ValueType::Status => {
                                entry.values.push(match IanaType::from(token) {
                                    IanaType::Iana(value) => ICalendarValue::Status(value),
                                    IanaType::Other(value) => ICalendarValue::Text(value),
                                });
                            }
                            ValueType::Transparency => {
                                entry.values.push(match IanaType::from(token) {
                                    IanaType::Iana(value) => ICalendarValue::Transparency(value),
                                    IanaType::Other(value) => ICalendarValue::Text(value),
                                });
                            }
                            ValueType::Action => {
                                entry.values.push(match IanaType::from(token) {
                                    IanaType::Iana(value) => ICalendarValue::Action(value),
                                    IanaType::Other(value) => ICalendarValue::Text(value),
                                });
                            }
                            ValueType::BusyType => {
                                entry.values.push(match IanaType::from(token) {
                                    IanaType::Iana(value) => ICalendarValue::BusyType(value),
                                    IanaType::Other(value) => ICalendarValue::Text(value),
                                });
                            }
                            ValueType::ParticipantType => {
                                entry.values.push(match IanaType::from(token) {
                                    IanaType::Iana(value) => ICalendarValue::ParticipantType(value),
                                    IanaType::Other(value) => ICalendarValue::Text(value),
                                });
                            }
                            ValueType::ResourceType => {
                                entry.values.push(match IanaType::from(token) {
                                    IanaType::Iana(value) => ICalendarValue::ResourceType(value),
                                    IanaType::Other(value) => ICalendarValue::Text(value),
                                });
                            }
                            ValueType::Proximity => {
                                entry.values.push(match IanaType::from(token) {
                                    IanaType::Iana(value) => ICalendarValue::Proximity(value),
                                    IanaType::Other(value) => ICalendarValue::Text(value),
                                });
                            }
                        }

                        if eol {
                            break;
                        }
                    }
                }
            }

            // Skip begin and end properties
            if !matches!(
                entry.name,
                ICalendarProperty::Begin | ICalendarProperty::End
            ) {
                // Add types
                if let Some(data_type) = params.data_type {
                    entry.params.push(ICalendarParameter {
                        name: ICalendarParameterName::Value,
                        value: match data_type {
                            IanaType::Iana(value) => ICalendarParameterValue::Value(value),
                            IanaType::Other(value) => ICalendarParameterValue::Text(value),
                        },
                    });
                }

                ical.entries.push(entry);
            }
        }

        if ical_stack.is_empty() || !self.strict {
            Entry::ICalendar(ICalendar {
                components: ical_components,
            })
        } else {
            Entry::UnterminatedComponent(ical.component_type.as_str().to_string().into())
        }
    }

    fn ical_parameters(&mut self, params: &mut Params) {
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
            if let Some(param_name) = ICalendarParameterName::try_parse(param_name.as_ref()) {
                if self.token_buf.is_empty() {
                    param_values.push(ICalendarParameter::new(
                        param_name.clone(),
                        ICalendarParameterValue::Null,
                    ));
                }
                for token in self.token_buf.drain(..) {
                    match &param_name {
                        ICalendarParameterName::Altrep
                        | ICalendarParameterName::DelegatedFrom
                        | ICalendarParameterName::DelegatedTo
                        | ICalendarParameterName::Dir
                        | ICalendarParameterName::Member
                        | ICalendarParameterName::SentBy
                        | ICalendarParameterName::Schema => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                ICalendarParameterValue::Uri(Uri::from(token)),
                            ));
                        }
                        ICalendarParameterName::Rsvp | ICalendarParameterName::Derived => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                match IanaType::<Boolean, String>::from(token) {
                                    IanaType::Iana(value) => ICalendarParameterValue::Bool(value.0),
                                    IanaType::Other(value) => ICalendarParameterValue::Text(value),
                                },
                            ));
                        }
                        ICalendarParameterName::Range => {
                            if token.text.as_ref().eq_ignore_ascii_case(b"THISANDFUTURE") {
                                param_values.push(ICalendarParameter::new(
                                    param_name.clone(),
                                    ICalendarParameterValue::Bool(true),
                                ));
                            }
                        }
                        ICalendarParameterName::Size | ICalendarParameterName::Order => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                match IanaType::<Integer, String>::from(token) {
                                    IanaType::Iana(value) => {
                                        ICalendarParameterValue::Integer(value.0.unsigned_abs())
                                    }
                                    IanaType::Other(value) => ICalendarParameterValue::Text(value),
                                },
                            ));
                        }
                        ICalendarParameterName::Gap => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                match IanaType::from(token) {
                                    IanaType::Iana(value) => {
                                        ICalendarParameterValue::Duration(value)
                                    }
                                    IanaType::Other(value) => ICalendarParameterValue::Text(value),
                                },
                            ));
                        }
                        ICalendarParameterName::Cutype => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                match IanaType::from(token) {
                                    IanaType::Iana(value) => ICalendarParameterValue::Cutype(value),
                                    IanaType::Other(value) => ICalendarParameterValue::Text(value),
                                },
                            ));
                        }
                        ICalendarParameterName::Fbtype => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                match IanaType::from(token) {
                                    IanaType::Iana(value) => ICalendarParameterValue::Fbtype(value),
                                    IanaType::Other(value) => ICalendarParameterValue::Text(value),
                                },
                            ));
                        }
                        ICalendarParameterName::Partstat => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                match IanaType::from(token) {
                                    IanaType::Iana(value) => {
                                        ICalendarParameterValue::Partstat(value)
                                    }
                                    IanaType::Other(value) => ICalendarParameterValue::Text(value),
                                },
                            ));
                        }
                        ICalendarParameterName::Related => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                match IanaType::from(token) {
                                    IanaType::Iana(value) => {
                                        ICalendarParameterValue::Related(value)
                                    }
                                    IanaType::Other(value) => ICalendarParameterValue::Text(value),
                                },
                            ));
                        }
                        ICalendarParameterName::Reltype => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                match IanaType::from(token) {
                                    IanaType::Iana(value) => {
                                        ICalendarParameterValue::Reltype(value)
                                    }
                                    IanaType::Other(value) => ICalendarParameterValue::Text(value),
                                },
                            ));
                        }
                        ICalendarParameterName::Role => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                match IanaType::from(token) {
                                    IanaType::Iana(value) => ICalendarParameterValue::Role(value),
                                    IanaType::Other(value) => ICalendarParameterValue::Text(value),
                                },
                            ));
                        }
                        ICalendarParameterName::ScheduleAgent => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                match IanaType::from(token) {
                                    IanaType::Iana(value) => {
                                        ICalendarParameterValue::ScheduleAgent(value)
                                    }
                                    IanaType::Other(value) => ICalendarParameterValue::Text(value),
                                },
                            ));
                        }
                        ICalendarParameterName::ScheduleForceSend => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                match IanaType::from(token) {
                                    IanaType::Iana(value) => {
                                        ICalendarParameterValue::ScheduleForceSend(value)
                                    }
                                    IanaType::Other(value) => ICalendarParameterValue::Text(value),
                                },
                            ));
                        }
                        ICalendarParameterName::Value => {
                            params.data_type = Some(token.into());
                        }
                        ICalendarParameterName::Display => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                match IanaType::from(token) {
                                    IanaType::Iana(value) => {
                                        ICalendarParameterValue::Display(value)
                                    }
                                    IanaType::Other(value) => ICalendarParameterValue::Text(value),
                                },
                            ));
                        }
                        ICalendarParameterName::Feature => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                match IanaType::from(token) {
                                    IanaType::Iana(value) => {
                                        ICalendarParameterValue::Feature(value)
                                    }
                                    IanaType::Other(value) => ICalendarParameterValue::Text(value),
                                },
                            ));
                        }
                        ICalendarParameterName::Linkrel => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                match IanaType::<LinkRelation, Uri>::from(token) {
                                    IanaType::Iana(value) => {
                                        ICalendarParameterValue::Linkrel(value)
                                    }
                                    IanaType::Other(value) => ICalendarParameterValue::Uri(value),
                                },
                            ));
                        }
                        _ => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                ICalendarParameterValue::Text(token.into_string()),
                            ));
                        }
                    }
                }
            } else if !param_name.is_empty() {
                match param_name.first() {
                    Some(b'c' | b'C') if param_name.as_ref().eq_ignore_ascii_case(b"charset") => {
                        for token in self.token_buf.drain(..) {
                            params.charset = token.into_string().into();
                        }
                    }
                    Some(b'e' | b'E') if param_name.as_ref().eq_ignore_ascii_case(b"encoding") => {
                        for token in self.token_buf.drain(..) {
                            params.encoding = Encoding::parse(token.text.as_ref());
                        }
                    }
                    _ => {
                        if params.encoding.is_none() && param_name.eq_ignore_ascii_case(b"base64") {
                            params.encoding = Some(Encoding::Base64);
                        } else {
                            let name =
                                ICalendarParameterName::Other(Token::new(param_name).into_string())
                                    .clone();

                            if !self.token_buf.is_empty() {
                                param_values.extend(self.token_buf.drain(..).map(|token| {
                                    ICalendarParameter::new(
                                        name.clone(),
                                        ICalendarParameterValue::Text(token.into_string()),
                                    )
                                }));
                            } else {
                                param_values.push(ICalendarParameter::new(
                                    name,
                                    ICalendarParameterValue::Null,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    fn rrule(&mut self) -> Result<ICalendarRecurrenceRule, String> {
        self.expect_rrule_value();

        let mut last_stop_char = StopChar::Equal;
        let mut is_valid = true;
        let mut has_freq = false;
        let mut rrule = ICalendarRecurrenceRule::default();

        let mut token_start = usize::MAX;
        let mut token_end = usize::MAX;

        while let Some(mut token) = self.token_until_lf(&mut last_stop_char) {
            if token_start == usize::MAX {
                token_start = token.start;
            }
            token_end = token.end;
            if !is_valid {
                continue;
            }
            if token.stop_char != StopChar::Equal {
                if !self.strict {
                    // Ignore unknown tokens
                    while let Some(token_) = self.token_until_lf(&mut last_stop_char) {
                        token_end = token.end;
                        if token_.stop_char == StopChar::Equal {
                            token = token_;
                            break;
                        }
                    }
                    if token.stop_char != StopChar::Equal {
                        is_valid = false;
                        continue;
                    }
                } else {
                    is_valid = false;
                    continue;
                }
            }

            hashify::fnc_map_ignore_case!(token.text.as_ref(),
                b"FREQ" => {
                    while let Some(value) = self.parse_value_until_lf(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Some(value)= value {
                            rrule.freq = value;
                            has_freq = true;
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"UNTIL" => {
                    while let Some(value) = self.parse_value_until_lf::<ICalendarDateTime>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Some(value)= value {
                            rrule.until = Some(value.0);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"COUNT" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Some(value)= value {
                            let count = value.0.unsigned_abs() as u32;
                            if count > 0 {
                                rrule.count = Some(count);
                            }
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"INTERVAL" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Some(value)= value {
                            let interval = value.0.unsigned_abs() as u16;
                            if interval > 0 {
                                rrule.interval = Some(interval);
                            }
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYSECOND" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Some(value)= value {
                            rrule.bysecond.push(value.0.unsigned_abs() as u8);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYMINUTE" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Some(value)= value {
                            rrule.byminute.push(value.0.unsigned_abs() as u8);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYHOUR" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Some(value)= value {
                            rrule.byhour.push(value.0.unsigned_abs() as u8);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYDAY" => {
                    while let Some(value) = self.parse_value_until_lf::<ICalendarDay>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Some(value)= value {
                            rrule.byday.push(value);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYMONTHDAY" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Some(value)= value {
                            rrule.bymonthday.push(value.0 as i8);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYYEARDAY" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Some(value)= value {
                            rrule.byyearday.push(value.0 as i16);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYWEEKNO" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Some(value)= value {
                            rrule.byweekno.push(value.0 as i8);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYMONTH" => {
                    while let Some(value) = self.parse_value_until_lf::<ICalendarMonth>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Some(value)= value {
                            rrule.bymonth.push(value);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYSETPOS" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Some(value)= value {
                            rrule.bysetpos.push(value.0 as i32);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"WKST" => {
                    while let Some(value) = self.parse_value_until_lf::<ICalendarWeekday>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Some(value)= value {
                            rrule.wkst = Some(value);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"RSCALE" => {
                    while let Some(value) = self.parse_value_until_lf::<CalendarScale>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Some(value)= value {
                            rrule.rscale = Some(value);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"SKIP" => {
                    while let Some(value) = self.parse_value_until_lf::<ICalendarSkip>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Some(value)= value {
                            rrule.skip = Some(value);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                _ => {
                    if !self.strict {
                        // Ignore unknown tokens
                        while let Some(token) = self.token_until_lf(&mut last_stop_char) {
                            token_end = token.end;
                            if token.stop_char == StopChar::Semicolon{
                                break;
                            }
                        }
                    } else {
                        is_valid = false;
                    }
                }
            );
        }

        if has_freq {
            Ok(rrule)
        } else if token_start != usize::MAX {
            Err(self
                .input
                .get(token_start..=token_end)
                .map(|slice| String::from_utf8_lossy(slice).into_owned())
                .unwrap_or_default())
        } else {
            Err("".to_string())
        }
    }
}

impl Token<'_> {
    pub(crate) fn into_ical_date(self) -> std::result::Result<PartialDateTime, String> {
        let mut dt = PartialDateTime::default();
        if dt.parse_ical_date(&mut self.text.iter().peekable()) {
            Ok(dt)
        } else {
            Err(self.into_string())
        }
    }

    pub(crate) fn into_ical_time(self) -> std::result::Result<PartialDateTime, String> {
        let mut dt = PartialDateTime::default();
        if dt.parse_ical_time(&mut self.text.iter().peekable()) {
            Ok(dt)
        } else {
            Err(self.into_string())
        }
    }
}

impl PartialDateTime {
    pub fn parse_ical_date(&mut self, iter: &mut Peekable<Iter<'_, u8>>) -> bool {
        parse_digits(iter, &mut self.year, 4, false)
            && parse_small_digits(iter, &mut self.month, 2, false)
            && parse_small_digits(iter, &mut self.day, 2, false)
    }

    pub fn parse_ical_time(&mut self, iter: &mut Peekable<Iter<'_, u8>>) -> bool {
        if parse_small_digits(iter, &mut self.hour, 2, false)
            && parse_small_digits(iter, &mut self.minute, 2, false)
            && parse_small_digits(iter, &mut self.second, 2, false)
        {
            self.parse_zone(iter);
            true
        } else {
            false
        }
    }
}

struct ICalendarDateTime(PartialDateTime);

impl IanaParse for ICalendarDateTime {
    fn parse(value: &[u8]) -> Option<Self> {
        let mut dt = PartialDateTime::default();
        if dt.parse_timestamp(&mut value.iter().peekable(), true) {
            Some(ICalendarDateTime(dt))
        } else {
            None
        }
    }
}

impl IanaParse for ICalendarPeriod {
    fn parse(value: &[u8]) -> Option<Self> {
        let mut iter = value.iter().peekable();
        let mut start = PartialDateTime::default();
        if start.parse_timestamp(&mut iter, true) {
            if let Some(duration) = ICalendarDuration::try_parse(&mut iter) {
                Some(ICalendarPeriod::Duration { start, duration })
            } else {
                let mut end = PartialDateTime::default();
                if end.parse_timestamp(&mut iter, true) {
                    Some(ICalendarPeriod::Range { start, end })
                } else {
                    None
                }
            }
        } else {
            None
        }
    }
}

impl IanaParse for ICalendarMonth {
    fn parse(value: &[u8]) -> Option<Self> {
        let mut num: i8 = 0;
        let mut is_leap = false;

        for (pos, ch) in value.iter().enumerate() {
            match ch {
                b'L' | b'l' if pos > 0 => {
                    is_leap = true;
                }
                b'0'..=b'9' => {
                    num = num.saturating_mul(10).saturating_add((*ch - b'0') as i8);
                }
                _ => {
                    if !ch.is_ascii_whitespace() {
                        return None;
                    }
                }
            }
        }

        Some(ICalendarMonth(if is_leap { -num } else { num }))
    }
}

impl ICalendarMonth {
    pub fn new(month: u8, is_leap: bool) -> Self {
        ICalendarMonth(if is_leap { -(month as i8) } else { month as i8 })
    }

    pub fn is_leap(&self) -> bool {
        self.0 < 0
    }

    pub fn month(&self) -> u8 {
        self.0.unsigned_abs()
    }
}

impl From<ICalendarMonth> for u8 {
    fn from(value: ICalendarMonth) -> Self {
        value.month()
    }
}

impl ICalendarDuration {
    fn try_parse(iter: &mut Peekable<Iter<'_, u8>>) -> Option<Self> {
        let mut dur = ICalendarDuration::default();
        loop {
            match iter.peek() {
                Some(b'P' | b'p') => {
                    iter.next();
                    break;
                }
                Some(b'+') => {
                    iter.next();
                }
                Some(b'-') if !dur.neg => {
                    iter.next();
                    dur.neg = true;
                }
                Some(b' ' | b'\t') => {
                    iter.next();
                }
                _ => {
                    return None;
                }
            }
        }

        let mut num: u32 = 0;
        for ch in iter {
            match ch {
                b'0'..=b'9' => {
                    num = num.saturating_mul(10).saturating_add((ch - b'0') as u32);
                }
                b'T' | b't' => {}
                b'W' | b'w' => {
                    dur.weeks = num;
                    num = 0;
                }
                b'D' | b'd' => {
                    dur.days = num;
                    num = 0;
                }
                b'H' | b'h' => {
                    dur.hours = num;
                    num = 0;
                }
                b'M' | b'm' => {
                    dur.minutes = num;
                    num = 0;
                }
                b'S' | b's' => {
                    dur.seconds = num;
                    num = 0;
                }
                _ => {
                    if !ch.is_ascii_whitespace() {
                        return None;
                    }
                }
            }
        }

        if !dur.is_empty() { Some(dur) } else { None }
    }
}

impl IanaParse for ICalendarDuration {
    fn parse(value: &[u8]) -> Option<Self> {
        ICalendarDuration::try_parse(&mut value.iter().peekable())
    }
}

impl ICalendarDuration {
    pub fn is_empty(&self) -> bool {
        self.weeks == 0
            && self.days == 0
            && self.hours == 0
            && self.minutes == 0
            && self.seconds == 0
    }
}

impl From<Token<'_>> for Uri {
    fn from(token: Token<'_>) -> Self {
        token
            .into_uri_bytes()
            .map(Uri::Data)
            .unwrap_or_else(Uri::Location)
    }
}

impl ICalendarParameterName {
    pub fn try_parse(input: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(input,
                b"ALTREP" => ICalendarParameterName::Altrep,
                b"CN" => ICalendarParameterName::Cn,
                b"CUTYPE" => ICalendarParameterName::Cutype,
                b"DELEGATED-FROM" => ICalendarParameterName::DelegatedFrom,
                b"DELEGATED-TO" => ICalendarParameterName::DelegatedTo,
                b"DIR" => ICalendarParameterName::Dir,
                b"FMTTYPE" => ICalendarParameterName::Fmttype,
                b"FBTYPE" => ICalendarParameterName::Fbtype,
                b"LANGUAGE" => ICalendarParameterName::Language,
                b"MEMBER" => ICalendarParameterName::Member,
                b"PARTSTAT" => ICalendarParameterName::Partstat,
                b"RANGE" => ICalendarParameterName::Range,
                b"RELATED" => ICalendarParameterName::Related,
                b"RELTYPE" => ICalendarParameterName::Reltype,
                b"ROLE" => ICalendarParameterName::Role,
                b"RSVP" => ICalendarParameterName::Rsvp,
                b"SCHEDULE-AGENT" => ICalendarParameterName::ScheduleAgent,
                b"SCHEDULE-FORCE-SEND" => ICalendarParameterName::ScheduleForceSend,
                b"SCHEDULE-STATUS" => ICalendarParameterName::ScheduleStatus,
                b"SENT-BY" => ICalendarParameterName::SentBy,
                b"TZID" => ICalendarParameterName::Tzid,
                b"VALUE" => ICalendarParameterName::Value,
                b"DISPLAY" => ICalendarParameterName::Display,
                b"EMAIL" => ICalendarParameterName::Email,
                b"FEATURE" => ICalendarParameterName::Feature,
                b"LABEL" => ICalendarParameterName::Label,
                b"SIZE" => ICalendarParameterName::Size,
                b"FILENAME" => ICalendarParameterName::Filename,
                b"MANAGED-ID" => ICalendarParameterName::ManagedId,
                b"ORDER" => ICalendarParameterName::Order,
                b"SCHEMA" => ICalendarParameterName::Schema,
                b"DERIVED" => ICalendarParameterName::Derived,
                b"GAP" => ICalendarParameterName::Gap,
                b"LINKREL" => ICalendarParameterName::Linkrel,
                b"JSPTR" => ICalendarParameterName::Jsptr,
                b"JSID" => ICalendarParameterName::Jsid,
        )
    }

    pub fn as_str(&self) -> &str {
        match self {
            ICalendarParameterName::Altrep => "ALTREP",
            ICalendarParameterName::Cn => "CN",
            ICalendarParameterName::Cutype => "CUTYPE",
            ICalendarParameterName::DelegatedFrom => "DELEGATED-FROM",
            ICalendarParameterName::DelegatedTo => "DELEGATED-TO",
            ICalendarParameterName::Dir => "DIR",
            ICalendarParameterName::Fmttype => "FMTTYPE",
            ICalendarParameterName::Fbtype => "FBTYPE",
            ICalendarParameterName::Language => "LANGUAGE",
            ICalendarParameterName::Member => "MEMBER",
            ICalendarParameterName::Partstat => "PARTSTAT",
            ICalendarParameterName::Range => "RANGE",
            ICalendarParameterName::Related => "RELATED",
            ICalendarParameterName::Reltype => "RELTYPE",
            ICalendarParameterName::Role => "ROLE",
            ICalendarParameterName::Rsvp => "RSVP",
            ICalendarParameterName::ScheduleAgent => "SCHEDULE-AGENT",
            ICalendarParameterName::ScheduleForceSend => "SCHEDULE-FORCE-SEND",
            ICalendarParameterName::ScheduleStatus => "SCHEDULE-STATUS",
            ICalendarParameterName::SentBy => "SENT-BY",
            ICalendarParameterName::Tzid => "TZID",
            ICalendarParameterName::Value => "VALUE",
            ICalendarParameterName::Display => "DISPLAY",
            ICalendarParameterName::Email => "EMAIL",
            ICalendarParameterName::Feature => "FEATURE",
            ICalendarParameterName::Label => "LABEL",
            ICalendarParameterName::Size => "SIZE",
            ICalendarParameterName::Filename => "FILENAME",
            ICalendarParameterName::ManagedId => "MANAGED-ID",
            ICalendarParameterName::Order => "ORDER",
            ICalendarParameterName::Schema => "SCHEMA",
            ICalendarParameterName::Derived => "DERIVED",
            ICalendarParameterName::Gap => "GAP",
            ICalendarParameterName::Linkrel => "LINKREL",
            ICalendarParameterName::Jsptr => "JSPTR",
            ICalendarParameterName::Jsid => "JSID",
            ICalendarParameterName::Other(name) => name.as_str(),
        }
    }

    pub fn parse(input: &str) -> Self {
        Self::try_parse(input.as_bytes())
            .unwrap_or_else(|| ICalendarParameterName::Other(input.to_string()))
    }
}

impl Uri {
    pub fn parse(value: impl Into<String>) -> Self {
        let uri = value.into();
        Data::try_parse(uri.as_bytes())
            .map(Uri::Data)
            .unwrap_or_else(|| Uri::Location(uri))
    }
}
