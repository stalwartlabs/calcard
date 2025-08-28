/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::{
    common::parser::{Boolean, Timestamp},
    vcard::*,
};

impl VCardEntry {
    pub fn new(name: VCardProperty) -> Self {
        Self {
            group: None,
            name,
            params: vec![],
            values: vec![],
        }
    }

    pub fn with_params(mut self, params: Vec<VCardParameter>) -> Self {
        self.params = params;
        self
    }

    pub fn with_group(mut self, group: Option<String>) -> Self {
        self.group = group;
        self
    }

    pub fn with_value(mut self, value: impl Into<VCardValue>) -> Self {
        self.values.push(value.into());
        self
    }

    pub fn with_values(mut self, values: Vec<VCardValue>) -> Self {
        self.values = values;
        self
    }

    pub fn with_param(mut self, param: impl Into<VCardParameter>) -> Self {
        self.params.push(param.into());
        self
    }

    pub fn add_param(&mut self, param: impl Into<VCardParameter>) {
        self.params.push(param.into());
    }

    pub fn with_param_opt(mut self, param: Option<impl Into<VCardParameter>>) -> Self {
        if let Some(param) = param {
            self.params.push(param.into());
        }
        self
    }

    pub fn is_type(&self, typ: &VCardValueType) -> bool {
        self.parameters(&VCardParameterName::Value)
            .any(|p| matches!(p, VCardParameterValue::ValueType(v) if v == typ))
    }
}

impl VCardParameter {
    pub fn new(name: VCardParameterName, value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name,
            value: value.into(),
        }
    }

    pub fn language(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Language,
            value: value.into(),
        }
    }

    pub fn value(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Value,
            value: value.into(),
        }
    }

    pub fn pref(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Pref,
            value: value.into(),
        }
    }

    pub fn altid(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Altid,
            value: value.into(),
        }
    }

    pub fn pid(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Pid,
            value: value.into(),
        }
    }

    pub fn typ(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Type,
            value: value.into(),
        }
    }

    pub fn mediatype(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Mediatype,
            value: value.into(),
        }
    }

    pub fn calscale(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Calscale,
            value: value.into(),
        }
    }

    pub fn sort_as(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::SortAs,
            value: value.into(),
        }
    }

    pub fn geo(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Geo,
            value: value.into(),
        }
    }

    pub fn tz(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Tz,
            value: value.into(),
        }
    }

    pub fn index(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Index,
            value: value.into(),
        }
    }

    pub fn level(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Level,
            value: value.into(),
        }
    }

    pub fn group(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Group,
            value: value.into(),
        }
    }

    pub fn cc(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Cc,
            value: value.into(),
        }
    }

    pub fn author(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Author,
            value: value.into(),
        }
    }

    pub fn author_name(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::AuthorName,
            value: value.into(),
        }
    }

    pub fn created(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Created,
            value: value.into(),
        }
    }

    pub fn derived(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Derived,
            value: value.into(),
        }
    }

    pub fn label(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Label,
            value: value.into(),
        }
    }

    pub fn phonetic(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Phonetic,
            value: value.into(),
        }
    }

    pub fn prop_id(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::PropId,
            value: value.into(),
        }
    }

    pub fn script(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Script,
            value: value.into(),
        }
    }

    pub fn service_type(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::ServiceType,
            value: value.into(),
        }
    }

    pub fn username(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Username,
            value: value.into(),
        }
    }

    pub fn jsptr(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Jsptr,
            value: value.into(),
        }
    }

    pub fn jscomps(value: impl Into<VCardParameterValue>) -> Self {
        VCardParameter {
            name: VCardParameterName::Jscomps,
            value: value.into(),
        }
    }
}

impl From<String> for VCardValue {
    fn from(text: String) -> Self {
        VCardValue::Text(text)
    }
}

impl From<VCardKind> for VCardValue {
    fn from(kind: VCardKind) -> Self {
        VCardValue::Kind(kind)
    }
}

impl From<String> for VCardParameterValue {
    fn from(value: String) -> Self {
        VCardParameterValue::Text(value)
    }
}

impl From<u32> for VCardParameterValue {
    fn from(value: u32) -> Self {
        VCardParameterValue::Integer(value)
    }
}

impl From<VCardType> for VCardParameterValue {
    fn from(value: VCardType) -> Self {
        VCardParameterValue::Type(value)
    }
}

impl From<VCardValueType> for VCardParameterValue {
    fn from(value: VCardValueType) -> Self {
        VCardParameterValue::ValueType(value)
    }
}

impl From<CalendarScale> for VCardParameterValue {
    fn from(value: CalendarScale) -> Self {
        VCardParameterValue::Calscale(value)
    }
}

impl From<VCardPhonetic> for VCardParameterValue {
    fn from(value: VCardPhonetic) -> Self {
        VCardParameterValue::Phonetic(value)
    }
}

impl From<VCardLevel> for VCardParameterValue {
    fn from(value: VCardLevel) -> Self {
        VCardParameterValue::Level(value)
    }
}

impl From<bool> for VCardParameterValue {
    fn from(value: bool) -> Self {
        VCardParameterValue::Bool(value)
    }
}

impl From<Vec<Jscomp>> for VCardParameterValue {
    fn from(value: Vec<Jscomp>) -> Self {
        VCardParameterValue::Jscomps(value)
    }
}

impl From<Timestamp> for VCardParameterValue {
    fn from(value: Timestamp) -> Self {
        VCardParameterValue::Timestamp(value.0)
    }
}

impl From<Boolean> for VCardParameterValue {
    fn from(value: Boolean) -> Self {
        VCardParameterValue::Bool(value.0)
    }
}

impl<T: Into<VCardParameterValue>> From<IanaType<T, String>> for VCardParameterValue {
    fn from(value: IanaType<T, String>) -> Self {
        match value {
            IanaType::Iana(v) => v.into(),
            IanaType::Other(s) => s.into(),
        }
    }
}
