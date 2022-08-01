//! Implements [OpenApi Responses][responses].
//!
//! [responses]: https://spec.openapis.org/oas/latest.html#responses-object
use std::collections::BTreeMap;

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
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        pub content: BTreeMap<String, Content>,
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

/// Trait with convenience functions for documenting response bodies.
///
/// This trait requires a feature-flag to enable:
/// ```text
/// [dependencies]
/// utoipa = { version = "1", features = ["openapi_extensions"] }
/// ```
///
/// Once enabled, with a single method call we can add [`Content`] to our ResponseBuilder
/// that references a responses (or component) schema using content-tpe "application/json":
///
/// ```rust
/// use utoipa::openapi::response::{ResponseBuilder, ResponseBuilderExt};
///
/// let request = ResponseBuilder::new()
///     .description("A sample response")
///     .json_response_ref("MyResponsePayload").build();
/// // Alternately for component, use
/// // let request = ResponseBuilder::new().json_component_ref("MyResponsePayload").build();
/// ```
///
/// If serialized to JSON, the above will result in a response schema like this:
///
/// ```json
/// {
///   "description": "A sample response",
///   "content": {
///     "application/json": {
///       "schema": {
///         "$ref": "#/components/responses/MyResponsePayload"
///       }
///     }
///   }
/// }
/// ```
///
#[cfg(feature = "openapi_extensions")]
pub trait ResponseBuilderExt {
    fn json_component_ref(self, ref_name: &str) -> Self;
    fn json_response_ref(self, ref_name: &str) -> Self;
}

#[cfg(feature = "openapi_extensions")]
impl ResponseBuilderExt for ResponseBuilder {
    /// Add [`Content`] to [`Response`] referring to a component-schema
    /// with Content-Type `application/json`.
    fn json_component_ref(self, ref_name: &str) -> ResponseBuilder {
        self.content(
            "application/json",
            Content::new(crate::openapi::Ref::from_component_name(ref_name)),
        )
    }

    /// Add [`Content`] to [`Response`] referring to a component-response
    /// with Content-Type `application/json`.
    fn json_response_ref(self, ref_name: &str) -> ResponseBuilder {
        self.content(
            "application/json",
            Content::new(crate::openapi::Ref::from_response_name(ref_name)),
        )
    }
}

#[cfg(test)]
mod tests {
    use assert_json_diff::assert_json_eq;
    use serde_json::json;
    use super::{Content, Responses, ResponseBuilder};

    #[test]
    fn responses_new() {
        let responses = Responses::new();

        assert!(responses.responses.is_empty());
    }

    #[test]
    fn response_builder() -> Result<(), serde_json::Error> {
        let request_body = ResponseBuilder::new()
            .description("A sample response")
            .content(
                "application/json",
                Content::new(crate::openapi::Ref::from_response_name("MyResponsePayload")),
            )
            .build();
        let serialized = serde_json::to_string_pretty(&request_body)?;
        println!("serialized json:\n {}", serialized);
        assert_json_eq!(
            request_body,
            json!({
            "description": "A sample response",
            "content": {
              "application/json": {
                "schema": {
                  "$ref": "#/components/responses/MyResponsePayload"
                }
              }
            }
          })
        );
        Ok(())
    }
    #[cfg(feature = "openapi_extensions")]
    use super::ResponseBuilderExt;

    #[cfg(feature = "openapi_extensions")]
    #[test]
    fn response_builder_ext() {
        let request_body = ResponseBuilder::new()
            .description("A sample response")
            .json_response_ref("MyResponsePayload").build();
        assert_json_eq!(
            request_body,
            json!({
                "description": "A sample response",
                "content": {
                  "application/json": {
                    "schema": {
                      "$ref": "#/components/responses/MyResponsePayload"
                    }
                  }
                }
              })
        );
    }
}
