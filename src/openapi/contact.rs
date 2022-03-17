use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

impl Contact {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_name<S: AsRef<str>>(mut self, name: S) -> Self {
        self.name = Some(name.as_ref().to_string());

        self
    }

    pub fn with_url<S: AsRef<str>>(mut self, url: S) -> Self {
        self.url = Some(url.as_ref().to_string());

        self
    }

    pub fn with_email<S: AsRef<str>>(mut self, email: S) -> Self {
        self.email = Some(email.as_ref().to_string());

        self
    }
}
