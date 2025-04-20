use crate::common::PartialDateTime;
use chrono::{DateTime, FixedOffset, Offset, TimeZone, Utc};
use std::str::FromStr;

#[derive(Clone, Copy)]
pub enum Tz {
    Naive,
    Fixed(FixedOffset),
    Tz(chrono_tz::Tz),
}

impl PartialDateTime {
    pub fn to_date_time_with_tz(&self, tz_id: Option<&str>) -> Option<DateTime<Tz>> {
        let dt = self.to_date_time()?;

        if let Some(offset) = dt.offset {
            if offset.local_minus_utc() == 0 {
                Tz::UTC.from_utc_datetime(&dt.date_time).into()
            } else {
                Tz::Fixed(offset)
                    .from_local_datetime(&dt.date_time)
                    .single()
            }
        } else if let Some(tz) = tz_id.and_then(|id| chrono_tz::Tz::from_str(id).ok()) {
            Tz::Tz(tz).from_local_datetime(&dt.date_time).single()
        } else {
            Tz::Naive.from_local_datetime(&dt.date_time).single()
        }
    }
}

impl Tz {
    pub fn name(&self) -> &str {
        match self {
            Self::Naive => "Naive",
            Self::Tz(tz) => tz.name(),
            Self::Fixed(_) => "Fixed",
        }
    }

    pub fn is_naive(&self) -> bool {
        matches!(self, Self::Naive)
    }

    pub const UTC: Self = Self::Tz(chrono_tz::UTC);
}

impl PartialEq for Tz {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Naive, Self::Naive) => true,
            (Self::Tz(l0), Self::Tz(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl From<Utc> for Tz {
    fn from(_tz: Utc) -> Self {
        Self::Tz(chrono_tz::UTC)
    }
}

impl From<chrono_tz::Tz> for Tz {
    fn from(tz: chrono_tz::Tz) -> Self {
        Self::Tz(tz)
    }
}

impl std::fmt::Debug for Tz {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Naive => write!(f, "Naive"),
            Self::Tz(tz) => tz.fmt(f),
            Self::Fixed(fixed_offset) => fixed_offset.fmt(f),
        }
    }
}

impl std::fmt::Display for Tz {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Naive => write!(f, "Naive"),
            Self::Tz(tz) => tz.fmt(f),
            Self::Fixed(fixed_offset) => fixed_offset.fmt(f),
        }
    }
}

#[derive(Clone, Copy)]
pub enum RRuleOffset {
    Fixed(FixedOffset),
    Tz(<chrono_tz::Tz as TimeZone>::Offset),
    Naive,
}

impl std::fmt::Debug for RRuleOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fixed(offset) => offset.fmt(f),
            Self::Tz(offset) => offset.fmt(f),
            Self::Naive => write!(f, "Naive"),
        }
    }
}

impl std::fmt::Display for RRuleOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fixed(offset) => offset.fmt(f),
            Self::Tz(offset) => offset.fmt(f),
            Self::Naive => write!(f, "Naive"),
        }
    }
}

impl Offset for RRuleOffset {
    fn fix(&self) -> FixedOffset {
        match self {
            Self::Fixed(tz) => tz.fix(),
            Self::Tz(tz) => tz.fix(),
            Self::Naive => FixedOffset::east_opt(0).unwrap(),
        }
    }
}

impl TimeZone for Tz {
    type Offset = RRuleOffset;

    fn from_offset(offset: &Self::Offset) -> Self {
        match offset {
            RRuleOffset::Tz(offset) => Self::Tz(chrono_tz::Tz::from_offset(offset)),
            RRuleOffset::Fixed(fixed_offset) => Self::Fixed(*fixed_offset),
            RRuleOffset::Naive => Self::Naive,
        }
    }

    #[allow(deprecated)]
    fn offset_from_local_date(
        &self,
        local: &chrono::NaiveDate,
    ) -> chrono::LocalResult<Self::Offset> {
        match self {
            Self::Fixed(tz) => tz
                .from_local_date(local)
                .map(|date| RRuleOffset::Fixed(*date.offset())),
            Self::Tz(tz) => tz
                .from_local_date(local)
                .map(|date| RRuleOffset::Tz(*date.offset())),
            Self::Naive => chrono::LocalResult::Single(RRuleOffset::Naive),
        }
    }

    fn offset_from_local_datetime(
        &self,
        local: &chrono::NaiveDateTime,
    ) -> chrono::LocalResult<Self::Offset> {
        match self {
            Self::Fixed(tz) => tz
                .from_local_datetime(local)
                .map(|date| RRuleOffset::Fixed(*date.offset())),
            Self::Tz(tz) => tz
                .from_local_datetime(local)
                .map(|date| RRuleOffset::Tz(*date.offset())),
            Self::Naive => chrono::LocalResult::Single(RRuleOffset::Naive),
        }
    }

    #[allow(deprecated)]
    fn offset_from_utc_date(&self, utc: &chrono::NaiveDate) -> Self::Offset {
        match self {
            Self::Fixed(tz) => RRuleOffset::Fixed(*tz.from_utc_date(utc).offset()),
            Self::Tz(tz) => RRuleOffset::Tz(*tz.from_utc_date(utc).offset()),
            Self::Naive => RRuleOffset::Naive,
        }
    }

    fn offset_from_utc_datetime(&self, utc: &chrono::NaiveDateTime) -> Self::Offset {
        match self {
            Self::Fixed(tz) => RRuleOffset::Fixed(*tz.from_utc_datetime(utc).offset()),
            Self::Tz(tz) => RRuleOffset::Tz(*tz.from_utc_datetime(utc).offset()),
            Self::Naive => RRuleOffset::Naive,
        }
    }
}
