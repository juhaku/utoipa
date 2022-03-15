use serde::{Deserialize, Serialize};

use super::external_docs::ExternalDocs;

/// Implements [OpenAPI Tag Object][tag].
///
/// Tag can be used to provide additional metadata for tags used by path operations.
///
/// [tag]: https://spec.openapis.org/oas/latest.html#tag-object
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocs>,
}

impl Tag {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Self {
            name: name.as_ref().to_string(),
            ..Default::default()
        }
    }

    pub fn with_description<S: AsRef<str>>(mut self, description: S) -> Self {
        self.description = Some(description.as_ref().to_string());

        self
    }

    pub fn with_external_docs(mut self, external_docs: ExternalDocs) -> Self {
        self.external_docs = Some(external_docs);

        self
    }
}
