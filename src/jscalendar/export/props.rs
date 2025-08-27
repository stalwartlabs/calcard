/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use jmap_tools::{JsonPointer, Value};

use crate::{
    icalendar::ICalendarEntry,
    jscalendar::{JSCalendarProperty, JSCalendarValue, export::State},
};

impl<'x> State<'x> {
    pub(super) fn insert_ical(&mut self, path: &[JSCalendarProperty], mut entry: ICalendarEntry) {
        todo!()
    }

    pub(super) fn insert_jsprop(
        &mut self,
        path: &[&str],
        value: Value<'x, JSCalendarProperty, JSCalendarValue>,
    ) {
        self.js_props
            .push((JsonPointer::<JSCalendarProperty>::encode(path), value));
    }

    pub(super) fn import_properties(
        &mut self,
        props: Vec<Value<'x, JSCalendarProperty, JSCalendarValue>>,
    ) {
        let todo = "implement";
    }
}
