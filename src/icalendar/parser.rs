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
        let mut next_component_id: u32 = 1;
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
                                IanaType::<Boolean, String>::from(token),
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
                                IanaType::<Integer, String>::from(token),
                            ));
                        }
                        ICalendarParameterName::Gap => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                IanaType::<ICalendarDuration, String>::from(token),
                            ));
                        }
                        ICalendarParameterName::Cutype => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                IanaType::<ICalendarUserTypes, String>::from(token),
                            ));
                        }
                        ICalendarParameterName::Fbtype => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                IanaType::<ICalendarFreeBusyType, String>::from(token),
                            ));
                        }
                        ICalendarParameterName::Partstat => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                IanaType::<ICalendarParticipationStatus, String>::from(token),
                            ));
                        }
                        ICalendarParameterName::Related => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                IanaType::<ICalendarRelated, String>::from(token),
                            ));
                        }
                        ICalendarParameterName::Reltype => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                IanaType::<ICalendarRelationshipType, String>::from(token),
                            ));
                        }
                        ICalendarParameterName::Role => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                IanaType::<ICalendarParticipationRole, String>::from(token),
                            ));
                        }
                        ICalendarParameterName::ScheduleAgent => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                IanaType::<ICalendarScheduleAgentValue, String>::from(token),
                            ));
                        }
                        ICalendarParameterName::ScheduleForceSend => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                IanaType::<ICalendarScheduleForceSendValue, String>::from(token),
                            ));
                        }
                        ICalendarParameterName::Value => {
                            params.data_type = Some(token.into());
                        }
                        ICalendarParameterName::Display => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                IanaType::<ICalendarDisplayType, String>::from(token),
                            ));
                        }
                        ICalendarParameterName::Feature => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                IanaType::<ICalendarFeatureType, String>::from(token),
                            ));
                        }
                        ICalendarParameterName::Linkrel => {
                            param_values.push(ICalendarParameter::new(
                                param_name.clone(),
                                IanaType::<LinkRelation, String>::from(token),
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

    pub(crate) fn rrule(&mut self) -> Result<ICalendarRecurrenceRule, String> {
        self.expect_rrule_value();

        let mut last_stop_char = StopChar::Equal;
        let mut is_valid = true;
        let mut has_freq = false;
        let mut rrule = ICalendarRecurrenceRule::default();

        let mut token_start = usize::MAX;

        while let Some(mut token) = self.token_until_lf(&mut last_stop_char) {
            if token_start == usize::MAX && !token.text.is_empty() {
                token_start = token.start;
            }
            if !is_valid {
                continue;
            }
            if token.text.is_empty() && token.stop_char != StopChar::Equal {
                continue;
            }
            if token.stop_char != StopChar::Equal {
                if !self.strict {
                    // Ignore unknown tokens
                    while let Some(token_) = self.token_until_lf(&mut last_stop_char) {
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
                    while let Some(value) = self.parse_value_until_lf(StopChar::Semicolon, &mut last_stop_char) {                        if let Some(value)= value {
                            rrule.freq = value;
                            has_freq = true;
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"UNTIL" => {
                    while let Some(value) = self.parse_value_until_lf::<ICalendarDateOrTime>(StopChar::Semicolon, &mut last_stop_char) {                        if let Some(value)= value {
                            rrule.until = Some(value.0);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"COUNT" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {                        if let Some(value)= value {
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
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {                        if let Some(value)= value {
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
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {                        if let Some(value)= value {
                            rrule.bysecond.push(value.0.unsigned_abs() as u8);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYMINUTE" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {                        if let Some(value)= value {
                            rrule.byminute.push(value.0.unsigned_abs() as u8);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYHOUR" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {                        if let Some(value)= value {
                            rrule.byhour.push(value.0.unsigned_abs() as u8);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYDAY" => {
                    while let Some(value) = self.parse_value_until_lf::<ICalendarDay>(StopChar::Semicolon, &mut last_stop_char) {                        if let Some(value)= value {
                            rrule.byday.push(value);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYMONTHDAY" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {                        if let Some(value)= value {
                            rrule.bymonthday.push(value.0 as i8);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYYEARDAY" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {                        if let Some(value)= value {
                            rrule.byyearday.push(value.0 as i16);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYWEEKNO" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {                        if let Some(value)= value {
                            rrule.byweekno.push(value.0 as i8);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYMONTH" => {
                    while let Some(value) = self.parse_value_until_lf::<ICalendarMonth>(StopChar::Semicolon, &mut last_stop_char) {                        if let Some(value)= value {
                            rrule.bymonth.push(value);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYSETPOS" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {                        if let Some(value)= value {
                            rrule.bysetpos.push(value.0 as i32);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"WKST" => {
                    while let Some(value) = self.parse_value_until_lf::<ICalendarWeekday>(StopChar::Semicolon, &mut last_stop_char) {                        if let Some(value)= value {
                            rrule.wkst = Some(value);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"RSCALE" => {
                    while let Some(value) = self.parse_value_until_lf::<CalendarScale>(StopChar::Semicolon, &mut last_stop_char) {                        if let Some(value)= value {
                            rrule.rscale = Some(value);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"SKIP" => {
                    while let Some(value) = self.parse_value_until_lf::<ICalendarSkip>(StopChar::Semicolon, &mut last_stop_char) {                        if let Some(value)= value {
                            rrule.skip = Some(value);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                _ => {
                    if !self.strict {
                        // Ignore unknown tokens
                        while let Some(token) = self.token_until_lf(&mut last_stop_char) {                            if token.stop_char == StopChar::Semicolon{
                                break;
                            }
                        }
                    } else {
                        is_valid = false;
                    }
                }
            );
        }

        if has_freq && is_valid {
            Ok(rrule)
        } else if token_start != usize::MAX && self.last_token_end >= token_start {
            Err(self
                .input
                .get(token_start..=self.last_token_end)
                .map(|slice| {
                    let mut buf = Vec::with_capacity(slice.len());
                    let mut iter = slice.iter().peekable();

                    while let Some(ch) = iter.next() {
                        match ch {
                            b'\r' => {}
                            b'\n' => {
                                iter.next_if(|ch| matches!(ch, b' ' | b'\t'));
                            }
                            _ => buf.push(*ch),
                        }
                    }

                    String::from_utf8(buf)
                        .unwrap_or_else(|err| String::from_utf8_lossy(err.as_bytes()).into_owned())
                })
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

struct ICalendarDateOrTime(PartialDateTime);

impl IanaParse for ICalendarDateOrTime {
    fn parse(value: &[u8]) -> Option<Self> {
        let mut dt = PartialDateTime::default();
        if dt.parse_timestamp(&mut value.iter().peekable(), false) {
            Some(ICalendarDateOrTime(dt))
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
        let mut saw_component = false;
        for ch in iter {
            match ch {
                b'0'..=b'9' => {
                    num = num.saturating_mul(10).saturating_add((ch - b'0') as u32);
                }
                b'T' | b't' => {}
                b'W' | b'w' => {
                    dur.weeks = num;
                    num = 0;
                    saw_component = true;
                }
                b'D' | b'd' => {
                    dur.days = num;
                    num = 0;
                    saw_component = true;
                }
                b'H' | b'h' => {
                    dur.hours = num;
                    num = 0;
                    saw_component = true;
                }
                b'M' | b'm' => {
                    dur.minutes = num;
                    num = 0;
                    saw_component = true;
                }
                b'S' | b's' => {
                    dur.seconds = num;
                    num = 0;
                    saw_component = true;
                }
                _ => {
                    if !ch.is_ascii_whitespace() {
                        return None;
                    }
                }
            }
        }

        if saw_component { Some(dur) } else { None }
    }
}

impl IanaParse for ICalendarDuration {
    fn parse(value: &[u8]) -> Option<Self> {
        ICalendarDuration::try_parse(&mut value.iter().peekable())
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

impl IanaParse for ICalendarDay {
    fn parse(value: &[u8]) -> Option<Self> {
        let mut iter = value.iter().enumerate();
        let mut is_negative = false;
        let mut has_ordwk = false;
        let mut ordwk: i16 = 0;

        loop {
            let (pos, ch) = iter.next()?;

            match ch {
                b'0'..=b'9' => {
                    ordwk = ordwk.saturating_mul(10).saturating_add((ch - b'0') as i16);
                    has_ordwk = true;
                }
                b'-' if pos == 0 => {
                    is_negative = true;
                }
                b'+' if pos == 0 => {}
                b'A'..=b'Z' | b'a'..=b'z' => {
                    return ICalendarWeekday::parse(value.get(pos..).unwrap_or_default()).map(
                        |weekday| ICalendarDay {
                            ordwk: has_ordwk.then_some(if is_negative { -ordwk } else { ordwk }),
                            weekday,
                        },
                    );
                }
                _ => return None,
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_ical() {
        // Read all .ics files in the test directory
        for entry in std::fs::read_dir("resources/ical").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "ics") {
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
                let mut output = std::fs::File::create(path.with_extension("ics.out")).unwrap();
                //let mut output_debug =
                //    std::fs::File::create(path.with_extension("ics.debug")).unwrap();
                let file_name = path.as_path().to_str().unwrap();

                loop {
                    match parser.entry() {
                        Entry::ICalendar(mut ical) => {
                            for item in &mut ical.components {
                                for item in &mut item.entries {
                                    if item.name == ICalendarProperty::Version {
                                        item.values = vec![ICalendarValue::Text("2.0".into())];
                                    }
                                }
                            }
                            let ical_text = ical.to_string();
                            crate::common::writer::assert_fold_width(&ical_text, file_name);
                            writeln!(output, "{}", ical_text).unwrap();
                            //writeln!(output_debug, "{:#?}", ical).unwrap();

                            // Roundtrip parsing
                            let mut parser = Parser::new(&ical_text);
                            match parser.entry() {
                                Entry::ICalendar(ical_) => {
                                    /*ical.components.iter_mut().for_each(|component| {
                                        component.entries.retain(|entry| {
                                            !matches!(entry.name, ICalendarProperty::Version)
                                        });
                                    });
                                    ical_.components.iter_mut().for_each(|component| {
                                        component.entries.retain(|entry| {
                                            !matches!(entry.name, ICalendarProperty::Version)
                                        });
                                    });*/

                                    compare_components(&ical, &ical_, file_name);
                                }
                                other => panic!("Expected iCal, got {other:?} for {file_name}"),
                            }

                            // Rkyv archiving tests
                            #[cfg(feature = "rkyv")]
                            {
                                let ical_bytes =
                                    rkyv::to_bytes::<rkyv::rancor::Error>(&ical).unwrap();
                                let ical_unarchived = rkyv::access::<
                                    crate::icalendar::ArchivedICalendar,
                                    rkyv::rancor::Error,
                                >(&ical_bytes)
                                .unwrap();
                                assert_eq!(ical_text, ical_unarchived.to_string());
                            }
                        }
                        Entry::InvalidLine(text) => {
                            println!("Invalid line in {file_name}: {text}");
                        }
                        Entry::Eof => break,
                        other => {
                            panic!("Expected iCal, got {other:?} for {file_name}");
                        }
                    }
                }
            }
        }
    }

    fn compare_components(a: &ICalendar, b: &ICalendar, file_name: &str) {
        assert_eq!(
            a.components.len(),
            b.components.len(),
            "failed for {file_name}"
        );

        for (a, b) in a.components.iter().zip(b.components.iter()) {
            assert_eq!(a, b, "failed for {file_name}");
        }
    }

    #[test]
    fn test_parse_rrule() {
        for (rule, expected) in [
            (
                "FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Monthly,
                    byday: vec![
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Monday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Tuesday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Wednesday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Thursday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Friday,
                        },
                    ],
                    bysetpos: vec![-1],
                    ..Default::default()
                },
            ),
            (
                "FREQ=YEARLY;BYMONTH=1,2",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Yearly,
                    bymonth: vec![ICalendarMonth(1), ICalendarMonth(2)],
                    ..Default::default()
                },
            ),
            (
                "FREQ=YEARLY;INTERVAL=2;BYMONTH=1;BYDAY=SU;BYHOUR=8,9;BYMINUTE=30",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Yearly,
                    interval: Some(2),
                    byminute: vec![30],
                    byhour: vec![8, 9],
                    byday: vec![ICalendarDay {
                        ordwk: None,
                        weekday: ICalendarWeekday::Sunday,
                    }],
                    bymonth: vec![ICalendarMonth(1)],
                    ..Default::default()
                },
            ),
            (
                "FREQ=DAILY;COUNT=10;INTERVAL=2",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Daily,
                    count: Some(10),
                    interval: Some(2),
                    ..Default::default()
                },
            ),
            (
                "FREQ=YEARLY",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Yearly,
                    ..Default::default()
                },
            ),
            (
                "FREQ=YEARLY;BYMONTH=4;BYDAY=-1SU;UNTIL=19730429T070000Z",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Yearly,
                    until: Some(PartialDateTime {
                        year: Some(1973),
                        month: Some(4),
                        day: Some(29),
                        hour: Some(7),
                        minute: Some(0),
                        second: Some(0),
                        tz_hour: Some(0),
                        tz_minute: Some(0),
                        tz_minus: false,
                    }),
                    byday: vec![ICalendarDay {
                        ordwk: Some(-1),
                        weekday: ICalendarWeekday::Sunday,
                    }],
                    bymonth: vec![ICalendarMonth(4)],
                    ..Default::default()
                },
            ),
            (
                "FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU;UNTIL=20061029T060000",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Yearly,
                    until: Some(PartialDateTime {
                        year: Some(2006),
                        month: Some(10),
                        day: Some(29),
                        hour: Some(6),
                        minute: Some(0),
                        second: Some(0),
                        tz_hour: None,
                        tz_minute: None,
                        tz_minus: false,
                    }),
                    byday: vec![ICalendarDay {
                        ordwk: Some(-1),
                        weekday: ICalendarWeekday::Sunday,
                    }],
                    bymonth: vec![ICalendarMonth(10)],
                    ..Default::default()
                },
            ),
            (
                "FREQ=YEARLY;BYMONTH=3;BYDAY=2SU",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Yearly,
                    byday: vec![ICalendarDay {
                        ordwk: Some(2),
                        weekday: ICalendarWeekday::Sunday,
                    }],
                    bymonth: vec![ICalendarMonth(3)],
                    ..Default::default()
                },
            ),
            (
                "FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Yearly,
                    byday: vec![ICalendarDay {
                        ordwk: Some(-1),
                        weekday: ICalendarWeekday::Sunday,
                    }],
                    bymonth: vec![ICalendarMonth(10)],
                    ..Default::default()
                },
            ),
            (
                "FREQ=DAILY;COUNT=10",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Daily,
                    count: Some(10),
                    ..Default::default()
                },
            ),
            (
                "FREQ=DAILY;INTERVAL=10;COUNT=5",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Daily,
                    count: Some(5),
                    interval: Some(10),
                    ..Default::default()
                },
            ),
            (
                "FREQ=YEARLY;UNTIL=20000131T140000Z;BYMONTH=1;BYDAY=SU,MO,TU,WE,TH,FR,SA",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Yearly,
                    until: Some(PartialDateTime {
                        year: Some(2000),
                        month: Some(1),
                        day: Some(31),
                        hour: Some(14),
                        minute: Some(0),
                        second: Some(0),
                        tz_hour: Some(0),
                        tz_minute: Some(0),
                        tz_minus: false,
                    }),
                    byday: vec![
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Sunday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Monday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Tuesday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Wednesday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Thursday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Friday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Saturday,
                        },
                    ],
                    bymonth: vec![ICalendarMonth(1)],
                    ..Default::default()
                },
            ),
            (
                "FREQ=DAILY;UNTIL=20000131T140000Z;BYMONTH=1",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Daily,
                    until: Some(PartialDateTime {
                        year: Some(2000),
                        month: Some(1),
                        day: Some(31),
                        hour: Some(14),
                        minute: Some(0),
                        second: Some(0),
                        tz_hour: Some(0),
                        tz_minute: Some(0),
                        tz_minus: false,
                    }),
                    bymonth: vec![ICalendarMonth(1)],
                    ..Default::default()
                },
            ),
            (
                "FREQ=WEEKLY;COUNT=10",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Weekly,
                    count: Some(10),
                    ..Default::default()
                },
            ),
            (
                "FREQ=WEEKLY;INTERVAL=2;WKST=SU",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Weekly,
                    interval: Some(2),
                    wkst: Some(ICalendarWeekday::Sunday),
                    ..Default::default()
                },
            ),
            (
                "FREQ=WEEKLY;UNTIL=19971007T000000Z;WKST=SU;BYDAY=TU,TH",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Weekly,
                    until: Some(PartialDateTime {
                        year: Some(1997),
                        month: Some(10),
                        day: Some(7),
                        hour: Some(0),
                        minute: Some(0),
                        second: Some(0),
                        tz_hour: Some(0),
                        tz_minute: Some(0),
                        tz_minus: false,
                    }),
                    byday: vec![
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Tuesday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Thursday,
                        },
                    ],
                    wkst: Some(ICalendarWeekday::Sunday),
                    ..Default::default()
                },
            ),
            (
                "FREQ=WEEKLY;COUNT=10;WKST=SU;BYDAY=TU,TH",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Weekly,
                    count: Some(10),
                    byday: vec![
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Tuesday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Thursday,
                        },
                    ],
                    wkst: Some(ICalendarWeekday::Sunday),
                    ..Default::default()
                },
            ),
            (
                "FREQ=WEEKLY;INTERVAL=2;UNTIL=19971224T000000Z;WKST=SU;BYDAY=MO,WE,FR",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Weekly,
                    until: Some(PartialDateTime {
                        year: Some(1997),
                        month: Some(12),
                        day: Some(24),
                        hour: Some(0),
                        minute: Some(0),
                        second: Some(0),
                        tz_hour: Some(0),
                        tz_minute: Some(0),
                        tz_minus: false,
                    }),
                    interval: Some(2),
                    byday: vec![
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Monday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Wednesday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Friday,
                        },
                    ],
                    wkst: Some(ICalendarWeekday::Sunday),
                    ..Default::default()
                },
            ),
            (
                "FREQ=WEEKLY;INTERVAL=2;COUNT=8;WKST=SU;BYDAY=TU,TH",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Weekly,
                    count: Some(8),
                    interval: Some(2),
                    byday: vec![
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Tuesday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Thursday,
                        },
                    ],
                    wkst: Some(ICalendarWeekday::Sunday),
                    ..Default::default()
                },
            ),
            (
                "FREQ=MONTHLY;COUNT=10;BYDAY=1FR",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Monthly,
                    count: Some(10),
                    byday: vec![ICalendarDay {
                        ordwk: Some(1),
                        weekday: ICalendarWeekday::Friday,
                    }],
                    ..Default::default()
                },
            ),
            (
                "FREQ=MONTHLY;INTERVAL=2;COUNT=10;BYDAY=1SU,-1SU",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Monthly,
                    count: Some(10),
                    interval: Some(2),
                    byday: vec![
                        ICalendarDay {
                            ordwk: Some(1),
                            weekday: ICalendarWeekday::Sunday,
                        },
                        ICalendarDay {
                            ordwk: Some(-1),
                            weekday: ICalendarWeekday::Sunday,
                        },
                    ],
                    ..Default::default()
                },
            ),
            (
                "FREQ=MONTHLY;COUNT=6;BYDAY=-2MO",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Monthly,
                    count: Some(6),
                    byday: vec![ICalendarDay {
                        ordwk: Some(-2),
                        weekday: ICalendarWeekday::Monday,
                    }],
                    ..Default::default()
                },
            ),
            (
                "FREQ=MONTHLY;BYMONTHDAY=-3",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Monthly,
                    bymonthday: vec![-3],
                    ..Default::default()
                },
            ),
            (
                "FREQ=MONTHLY;COUNT=10;BYMONTHDAY=2,15",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Monthly,
                    count: Some(10),
                    bymonthday: vec![2, 15],
                    ..Default::default()
                },
            ),
            (
                "FREQ=MONTHLY;COUNT=10;BYMONTHDAY=1,-1",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Monthly,
                    count: Some(10),
                    bymonthday: vec![1, -1],
                    ..Default::default()
                },
            ),
            (
                "FREQ=MONTHLY;INTERVAL=18;COUNT=10;BYMONTHDAY=10,11,12,13,14,15",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Monthly,
                    count: Some(10),
                    interval: Some(18),
                    bymonthday: vec![10, 11, 12, 13, 14, 15],
                    ..Default::default()
                },
            ),
            (
                "FREQ=MONTHLY;INTERVAL=2;BYDAY=TU",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Monthly,
                    interval: Some(2),
                    byday: vec![ICalendarDay {
                        ordwk: None,
                        weekday: ICalendarWeekday::Tuesday,
                    }],
                    ..Default::default()
                },
            ),
            (
                "FREQ=YEARLY;COUNT=10;BYMONTH=6,7",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Yearly,
                    count: Some(10),
                    bymonth: vec![ICalendarMonth(6), ICalendarMonth(7)],
                    ..Default::default()
                },
            ),
            (
                "FREQ=YEARLY;INTERVAL=2;COUNT=10;BYMONTH=1,2,3",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Yearly,
                    count: Some(10),
                    interval: Some(2),
                    bymonth: vec![ICalendarMonth(1), ICalendarMonth(2), ICalendarMonth(3)],
                    ..Default::default()
                },
            ),
            (
                "FREQ=YEARLY;INTERVAL=3;COUNT=10;BYYEARDAY=1,100,200",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Yearly,
                    count: Some(10),
                    interval: Some(3),
                    byyearday: vec![1, 100, 200],
                    ..Default::default()
                },
            ),
            (
                "FREQ=YEARLY;BYDAY=20MO",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Yearly,
                    byday: vec![ICalendarDay {
                        ordwk: Some(20),
                        weekday: ICalendarWeekday::Monday,
                    }],
                    ..Default::default()
                },
            ),
            (
                "FREQ=YEARLY;BYWEEKNO=20;BYDAY=MO",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Yearly,
                    byday: vec![ICalendarDay {
                        ordwk: None,
                        weekday: ICalendarWeekday::Monday,
                    }],
                    byweekno: vec![20],
                    ..Default::default()
                },
            ),
            (
                "FREQ=YEARLY;BYMONTH=3;BYDAY=TH",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Yearly,
                    byday: vec![ICalendarDay {
                        ordwk: None,
                        weekday: ICalendarWeekday::Thursday,
                    }],
                    bymonth: vec![ICalendarMonth(3)],
                    ..Default::default()
                },
            ),
            (
                "FREQ=YEARLY;BYDAY=TH;BYMONTH=6,7,8",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Yearly,
                    byday: vec![ICalendarDay {
                        ordwk: None,
                        weekday: ICalendarWeekday::Thursday,
                    }],
                    bymonth: vec![ICalendarMonth(6), ICalendarMonth(7), ICalendarMonth(8)],
                    ..Default::default()
                },
            ),
            (
                "FREQ=MONTHLY;BYDAY=FR;BYMONTHDAY=13",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Monthly,
                    byday: vec![ICalendarDay {
                        ordwk: None,
                        weekday: ICalendarWeekday::Friday,
                    }],
                    bymonthday: vec![13],
                    ..Default::default()
                },
            ),
            (
                "FREQ=MONTHLY;BYDAY=SA;BYMONTHDAY=7,8,9,10,11,12,13",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Monthly,
                    byday: vec![ICalendarDay {
                        ordwk: None,
                        weekday: ICalendarWeekday::Saturday,
                    }],
                    bymonthday: vec![7, 8, 9, 10, 11, 12, 13],
                    ..Default::default()
                },
            ),
            (
                "FREQ=YEARLY;INTERVAL=4;BYMONTH=11;BYDAY=TU;BYMONTHDAY=2,3,4,5,6,7,8",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Yearly,
                    interval: Some(4),
                    byday: vec![ICalendarDay {
                        ordwk: None,
                        weekday: ICalendarWeekday::Tuesday,
                    }],
                    bymonthday: vec![2, 3, 4, 5, 6, 7, 8],
                    bymonth: vec![ICalendarMonth(11)],
                    ..Default::default()
                },
            ),
            (
                "FREQ=MONTHLY;COUNT=3;BYDAY=TU,WE,TH;BYSETPOS=3",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Monthly,
                    count: Some(3),
                    byday: vec![
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Tuesday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Wednesday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Thursday,
                        },
                    ],
                    bysetpos: vec![3],
                    ..Default::default()
                },
            ),
            (
                "FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-2",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Monthly,
                    byday: vec![
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Monday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Tuesday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Wednesday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Thursday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Friday,
                        },
                    ],
                    bysetpos: vec![-2],
                    ..Default::default()
                },
            ),
            (
                "FREQ=HOURLY;INTERVAL=3;UNTIL=19970902T170000Z",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Hourly,
                    until: Some(PartialDateTime {
                        year: Some(1997),
                        month: Some(9),
                        day: Some(2),
                        hour: Some(17),
                        minute: Some(0),
                        second: Some(0),
                        tz_hour: Some(0),
                        tz_minute: Some(0),
                        tz_minus: false,
                    }),
                    interval: Some(3),
                    ..Default::default()
                },
            ),
            (
                "FREQ=MINUTELY;INTERVAL=15;COUNT=6",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Minutely,
                    count: Some(6),
                    interval: Some(15),
                    ..Default::default()
                },
            ),
            (
                "FREQ=MINUTELY;INTERVAL=90;COUNT=4",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Minutely,
                    count: Some(4),
                    interval: Some(90),
                    ..Default::default()
                },
            ),
            (
                "FREQ=DAILY;BYHOUR=9,10,11,12,13,14,15,16;BYMINUTE=0,20,40",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Daily,
                    byminute: vec![0, 20, 40],
                    byhour: vec![9, 10, 11, 12, 13, 14, 15, 16],
                    ..Default::default()
                },
            ),
            (
                "FREQ=MINUTELY;INTERVAL=20;BYHOUR=9,10,11,12,13,14,15,16",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Minutely,
                    interval: Some(20),
                    byhour: vec![9, 10, 11, 12, 13, 14, 15, 16],
                    ..Default::default()
                },
            ),
            (
                "FREQ=WEEKLY;INTERVAL=2;COUNT=4;BYDAY=TU,SU;WKST=MO",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Weekly,
                    count: Some(4),
                    interval: Some(2),
                    byday: vec![
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Tuesday,
                        },
                        ICalendarDay {
                            ordwk: None,
                            weekday: ICalendarWeekday::Sunday,
                        },
                    ],
                    wkst: Some(ICalendarWeekday::Monday),
                    ..Default::default()
                },
            ),
            (
                "FREQ=MONTHLY;BYMONTHDAY=15,30;COUNT=5",
                ICalendarRecurrenceRule {
                    freq: ICalendarFrequency::Monthly,
                    count: Some(5),
                    bymonthday: vec![15, 30],
                    ..Default::default()
                },
            ),
        ] {
            assert_eq!(
                Parser::new(rule).strict().rrule().unwrap(),
                expected,
                "failed for {rule}"
            );
        }
    }

    #[test]
    fn test_parse_period() {
        for (rule, expected) in [
            (
                "19970308T160000Z/PT8H30M",
                ICalendarPeriod::Duration {
                    start: PartialDateTime {
                        year: Some(1997),
                        month: Some(3),
                        day: Some(8),
                        hour: Some(16),
                        minute: Some(0),
                        second: Some(0),
                        tz_hour: Some(0),
                        tz_minute: Some(0),
                        tz_minus: false,
                    },
                    duration: ICalendarDuration {
                        neg: false,
                        weeks: 0,
                        days: 0,
                        hours: 8,
                        minutes: 30,
                        seconds: 0,
                    },
                },
            ),
            (
                "19970308T160000/PT3H",
                ICalendarPeriod::Duration {
                    start: PartialDateTime {
                        year: Some(1997),
                        month: Some(3),
                        day: Some(8),
                        hour: Some(16),
                        minute: Some(0),
                        second: Some(0),
                        tz_hour: None,
                        tz_minute: None,
                        tz_minus: false,
                    },
                    duration: ICalendarDuration {
                        neg: false,
                        weeks: 0,
                        days: 0,
                        hours: 3,
                        minutes: 0,
                        seconds: 0,
                    },
                },
            ),
            (
                "19970308T200000Z/PT1H",
                ICalendarPeriod::Duration {
                    start: PartialDateTime {
                        year: Some(1997),
                        month: Some(3),
                        day: Some(8),
                        hour: Some(20),
                        minute: Some(0),
                        second: Some(0),
                        tz_hour: Some(0),
                        tz_minute: Some(0),
                        tz_minus: false,
                    },
                    duration: ICalendarDuration {
                        neg: false,
                        weeks: 0,
                        days: 0,
                        hours: 1,
                        minutes: 0,
                        seconds: 0,
                    },
                },
            ),
            (
                "19970308T230000Z/19970309T000000Z",
                ICalendarPeriod::Range {
                    start: PartialDateTime {
                        year: Some(1997),
                        month: Some(3),
                        day: Some(8),
                        hour: Some(23),
                        minute: Some(0),
                        second: Some(0),
                        tz_hour: Some(0),
                        tz_minute: Some(0),
                        tz_minus: false,
                    },
                    end: PartialDateTime {
                        year: Some(1997),
                        month: Some(3),
                        day: Some(9),
                        hour: Some(0),
                        minute: Some(0),
                        second: Some(0),
                        tz_hour: Some(0),
                        tz_minute: Some(0),
                        tz_minus: false,
                    },
                },
            ),
            (
                "19970101T180000Z/PT5H30M",
                ICalendarPeriod::Duration {
                    start: PartialDateTime {
                        year: Some(1997),
                        month: Some(1),
                        day: Some(1),
                        hour: Some(18),
                        minute: Some(0),
                        second: Some(0),
                        tz_hour: Some(0),
                        tz_minute: Some(0),
                        tz_minus: false,
                    },
                    duration: ICalendarDuration {
                        neg: false,
                        weeks: 0,
                        days: 0,
                        hours: 5,
                        minutes: 30,
                        seconds: 0,
                    },
                },
            ),
            (
                "19971015T050000Z/PT8H30M",
                ICalendarPeriod::Duration {
                    start: PartialDateTime {
                        year: Some(1997),
                        month: Some(10),
                        day: Some(15),
                        hour: Some(5),
                        minute: Some(0),
                        second: Some(0),
                        tz_hour: Some(0),
                        tz_minute: Some(0),
                        tz_minus: false,
                    },
                    duration: ICalendarDuration {
                        neg: false,
                        weeks: 0,
                        days: 0,
                        hours: 8,
                        minutes: 30,
                        seconds: 0,
                    },
                },
            ),
            (
                "19980314T233000Z/19980315T003000Z",
                ICalendarPeriod::Range {
                    start: PartialDateTime {
                        year: Some(1998),
                        month: Some(3),
                        day: Some(14),
                        hour: Some(23),
                        minute: Some(30),
                        second: Some(0),
                        tz_hour: Some(0),
                        tz_minute: Some(0),
                        tz_minus: false,
                    },
                    end: PartialDateTime {
                        year: Some(1998),
                        month: Some(3),
                        day: Some(15),
                        hour: Some(0),
                        minute: Some(30),
                        second: Some(0),
                        tz_hour: Some(0),
                        tz_minute: Some(0),
                        tz_minus: false,
                    },
                },
            ),
        ] {
            let mut parser = Parser::new(rule);
            let token = parser.token().unwrap();

            assert_eq!(
                ICalendarPeriod::parse(token.text.as_ref()).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn test_parse_duration() {
        for (rule, expected) in [
            (
                "P1W3DT4H5M6S",
                ICalendarDuration {
                    neg: false,
                    weeks: 1,
                    days: 3,
                    hours: 4,
                    minutes: 5,
                    seconds: 6,
                },
            ),
            (
                "P15DT5H0M20S",
                ICalendarDuration {
                    neg: false,
                    weeks: 0,
                    days: 15,
                    hours: 5,
                    minutes: 0,
                    seconds: 20,
                },
            ),
            (
                "P7W",
                ICalendarDuration {
                    neg: false,
                    weeks: 7,
                    days: 0,
                    hours: 0,
                    minutes: 0,
                    seconds: 0,
                },
            ),
            (
                "-P7W",
                ICalendarDuration {
                    neg: true,
                    weeks: 7,
                    days: 0,
                    hours: 0,
                    minutes: 0,
                    seconds: 0,
                },
            ),
            (
                "PT0S",
                ICalendarDuration {
                    neg: false,
                    weeks: 0,
                    days: 0,
                    hours: 0,
                    minutes: 0,
                    seconds: 0,
                },
            ),
        ] {
            let mut parser = Parser::new(rule);
            let token = parser.token().unwrap();

            assert_eq!(
                ICalendarDuration::parse(token.text.as_ref()).expect(rule),
                expected,
                "Failed to parse: {rule}",
            );
        }

        assert!(ICalendarDuration::parse(b"P").is_none());

        // Duration zero
        let zero = ICalendarDuration::default();
        assert_eq!(zero.to_string(), "PT0S");

        let reparsed = ICalendarDuration::parse(zero.to_string().as_bytes()).unwrap();
        assert_eq!(reparsed, zero);
    }
}
