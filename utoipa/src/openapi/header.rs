//! Implements [OpenAPI Header Object][header] types.
//!
//! [header]: https://spec.openapis.org/oas/latest.html#header-object

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use super::{
    builder, content::Content, example::Example, extensions::Extensions, path::ParameterStyle,
    set_value, Deprecated, Object, RefOr, Schema, Type,
};

builder! {
    HeaderBuilder;

    /// Implements [OpenAPI Header Object][header] for response headers.
    ///
    /// [header]: https://spec.openapis.org/oas/latest.html#header-object
    #[non_exhaustive]
    #[derive(Serialize, Deserialize, Clone, PartialEq)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Header {
        /// Schema of header type.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub schema: Option<RefOr<Schema>>,

        /// Additional description of the header value.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,

        /// Declares the header deprecated status.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub deprecated: Option<Deprecated>,

        /// Describes how the header value will be serialized.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub style: Option<ParameterStyle>,

        /// When _`true`_ it will generate separate header value for each value with _`array`_ and _`object`_ type.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub explode: Option<bool>,

        /// Example of the header potential value.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub example: Option<Value>,

        /// Examples of the header potential values.
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        pub examples: BTreeMap<String, RefOr<Example>>,

        /// A map containing the representations for the header.
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        pub content: BTreeMap<String, Content>,

        /// Optional extensions "x-something".
        #[serde(skip_serializing_if = "Option::is_none", flatten)]
        pub extensions: Option<Extensions>,
    }
}

impl Header {
    /// Construct a new [`Header`] with custom schema. If you wish to construct a default
    /// header with `String` type you can use [`Header::default`] function.
    ///
    /// # Examples
    ///
    /// Create new [`Header`] with integer type.
    /// ```rust
    /// # use utoipa::openapi::header::Header;
    /// # use utoipa::openapi::{Object, Type};
    /// let header = Header::new(Object::with_type(Type::Integer));
    /// ```
    ///
    /// Create a new [`Header`] with default type `String`
    /// ```rust
    /// # use utoipa::openapi::header::Header;
    /// let header = Header::default();
    /// ```
    pub fn new<C: Into<RefOr<Schema>>>(component: C) -> Self {
        Self {
            schema: Some(component.into()),
            ..Default::default()
        }
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            schema: Some(Object::with_type(Type::String).into()),
            description: None,
            deprecated: None,
            style: None,
            explode: None,
            example: None,
            examples: BTreeMap::new(),
            content: BTreeMap::new(),
            extensions: None,
        }
    }
}

impl HeaderBuilder {
    /// Add schema of header.
    pub fn schema<I: Into<RefOr<Schema>>>(mut self, component: I) -> Self {
        set_value!(self schema Some(component.into()))
    }

    /// Add additional description for header.
    pub fn description<S: Into<String>>(mut self, description: Option<S>) -> Self {
        set_value!(self description description.map(|description| description.into()))
    }

    /// Add or change [`Header`] deprecated status.
    pub fn deprecated(mut self, deprecated: Option<Deprecated>) -> Self {
        set_value!(self deprecated deprecated)
    }

    /// Add or change serialization style of [`Header`].
    pub fn style(mut self, style: Option<ParameterStyle>) -> Self {
        set_value!(self style style)
    }

    /// Define whether [`Header`]s are exploded or not.
    pub fn explode(mut self, explode: Option<bool>) -> Self {
        set_value!(self explode explode)
    }

    /// Add or change example of [`Header`]'s potential value.
    pub fn example(mut self, example: Option<Value>) -> Self {
        set_value!(self example example)
    }

    /// Add examples from iterator.
    pub fn examples_from_iter<
        E: IntoIterator<Item = (N, V)>,
        N: Into<String>,
        V: Into<RefOr<Example>>,
    >(
        mut self,
        examples: E,
    ) -> Self {
        self.examples.extend(
            examples
                .into_iter()
                .map(|(name, example)| (name.into(), example.into())),
        );

        self
    }

    /// Add media type content representation to [`Header`].
    pub fn content_from_iter<E: IntoIterator<Item = (N, V)>, N: Into<String>, V: Into<Content>>(
        mut self,
        content: E,
    ) -> Self {
        self.content.extend(
            content
                .into_iter()
                .map(|(name, content)| (name.into(), content.into())),
        );

        self
    }

    /// Add openapi extensions (x-something) to the [`Header`].
    pub fn extensions(mut self, extensions: Option<Extensions>) -> Self {
        set_value!(self extensions extensions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    use crate::openapi::content::ContentBuilder;
    use crate::openapi::example::ExampleBuilder;

    #[test]
    fn test_header_builder_and_serialization() {
        let header = HeaderBuilder::new()
            .description(Some("custom header"))
            .deprecated(Some(Deprecated::True))
            .style(Some(ParameterStyle::Simple))
            .explode(Some(true))
            .example(Some(json!("example-value")))
            .build();

        insta::assert_json_snapshot!(&header, @r#"
        {
          "schema": {
            "type": "string"
          },
          "description": "custom header",
          "deprecated": true,
          "style": "simple",
          "explode": true,
          "example": "example-value"
        }
        "#);
    }

    #[test]
    fn test_header_with_content_and_examples() {
        let content = ContentBuilder::new()
            .schema(Some(Object::with_type(Type::Integer)))
            .build();
        let example = ExampleBuilder::new().value(Some(json!("test"))).build();

        let header = Header {
            schema: None,
            content: BTreeMap::from_iter([("application/json".to_string(), content)]),
            examples: BTreeMap::from_iter([("test_example".to_string(), example.into())]),
            ..Default::default()
        };

        insta::assert_json_snapshot!(&header, @r#"
        {
          "examples": {
            "test_example": {
              "value": "test"
            }
          },
          "content": {
            "application/json": {
              "schema": {
                "type": "integer"
              }
            }
          }
        }
        "#);
    }
}
