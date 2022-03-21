//! Implements content object for request body and response.
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde_json")]
use serde_json::Value;

use super::{add_value, build_fn, from, new, Component};

/// Content holds request body content or response content.
#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[non_exhaustive]
pub struct Content {
    /// Schema used in response body or request body.
    pub schema: Component,

    /// Example for request body or response body.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "serde_json")]
    pub example: Option<Value>,

    /// Example for request body or response body.
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
}

/// Builder for [`Content`] with chainable configuration methods to create a new [`Content`].
#[derive(Default)]
pub struct ContentBuilder {
    schema: Component,

    #[cfg(feature = "serde_json")]
    example: Option<Value>,

    #[cfg(not(feature = "serde_json"))]
    example: Option<String>,
}

from!(Content ContentBuilder schema, example);

impl ContentBuilder {
    new!(pub ContentBuilder);

    /// Add schema.
    pub fn schema<I: Into<Component>>(mut self, component: I) -> Self {
        add_value!(self schema component.into())
    }

    /// Add example of schema.
    #[cfg(feature = "serde_json")]
    pub fn example(mut self, example: Option<Value>) -> Self {
        add_value!(self example example)
    }

    /// Add example of schema.
    #[cfg(not(feature = "serde_json"))]
    pub fn example<S: Into<String>>(mut self, example: Option<S>) -> Self {
        add_value!(self example example.into())
    }

    build_fn!(pub Content schema, example);
}
