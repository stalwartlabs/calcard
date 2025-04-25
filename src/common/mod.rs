use crate::Token;
use chrono::{FixedOffset, NaiveDate, NaiveDateTime};

pub mod parser;
pub mod timezone;
pub mod tokenizer;
pub mod writer;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub struct PartialDateTime {
    pub year: Option<u16>,
    pub month: Option<u8>,
    pub day: Option<u8>,
    pub hour: Option<u8>,
    pub minute: Option<u8>,
    pub second: Option<u8>,
    pub tz_hour: Option<u8>,
    pub tz_minute: Option<u8>,
    pub tz_minus: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
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

#[cfg(feature = "rkyv")]
impl ArchivedCalendarScale {
    pub fn as_str(&self) -> &str {
        match self {
            ArchivedCalendarScale::Gregorian => "GREGORIAN",
            ArchivedCalendarScale::Chinese => "CHINESE",
            ArchivedCalendarScale::IslamicCivil => "ISLAMIC-CIVIL",
            ArchivedCalendarScale::Hebrew => "HEBREW",
            ArchivedCalendarScale::Ethiopic => "ETHIOPIC",
            ArchivedCalendarScale::Other(ref s) => s,
        }
    }
}

#[cfg(feature = "rkyv")]
impl AsRef<str> for ArchivedCalendarScale {
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
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub struct Data {
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}

impl PartialDateTime {
    pub fn to_date_time(&self) -> Option<DateTimeResult> {
        let mut dt = DateTimeResult {
            date_time: NaiveDate::from_ymd_opt(
                self.year? as i32,
                self.month? as u32,
                self.day? as u32,
            )?
            .and_hms_opt(
                self.hour.unwrap_or(0) as u32,
                self.minute.unwrap_or(0) as u32,
                self.second.unwrap_or(0) as u32,
            )?,
            offset: None,
        };
        if let Some(tz_hour) = self.tz_hour {
            let secs = (tz_hour as i32 * 3600) + (self.tz_minute.unwrap_or(0) as i32 * 60);
            dt.offset = if self.tz_minus {
                FixedOffset::west_opt(secs)?
            } else {
                FixedOffset::east_opt(secs)?
            }
            .into();
        }
        Some(dt)
    }
}

#[cfg(feature = "rkyv")]
impl ArchivedPartialDateTime {
    pub fn to_date_time(&self) -> Option<DateTimeResult> {
        let mut dt = DateTimeResult {
            date_time: NaiveDate::from_ymd_opt(
                self.year.as_ref()?.to_native() as i32,
                *self.month.as_ref()? as u32,
                *self.day.as_ref()? as u32,
            )?
            .and_hms_opt(
                self.hour.unwrap_or(0) as u32,
                self.minute.unwrap_or(0) as u32,
                self.second.unwrap_or(0) as u32,
            )?,
            offset: None,
        };
        if let Some(tz_hour) = self.tz_hour.as_ref() {
            let secs = (*tz_hour as i32 * 3600) + (self.tz_minute.unwrap_or(0) as i32 * 60);
            dt.offset = if self.tz_minus {
                FixedOffset::west_opt(secs)?
            } else {
                FixedOffset::east_opt(secs)?
            }
            .into();
        }
        Some(dt)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeResult {
    pub date_time: NaiveDateTime,
    pub offset: Option<FixedOffset>,
}
