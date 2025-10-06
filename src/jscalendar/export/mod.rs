/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::timezone::Tz,
    icalendar::ICalendarComponentType,
    jscalendar::{JSCalendarId, JSCalendarProperty, JSCalendarValue},
};
use chrono::DateTime;
use jmap_tools::{Key, Map, Value};

pub mod convert;
pub mod params;
pub mod props;

struct State<'x, I: JSCalendarId, B: JSCalendarId> {
    entries: Map<'x, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
    default_component_type: ICalendarComponentType,
    uid: Option<&'x str>,
    tz: Option<Tz>,
    tz_end: Option<Tz>,
    tz_rid: Option<Tz>,
    start: Option<DateTime<Tz>>,
    recurrence_id: Option<DateTime<Tz>>,
}

#[allow(clippy::type_complexity)]
struct ConvertedComponent<'x, I: JSCalendarId, B: JSCalendarId> {
    pub(super) name: ICalendarComponentType,
    pub(super) converted_props: Vec<(
        Vec<Key<'x, JSCalendarProperty<I>>>,
        Value<'x, JSCalendarProperty<I>, JSCalendarValue<I, B>>,
    )>,
    pub(super) converted_props_count: usize,
    pub(super) properties: Vec<Value<'x, JSCalendarProperty<I>, JSCalendarValue<I, B>>>,
    pub(super) components: Vec<Value<'x, JSCalendarProperty<I>, JSCalendarValue<I, B>>>,
}
