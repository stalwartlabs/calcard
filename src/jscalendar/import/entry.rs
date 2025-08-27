/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    icalendar::{
        ICalendarAction, ICalendarComponent, ICalendarComponentType, ICalendarParameter,
        ICalendarParameterName, ICalendarParameterValue, ICalendarProperty, ICalendarValue,
    },
    jscalendar::{
        JSCalendar, JSCalendarAlertAction, JSCalendarDateTime, JSCalendarProperty, JSCalendarType,
        JSCalendarValue,
        import::{EntryState, State},
    },
};
use jmap_tools::{JsonPointer, Key, Value};

impl ICalendarComponent {
    pub(crate) fn entries_to_jscalendar(&mut self) -> State {
        let mut state = State::default();

        for entry in std::mem::take(&mut self.entries) {
            let mut entry = EntryState::new(entry);
            let mut values = std::mem::take(&mut entry.entry.values).into_iter();

            match (&entry.entry.name, values.next(), &self.component_type) {
                (
                    ICalendarProperty::Acknowledged,
                    Some(ICalendarValue::PartialDateTime(value)),
                    ICalendarComponentType::VAlarm,
                ) if value.has_date_and_time() => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Acknowledged),
                        Value::Element(JSCalendarValue::DateTime(JSCalendarDateTime::new(
                            value.to_timestamp().unwrap(),
                            false,
                        ))),
                    );
                    entry
                        .set_converted_to(&[JSCalendarProperty::Acknowledged.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Action,
                    Some(ICalendarValue::Action(
                        value @ (ICalendarAction::Display | ICalendarAction::Email),
                    )),
                    ICalendarComponentType::VAlarm,
                ) => {
                    state.entries.insert(
                        Key::Property(JSCalendarProperty::Action),
                        Value::Element(JSCalendarValue::AlertAction(match value {
                            ICalendarAction::Display => JSCalendarAlertAction::Display,
                            ICalendarAction::Email => JSCalendarAlertAction::Email,
                            _ => unreachable!(),
                        })),
                    );
                    entry.set_converted_to(&[JSCalendarProperty::Action.to_string().as_ref()]);
                }
                (
                    ICalendarProperty::Attach,
                    Some(ICalendarValue::Uri(uri)),
                    ICalendarComponentType::VEvent
                    | ICalendarComponentType::VTodo
                    | ICalendarComponentType::Participant
                    | ICalendarComponentType::VLocation
                    | ICalendarComponentType::VCalendar,
                ) => {
                    state.map_named_entry(
                        &mut entry,
                        &[
                            ICalendarParameterName::Fmttype,
                            ICalendarParameterName::Label,
                            ICalendarParameterName::Linkrel,
                            ICalendarParameterName::Size,
                        ],
                        JSCalendarProperty::Links,
                        (
                            Key::Property(JSCalendarProperty::Href),
                            Value::Str(uri.to_string().into()),
                        ),
                        [(
                            Key::Property(JSCalendarProperty::Type),
                            Value::Element(JSCalendarValue::Type(JSCalendarType::Link)),
                        )],
                    );
                    entry.set_map_name();
                }
                (
                    ICalendarProperty::Calscale,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Method,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Prodid,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Version,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}

                (
                    ICalendarProperty::Categories,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Class,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Comment,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Description,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Geo,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Location,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::PercentComplete,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Priority,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Resources,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Status,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Summary,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Completed,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Dtend,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Due,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Dtstart,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Duration,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Freebusy,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Transp,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Tzid,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Tzname,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Tzoffsetfrom,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Tzoffsetto,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Tzurl,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Attendee,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Contact,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Organizer,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::RecurrenceId,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::RelatedTo,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Url,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Uid,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Exdate,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Exrule,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Rdate,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Rrule,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}

                (
                    ICalendarProperty::Repeat,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Trigger,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Created,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Dtstamp,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::LastModified,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Sequence,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::RequestStatus,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Xml,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Tzuntil,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::TzidAliasOf,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Busytype,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Name,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::RefreshInterval,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Source,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Color,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Image,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Conference,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::CalendarAddress,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::LocationType,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::ParticipantType,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::ResourceType,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::StructuredData,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::StyledDescription,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}

                (
                    ICalendarProperty::Proximity,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Concept,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Link,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Refid,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::Coordinates,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (
                    ICalendarProperty::ShowWithoutTime,
                    Some(ICalendarValue::Text(value)),
                    ICalendarComponentType::VEvent | ICalendarComponentType::VTodo,
                ) => {}
                (ICalendarProperty::Jsid, Some(ICalendarValue::Text(value)), _) => {
                    state.jsid = Some(value);
                }
                (ICalendarProperty::Jsprop, Some(ICalendarValue::Text(value)), _) => {
                    if let Some(ICalendarParameter {
                        name: ICalendarParameterName::Jsptr,
                        value: ICalendarParameterValue::Text(ptr),
                    }) = entry.entry.params.first()
                    {
                        let ptr = JsonPointer::<JSCalendarProperty>::parse(ptr);

                        if let Ok(jscalendar) = JSCalendar::parse(&value) {
                            state.patch_objects.push((ptr, jscalendar.0.into_owned()));
                            continue;
                        }
                    }
                    entry.entry.values = [ICalendarValue::Text(value)]
                        .into_iter()
                        .chain(values)
                        .collect();
                }
                (ICalendarProperty::Begin | ICalendarProperty::End, _, _) => {
                    continue;
                }

                (_, value, _) => {
                    entry.entry.values = [value].into_iter().flatten().chain(values).collect();
                }
            }

            state.add_conversion_props(entry);
        }

        state
    }
}
