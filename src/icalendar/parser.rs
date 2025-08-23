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
        parser::{Integer, parse_digits, parse_small_digits},
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
    data_type: Option<ICalendarValueType>,
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
            let name = match ICalendarProperty::try_from(name.as_ref()) {
                Ok(ICalendarProperty::Begin) => {
                    if params.stop_char == StopChar::Colon {
                        self.expect_single_value();
                        if let Some(token) = self.token() {
                            let component_type =
                                ICalendarComponentType::try_from(token.text.as_ref())
                                    .unwrap_or_else(|_| {
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
                Ok(ICalendarProperty::End) => {
                    if params.stop_char == StopChar::Colon {
                        self.expect_single_value();
                        if let Some(token) = self.token() {
                            let component_type =
                                ICalendarComponentType::try_from(token.text.as_ref())
                                    .unwrap_or_else(|_| {
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
                Ok(name) => name,
                Err(_) => {
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
                    (Some(ICalendarValueType::Recur), _)
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
                                let value = match params.data_type.as_ref().unwrap_or(default_type)
                                {
                                    ICalendarValueType::Date => token
                                        .into_ical_date()
                                        .map(|data| ICalendarValue::PartialDateTime(Box::new(data)))
                                        .unwrap_or_else(ICalendarValue::Text),
                                    ICalendarValueType::DateTime => {
                                        match token.into_timestamp(false) {
                                            Ok(timestamp) => {
                                                if !timestamp.has_time() {
                                                    params.data_type =
                                                        Some(ICalendarValueType::Date);
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
                                    ICalendarValueType::Other(_) => {
                                        ICalendarValue::Text(token.into_string())
                                    }
                                    ICalendarValueType::Duration => {
                                        if let Ok(duration) =
                                            ICalendarDuration::try_from(token.text.as_ref())
                                        {
                                            ICalendarValue::Duration(duration)
                                        } else {
                                            ICalendarValue::Text(token.into_string())
                                        }
                                    }
                                    ICalendarValueType::Period => {
                                        if let Ok(period) =
                                            ICalendarPeriod::try_from(token.text.as_ref())
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
                                entry.values.push(ICalendarValue::CalendarScale(
                                    CalendarScale::from(token),
                                ));
                            }
                            ValueType::Method => {
                                entry
                                    .values
                                    .push(ICalendarValue::Method(ICalendarMethod::from(token)));
                            }
                            ValueType::Classification => {
                                entry.values.push(ICalendarValue::Classification(
                                    ICalendarClassification::from(token),
                                ));
                            }
                            ValueType::Status => {
                                entry
                                    .values
                                    .push(ICalendarValue::Status(ICalendarStatus::from(token)));
                            }
                            ValueType::Transparency => {
                                entry.values.push(ICalendarValue::Transparency(
                                    ICalendarTransparency::from(token),
                                ));
                            }
                            ValueType::Action => {
                                entry
                                    .values
                                    .push(ICalendarValue::Action(ICalendarAction::from(token)));
                            }
                            ValueType::BusyType => {
                                entry.values.push(ICalendarValue::BusyType(
                                    ICalendarFreeBusyType::from(token),
                                ));
                            }
                            ValueType::ParticipantType => {
                                entry.values.push(ICalendarValue::ParticipantType(
                                    ICalendarParticipantType::from(token),
                                ));
                            }
                            ValueType::ResourceType => {
                                entry.values.push(ICalendarValue::ResourceType(
                                    ICalendarResourceType::from(token),
                                ));
                            }
                            ValueType::Proximity => {
                                entry.values.push(ICalendarValue::Proximity(
                                    ICalendarProximityValue::from(token),
                                ));
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
                    entry.params.push(ICalendarParameter::Value(data_type));
                }

                ical.entries.push(entry);
            }
        }

        if ical_stack.is_empty() || !self.strict {
            Entry::ICalendar(ICalendar {
                components: ical_components,
            })
        } else {
            Entry::UnterminatedComponent(ical.component_type.clone().into_str())
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

            hashify::fnc_map_ignore_case!(param_name.as_ref(),
                b"ALTREP" => {
                    if let Some(value) = self.buf_parse_one() {
                        param_values.push(ICalendarParameter::Altrep(value));
                    }
                },
                b"CN" => {
                    param_values.push(ICalendarParameter::Cn(self.buf_to_string()));
                },
                b"CUTYPE" => {
                    if let Some(value) = self.buf_parse_one() {
                        param_values.push(ICalendarParameter::Cutype(value));
                    }
                },
                b"DELEGATED-FROM" => {
                    param_values.push(ICalendarParameter::DelegatedFrom(self.buf_parse_many()));
                },
                b"DELEGATED-TO" => {
                    param_values.push(ICalendarParameter::DelegatedTo(self.buf_parse_many()));
                },
                b"DIR" => {
                    if let Some(value) = self.buf_parse_one() {
                        param_values.push(ICalendarParameter::Dir(value));
                    }
                },
                b"FMTTYPE" => {
                    param_values.push(ICalendarParameter::Fmttype(self.buf_to_string()));
                },
                b"FBTYPE" => {
                    if let Some(value) = self.buf_parse_one() {
                        param_values.push(ICalendarParameter::Fbtype(value));
                    }
                },
                b"LANGUAGE" => {
                    param_values.push(ICalendarParameter::Language(self.buf_to_string()));
                },
                b"MEMBER" => {
                    param_values.push(ICalendarParameter::Member(self.buf_parse_many()));
                },
                b"PARTSTAT" => {
                    if let Some(value) = self.buf_parse_one() {
                        param_values.push(ICalendarParameter::Partstat(value));
                    }
                },
                b"RANGE" => {
                    if self.token_buf.first().is_some_and(|token| {
                        token.text.as_ref().eq_ignore_ascii_case(b"THISANDFUTURE")
                    }) {
                        param_values.push(ICalendarParameter::Range);
                    }
                    self.token_buf.clear();
                },
                b"RELATED" => {
                    if let Some(gap) = self.buf_try_parse_one() {
                        param_values.push(ICalendarParameter::Related(gap));
                    }
                },
                b"RELTYPE" => {
                    if let Some(value) = self.buf_parse_one() {
                        param_values.push(ICalendarParameter::Reltype(value));
                    }
                },
                b"ROLE" => {
                    if let Some(value) = self.buf_parse_one() {
                        param_values.push(ICalendarParameter::Role(value));
                    }
                },
                b"RSVP" => {
                    param_values.push(ICalendarParameter::Rsvp(self.buf_to_bool()));
                },
                b"SCHEDULE-AGENT" => {
                    if let Some(value) = self.buf_parse_one() {
                        param_values.push(ICalendarParameter::ScheduleAgent(value));
                    }
                },
                b"SCHEDULE-FORCE-SEND" => {
                    if let Some(value) = self.buf_parse_one() {
                        param_values.push(ICalendarParameter::ScheduleForceSend(value));
                    }
                },
                b"SCHEDULE-STATUS" => {
                    param_values.push(ICalendarParameter::ScheduleStatus(self.buf_to_string()));
                },
                b"SENT-BY" => {
                    if let Some(value) = self.buf_parse_one() {
                        param_values.push(ICalendarParameter::SentBy(value));
                    }
                },
                b"TZID" => {
                    param_values.push(ICalendarParameter::Tzid(self.buf_to_string()));
                },
                b"VALUE" => {
                    if let Some(value) = self.buf_parse_one::<ICalendarValueType>() {
                        params.data_type = value.into();
                    }
                },
                b"DISPLAY" => {
                    param_values.push(ICalendarParameter::Display(self.buf_parse_many()));
                },
                b"EMAIL" => {
                    param_values.push(ICalendarParameter::Email(self.buf_to_string()));
                },
                b"FEATURE" => {
                    param_values.push(ICalendarParameter::Feature(self.buf_parse_many()));
                },
                b"LABEL" => {
                    param_values.push(ICalendarParameter::Label(self.buf_to_string()));
                },
                b"SIZE" => {
                    param_values.push(ICalendarParameter::Size(self.buf_to_other().unwrap_or_default()));
                },
                b"FILENAME" => {
                    param_values.push(ICalendarParameter::Filename(self.buf_to_string()));
                },
                b"MANAGED-ID" => {
                    param_values.push(ICalendarParameter::ManagedId(self.buf_to_string()));
                },
                b"ORDER" => {
                    param_values.push(ICalendarParameter::Order(self.buf_to_other().unwrap_or_default()));
                },
                b"SCHEMA" => {
                    if let Some(value) = self.buf_parse_one() {
                        param_values.push(ICalendarParameter::Schema(value));
                    }
                },
                b"DERIVED" => {
                    param_values.push(ICalendarParameter::Derived(self.buf_to_bool()));
                },
                b"GAP" => {
                    if let Some(gap) = self.buf_try_parse_one() {
                        param_values.push(ICalendarParameter::Gap(gap));
                    }
                },
                b"LINKREL" => {
                    if let Some(value) = self.buf_parse_one() {
                        param_values.push(ICalendarParameter::Linkrel(value));
                    }
                },
                b"JSPTR" => {
                     param_values.push(ICalendarParameter::Jsptr(self.buf_to_string()));
                },
                b"JSID" => {
                     param_values.push(ICalendarParameter::Jsid(self.buf_to_string()));
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
                    if !param_name.is_empty() {
                        if params.encoding.is_none() && param_name.eq_ignore_ascii_case(b"base64") {
                            params.encoding = Some(Encoding::Base64);
                        } else {
                            param_values.push(ICalendarParameter::Other(
                                [Token::new(param_name).into_string()]
                                    .into_iter()
                                    .chain(self.token_buf.drain(..).map(|token| token.into_string()))
                                    .collect(),
                            ));
                        }
                    }
                }
            );
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
                        if let Ok(value) = value {
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
                        if let Ok(value) = value {
                            rrule.until = Some(value.0);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"COUNT" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Ok(value) = value {
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
                        if let Ok(value) = value {
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
                        if let Ok(value) = value {
                            rrule.bysecond.push(value.0.unsigned_abs() as u8);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYMINUTE" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Ok(value) = value {
                            rrule.byminute.push(value.0.unsigned_abs() as u8);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYHOUR" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Ok(value) = value {
                            rrule.byhour.push(value.0.unsigned_abs() as u8);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYDAY" => {
                    while let Some(value) = self.parse_value_until_lf::<ICalendarDay>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Ok(value) = value {
                            rrule.byday.push(value);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYMONTHDAY" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Ok(value) = value {
                            rrule.bymonthday.push(value.0 as i8);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYYEARDAY" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Ok(value) = value {
                            rrule.byyearday.push(value.0 as i16);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYWEEKNO" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Ok(value) = value {
                            rrule.byweekno.push(value.0 as i8);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYMONTH" => {
                    while let Some(value) = self.parse_value_until_lf::<ICalendarMonth>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Ok(value) = value {
                            rrule.bymonth.push(value);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"BYSETPOS" => {
                    while let Some(value) = self.parse_value_until_lf::<Integer>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Ok(value) = value {
                            rrule.bysetpos.push(value.0 as i32);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"WKST" => {
                    while let Some(value) = self.parse_value_until_lf::<ICalendarWeekday>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Ok(value) = value {
                            rrule.wkst = Some(value);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"RSCALE" => {
                    while let Some(value) = self.parse_value_until_lf::<CalendarScale>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Ok(value) = value {
                            rrule.rscale = Some(value);
                        } else if !self.strict {
                            is_valid = false;
                        }
                    }
                },
                b"SKIP" => {
                    while let Some(value) = self.parse_value_until_lf::<ICalendarSkip>(StopChar::Semicolon, &mut last_stop_char) {
                        token_end = token.end;
                        if let Ok(value) = value {
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

impl TryFrom<&[u8]> for ICalendarDateTime {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut dt = PartialDateTime::default();
        if dt.parse_timestamp(&mut value.iter().peekable(), true) {
            Ok(ICalendarDateTime(dt))
        } else {
            Err(())
        }
    }
}

impl TryFrom<&[u8]> for ICalendarPeriod {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut iter = value.iter().peekable();
        let mut start = PartialDateTime::default();
        if start.parse_timestamp(&mut iter, true) {
            if let Some(duration) = ICalendarDuration::try_parse(&mut iter) {
                Ok(ICalendarPeriod::Duration { start, duration })
            } else {
                let mut end = PartialDateTime::default();
                if end.parse_timestamp(&mut iter, true) {
                    Ok(ICalendarPeriod::Range { start, end })
                } else {
                    Err(())
                }
            }
        } else {
            Err(())
        }
    }
}

impl TryFrom<&[u8]> for ICalendarMonth {
    type Error = ();

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
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
                        return Err(());
                    }
                }
            }
        }

        Ok(ICalendarMonth(if is_leap { -num } else { num }))
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

impl TryFrom<&[u8]> for ICalendarDuration {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if let Some(duration) = ICalendarDuration::try_parse(&mut value.iter().peekable()) {
            Ok(duration)
        } else {
            Err(())
        }
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
    pub fn parse(input: &str) -> Self {
        hashify::tiny_map_ignore_case!(input.as_bytes(),
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
        .unwrap_or_else(|| ICalendarParameterName::Other(input.to_string()))
    }
}
