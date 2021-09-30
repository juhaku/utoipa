use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::Required;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub content: HashMap<String, Content>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Required>,
}

impl RequestBody {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_description<S: AsRef<str>>(mut self, description: S) -> Self {
        self.description = Some(description.as_ref().to_string());

        self
    }

    pub fn with_required(mut self, required: Required) -> Self {
        self.required = Some(required);

        self
    }

    pub fn with_content<S: AsRef<str>>(mut self, content_type: S, content: Content) -> Self {
        self.content
            .insert(content_type.as_ref().to_string(), content);

        self
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Content {
    // TODO implement schema somehow
    pub schema: String,
}
