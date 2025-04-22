use super::{
    ICalendar, ICalendarComponent, ICalendarComponentType, ICalendarEntry, ICalendarParameter,
    ICalendarProperty, ICalendarValue,
};
use crate::common::timezone::Tz;
use std::{collections::HashMap, str::FromStr};

pub struct TzResolver<'x> {
    tzs: HashMap<&'x str, Tz>,
    default: Tz,
}

impl ICalendar {
    pub fn timezones(&self) -> impl Iterator<Item = &ICalendarComponent> {
        self.components
            .iter()
            .filter(|comp| matches!(comp.component_type, ICalendarComponentType::VTimezone))
    }

    pub fn is_timezone(&self) -> bool {
        self.timezones().count() == 1
    }

    pub fn build_tz_resolver(&self) -> TzResolver<'_> {
        TzResolver {
            tzs: self.timezones().filter_map(|tz| tz.timezone()).collect(),
            default: Tz::Floating,
        }
    }
}

impl TzResolver<'_> {
    pub fn resolve(&self, tz_name: Option<&str>) -> Tz {
        tz_name
            .and_then(|tz_name| {
                self.tzs
                    .get(tz_name)
                    .copied()
                    .or_else(|| Tz::from_str(tz_name).ok())
            })
            .unwrap_or(self.default)
    }

    pub fn with_default(mut self, default: Tz) -> Self {
        self.default = default;
        self
    }
}

impl ICalendarComponent {
    pub fn timezone(&self) -> Option<(&str, Tz)> {
        let mut tz_name = None;
        let mut tz_id = None;
        let mut tz_lic = None;
        let mut tz_cdo_id = None;

        for entry in &self.entries {
            match (&entry.name, entry.values.first()) {
                (ICalendarProperty::Tzid, Some(ICalendarValue::Text(id))) => {
                    tz_id = Tz::from_str(id).ok();
                    tz_name = Some(id.as_str());
                }
                (ICalendarProperty::Other(value), Some(ICalendarValue::Text(id))) => {
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

impl ICalendarEntry {
    pub fn tz_id(&self) -> Option<&str> {
        self.params.iter().find_map(|param| {
            if let ICalendarParameter::Tzid(tzid) = param {
                Some(tzid.as_str())
            } else {
                None
            }
        })
    }
}
