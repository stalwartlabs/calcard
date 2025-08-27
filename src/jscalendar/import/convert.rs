/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{icalendar::ICalendar, jscalendar::JSCalendar};

impl ICalendar {
    pub fn into_jscalendar(mut self) -> JSCalendar<'static> {
        todo!()
    }
}
