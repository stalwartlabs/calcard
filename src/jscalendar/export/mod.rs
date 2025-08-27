/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    icalendar::ICalendarComponent,
    jscalendar::{JSCalendarProperty, JSCalendarType, JSCalendarValue},
};
use jmap_tools::{Key, Value};

pub mod convert;
pub mod entry;
pub mod props;

#[derive(Default)]
#[allow(clippy::type_complexity)]
struct State<'x> {
    pub(super) component: ICalendarComponent,
    pub(super) js_props: Vec<(String, Value<'x, JSCalendarProperty, JSCalendarValue>)>,
    pub(super) converted_props: Vec<(
        Vec<Key<'static, JSCalendarProperty>>,
        Value<'x, JSCalendarProperty, JSCalendarValue>,
    )>,
    pub(super) converted_props_count: usize,
    pub(super) typ: JSCalendarType,
}
