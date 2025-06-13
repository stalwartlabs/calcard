/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::PartialDateTime,
    icalendar::{
        ICalendarComponent, ICalendarEntry, ICalendarParameter, ICalendarProperty, ICalendarValue,
    },
};

impl ICalendarComponent {
    pub fn add_dtstamp(&mut self, dt_stamp: PartialDateTime) {
        self.entries.push(ICalendarEntry {
            name: ICalendarProperty::Dtstamp,
            params: vec![],
            values: vec![ICalendarValue::PartialDateTime(Box::new(dt_stamp))],
        });
    }

    pub fn add_sequence(&mut self, sequence: u32) {
        self.entries.push(ICalendarEntry {
            name: ICalendarProperty::Sequence,
            params: vec![],
            values: vec![ICalendarValue::Integer(sequence as i64)],
        });
    }

    pub fn add_uid(&mut self, uid: &str) {
        self.entries.push(ICalendarEntry {
            name: ICalendarProperty::Uid,
            params: vec![],
            values: vec![ICalendarValue::Text(uid.to_string())],
        });
    }

    pub fn add_property(&mut self, name: ICalendarProperty, value: ICalendarValue) {
        self.entries.push(ICalendarEntry {
            name,
            params: vec![],
            values: vec![value],
        });
    }

    pub fn add_property_with_params(
        &mut self,
        name: ICalendarProperty,
        params: impl IntoIterator<Item = ICalendarParameter>,
        value: ICalendarValue,
    ) {
        self.entries.push(ICalendarEntry {
            name,
            params: params.into_iter().collect(),
            values: vec![value],
        });
    }
}
