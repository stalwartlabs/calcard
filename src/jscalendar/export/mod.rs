/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    icalendar::ICalendarComponentType,
    jscalendar::{JSCalendarProperty, JSCalendarValue},
};
use jmap_tools::{Key, Value};

pub mod convert;
pub mod entry;
pub mod params;
pub mod props;

#[allow(clippy::type_complexity)]
struct ConvertedComponent<'x> {
    pub(super) name: ICalendarComponentType,
    pub(super) converted_props: Vec<(
        Vec<Key<'static, JSCalendarProperty>>,
        Value<'x, JSCalendarProperty, JSCalendarValue>,
    )>,
    pub(super) converted_props_count: usize,
    pub(super) properties: Vec<Value<'x, JSCalendarProperty, JSCalendarValue>>,
    pub(super) components: Vec<Value<'x, JSCalendarProperty, JSCalendarValue>>,
}
