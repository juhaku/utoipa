//! Implements [OpenAPI Request Body][request_body] types.
//!
//! [request_body]: https://spec.openapis.org/oas/latest.html#request-body-object
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{build_fn, builder, from, new, set_value, Content, Required};

builder! {
    RequestBodyBuilder;

    /// Implements [OpenAPI Request Body][request_body].
    ///
    /// [request_body]: https://spec.openapis.org/oas/latest.html#request-body-object
    #[non_exhaustive]
    #[derive(Serialize, Deserialize, Default, Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[serde(rename_all = "camelCase")]
    pub struct RequestBody {
        /// Additional description of [`RequestBody`] supporting markdown syntax.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,

        /// Map of request body contents mapped by content type e.g. `application/json`.
        pub content: BTreeMap<String, Content>,

        /// Determines whether request body is reuqired in the request or not.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub required: Option<Required>,
    }
}

impl RequestBody {
    /// Construct a new [`RequestBody`].
    pub fn new() -> Self {
        Default::default()
    }
}

impl RequestBodyBuilder {
    /// Add description for [`RequestBody`].
    pub fn description<S: Into<String>>(mut self, description: Option<S>) -> Self {
        set_value!(self description description.map(|description| description.into()))
    }

    /// Define [`RequestBody`] required.
    pub fn required(mut self, required: Option<Required>) -> Self {
        set_value!(self required required)
    }

    /// Add [`Content`] by content type e.g `application/json` to [`RequestBody`].
    pub fn content<S: Into<String>>(mut self, content_type: S, content: Content) -> Self {
        self.content.insert(content_type.into(), content);

        self
    }
}

/// Trait with convenience functions for documenting request bodies.
///
/// This trait requires a feature-flag to enable:
/// ```text
/// [dependencies]
/// utoipa = { version = "1", features = ["openapi_extensions"] }
/// ```
///
/// Once enabled, with a single method call we can add [`Content`] to our RequestBodyBuilder
/// that references a [`crate::ToSchema`] schema using content-tpe "application/json":
///
/// ```rust
/// use utoipa::openapi::request_body::{RequestBodyBuilder, RequestBodyExt};
///
/// let request = RequestBodyBuilder::new().json_component_ref("EmailPayload").build();
/// ```
///
/// If serialized to JSON, the above will result in a requestBody schema like this:
///
/// ```json
/// {
///   "content": {
///     "application/json": {
///       "schema": {
///         "$ref": "#/components/schemas/EmailPayload"
///       }
///     }
///   }
/// }
/// ```
///
#[cfg(feature = "openapi_extensions")]
pub trait RequestBodyExt {
    /// Add [`Content`] to [`RequestBody`] referring to a schema
    /// with Content-Type `application/json`.
    fn json_component_ref(self, ref_name: &str) -> Self;
}

#[cfg(feature = "openapi_extensions")]
impl RequestBodyExt for RequestBody {
    fn json_component_ref(mut self, ref_name: &str) -> RequestBody {
        self.content.insert(
            "application/json".to_string(),
            crate::openapi::Content::new(crate::openapi::Ref::from_schema_name(ref_name)),
        );
        self
    }
}

#[cfg(feature = "openapi_extensions")]
impl RequestBodyExt for RequestBodyBuilder {
    fn json_component_ref(self, ref_name: &str) -> RequestBodyBuilder {
        self.content(
            "application/json",
            crate::openapi::Content::new(crate::openapi::Ref::from_schema_name(ref_name)),
        )
    }
}

#[cfg(test)]
mod tests {
    use assert_json_diff::assert_json_eq;
    use serde_json::json;

    use super::{Content, RequestBody, RequestBodyBuilder, Required};

    #[test]
    fn request_body_new() {
        let request_body = RequestBody::new();

        assert!(request_body.content.is_empty());
        assert_eq!(request_body.description, None);
        assert!(request_body.required.is_none());
    }

    #[test]
    fn request_body_builder() -> Result<(), serde_json::Error> {
        let request_body = RequestBodyBuilder::new()
            .description(Some("A sample requestBody"))
            .required(Some(Required::True))
            .content(
                "application/json",
                Content::new(crate::openapi::Ref::from_schema_name("EmailPayload")),
            )
            .build();
        let serialized = serde_json::to_string_pretty(&request_body)?;
        println!("serialized json:\n {}", serialized);
        assert_json_eq!(
            request_body,
            json!({
              "description": "A sample requestBody",
              "content": {
                "application/json": {
                  "schema": {
                    "$ref": "#/components/schemas/EmailPayload"
                  }
                }
              },
              "required": true
            })
        );
        Ok(())
    }
    #[cfg(feature = "openapi_extensions")]
    use super::RequestBodyExt;

    #[cfg(feature = "openapi_extensions")]
    #[test]
    fn request_body_ext() {
        let request_body = RequestBodyBuilder::new()
            .build()
            // build a RequestBody first to test the method
            .json_component_ref("EmailPayload");
        assert_json_eq!(
            request_body,
            json!({
              "content": {
                "application/json": {
                  "schema": {
                    "$ref": "#/components/schemas/EmailPayload"
                  }
                }
              }
            })
        );
    }

    #[cfg(feature = "openapi_extensions")]
    #[test]
    fn request_body_builder_ext() {
        let request_body = RequestBodyBuilder::new()
            .json_component_ref("EmailPayload")
            .build();
        assert_json_eq!(
            request_body,
            json!({
              "content": {
                "application/json": {
                  "schema": {
                    "$ref": "#/components/schemas/EmailPayload"
                  }
                }
              }
            })
        );
    }
}
