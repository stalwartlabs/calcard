use std::{borrow::Cow, str::FromStr};

use mail_parser::decoders::{
    base64::base64_decode, charsets::map::charset_decoder,
    quoted_printable::quoted_printable_decode,
};

use crate::{
    parser::{Parser, Timestamp},
    tokenizer::{StopChar, Token},
    vcard::VCardProperty,
};

use super::{
    VCard, VCardBinary, VCardEntry, VCardParameter, VCardType, VCardValue, VCardValueType,
    ValueSeparator, ValueType,
};

struct Params {
    params: Vec<VCardParameter>,
    stop_char: StopChar,
    data_types: Vec<VCardValueType>,
    charset: Option<String>,
    encoding: Option<Encoding>,
    group_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Encoding {
    QuotedPrintable,
    Base64,
}

impl<'x> Parser<'x> {
    pub fn vcard(&mut self) -> VCard {
        let mut vcard = VCard::default();
        let mut is_v4 = true;

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
                    self.parameters(&mut params);
                }
                StopChar::Colon => {}
                StopChar::Lf => {
                    // Invalid line
                    continue;
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
                                entry.values.push(VCardValue::Binary(VCardBinary {
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
                            .into_date()
                            .map(VCardValue::PartialDateTime)
                            .unwrap_or_else(VCardValue::Text),
                        VCardValueType::DateAndOrTime if is_v4 => token
                            .into_date_and_or_datetime()
                            .map(VCardValue::PartialDateTime)
                            .unwrap_or_else(VCardValue::Text),
                        VCardValueType::DateTime if is_v4 => token
                            .into_date_time()
                            .map(VCardValue::PartialDateTime)
                            .unwrap_or_else(VCardValue::Text),
                        VCardValueType::Time if is_v4 => token
                            .into_time()
                            .map(VCardValue::PartialDateTime)
                            .unwrap_or_else(VCardValue::Text),
                        VCardValueType::Timestamp if is_v4 => token
                            .into_timestamp()
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
                            .into_datetime_or_legacy()
                            .map(VCardValue::PartialDateTime)
                            .unwrap_or_else(VCardValue::Text),
                        VCardValueType::Timestamp => token
                            .into_timestamp_or_legacy()
                            .map(VCardValue::PartialDateTime)
                            .unwrap_or_else(VCardValue::Text),
                        VCardValueType::UtcOffset => token
                            .into_offset_or_legacy()
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

        vcard
    }

    fn parameters(&mut self, params: &mut Params) {
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

    fn buf_to_string(&mut self) -> String {
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

    fn buf_to_other<T: FromStr>(&mut self) -> Option<T> {
        let result = self.token_buf.first().and_then(|token| {
            std::str::from_utf8(token.text.as_ref())
                .ok()
                .and_then(|s| s.parse().ok())
        });
        self.token_buf.clear();
        result
    }

    fn buf_to_bool(&mut self) -> bool {
        let result = self
            .token_buf
            .pop()
            .is_some_and(|token| token.text.as_ref().eq_ignore_ascii_case(b"TRUE"));
        self.token_buf.clear();
        result
    }

    fn buf_parse_many<T: From<Token<'x>>>(&mut self) -> Vec<T> {
        self.token_buf.drain(..).map(T::from).collect()
    }

    fn buf_parse_one<T: From<Token<'x>>>(&mut self) -> Option<T> {
        let result = self.token_buf.pop().map(T::from);
        self.token_buf.clear();
        result
    }

    fn buf_try_parse_one<T: for<'y> TryFrom<&'y [u8]>>(&mut self) -> Option<T> {
        let result = self
            .token_buf
            .first()
            .and_then(|t| T::try_from(t.text.as_ref()).ok());
        self.token_buf.clear();
        result
    }
}

impl Encoding {
    pub fn parse(value: &[u8]) -> Option<Self> {
        hashify::tiny_map_ignore_case!(value,
            b"QUOTED-PRINTABLE" => Encoding::QuotedPrintable,
            b"BASE64" => Encoding::Base64,
            b"Q" => Encoding::QuotedPrintable,
            b"B" => Encoding::Base64,
        )
    }
}
