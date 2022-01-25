use serde::{Deserialize, Serialize};

use super::{Component, ComponentType, Property};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
pub struct Header {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub schema: Component,
}

impl Header {
    pub fn new<C: Into<Component>>(component: C) -> Self {
        Self {
            schema: component.into(),
            ..Default::default()
        }
    }

    pub fn with_description<S: AsRef<str>>(mut self, description: S) -> Self {
        self.description = Some(description.as_ref().to_string());

        self
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            description: Default::default(),
            schema: Property::new(ComponentType::String).into(),
        }
    }
}
