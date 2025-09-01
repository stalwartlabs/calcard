/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{IanaParse, LinkRelation, parser::Boolean},
    icalendar::*,
    jscalendar::{JSCalendarProperty, JSCalendarValue, export::ConvertedComponent, uuid5},
    jscontact::export::params::ParamValue,
};
use jmap_tools::{Key, Value};

impl ICalendarEntry {
    pub(super) fn import_converted(
        mut self,
        path: &[JSCalendarProperty],
        conversions: &mut Option<ConvertedComponent<'_>>,
    ) -> Self {
        let Some(conversions) = conversions
            .as_mut()
            .filter(|c| c.converted_props_count < c.converted_props.len())
        else {
            return self;
        };

        // Obtain jsid
        let value = self.values.first().and_then(|v| v.as_text());
        let (jsid, is_uuid5_jsid) = if matches!(self.name, ICalendarProperty::RelatedTo) {
            (value, false)
        } else {
            let jsid = self.jsid();
            (
                jsid,
                jsid.is_some_and(|id| value.is_some_and(|v| uuid5(v) == id)),
            )
        };

        let mut matched_once = false;

        'outer: for (keys, value) in conversions.converted_props.iter_mut() {
            if matches!(value, Value::Null) {
                continue;
            }

            for (pos, item) in path.iter().enumerate() {
                if !keys
                    .iter()
                    .any(|k| matches!(k, Key::Property(p) if p == item))
                {
                    if pos == 0 && matched_once {
                        // Array is sorted, so if we didn't match the first item,
                        // we won't match any further.
                        break 'outer;
                    } else {
                        continue 'outer;
                    }
                } else {
                    matched_once = true;
                }
            }

            if jsid
                .map(Key::Borrowed)
                .is_none_or(|prop_id| keys.iter().any(|k| k == &prop_id))
            {
                self.import_converted_properties(std::mem::take(value));
                conversions.converted_props_count += 1;
                break;
            }
        }

        if is_uuid5_jsid {
            self.params
                .retain(|p| !matches!(p.name, ICalendarParameterName::Jsid));
        }

        self
    }

    pub(super) fn import_converted_properties(
        &mut self,
        props: Value<'_, JSCalendarProperty, JSCalendarValue>,
    ) {
        for (key, value) in props.into_expanded_object() {
            match key {
                Key::Property(JSCalendarProperty::Name) => {
                    if let Some(name) = value.into_string() {
                        self.name = ICalendarProperty::parse(name.as_bytes())
                            .unwrap_or(ICalendarProperty::Other(name.into_owned()));
                    }
                }
                Key::Property(JSCalendarProperty::Parameters) => {
                    self.import_jcal_params(value);
                }
                _ => {}
            }
        }
    }

    pub(super) fn import_jcal_params(
        &mut self,
        params: Value<'_, JSCalendarProperty, JSCalendarValue>,
    ) {
        for (key, value) in params.into_expanded_object() {
            let mut values = match value {
                Value::Array(values) => values.into_iter().filter_map(ParamValue::try_from_value),
                value => vec![value]
                    .into_iter()
                    .filter_map(ParamValue::try_from_value),
            }
            .peekable();

            if values.peek().is_none() {
                continue;
            }

            let key = key.to_string();
            let Some(param) = ICalendarParameterName::try_parse(key.as_bytes()) else {
                let key = key.into_owned();

                for value in values {
                    self.params.push(ICalendarParameter {
                        name: ICalendarParameterName::Other(key.clone()),
                        value: value.into_string().into_owned().into(),
                    });
                }

                continue;
            };

            for value in values {
                let value = match &param {
                    ICalendarParameterName::Value => {
                        let value = value.into_string();
                        ICalendarValueType::parse(value.as_bytes())
                            .map(ICalendarParameterValue::Value)
                            .unwrap_or_else(|| ICalendarParameterValue::Text(value.into_owned()))
                    }
                    ICalendarParameterName::Size | ICalendarParameterName::Order => {
                        match value.into_number() {
                            Ok(n) => ICalendarParameterValue::Integer(n.unsigned_abs()),
                            Err(value) => {
                                ICalendarParameterValue::Text(value.into_string().into_owned())
                            }
                        }
                    }
                    ICalendarParameterName::Rsvp | ICalendarParameterName::Derived => {
                        let value = value.into_string();
                        Boolean::parse(value.as_bytes())
                            .map(|v| ICalendarParameterValue::Bool(v.0))
                            .unwrap_or_else(|| ICalendarParameterValue::Text(value.into_owned()))
                    }
                    ICalendarParameterName::Altrep
                    | ICalendarParameterName::DelegatedFrom
                    | ICalendarParameterName::DelegatedTo
                    | ICalendarParameterName::Dir
                    | ICalendarParameterName::Member
                    | ICalendarParameterName::SentBy
                    | ICalendarParameterName::Schema => {
                        ICalendarParameterValue::Uri(Uri::parse(value.into_string().into_owned()))
                    }
                    ICalendarParameterName::Range => {
                        let value = value.into_string();
                        if value.eq_ignore_ascii_case("THISANDFUTURE") {
                            ICalendarParameterValue::Bool(true)
                        } else {
                            ICalendarParameterValue::Text(value.into_owned())
                        }
                    }
                    ICalendarParameterName::Gap => {
                        let value = value.into_string();
                        ICalendarDuration::parse(value.as_bytes())
                            .map(ICalendarParameterValue::Duration)
                            .unwrap_or_else(|| ICalendarParameterValue::Text(value.into_owned()))
                    }
                    ICalendarParameterName::Cutype => {
                        let value = value.into_string();
                        ICalendarUserTypes::parse(value.as_bytes())
                            .map(ICalendarParameterValue::Cutype)
                            .unwrap_or_else(|| ICalendarParameterValue::Text(value.into_owned()))
                    }
                    ICalendarParameterName::Fbtype => {
                        let value = value.into_string();
                        ICalendarFreeBusyType::parse(value.as_bytes())
                            .map(ICalendarParameterValue::Fbtype)
                            .unwrap_or_else(|| ICalendarParameterValue::Text(value.into_owned()))
                    }
                    ICalendarParameterName::Partstat => {
                        let value = value.into_string();
                        ICalendarParticipationStatus::parse(value.as_bytes())
                            .map(ICalendarParameterValue::Partstat)
                            .unwrap_or_else(|| ICalendarParameterValue::Text(value.into_owned()))
                    }
                    ICalendarParameterName::Related => {
                        let value = value.into_string();
                        ICalendarRelated::parse(value.as_bytes())
                            .map(ICalendarParameterValue::Related)
                            .unwrap_or_else(|| ICalendarParameterValue::Text(value.into_owned()))
                    }
                    ICalendarParameterName::Reltype => {
                        let value = value.into_string();
                        ICalendarRelationshipType::parse(value.as_bytes())
                            .map(ICalendarParameterValue::Reltype)
                            .unwrap_or_else(|| ICalendarParameterValue::Text(value.into_owned()))
                    }
                    ICalendarParameterName::Role => {
                        let value = value.into_string();
                        ICalendarParticipationRole::parse(value.as_bytes())
                            .map(ICalendarParameterValue::Role)
                            .unwrap_or_else(|| ICalendarParameterValue::Text(value.into_owned()))
                    }
                    ICalendarParameterName::ScheduleAgent => {
                        let value = value.into_string();
                        ICalendarScheduleAgentValue::parse(value.as_bytes())
                            .map(ICalendarParameterValue::ScheduleAgent)
                            .unwrap_or_else(|| ICalendarParameterValue::Text(value.into_owned()))
                    }
                    ICalendarParameterName::ScheduleForceSend => {
                        let value = value.into_string();
                        ICalendarScheduleForceSendValue::parse(value.as_bytes())
                            .map(ICalendarParameterValue::ScheduleForceSend)
                            .unwrap_or_else(|| ICalendarParameterValue::Text(value.into_owned()))
                    }
                    ICalendarParameterName::Display => {
                        let value = value.into_string();
                        ICalendarDisplayType::parse(value.as_bytes())
                            .map(ICalendarParameterValue::Display)
                            .unwrap_or_else(|| ICalendarParameterValue::Text(value.into_owned()))
                    }
                    ICalendarParameterName::Feature => {
                        let value = value.into_string();
                        ICalendarFeatureType::parse(value.as_bytes())
                            .map(ICalendarParameterValue::Feature)
                            .unwrap_or_else(|| ICalendarParameterValue::Text(value.into_owned()))
                    }
                    ICalendarParameterName::Linkrel => {
                        let value = value.into_string();
                        LinkRelation::parse(value.as_bytes())
                            .map(ICalendarParameterValue::Linkrel)
                            .unwrap_or_else(|| ICalendarParameterValue::Text(value.into_owned()))
                    }
                    _ => ICalendarParameterValue::Text(value.into_string().into_owned()),
                };

                self.params.push(ICalendarParameter {
                    name: param.clone(),
                    value,
                });
            }
        }
    }
}
