//! Implements [OpenApi Responses][responses].
//!
//! [responses]: https://spec.openapis.org/oas/latest.html#responses-object
use std::collections::BTreeMap;
use indexmap::{IndexMap};

use serde::{Deserialize, Serialize};

use super::{build_fn, builder, from, header::Header, new, set_value, Content};

builder! {
    ResponsesBuilder;

    /// Implements [OpenAPI Responses Object][responses].
    ///
    /// Responses is a map holding api operation responses identified by their status code.
    ///
    /// [responses]: https://spec.openapis.org/oas/latest.html#responses-object
    #[non_exhaustive]
    #[derive(Serialize, Deserialize, Default, Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[serde(rename_all = "camelCase")]
    pub struct Responses {
        /// Map containing status code as a key with represented response as a value.
        #[serde(flatten)]
        pub responses: BTreeMap<String, Response>,
    }
}

impl Responses {
    pub fn new() -> Self {
        Default::default()
    }
}

impl ResponsesBuilder {
    /// Add response to responses.
    pub fn response<S: Into<String>, R: Into<Response>>(mut self, code: S, response: R) -> Self {
        self.responses.insert(code.into(), response.into());

        self
    }
}

impl<C> FromIterator<(C, Response)> for Responses
where
    C: Into<String>,
{
    fn from_iter<T: IntoIterator<Item = (C, Response)>>(iter: T) -> Self {
        Self {
            responses: BTreeMap::from_iter(
                iter.into_iter()
                    .map(|(code, response)| (code.into(), response)),
            ),
        }
    }
}

builder! {
    ResponseBuilder;

    /// Implements [OpenAPI Response Object][response].
    ///
    /// Response is api operation response.
    ///
    /// [response]: https://spec.openapis.org/oas/latest.html#response-object
    #[non_exhaustive]
    #[derive(Serialize, Deserialize, Default, Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[serde(rename_all = "camelCase")]
    pub struct Response {
        /// Description of the response. Response support markdown syntax.
        pub description: String,

        /// Map of headers identified by their name. `Content-Type` header will be ignored.
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        pub headers: BTreeMap<String, Header>,

        /// Map of response [`Content`] objects identified by response body content type e.g `application/json`.
        #[serde(skip_serializing_if = "IndexMap::is_empty")]
        pub content: IndexMap<String, Content>,
    }
}

impl Response {
    /// Construct a new [`Response`].
    ///
    /// Function takes description as argument.
    pub fn new<S: Into<String>>(description: S) -> Self {
        Self {
            description: description.into(),
            ..Default::default()
        }
    }
}

impl ResponseBuilder {
    /// Add description. Description supports markdown syntax.
    pub fn description<I: Into<String>>(mut self, description: I) -> Self {
        set_value!(self description description.into())
    }

    /// Add [`Content`] of the [`Response`] with content type e.g `application/json`.
    pub fn content<S: Into<String>>(mut self, content_type: S, content: Content) -> Self {
        self.content.insert(content_type.into(), content);

        self
    }

    /// Add response [`Header`].
    pub fn header<S: Into<String>>(mut self, name: S, header: Header) -> Self {
        self.headers.insert(name.into(), header);

        self
    }
}

#[cfg(test)]
mod tests {
    use super::Responses;

    #[test]
    fn responses_new() {
        let responses = Responses::new();

        assert!(responses.responses.is_empty());
    }
}
