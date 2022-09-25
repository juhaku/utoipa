//! Implements content object for request body and response.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[cfg(feature = "serde_json")]
use serde_json::Value;

use super::{build_fn, encoding::Encoding, from, new, schema::RefOr, set_value, Schema};

/// Content holds request body content or response content.
#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[non_exhaustive]
pub struct Content {
    /// Schema used in response body or request body.
    pub schema: RefOr<Schema>,

    /// Example for request body or response body.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "serde_json")]
    pub example: Option<Value>,

    /// Example for request body or response body.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(not(feature = "serde_json"))]
    pub example: Option<String>,

    /// A map between a property name and its encoding information.
    ///
    /// The key, being the property name, MUST exist in the [`Content::schema`] as a property, with
    /// `schema` being a [`Schema::Object`] and this object containing the same property key in
    /// [`Object::properties`](crate::openapi::schema::Object::properties).
    ///
    /// The encoding object SHALL only apply to `request_body` objects when the media type is
    /// multipart or `application/x-www-form-urlencoded`.
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub encoding: BTreeMap<String, Encoding>,
}

impl Content {
    pub fn new<I: Into<RefOr<Schema>>>(schema: I) -> Self {
        Self {
            schema: schema.into(),
            ..Self::default()
        }
    }
}

/// Builder for [`Content`] with chainable configuration methods to create a new [`Content`].
#[derive(Default)]
pub struct ContentBuilder {
    schema: RefOr<Schema>,

    #[cfg(feature = "serde_json")]
    example: Option<Value>,

    #[cfg(not(feature = "serde_json"))]
    example: Option<String>,

    encoding: BTreeMap<String, Encoding>,
}

from!(Content ContentBuilder schema, example, encoding);

impl ContentBuilder {
    new!(pub ContentBuilder);

    /// Add schema.
    pub fn schema<I: Into<RefOr<Schema>>>(mut self, component: I) -> Self {
        set_value!(self schema component.into())
    }

    /// Add example of schema.
    #[cfg(feature = "serde_json")]
    pub fn example(mut self, example: Option<Value>) -> Self {
        set_value!(self example example)
    }

    /// Add example of schema.
    #[cfg(not(feature = "serde_json"))]
    pub fn example<S: Into<String>>(mut self, example: Option<S>) -> Self {
        set_value!(self example example.map(|example| example.into()))
    }

    /// Add an encoding.
    ///
    /// The `property_name` MUST exist in the [`Content::schema`] as a property,
    /// with `schema` being a [`Schema::Object`] and this object containing the same property
    /// key in [`Object::properties`](crate::openapi::schema::Object::properties).
    ///
    /// The encoding object SHALL only apply to `request_body` objects when the media type is
    /// multipart or `application/x-www-form-urlencoded`.
    pub fn encoding<S: Into<String>, E: Into<Encoding>>(
        mut self,
        property_name: S,
        encoding: E,
    ) -> Self {
        self.encoding.insert(property_name.into(), encoding.into());
        self
    }

    build_fn!(pub Content schema, example, encoding);
}
