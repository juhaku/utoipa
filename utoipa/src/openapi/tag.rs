//! Implements [OpenAPI Tag Object][tag] types.
//!
//! [tag]: https://spec.openapis.org/oas/latest.html#tag-object
use serde::{Deserialize, Serialize};

use super::{builder, extensions::Extensions, external_docs::ExternalDocs, set_value};

builder! {
    TagBuilder;

    /// Implements [OpenAPI Tag Object][tag].
    ///
    /// Tag can be used to provide additional metadata for tags used by path operations.
    ///
    /// [tag]: https://spec.openapis.org/oas/latest.html#tag-object
    #[non_exhaustive]
    #[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[serde(rename_all = "camelCase")]
    pub struct Tag {
        /// Name of the tag. Should match to tag of **operation**.
        pub name: String,

        /// Short summary for the tag.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub summary: Option<String>,

        /// Additional description for the tag shown in the document.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,

        /// Name of this tag's parent tag.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub parent: Option<String>,

        /// Kind of tag, allowing tooling to distinguish grouping tags from other tag purposes.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub kind: Option<String>,

        /// Additional external documentation for the tag.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub external_docs: Option<ExternalDocs>,

        /// Optional extensions "x-something".
        #[serde(skip_serializing_if = "Option::is_none", flatten)]
        pub extensions: Option<Extensions>,
    }
}

impl Tag {
    /// Construct a new [`Tag`] with given name.
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Self {
            name: name.as_ref().to_string(),
            ..Default::default()
        }
    }
}

impl TagBuilder {
    /// Add name of the tag.
    pub fn name<I: Into<String>>(mut self, name: I) -> Self {
        set_value!(self name name.into())
    }

    /// Add short summary for the tag.
    pub fn summary<S: Into<String>>(mut self, summary: Option<S>) -> Self {
        set_value!(self summary summary.map(|summary| summary.into()))
    }

    /// Add additional description for the tag.
    pub fn description<S: Into<String>>(mut self, description: Option<S>) -> Self {
        set_value!(self description description.map(|description| description.into()))
    }

    /// Add parent tag name.
    pub fn parent<S: Into<String>>(mut self, parent: Option<S>) -> Self {
        set_value!(self parent parent.map(|parent| parent.into()))
    }

    /// Add tag kind.
    pub fn kind<S: Into<String>>(mut self, kind: Option<S>) -> Self {
        set_value!(self kind kind.map(|kind| kind.into()))
    }

    /// Add additional external documentation for the tag.
    pub fn external_docs(mut self, external_docs: Option<ExternalDocs>) -> Self {
        set_value!(self external_docs external_docs)
    }

    /// Add openapi extensions (x-something) to the tag.
    pub fn extensions(mut self, extensions: Option<Extensions>) -> Self {
        set_value!(self extensions extensions)
    }
}
