/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    icalendar::ICalendarEntry,
    jscalendar::{JSCalendarProperty, export::ConvertedComponent},
};

impl ICalendarEntry {
    pub(super) fn import_converted(
        mut self,
        path: &[JSCalendarProperty],
        conversions: &[&mut Option<ConvertedComponent<'_>>],
    ) -> Self {
        let todo = "relation uses value, not jsid";
        todo!()
    }
}
