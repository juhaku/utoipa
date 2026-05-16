//! Implements [OpenAPI Header Object][header] types.
//!
//! [header]: https://spec.openapis.org/oas/latest.html#header-object

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{builder, set_value, Content, Object, RefOr, Schema, Type};

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

        /// Map of media type representations for the header.
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        pub content: BTreeMap<String, RefOr<Content>>,

        /// Additional description of the header value.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
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
            description: Default::default(),
            schema: Some(Object::with_type(Type::String).into()),
            content: Default::default(),
        }
    }
}

impl HeaderBuilder {
    /// Add schema of header.
    pub fn schema<I: Into<RefOr<Schema>>>(mut self, component: Option<I>) -> Self {
        set_value!(self schema component.map(Into::into))
    }

    /// Add media type representation for the header.
    pub fn content<S: Into<String>, C: Into<RefOr<Content>>>(
        mut self,
        content_type: S,
        content: C,
    ) -> Self {
        self.content.insert(content_type.into(), content.into());
        self
    }

    /// Add additional description for header.
    pub fn description<S: Into<String>>(mut self, description: Option<S>) -> Self {
        set_value!(self description description.map(|description| description.into()))
    }
}
