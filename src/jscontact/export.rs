/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    jscontact::{JSContact, JSContactKind, JSContactProperty, JSContactValue},
    vcard::{VCard, VCardEntry, VCardKind, VCardProperty},
};
use jmap_tools::{Key, Value};

impl JSContact<'_> {
    pub fn into_vcard(self) -> Option<VCard> {
        let mut vcard = VCard::default();

        for (property, value) in self.0.into_object()?.into_vec() {
            let Key::Property(property) = property else {
                continue;
            };

            match property {
                JSContactProperty::Uid => {
                    if let Some(text) = value.into_string() {
                        vcard
                            .entries
                            .push(VCardEntry::new(VCardProperty::Uid).with_value(text));
                    }
                }
                JSContactProperty::Kind => match value {
                    Value::Element(JSContactValue::Kind(kind)) => {
                        let kind = match kind {
                            JSContactKind::Application => VCardKind::Application,
                            JSContactKind::Device => VCardKind::Device,
                            JSContactKind::Group => VCardKind::Group,
                            JSContactKind::Individual => VCardKind::Individual,
                            JSContactKind::Location => VCardKind::Location,
                            JSContactKind::Org => VCardKind::Org,
                            _ => continue,
                        };
                        vcard
                            .entries
                            .push(VCardEntry::new(VCardProperty::Kind).with_value(kind));
                    }
                    Value::Str(text) => {
                        vcard.entries.push(
                            VCardEntry::new(VCardProperty::Kind).with_value(text.into_owned()),
                        );
                    }
                    _ => (),
                },
                JSContactProperty::Language => todo!(),
                JSContactProperty::Members => todo!(),
                JSContactProperty::ProdId => todo!(),
                JSContactProperty::Created => todo!(),
                JSContactProperty::Updated => todo!(),
                JSContactProperty::Name => todo!(),
                JSContactProperty::Nicknames => todo!(),
                JSContactProperty::Organizations => todo!(),
                JSContactProperty::SpeakToAs => todo!(),
                JSContactProperty::Titles => todo!(),
                JSContactProperty::Emails => todo!(),
                JSContactProperty::OnlineServices => todo!(),
                JSContactProperty::Phones => todo!(),
                JSContactProperty::PreferredLanguages => todo!(),
                JSContactProperty::Calendars => todo!(),
                JSContactProperty::SchedulingAddresses => todo!(),
                JSContactProperty::Addresses => todo!(),
                JSContactProperty::Directories => todo!(),
                JSContactProperty::Links => todo!(),
                JSContactProperty::Media => todo!(),
                JSContactProperty::Localizations => todo!(),
                JSContactProperty::Keywords => todo!(),
                JSContactProperty::Anniversaries => todo!(),
                JSContactProperty::Notes => todo!(),
                JSContactProperty::PersonalInfo => todo!(),
                JSContactProperty::RelatedTo => todo!(),
                JSContactProperty::VCardName => todo!(),
                JSContactProperty::VCardParams => todo!(),
                JSContactProperty::VCardProps => todo!(),
                _ => (),
            }
        }

        Some(vcard)
    }
}
