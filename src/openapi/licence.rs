use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Licence {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl Licence {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Self {
            name: name.as_ref().to_string(),
            ..Default::default()
        }
    }

    pub fn with_url<S: AsRef<str>>(mut self, url: S) -> Self {
        self.url = Some(url.as_ref().to_string());

        self
    }
}
