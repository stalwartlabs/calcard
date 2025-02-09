use std::{borrow::Cow, str::FromStr};

use mail_parser::decoders::{base64::base64_decode, hex::decode_hex};

use crate::Parser;

use super::{tokenizer::Token, Data};

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
}
