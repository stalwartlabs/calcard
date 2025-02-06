use std::{
    iter::{Enumerate, Peekable},
    slice::Iter,
    str::FromStr,
};

use crate::{
    tokenizer::{StopChar, Token},
    vcard::{VCard, VCardPartialDateTime},
};

pub enum ParseError {
    InvalidToken(String),
    Eof,
}

pub enum Entry {
    VCard(VCard),
}

pub type Result<T> = std::result::Result<T, ParseError>;

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

    pub fn entry(&mut self) -> Result<Entry> {
        self.expect_iana_token();
        let mut token = self.token().ok_or(ParseError::Eof)?;
        if token.text.eq_ignore_ascii_case(b"BEGIN") && token.stop_char == StopChar::Colon {
            token = self.token().ok_or(ParseError::Eof)?;

            if token.stop_char == StopChar::Lf {
                hashify::fnc_map_ignore_case!(token.text.as_ref(),
                    b"VCARD" => {
                        return self.vcard().map(Entry::VCard);
                    },
                    b"VCALENDAR" => {
                        //self.vcalendar()
                        todo!()
                    }
                    _ => {}
                )
            }
        }

        Err(ParseError::InvalidToken(token.into_string()))
    }
}

impl Token<'_> {
    pub(crate) fn into_uri_bytes(self) -> std::result::Result<Vec<u8>, String> {
        todo!()
    }

    pub(crate) fn into_boolean(self) -> bool {
        todo!()
    }

    pub(crate) fn into_date(self) -> std::result::Result<VCardPartialDateTime, String> {
        todo!()
    }

    pub(crate) fn into_date_and_or_datetime(
        self,
    ) -> std::result::Result<VCardPartialDateTime, String> {
        todo!()
    }

    pub(crate) fn into_date_time(self) -> std::result::Result<VCardPartialDateTime, String> {
        todo!()
    }

    pub(crate) fn into_float(self) -> std::result::Result<f64, String> {
        todo!()
    }

    pub(crate) fn into_integer(self) -> std::result::Result<i64, String> {
        todo!()
    }

    pub(crate) fn into_time(self) -> std::result::Result<VCardPartialDateTime, String> {
        todo!()
    }

    pub(crate) fn into_offset(self) -> std::result::Result<i16, String> {
        todo!()
    }

    pub(crate) fn into_timestamp(self) -> std::result::Result<i64, String> {
        todo!()
    }
}

#[derive(Default)]
pub(crate) struct Timestamp(pub i64);

impl FromStr for Timestamp {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        todo!()
    }
}
