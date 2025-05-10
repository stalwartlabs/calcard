/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::{
    ArchivedICalendar, ArchivedICalendarComponent, ArchivedICalendarComponentType,
    ArchivedICalendarEntry, ArchivedICalendarParameter, ArchivedICalendarProperty,
    ArchivedICalendarValue,
};
use crate::common::timezone::Tz;
use std::str::FromStr;

impl ArchivedICalendar {
    pub fn timezones(&self) -> impl Iterator<Item = &ArchivedICalendarComponent> {
        self.components.iter().filter(|comp| {
            matches!(
                comp.component_type,
                ArchivedICalendarComponentType::VTimezone
            )
        })
    }

    pub fn is_timezone(&self) -> bool {
        self.timezones().count() == 1
    }
}

impl ArchivedICalendarComponent {
    pub fn timezone(&self) -> Option<(&str, Tz)> {
        let mut tz_name = None;
        let mut tz_id = None;
        let mut tz_lic = None;
        let mut tz_cdo_id = None;

        for entry in self.entries.iter() {
            match (&entry.name, entry.values.first()) {
                (ArchivedICalendarProperty::Tzid, Some(ArchivedICalendarValue::Text(id))) => {
                    tz_id = Tz::from_str(id.as_str()).ok();
                    tz_name = Some(id.as_str());
                }
                (
                    ArchivedICalendarProperty::Other(value),
                    Some(ArchivedICalendarValue::Text(id)),
                ) => {
                    hashify::fnc_map!(value.as_bytes(),
                        "X-LIC-LOCATION" => {
                            tz_lic = Tz::from_str(id.strip_prefix("SystemV/").unwrap_or(id.as_str())).ok();
                        },
                        "X-MICROSOFT-CDO-TZID" => {
                            tz_cdo_id = Tz::from_ms_cdo_zone_id(id);
                        },
                        _ => {}
                    );
                }
                _ => (),
            }
        }

        tz_name.and_then(|name| tz_id.or(tz_lic).or(tz_cdo_id).map(|tz| (name, tz)))
    }
}

impl ArchivedICalendarEntry {
    pub fn tz_id(&self) -> Option<&str> {
        self.params.iter().find_map(|param| {
            if let ArchivedICalendarParameter::Tzid(tzid) = param {
                Some(tzid.as_str())
            } else {
                None
            }
        })
    }
}
