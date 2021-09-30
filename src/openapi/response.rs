use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Responses {
    #[serde(flatten)]
    pub inner: HashMap<String, Response>,
}

impl Responses {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_response<S: AsRef<str>>(mut self, code: S, response: Response) -> Self {
        self.inner.insert(code.as_ref().to_string(), response);

        self
    }
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    // TODO add missing fields
    pub description: String,
}

impl Response {
    pub fn new<S: AsRef<str>>(description: S) -> Self {
        Self {
            description: description.as_ref().to_string(),
        }
    }
}
