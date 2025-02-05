use std::{
    iter::{Enumerate, Peekable},
    slice::Iter,
};

use crate::{
    jscontact::{JSContact, Value},
    tokenizer::{StopChar, Token},
};

pub enum ParseError {
    InvalidToken(String),
    Eof,
}

pub enum Entry {
    VCard(JSContact),
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

impl Value {
    pub(crate) fn try_uint(token: Token<'_>) -> Self {
        std::str::from_utf8(token.text.as_ref())
            .ok()
            .and_then(|s| s.parse().ok())
            .map(Value::UInt)
            .unwrap_or_else(|| Value::Text(token.into_string()))
    }
    pub(crate) fn bool(token: Token<'_>) -> Self {
        Value::Boolean(token.text.as_ref().eq_ignore_ascii_case(b"TRUE"))
    }

    pub(crate) fn try_timestamp(token: Token<'_>) -> Self {
        todo!("parse timestamp")
    }

    pub(crate) fn text(token: Token<'_>) -> Self {
        Value::Text(token.into_string())
    }
}
