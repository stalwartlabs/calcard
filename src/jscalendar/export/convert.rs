/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    icalendar::{ICalendar, ICalendarComponent},
    jscalendar::{JSCalendar, export::State},
};

impl JSCalendar<'_> {
    pub fn into_icalendar(self) -> Option<ICalendar> {
        ICalendarComponent::new(crate::icalendar::ICalendarComponentType::Available)
            .entries_from_jscalendar(
                State {
                    tz: Default::default(),
                    tz_end: Default::default(),
                    tz_rid: Default::default(),
                    start: Default::default(),
                    recurrence_id: Default::default(),
                    components: &mut Vec::new(),
                },
                self.0.into_object().unwrap(),
            );
        todo!()
    }
}
