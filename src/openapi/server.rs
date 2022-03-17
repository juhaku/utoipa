use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>, // TODO check the correct type here
}

impl Server {
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
