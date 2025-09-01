/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{icalendar::ICalendar, jscalendar::JSCalendar};

impl ICalendar {
    pub fn into_jscalendar(self) -> JSCalendar<'static> {
        let resolver = self.build_owned_tz_resolver();
        self.components
            .into_iter()
            .next()
            .unwrap()
            .entries_to_jscalendar(&resolver)
            .into_object(crate::icalendar::ICalendarComponentType::Available);

        todo!()
    }
}
