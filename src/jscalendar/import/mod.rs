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
    icalendar::{
        ICalendarComponentType, ICalendarEntry, ICalendarParameterName, ICalendarProperty,
    },
    jscalendar::{JSCalendarId, JSCalendarProperty, JSCalendarValue},
};

pub mod convert;
pub mod params;
pub mod props;

#[derive(Default)]
#[allow(clippy::type_complexity)]
struct State<I: JSCalendarId, B: JSCalendarId> {
    component_type: ICalendarComponentType,
    entries: AHashMap<
        Key<'static, JSCalendarProperty<I>>,
        Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
    >,
    ical_converted_properties: AHashMap<String, ICalendarConvertedProperty<I, B>>,
    ical_properties: Vec<Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>>,
    ical_components: Option<Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>>,
    patch_objects: Vec<(
        JsonPointer<JSCalendarProperty<I>>,
        Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
    )>,
    jsid: Option<String>,
    uid: Option<String>,
    recurrence_id: Option<DateTime<Tz>>,
    tz_start: Option<Tz>,
    tz_end: Option<Tz>,
    has_dates: bool,
    map_component: bool,
    is_recurrence_instance: bool,
    include_ical_converted: bool,
}

#[derive(Debug, Default)]
struct ICalendarConvertedProperty<I: JSCalendarId, B: JSCalendarId> {
    name: Option<ICalendarProperty>,
    params: ICalendarParams<I, B>,
}

#[derive(Debug, Default)]
#[allow(clippy::type_complexity)]
struct ICalendarParams<I: JSCalendarId, B: JSCalendarId>(
    AHashMap<
        ICalendarParameterName,
        Vec<Value<'static, JSCalendarProperty<I>, JSCalendarValue<I, B>>>,
    >,
);

#[derive(Debug, Clone)]
struct EntryState {
    entry: ICalendarEntry,
    converted_to: Option<String>,
    map_name: bool,
}
