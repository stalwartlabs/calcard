/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */
#![doc = include_str!("../README.md")]
#![deny(rust_2018_idioms)]
#![forbid(unsafe_code)]
use common::tokenizer::{StopChar, Token};
use icalendar::{ICalendar, ICalendarComponentType};
use std::{
    iter::{Enumerate, Peekable},
    slice::Iter,
};
use vcard::VCard;

pub mod common;
pub mod datecalc;
pub mod icalendar;
pub mod vcard;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    VCard(VCard),
    ICalendar(ICalendar),
    InvalidLine(String),
    UnexpectedComponentEnd {
        expected: ICalendarComponentType,
        found: ICalendarComponentType,
    },
    UnterminatedComponent(ICalendarComponentType),
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
    pub(crate) stop_dot: bool,
    pub(crate) unfold_qp: bool,
    pub(crate) unquote: bool,
    pub(crate) token_buf: Vec<Token<'x>>,
}

impl<'x> Parser<'x> {
    pub fn new(input: &'x str) -> Self {
        let input = input.as_bytes();
        Self {
            input,
            iter: input.iter().enumerate().peekable(),
            strict: false,
            stop_colon: true,
            stop_semicolon: true,
            stop_comma: true,
            stop_equal: true,
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
                if (token.text.eq_ignore_ascii_case(b"BEGIN")
                    || token.text.eq_ignore_ascii_case("\u{feff}BEGIN".as_bytes()))
                    && token.stop_char == StopChar::Colon
                {
                    if let Some(token) = self.token() {
                        if token.stop_char == StopChar::Lf {
                            hashify::fnc_map_ignore_case!(token.text.as_ref(),
                                b"VCARD" => { return self.vcard(); },
                                b"VCALENDAR" => { return self.icalendar(ICalendarComponentType::VCalendar); },
                                b"VEVENT" => { return self.icalendar(ICalendarComponentType::VEvent); },
                                b"VTODO" => { return self.icalendar(ICalendarComponentType::VTodo); },
                                b"VJOURNAL" => { return self.icalendar(ICalendarComponentType::VJournal); },
                                b"VFREEBUSY" => { return self.icalendar(ICalendarComponentType::VFreebusy); },
                                b"VTIMEZONE" => { return self.icalendar(ICalendarComponentType::VTimezone); },
                                b"VALARM" => { return self.icalendar(ICalendarComponentType::VAlarm); },
                                b"STANDARD" => { return self.icalendar(ICalendarComponentType::Standard); },
                                b"DAYLIGHT" => { return self.icalendar(ICalendarComponentType::Daylight); },
                                b"VAVAILABILITY" => { return self.icalendar(ICalendarComponentType::VAvailability); },
                                b"AVAILABLE" => { return self.icalendar(ICalendarComponentType::Available); },
                                b"PARTICIPANT" => { return self.icalendar(ICalendarComponentType::Participant); },
                                b"VLOCATION" => { return self.icalendar(ICalendarComponentType::VLocation); },
                                b"VRESOURCE" => { return self.icalendar(ICalendarComponentType::VResource); },
                                _ => {
                                    return self.icalendar(ICalendarComponentType::Other(token.into_string()));
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
                    std::str::from_utf8(
                        self.input.get(token_start..=token_end).unwrap_or_default(),
                    )
                    .unwrap_or_default()
                    .to_string(),
                );
            } else {
                return Entry::Eof;
            }
        }
    }
}
