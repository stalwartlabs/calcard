use crate::Token;

pub mod parser;
pub mod tokenizer;
pub mod writer;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PartialDateTime {
    pub year: Option<u16>,
    pub month: Option<u16>,
    pub day: Option<u16>,
    pub hour: Option<u16>,
    pub minute: Option<u16>,
    pub second: Option<u16>,
    pub tz_hour: Option<u16>,
    pub tz_minute: Option<u16>,
    pub tz_minus: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum CalendarScale {
    #[default]
    Gregorian,
    Chinese,
    IslamicCivil,
    Hebrew,
    Ethiopic,
    Other(String),
}

impl CalendarScale {
    pub fn as_str(&self) -> &str {
        match self {
            CalendarScale::Gregorian => "GREGORIAN",
            CalendarScale::Chinese => "CHINESE",
            CalendarScale::IslamicCivil => "ISLAMIC-CIVIL",
            CalendarScale::Hebrew => "HEBREW",
            CalendarScale::Ethiopic => "ETHIOPIC",
            CalendarScale::Other(ref s) => s,
        }
    }
}

impl AsRef<str> for CalendarScale {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<Token<'_>> for CalendarScale {
    fn from(token: Token<'_>) -> Self {
        hashify::tiny_map_ignore_case!(token.text.as_ref(),
            "gregorian" => CalendarScale::Gregorian,
            "chinese" => CalendarScale::Chinese,
            "islamic-civil" => CalendarScale::IslamicCivil,
            "hebrew" => CalendarScale::Hebrew,
            "ethiopic" => CalendarScale::Ethiopic,
        )
        .unwrap_or_else(|| CalendarScale::Other(token.into_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Encoding {
    QuotedPrintable,
    Base64,
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Data {
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}
