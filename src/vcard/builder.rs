/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use crate::vcard::{
    VCardEntry, VCardKind, VCardParameter, VCardProperty, VCardValue, VCardValueType,
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
        self.params
            .iter()
            .any(|p| matches!(p, VCardParameter::Value(v) if v.contains(typ)))
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
