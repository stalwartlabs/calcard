/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    icalendar::ICalendarComponentType,
    jscalendar::{JSCalendarDateTime, JSCalendarProperty, JSCalendarValue},
};
use chrono::{DateTime, NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use jmap_tools::{Key, Value};

pub mod convert;
pub mod entry;
pub mod params;
pub mod props;

#[allow(clippy::type_complexity)]
struct ConvertedComponent<'x> {
    pub(super) name: ICalendarComponentType,
    pub(super) converted_props: Vec<(
        Vec<Key<'x, JSCalendarProperty>>,
        Value<'x, JSCalendarProperty, JSCalendarValue>,
    )>,
    pub(super) converted_props_count: usize,
    pub(super) properties: Vec<Value<'x, JSCalendarProperty, JSCalendarValue>>,
    pub(super) components: Vec<Value<'x, JSCalendarProperty, JSCalendarValue>>,
}

impl JSCalendarDateTime {
    pub fn to_utc_date_time(&self) -> Option<DateTime<Tz>> {
        DateTime::from_timestamp(self.timestamp, 0)
            .and_then(|local| Tz::UTC.from_local_datetime(&local.naive_utc()).single())
    }

    pub fn to_naive_date_time(&self) -> Option<NaiveDateTime> {
        DateTime::from_timestamp(self.timestamp, 0).map(|local| local.naive_utc())
    }
}
