//! Implements [OpenAPI Metadata][info] types.
//!
//! Refer to [`OpenApi`][openapi_trait] trait and [derive documentation][derive]
//! for examples and usage details.
//!
//! [info]: <https://spec.openapis.org/oas/latest.html#info-object>
//! [openapi_trait]: ../../trait.OpenApi.html
//! [derive]: ../../derive.OpenApi.html
use serde::{Deserialize, Serialize};

use super::{builder, set_value};

builder! {
    /// # Examples
    ///
    /// Create [`Info`] using [`InfoBuilder`].
    /// ```rust
    /// # use utoipa::openapi::{Info, InfoBuilder, ContactBuilder};
    /// let info = InfoBuilder::new()
    ///      .title("My api")
    ///      .version("1.0.0")
    ///      .contact(Some(ContactBuilder::new()
    ///           .name(Some("Admin Admin"))
    ///           .email(Some("amdin@petapi.com"))
    ///           .build()
    ///       ))
    ///      .build();
    /// ```
    InfoBuilder;

    /// OpenAPI [Info][info] object represents metadata of the API.
    ///
    /// You can use [`Info::new`] to construct a new [`Info`] object or alternatively use [`InfoBuilder::new`]
    /// to construct a new [`Info`] with chainable configuration methods.
    ///
    /// [info]: <https://spec.openapis.org/oas/latest.html#info-object>
    #[non_exhaustive]
    #[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[serde(rename_all = "camelCase")]
    pub struct Info {
        /// Title of the API.
        pub title: String,

        /// Optional description of the API.
        ///
        /// Value supports markdown syntax.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,

        /// Optional url for terms of service.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub terms_of_service: Option<String>,

        /// Contact information of exposed API.
        ///
        /// See more details at: <https://spec.openapis.org/oas/latest.html#contact-object>.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub contact: Option<Contact>,

        /// License of the API.
        ///
        /// See more details at: <https://spec.openapis.org/oas/latest.html#license-object>.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub license: Option<License>,

        /// Document version typically the API version.
        pub version: String,
    }
}

impl Info {
    /// Construct a new [`Info`] object.
    ///
    /// This function accepts two arguments. One which is the title of the API and two the
    /// version of the api document typically the API version.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use utoipa::openapi::Info;
    /// let info = Info::new("Pet api", "1.1.0");
    /// ```
    pub fn new<S: Into<String>>(title: S, version: S) -> Self {
        Self {
            title: title.into(),
            version: version.into(),
            ..Default::default()
        }
    }
}

impl InfoBuilder {
    /// Add title of the API.
    pub fn title<I: Into<String>>(mut self, title: I) -> Self {
        set_value!(self title title.into())
    }

    /// Add version of the api document typically the API version.
    pub fn version<I: Into<String>>(mut self, version: I) -> Self {
        set_value!(self version version.into())
    }

    /// Add description of the API.
    pub fn description<S: Into<String>>(mut self, description: Option<S>) -> Self {
        set_value!(self description description.map(|description| description.into()))
    }

    /// Add url for terms of the API.
    pub fn terms_of_service<S: Into<String>>(mut self, terms_of_service: Option<S>) -> Self {
        set_value!(self terms_of_service terms_of_service.map(|terms_of_service| terms_of_service.into()))
    }

    /// Add contact information of the API.
    pub fn contact(mut self, contact: Option<Contact>) -> Self {
        set_value!(self contact contact)
    }

    /// Add license of the API.
    pub fn license(mut self, license: Option<License>) -> Self {
        set_value!(self license license)
    }
}

builder! {
    /// See the [`InfoBuilder`] for combined usage example.
    ContactBuilder;

    /// OpenAPI [Contact][contact] information of the API.
    ///
    /// You can use [`Contact::new`] to construct a new [`Contact`] object or alternatively
    /// use [`ContactBuilder::new`] to construct a new [`Contact`] with chainable configuration methods.
    ///
    /// [contact]: <https://spec.openapis.org/oas/latest.html#contact-object>
    #[non_exhaustive]
    #[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[serde(rename_all = "camelCase")]
    pub struct Contact {
        /// Identifying name of the contact person or organization of the API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,

        /// Url pointing to contact information of the API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub url: Option<String>,

        /// Email of the contact person or the organization of the API.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub email: Option<String>,
    }
}

impl Contact {
    /// Construct a new [`Contact`].
    pub fn new() -> Self {
        Default::default()
    }
}

impl ContactBuilder {
    /// Add name contact person or organization of the API.
    pub fn name<S: Into<String>>(mut self, name: Option<S>) -> Self {
        set_value!(self name name.map(|name| name.into()))
    }

    /// Add url pointing to the contact information of the API.
    pub fn url<S: Into<String>>(mut self, url: Option<S>) -> Self {
        set_value!(self url url.map(|url| url.into()))
    }

    /// Add email of the contact person or organization of the API.
    pub fn email<S: Into<String>>(mut self, email: Option<S>) -> Self {
        set_value!(self email email.map(|email| email.into()))
    }
}

builder! {
    LicenseBuilder;

    /// OpenAPI [License][license] information of the API.
    ///
    /// [license]: <https://spec.openapis.org/oas/latest.html#license-object>
    #[non_exhaustive]
    #[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[serde(rename_all = "camelCase")]
    pub struct License {
        /// Name of the license used e.g MIT or Apache-2.0
        pub name: String,

        /// Optional url pointing to the license.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub url: Option<String>,
    }
}

impl License {
    /// Construct a new [`License`] object.
    ///
    /// Function takes name of the license as an argument e.g MIT.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

impl LicenseBuilder {
    /// Add name of the license used in API.
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        set_value!(self name name.into())
    }

    /// Add url pointing to the license used in API.
    pub fn url<S: Into<String>>(mut self, url: Option<S>) -> Self {
        set_value!(self url url.map(|url| url.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::Contact;

    #[test]
    fn contact_new() {
        let contact = Contact::new();

        assert!(contact.name.is_none());
        assert!(contact.url.is_none());
        assert!(contact.email.is_none());
    }
}
