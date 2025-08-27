/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{icalendar::ICalendar, jscalendar::JSCalendar};

impl JSCalendar<'_> {
    pub fn into_icalendar(self) -> Option<ICalendar> {
        todo!()
    }
}
