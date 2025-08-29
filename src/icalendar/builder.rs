/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::{
        IanaType, PartialDateTime,
        parser::{Boolean, Integer},
    },
    icalendar::*,
};
use ahash::{AHashMap, AHashSet};

impl ICalendar {
    pub fn remove_component_ids(&mut self, component_ids: &[u32]) {
        // Validate component IDs
        let max_component_id = self.components.len() as u32;
        let mut remove_component_ids = AHashSet::from_iter(
            component_ids
                .iter()
                .filter(|id| **id < max_component_id)
                .cloned(),
        );

        // Add sub-components to the set
        for (component_id, component) in self.components.iter().enumerate() {
            if remove_component_ids.contains(&(component_id as u32)) {
                remove_component_ids.extend(&component.component_ids);
            }
        }

        if !remove_component_ids.is_empty() {
            let id_mappings = (0..max_component_id)
                .filter(|i| !remove_component_ids.contains(i))
                .enumerate()
                .map(|(new_id, old_id)| (old_id, new_id as u32))
                .collect::<AHashMap<_, _>>();

            for (component_id, mut component) in
                std::mem::replace(&mut self.components, Vec::with_capacity(id_mappings.len()))
                    .into_iter()
                    .enumerate()
            {
                if !remove_component_ids.contains(&(component_id as u32)) {
                    let component_ids = component
                        .component_ids
                        .iter()
                        .filter_map(|id| id_mappings.get(id).cloned())
                        .collect();
                    component.component_ids = component_ids;
                    self.components.push(component);
                }
            }
        }
    }

    pub fn copy_timezones(&mut self, other: &ICalendar) {
        for component in &other.components {
            if component.component_type == ICalendarComponentType::VTimezone {
                let tz_component_id = self.components.len();
                self.components[0]
                    .component_ids
                    .insert(1, tz_component_id as u32);
                self.components.push(ICalendarComponent {
                    component_type: ICalendarComponentType::VTimezone,
                    entries: component.entries.clone(),
                    component_ids: vec![],
                });
                for component_id in &component.component_ids {
                    let item_id = self.components.len() as u32;
                    let item = &other.components[*component_id as usize];
                    self.components.push(ICalendarComponent {
                        component_type: item.component_type.clone(),
                        entries: item.entries.clone(),
                        component_ids: vec![],
                    });
                    self.components[tz_component_id].component_ids.push(item_id);
                }
            }
        }
    }
}

impl ICalendarComponent {
    pub fn new(component_type: ICalendarComponentType) -> Self {
        Self {
            component_type,
            entries: Vec::new(),
            component_ids: Vec::new(),
        }
    }

    pub fn add_dtstamp(&mut self, dt_stamp: PartialDateTime) {
        self.entries.push(ICalendarEntry {
            name: ICalendarProperty::Dtstamp,
            params: vec![],
            values: vec![ICalendarValue::PartialDateTime(Box::new(dt_stamp))],
        });
    }

    pub fn add_sequence(&mut self, sequence: i64) {
        self.entries.push(ICalendarEntry {
            name: ICalendarProperty::Sequence,
            params: vec![],
            values: vec![ICalendarValue::Integer(sequence)],
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

impl ICalendarEntry {
    pub fn new(name: ICalendarProperty) -> Self {
        Self {
            name,
            params: vec![],
            values: vec![],
        }
    }

    pub fn with_params(mut self, params: Vec<ICalendarParameter>) -> Self {
        self.params = params;
        self
    }

    pub fn with_value(mut self, value: impl Into<ICalendarValue>) -> Self {
        self.values.push(value.into());
        self
    }

    pub fn with_values(mut self, values: Vec<ICalendarValue>) -> Self {
        self.values = values;
        self
    }

    pub fn with_param(mut self, param: impl Into<ICalendarParameter>) -> Self {
        self.params.push(param.into());
        self
    }

    pub fn add_param(&mut self, param: impl Into<ICalendarParameter>) {
        self.params.push(param.into());
    }

    pub fn with_param_opt(mut self, param: Option<impl Into<ICalendarParameter>>) -> Self {
        if let Some(param) = param {
            self.params.push(param.into());
        }
        self
    }

    pub fn is_type(&self, typ: &ICalendarValueType) -> bool {
        self.parameters(&ICalendarParameterName::Value)
            .any(|p| matches!(p, ICalendarParameterValue::Value(v) if v == typ))
    }
}

impl ICalendarParameter {
    pub fn new(name: ICalendarParameterName, value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name,
            value: value.into(),
        }
    }

    pub fn altrep(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Altrep,
            value: value.into(),
        }
    }

    pub fn cn(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Cn,
            value: value.into(),
        }
    }

    pub fn cutype(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Cutype,
            value: value.into(),
        }
    }

    pub fn delegated_from(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::DelegatedFrom,
            value: value.into(),
        }
    }

    pub fn delegated_to(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::DelegatedTo,
            value: value.into(),
        }
    }

    pub fn dir(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Dir,
            value: value.into(),
        }
    }

    pub fn fmttype(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Fmttype,
            value: value.into(),
        }
    }

    pub fn fbtype(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Fbtype,
            value: value.into(),
        }
    }

    pub fn language(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Language,
            value: value.into(),
        }
    }

    pub fn member(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Member,
            value: value.into(),
        }
    }

    pub fn partstat(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Partstat,
            value: value.into(),
        }
    }

    pub fn range(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Range,
            value: value.into(),
        }
    }

    pub fn related(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Related,
            value: value.into(),
        }
    }

    pub fn reltype(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Reltype,
            value: value.into(),
        }
    }

    pub fn role(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Role,
            value: value.into(),
        }
    }

    pub fn rsvp(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Rsvp,
            value: value.into(),
        }
    }

    pub fn schedule_agent(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::ScheduleAgent,
            value: value.into(),
        }
    }

    pub fn schedule_force_send(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::ScheduleForceSend,
            value: value.into(),
        }
    }

    pub fn schedule_status(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::ScheduleStatus,
            value: value.into(),
        }
    }

    pub fn sent_by(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::SentBy,
            value: value.into(),
        }
    }

    pub fn tzid(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Tzid,
            value: value.into(),
        }
    }

    pub fn value(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Value,
            value: value.into(),
        }
    }

    pub fn display(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Display,
            value: value.into(),
        }
    }

    pub fn email(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Email,
            value: value.into(),
        }
    }

    pub fn feature(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Feature,
            value: value.into(),
        }
    }

    pub fn label(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Label,
            value: value.into(),
        }
    }

    pub fn size(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Size,
            value: value.into(),
        }
    }

    pub fn filename(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Filename,
            value: value.into(),
        }
    }

    pub fn managed_id(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::ManagedId,
            value: value.into(),
        }
    }

    pub fn order(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Order,
            value: value.into(),
        }
    }

    pub fn schema(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Schema,
            value: value.into(),
        }
    }

    pub fn derived(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Derived,
            value: value.into(),
        }
    }

    pub fn gap(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Gap,
            value: value.into(),
        }
    }

    pub fn linkrel(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Linkrel,
            value: value.into(),
        }
    }

    pub fn jsptr(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Jsptr,
            value: value.into(),
        }
    }

    pub fn jsid(value: impl Into<ICalendarParameterValue>) -> Self {
        ICalendarParameter {
            name: ICalendarParameterName::Jsid,
            value: value.into(),
        }
    }
}

impl From<String> for ICalendarParameterValue {
    fn from(text: String) -> Self {
        ICalendarParameterValue::Text(text)
    }
}

impl From<u64> for ICalendarParameterValue {
    fn from(integer: u64) -> Self {
        ICalendarParameterValue::Integer(integer)
    }
}

impl From<Integer> for ICalendarParameterValue {
    fn from(integer: Integer) -> Self {
        ICalendarParameterValue::Integer(integer.0.unsigned_abs())
    }
}

impl From<Boolean> for ICalendarParameterValue {
    fn from(boolean: Boolean) -> Self {
        ICalendarParameterValue::Bool(boolean.0)
    }
}

impl From<bool> for ICalendarParameterValue {
    fn from(boolean: bool) -> Self {
        ICalendarParameterValue::Bool(boolean)
    }
}

impl From<Uri> for ICalendarParameterValue {
    fn from(uri: Uri) -> Self {
        ICalendarParameterValue::Uri(uri)
    }
}

impl From<ICalendarUserTypes> for ICalendarParameterValue {
    fn from(cutype: ICalendarUserTypes) -> Self {
        ICalendarParameterValue::Cutype(cutype)
    }
}

impl From<ICalendarFreeBusyType> for ICalendarParameterValue {
    fn from(fbtype: ICalendarFreeBusyType) -> Self {
        ICalendarParameterValue::Fbtype(fbtype)
    }
}

impl From<ICalendarParticipationStatus> for ICalendarParameterValue {
    fn from(partstat: ICalendarParticipationStatus) -> Self {
        ICalendarParameterValue::Partstat(partstat)
    }
}

impl From<ICalendarRelated> for ICalendarParameterValue {
    fn from(related: ICalendarRelated) -> Self {
        ICalendarParameterValue::Related(related)
    }
}

impl From<ICalendarRelationshipType> for ICalendarParameterValue {
    fn from(reltype: ICalendarRelationshipType) -> Self {
        ICalendarParameterValue::Reltype(reltype)
    }
}

impl From<ICalendarParticipationRole> for ICalendarParameterValue {
    fn from(role: ICalendarParticipationRole) -> Self {
        ICalendarParameterValue::Role(role)
    }
}

impl From<ICalendarScheduleAgentValue> for ICalendarParameterValue {
    fn from(schedule_agent: ICalendarScheduleAgentValue) -> Self {
        ICalendarParameterValue::ScheduleAgent(schedule_agent)
    }
}

impl From<ICalendarScheduleForceSendValue> for ICalendarParameterValue {
    fn from(schedule_force_send: ICalendarScheduleForceSendValue) -> Self {
        ICalendarParameterValue::ScheduleForceSend(schedule_force_send)
    }
}

impl From<ICalendarValueType> for ICalendarParameterValue {
    fn from(value: ICalendarValueType) -> Self {
        ICalendarParameterValue::Value(value)
    }
}

impl From<ICalendarDisplayType> for ICalendarParameterValue {
    fn from(display: ICalendarDisplayType) -> Self {
        ICalendarParameterValue::Display(display)
    }
}

impl From<ICalendarFeatureType> for ICalendarParameterValue {
    fn from(feature: ICalendarFeatureType) -> Self {
        ICalendarParameterValue::Feature(feature)
    }
}

impl From<ICalendarDuration> for ICalendarParameterValue {
    fn from(duration: ICalendarDuration) -> Self {
        ICalendarParameterValue::Duration(duration)
    }
}

impl From<LinkRelation> for ICalendarParameterValue {
    fn from(linkrel: LinkRelation) -> Self {
        ICalendarParameterValue::Linkrel(linkrel)
    }
}

impl<T: Into<ICalendarParameterValue>> From<IanaType<T, String>> for ICalendarParameterValue {
    fn from(value: IanaType<T, String>) -> Self {
        match value {
            IanaType::Iana(v) => v.into(),
            IanaType::Other(s) => s.into(),
        }
    }
}

// From implementations for ICalendarValue enum

impl From<Vec<u8>> for ICalendarValue {
    fn from(value: Vec<u8>) -> Self {
        ICalendarValue::Binary(value)
    }
}

impl From<bool> for ICalendarValue {
    fn from(value: bool) -> Self {
        ICalendarValue::Boolean(value)
    }
}

impl From<Uri> for ICalendarValue {
    fn from(value: Uri) -> Self {
        ICalendarValue::Uri(value)
    }
}

impl From<PartialDateTime> for ICalendarValue {
    fn from(value: PartialDateTime) -> Self {
        ICalendarValue::PartialDateTime(Box::new(value))
    }
}

impl From<Box<PartialDateTime>> for ICalendarValue {
    fn from(value: Box<PartialDateTime>) -> Self {
        ICalendarValue::PartialDateTime(value)
    }
}

impl From<ICalendarDuration> for ICalendarValue {
    fn from(value: ICalendarDuration) -> Self {
        ICalendarValue::Duration(value)
    }
}

impl From<ICalendarRecurrenceRule> for ICalendarValue {
    fn from(value: ICalendarRecurrenceRule) -> Self {
        ICalendarValue::RecurrenceRule(Box::new(value))
    }
}

impl From<Box<ICalendarRecurrenceRule>> for ICalendarValue {
    fn from(value: Box<ICalendarRecurrenceRule>) -> Self {
        ICalendarValue::RecurrenceRule(value)
    }
}

impl From<ICalendarPeriod> for ICalendarValue {
    fn from(value: ICalendarPeriod) -> Self {
        ICalendarValue::Period(value)
    }
}

impl From<f64> for ICalendarValue {
    fn from(value: f64) -> Self {
        ICalendarValue::Float(value)
    }
}

impl From<i64> for ICalendarValue {
    fn from(value: i64) -> Self {
        ICalendarValue::Integer(value)
    }
}

impl From<String> for ICalendarValue {
    fn from(value: String) -> Self {
        ICalendarValue::Text(value)
    }
}

impl From<&str> for ICalendarValue {
    fn from(value: &str) -> Self {
        ICalendarValue::Text(value.to_string())
    }
}

impl From<CalendarScale> for ICalendarValue {
    fn from(value: CalendarScale) -> Self {
        ICalendarValue::CalendarScale(value)
    }
}

impl From<ICalendarMethod> for ICalendarValue {
    fn from(value: ICalendarMethod) -> Self {
        ICalendarValue::Method(value)
    }
}

impl From<ICalendarClassification> for ICalendarValue {
    fn from(value: ICalendarClassification) -> Self {
        ICalendarValue::Classification(value)
    }
}

impl From<ICalendarStatus> for ICalendarValue {
    fn from(value: ICalendarStatus) -> Self {
        ICalendarValue::Status(value)
    }
}

impl From<ICalendarTransparency> for ICalendarValue {
    fn from(value: ICalendarTransparency) -> Self {
        ICalendarValue::Transparency(value)
    }
}

impl From<ICalendarAction> for ICalendarValue {
    fn from(value: ICalendarAction) -> Self {
        ICalendarValue::Action(value)
    }
}

impl From<ICalendarFreeBusyType> for ICalendarValue {
    fn from(value: ICalendarFreeBusyType) -> Self {
        ICalendarValue::BusyType(value)
    }
}

impl From<ICalendarParticipantType> for ICalendarValue {
    fn from(value: ICalendarParticipantType) -> Self {
        ICalendarValue::ParticipantType(value)
    }
}

impl From<ICalendarResourceType> for ICalendarValue {
    fn from(value: ICalendarResourceType) -> Self {
        ICalendarValue::ResourceType(value)
    }
}

impl From<ICalendarProximityValue> for ICalendarValue {
    fn from(value: ICalendarProximityValue) -> Self {
        ICalendarValue::Proximity(value)
    }
}
