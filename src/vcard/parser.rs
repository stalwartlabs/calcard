use crate::{
    jscontact::{CalendarScale, JSContact, Kind, PhoneticSystem, Property, VCardParams, Value},
    parser::Parser,
    tokenizer::StopChar,
    VecMap,
};

use super::{VCardLevel, VCardParameter, VCardType, VCardValueDataTypes};

struct Params<'x> {
    vcard: &'x mut JSContact,
    stop_char: StopChar,
    group_name: Option<String>,
    params: VCardParams,
    types: Vec<VCardType>,
    data_types: Vec<VCardValueDataTypes>,
    levels: Vec<VCardLevel>,
    charset: Option<String>,
    encoding: Option<Encoding>,
}

enum Encoding {
    QuotedPrintable,
    Base64,
}

impl Parser<'_> {
    pub fn vcard(&mut self) -> crate::parser::Result<JSContact> {
        let mut vcard = JSContact::default();

        loop {
            // Fetch property name
            self.expect_iana_token();
            let mut token = match self.token() {
                Some(token) => token,
                None => break,
            };

            let mut params = Params {
                group_name: None,
                params: VCardParams::default(),
                vcard: &mut vcard,
                stop_char: token.stop_char,
                data_types: Vec::new(),
                types: Vec::new(),
                levels: Vec::new(),
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
            if params.stop_char != StopChar::Colon {
                params.stop_char = self.seek_value_or_eol();
            }

            // Parse property
            hashify::fnc_map_ignore_case!(name.as_ref(),
                b"KIND" => {
                    if let Some(token) = self.single_value(params.stop_char) {
                        params.vcard.items.insert(
                            Property::Kind,
                            Value::Kind(
                                Kind::try_from(token.text.as_ref())
                                    .unwrap_or_else(|_| Kind::Other(token.into_string())),
                            ),
                        );
                    }
                },
                b"SOURCE" => {
                    if let Some(token) = self.single_value(params.stop_char) {
                        //params.vcard.directories.0.insert()
                    }
                },

                /*
                b"XML" => { self.Xml(); },
                b"FN" => { self.Fn(); },
                b"N" => { self.N(); },
                b"NICKNAME" => { self.Nickname(); },
                b"PHOTO" => { self.Photo(); },
                b"BDAY" => { self.Bday(); },
                b"ANNIVERSARY" => { self.Anniversary(); },
                b"GENDER" => { self.Gender(); },
                b"ADR" => { self.Adr(); },
                b"TEL" => { self.Tel(); },
                b"EMAIL" => { self.Email(); },
                b"IMPP" => { self.Impp(); },
                b"LANG" => { self.Lang(); },
                b"TZ" => { self.Tz(); },
                b"GEO" => { self.Geo(); },
                b"TITLE" => { self.Title(); },
                b"ROLE" => { self.Role(); },
                b"LOGO" => { self.Logo(); },
                b"ORG" => { self.Org(); },
                b"MEMBER" => { self.Member(); },
                b"RELATED" => { self.Related(); },
                b"CATEGORIES" => { self.Categories(); },
                b"NOTE" => { self.Note(); },
                b"PRODID" => { self.Prodid(); },
                b"REV" => { self.Rev(); },
                b"SOUND" => { self.Sound(); },
                b"UID" => { self.Uid(); },
                b"CLIENTPIDMAP" => { self.Clientpidmap(); },
                b"URL" => { self.Url(); },
                b"VERSION" => { self.Version(); },
                b"KEY" => { self.Key(); },
                b"FBURL" => { self.Fburl(); },
                b"CALADRURI" => { self.Caladruri(); },
                b"CALURI" => { self.Caluri(); },
                b"BIRTHPLACE" => { self.Birthplace(); },
                b"DEATHPLACE" => { self.Deathplace(); },
                b"DEATHDATE" => { self.Deathdate(); },
                b"EXPERTISE" => { self.Expertise(); },
                b"HOBBY" => { self.Hobby(); },
                b"INTEREST" => { self.Interest(); },
                b"ORG-DIRECTORY" => { self.OrgDirectory(); },
                b"CONTACT-URI" => { self.ContactUri(); },
                b"CREATED" => { self.Created(); },
                b"GRAMGENDER" => { self.Gramgender(); },
                b"LANGUAGE" => { self.Language(); },
                b"PRONOUNS" => { self.Pronouns(); },
                b"SOCIALPROFILE" => { self.Socialprofile(); },
                b"JSPROP" => { self.Jsprop(); },*/
                b"END" => {
                    self.seek_lf();
                    break;
                },
                _ => {
                    self.unknown(params);
                }
            );
        }

        Ok(vcard)
    }

    fn parameters(&mut self, params: &mut Params<'_>) {
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

            let mut text_param_name = None;
            let param_values = &mut params.params.0;
            hashify::fnc_map_ignore_case!(param_name.as_ref(),
                b"LANGUAGE" => {
                    text_param_name = VCardParameter::Language.into();
                },
                b"VALUE" => {
                    params.data_types.extend(
                        self.token_buf
                            .drain(..)
                            .map(Into::into),
                    );
                },
                b"PREF" => {
                    param_values.get_mut_or_default(VCardParameter::Pref).extend(
                        self.token_buf
                            .drain(..)
                            .map(Value::try_uint),
                    );
                },
                b"ALTID" => {
                    text_param_name = VCardParameter::Altid.into();
                },
                b"PID" => {
                    text_param_name = VCardParameter::Pid.into();
                },
                b"TYPE" => {
                    params.types.extend(
                        self.token_buf
                            .drain(..)
                            .map(Into::into),
                    );
                },
                b"MEDIATYPE" => {
                    text_param_name = VCardParameter::Mediatype.into();
                },
                b"CALSCALE" => {
                    param_values
                        .get_mut_or_default(VCardParameter::Calscale)
                        .extend(self.token_buf.drain(..).map(|token| {
                            Value::CalendarScale(
                                CalendarScale::try_from(token.text.as_ref())
                                    .unwrap_or_else(|_| CalendarScale::Other(token.into_string())),
                            )
                        }));
                },
                b"SORT-AS" => {
                    text_param_name = VCardParameter::SortAs.into();
                },
                b"GEO" => {
                    text_param_name = VCardParameter::Geo.into();
                },
                b"TZ" => {
                    text_param_name = VCardParameter::Tz.into();
                },
                b"INDEX" => {
                    param_values.get_mut_or_default(VCardParameter::Index).extend(
                        self.token_buf
                            .drain(..)
                            .map(Value::try_uint),
                    );
                },
                b"LEVEL" => {
                    for token in self.token_buf.drain(..) {
                        if let Ok(level) = VCardLevel::try_from(token.text.as_ref()) {
                            params.levels.push(level);
                        }
                    }
                },
                b"GROUP" => {
                    param_values.get_mut_or_default(VCardParameter::Group).extend(
                        self.token_buf
                            .drain(..)
                            .map(Value::try_uint),
                    );
                },
                b"CC" => {
                    text_param_name = VCardParameter::Cc.into();
                },
                b"AUTHOR" => {
                    text_param_name = VCardParameter::Author.into();
                },
                b"AUTHOR-NAME" => {
                    text_param_name = VCardParameter::AuthorName.into();
                },
                b"CREATED" => {
                    param_values.get_mut_or_default(VCardParameter::Created).extend(
                        self.token_buf
                            .drain(..)
                            .map(Value::try_timestamp),
                    );
                },
                b"DERIVED" => {
                    param_values.get_mut_or_default(VCardParameter::Derived).extend(
                        self.token_buf
                            .drain(..)
                            .map(Value::bool),
                    );
                },
                b"LABEL" => {
                    text_param_name = VCardParameter::Label.into();
                },
                b"PHONETIC" => {
                    param_values
                        .get_mut_or_default(VCardParameter::Phonetic)
                        .extend(self.token_buf.drain(..).map(|token| {
                            Value::Phonetic(
                                PhoneticSystem::try_from(token.text.as_ref())
                                    .unwrap_or_else(|_| PhoneticSystem::Other(token.into_string())),
                            )
                        }));
                },
                b"PROP-ID" => {
                    text_param_name = VCardParameter::PropId.into();
                },
                b"SCRIPT" => {
                    text_param_name = VCardParameter::Script.into();
                },
                b"SERVICE-TYPE" => {
                    text_param_name = VCardParameter::ServiceType.into();
                },
                b"USERNAME" => {
                    text_param_name = VCardParameter::Username.into();
                },
                b"JSPTR" => {
                    text_param_name = VCardParameter::Jsptr.into();
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
                            params.types.push(typ);
                        },
                        _ => {
                            param_values
                            .get_mut_or_default(VCardParameter::Other(
                                String::from_utf8(param_name.into_owned()).unwrap_or_default(),
                            ))
                            .extend(self.token_buf.drain(..).map(Value::text));
                        }
                    }
                }
            );

            if let Some(text_param_name) = text_param_name {
                param_values
                    .get_mut_or_default(text_param_name)
                    .extend(self.token_buf.drain(..).map(Value::text));
            }
        }
    }

    fn unknown(&mut self, params: Params<'_>) {
        todo!()
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
