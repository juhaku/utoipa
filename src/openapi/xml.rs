use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Xml {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Cow<'static, str>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<Cow<'static, str>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<Cow<'static, str>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrapped: Option<bool>,
}

impl Xml {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn name<S: Into<Cow<'static, str>>>(mut self, name: Option<S>) -> Self {
        self.name = name.map(|name| name.into());

        self
    }

    pub fn namespace<S: Into<Cow<'static, str>>>(mut self, namespace: Option<S>) -> Self {
        self.namespace = namespace.map(|namespace| namespace.into());

        self
    }

    pub fn prefix<S: Into<Cow<'static, str>>>(mut self, prefix: Option<S>) -> Self {
        self.prefix = prefix.map(|prefix| prefix.into());

        self
    }

    pub fn attribute(mut self, attribute: Option<bool>) -> Self {
        self.attribute = attribute;

        self
    }

    pub fn wrapped(mut self, wrapped: Option<bool>) -> Self {
        self.wrapped = wrapped;

        self
    }
}
