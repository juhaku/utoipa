use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDocs {
    pub url: String,
    pub description: Option<String>,
}

impl ExternalDocs {
    pub fn new<S: AsRef<str>>(url: S) -> Self {
        Self {
            url: url.as_ref().to_string(),
            ..Default::default()
        }
    }

    pub fn with_description<S: AsRef<str>>(mut self, description: S) -> Self {
        self.description = Some(description.as_ref().to_string());

        self
    }
}
