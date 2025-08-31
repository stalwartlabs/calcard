/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use ahash::AHashMap;
use chrono::DateTime;
use jmap_tools::{JsonPointer, Key, Value};

use crate::{
    common::timezone::Tz,
    icalendar::{ICalendarEntry, ICalendarMethod, ICalendarParameterName, ICalendarProperty},
    jscalendar::{JSCalendarProperty, JSCalendarValue},
};

pub mod convert;
pub mod entry;
pub mod params;
pub mod props;

#[derive(Default)]
#[allow(clippy::type_complexity)]
struct State {
    entries: AHashMap<
        Key<'static, JSCalendarProperty>,
        Value<'static, JSCalendarProperty, JSCalendarValue>,
    >,
    ical_converted_properties: AHashMap<String, ICalendarConvertedProperty>,
    ical_properties: Vec<Value<'static, JSCalendarProperty, JSCalendarValue>>,
    patch_objects: Vec<(
        JsonPointer<JSCalendarProperty>,
        Value<'static, JSCalendarProperty, JSCalendarValue>,
    )>,
    jsid: Option<String>,
    uid: Option<String>,
    recurrence_id: Option<DateTime<Tz>>,
    tz_start: Option<Tz>,
    tz_end: Option<Tz>,
    has_dates: bool,
    method: Option<ICalendarMethod>,
}

#[derive(Debug, Default)]
struct ICalendarConvertedProperty {
    name: Option<ICalendarProperty>,
    params: ICalendarParams,
}

#[derive(Debug, Default)]
struct ICalendarParams(
    AHashMap<ICalendarParameterName, Vec<Value<'static, JSCalendarProperty, JSCalendarValue>>>,
);

#[derive(Debug, Clone)]
struct EntryState {
    entry: ICalendarEntry,
    converted_to: Option<String>,
    map_name: bool,
}
