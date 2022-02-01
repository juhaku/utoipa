use serde::{Deserialize, Serialize};

use super::{contact::Contact, licence::License};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_of_service: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,

    pub version: String,
}

impl Info {
    pub fn new<S: AsRef<str>>(title: S, version: S) -> Self {
        Self {
            title: title.as_ref().to_string(),
            version: version.as_ref().to_string(),
            ..Default::default()
        }
    }

    pub fn with_description<S: AsRef<str>>(mut self, description: S) -> Self {
        self.description = Some(description.as_ref().to_string());

        self
    }

    pub fn with_terms_of_service<S: AsRef<str>>(mut self, terms_of_service: S) -> Self {
        self.terms_of_service = Some(terms_of_service.as_ref().to_string());

        self
    }

    pub fn with_contact(mut self, contanct: Contact) -> Self {
        self.contact = Some(contanct);

        self
    }

    pub fn with_license(mut self, license: License) -> Self {
        self.license = Some(license);

        self
    }
}
