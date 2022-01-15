use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use super::{header::Header, Component, Content};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Responses {
    #[serde(flatten)]
    pub inner: BTreeMap<String, Response>,
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
    pub description: String,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, Header>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub content: HashMap<String, Content>,
}

impl Response {
    pub fn new<S: AsRef<str>>(description: S) -> Self {
        Self {
            description: description.as_ref().to_string(),
            ..Default::default()
        }
    }

    pub fn with_content<C: Into<Component>, S: AsRef<str>>(
        mut self,
        content_type: S,
        component: C,
    ) -> Self {
        self.content.insert(
            content_type.as_ref().to_string(),
            Content::new(component.into()),
        );

        self
    }

    pub fn with_header<S: AsRef<str>>(mut self, name: S, header: Header) -> Self {
        self.headers.insert(name.as_ref().to_string(), header);

        self
    }
}
