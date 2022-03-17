use serde::{Deserialize, Serialize};

#[cfg(feature = "serde_json")]
use serde_json::Value;

use super::Component;

#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[non_exhaustive]
pub struct Content {
    pub schema: Component,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "serde_json")]
    pub example: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(not(feature = "serde_json"))]
    pub example: Option<String>,
}

impl Content {
    pub fn new<I: Into<Component>>(schema: I) -> Self {
        Self {
            schema: schema.into(),
            example: None,
        }
    }

    #[cfg(feature = "serde_json")]
    pub fn with_example(mut self, example: Value) -> Self {
        self.example = Some(example);

        self
    }

    #[cfg(not(feature = "serde_json"))]
    pub fn with_example<S: Into<String>>(mut self, example: S) -> Self {
        self.example = Some(example.into());

        self
    }
}
