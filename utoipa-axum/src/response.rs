//! Response types documenting their status code and body in the OpenAPI document.
//!
//! [`Status`] wraps a response body and sets the status code of the response. It implements
//! [`IntoResponses`], so the handler return type documents the response with the status code and
//! the content of the body, either in `responses(...)` of `#[utoipa::path]` or automatically with
//! the `auto_into_responses` feature of `utoipa`.
//!
//! # Examples
//!
//! _**Document a `201 Created` response with a JSON body.**_
//!
//! ```rust
//! use axum::Json;
//! use utoipa::OpenApi;
//! use utoipa_axum::response::{Created, Status};
//!
//! #[derive(serde::Serialize, utoipa::ToSchema)]
//! struct Pet {
//!     name: String,
//! }
//!
//! #[utoipa::path(post, path = "/pet", responses(Created<Json<Pet>>))]
//! async fn create_pet() -> Created<Json<Pet>> {
//!     Status(Json(Pet {
//!         name: String::from("Doggo"),
//!     }))
//! }
//!
//! #[derive(OpenApi)]
//! #[openapi(paths(create_pet))]
//! struct ApiDoc;
//!
//! let responses = ApiDoc::openapi().paths.paths["/pet"].post.as_ref().unwrap().responses.clone();
//! assert!(responses.responses.contains_key("201"));
//! ```
//!
//! _**Use any status code with the constants of the [`status`] module.**_
//!
//! ```rust
//! use axum::Json;
//! use utoipa_axum::response::{status, Status};
//!
//! type PetNotFound = Status<{ status::NOT_FOUND }, Json<String>>;
//! ```

use std::collections::BTreeMap;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use utoipa::openapi::content::ContentBuilder;
use utoipa::openapi::response::{Response, ResponseBuilder, ResponsesBuilder};
use utoipa::openapi::schema::Schema;
use utoipa::openapi::RefOr;
use utoipa::{IntoResponses, PartialSchema, ToSchema};

/// Status code constants for [`Status`], named like the constants of [`StatusCode`].
pub mod status {
    use axum::http::StatusCode;

    macro_rules! status_codes {
        ( $( $(#[$doc:meta])* $name:ident; )* ) => {
            $(
                $(#[$doc])*
                pub const $name: u16 = StatusCode::$name.as_u16();
            )*
        };
    }

    status_codes! {
        /// `100 Continue`
        CONTINUE;
        /// `101 Switching Protocols`
        SWITCHING_PROTOCOLS;
        /// `102 Processing`
        PROCESSING;
        /// `103 Early Hints`
        EARLY_HINTS;
        /// `200 OK`
        OK;
        /// `201 Created`
        CREATED;
        /// `202 Accepted`
        ACCEPTED;
        /// `203 Non Authoritative Information`
        NON_AUTHORITATIVE_INFORMATION;
        /// `204 No Content`
        NO_CONTENT;
        /// `205 Reset Content`
        RESET_CONTENT;
        /// `206 Partial Content`
        PARTIAL_CONTENT;
        /// `207 Multi-Status`
        MULTI_STATUS;
        /// `208 Already Reported`
        ALREADY_REPORTED;
        /// `226 IM Used`
        IM_USED;
        /// `300 Multiple Choices`
        MULTIPLE_CHOICES;
        /// `301 Moved Permanently`
        MOVED_PERMANENTLY;
        /// `302 Found`
        FOUND;
        /// `303 See Other`
        SEE_OTHER;
        /// `304 Not Modified`
        NOT_MODIFIED;
        /// `305 Use Proxy`
        USE_PROXY;
        /// `307 Temporary Redirect`
        TEMPORARY_REDIRECT;
        /// `308 Permanent Redirect`
        PERMANENT_REDIRECT;
        /// `400 Bad Request`
        BAD_REQUEST;
        /// `401 Unauthorized`
        UNAUTHORIZED;
        /// `402 Payment Required`
        PAYMENT_REQUIRED;
        /// `403 Forbidden`
        FORBIDDEN;
        /// `404 Not Found`
        NOT_FOUND;
        /// `405 Method Not Allowed`
        METHOD_NOT_ALLOWED;
        /// `406 Not Acceptable`
        NOT_ACCEPTABLE;
        /// `407 Proxy Authentication Required`
        PROXY_AUTHENTICATION_REQUIRED;
        /// `408 Request Timeout`
        REQUEST_TIMEOUT;
        /// `409 Conflict`
        CONFLICT;
        /// `410 Gone`
        GONE;
        /// `411 Length Required`
        LENGTH_REQUIRED;
        /// `412 Precondition Failed`
        PRECONDITION_FAILED;
        /// `413 Payload Too Large`
        PAYLOAD_TOO_LARGE;
        /// `414 URI Too Long`
        URI_TOO_LONG;
        /// `415 Unsupported Media Type`
        UNSUPPORTED_MEDIA_TYPE;
        /// `416 Range Not Satisfiable`
        RANGE_NOT_SATISFIABLE;
        /// `417 Expectation Failed`
        EXPECTATION_FAILED;
        /// `418 I'm a teapot`
        IM_A_TEAPOT;
        /// `421 Misdirected Request`
        MISDIRECTED_REQUEST;
        /// `422 Unprocessable Entity`
        UNPROCESSABLE_ENTITY;
        /// `423 Locked`
        LOCKED;
        /// `424 Failed Dependency`
        FAILED_DEPENDENCY;
        /// `425 Too Early`
        TOO_EARLY;
        /// `426 Upgrade Required`
        UPGRADE_REQUIRED;
        /// `428 Precondition Required`
        PRECONDITION_REQUIRED;
        /// `429 Too Many Requests`
        TOO_MANY_REQUESTS;
        /// `431 Request Header Fields Too Large`
        REQUEST_HEADER_FIELDS_TOO_LARGE;
        /// `451 Unavailable For Legal Reasons`
        UNAVAILABLE_FOR_LEGAL_REASONS;
        /// `500 Internal Server Error`
        INTERNAL_SERVER_ERROR;
        /// `501 Not Implemented`
        NOT_IMPLEMENTED;
        /// `502 Bad Gateway`
        BAD_GATEWAY;
        /// `503 Service Unavailable`
        SERVICE_UNAVAILABLE;
        /// `504 Gateway Timeout`
        GATEWAY_TIMEOUT;
        /// `505 HTTP Version Not Supported`
        HTTP_VERSION_NOT_SUPPORTED;
        /// `506 Variant Also Negotiates`
        VARIANT_ALSO_NEGOTIATES;
        /// `507 Insufficient Storage`
        INSUFFICIENT_STORAGE;
        /// `508 Loop Detected`
        LOOP_DETECTED;
        /// `510 Not Extended`
        NOT_EXTENDED;
        /// `511 Network Authentication Required`
        NETWORK_AUTHENTICATION_REQUIRED;
    }
}

/// Body of a response which can be documented as the content of an OpenAPI response.
pub trait ResponseContent {
    /// Media type and schema of the body. `T` of `Json<T>` is inlined, and the schemas it
    /// references are collected with [`ResponseContent::schemas`].
    fn content() -> (&'static str, RefOr<Schema>);

    /// Add the schemas referenced by the content to `schemas`, see [`ToSchema::schemas`].
    #[allow(unused)]
    fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
        // nothing by default
    }
}

impl<T: ToSchema> ResponseContent for Json<T> {
    fn content() -> (&'static str, RefOr<Schema>) {
        ("application/json", T::schema())
    }

    fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
        T::schemas(schemas);
    }
}

impl ResponseContent for String {
    fn content() -> (&'static str, RefOr<Schema>) {
        ("text/plain", String::schema())
    }
}

/// A response with the status code `CODE` and the body `B`.
///
/// The status code of the response of `B` is replaced with `CODE`, the same way as returning
/// `(StatusCode, B)` from a handler does.
pub struct Status<const CODE: u16, B>(pub B);

/// A `201 Created` response with the body `B`.
pub type Created<B> = Status<{ status::CREATED }, B>;

/// A `202 Accepted` response with the body `B`.
pub type Accepted<B> = Status<{ status::ACCEPTED }, B>;

impl<const CODE: u16, B> Status<CODE, B> {
    fn status_code() -> StatusCode {
        StatusCode::from_u16(CODE).expect("`Status` must have a valid status code")
    }
}

impl<const CODE: u16, B: ResponseContent> IntoResponses for Status<CODE, B> {
    fn responses() -> BTreeMap<String, RefOr<Response>> {
        let (content_type, schema) = B::content();
        let response = ResponseBuilder::new()
            .description(Self::status_code().canonical_reason().unwrap_or_default())
            .content(
                content_type,
                ContentBuilder::new().schema(Some(schema)).build(),
            );

        ResponsesBuilder::new()
            .response(CODE.to_string(), response)
            .build()
            .into()
    }

    fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
        B::schemas(schemas);
    }
}

impl<const CODE: u16, B: IntoResponse> IntoResponse for Status<CODE, B> {
    fn into_response(self) -> axum::response::Response {
        (Self::status_code(), self.0).into_response()
    }
}

/// A `204 No Content` response without a body.
pub struct NoContent;

impl IntoResponses for NoContent {
    fn responses() -> BTreeMap<String, RefOr<Response>> {
        let status_code = StatusCode::NO_CONTENT;
        let response =
            ResponseBuilder::new().description(status_code.canonical_reason().unwrap_or_default());

        ResponsesBuilder::new()
            .response(status_code.as_u16().to_string(), response)
            .build()
            .into()
    }
}

impl IntoResponse for NoContent {
    fn into_response(self) -> axum::response::Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use utoipa::OpenApi;

    use super::*;

    #[derive(serde::Serialize, utoipa::ToSchema)]
    struct Inner {
        value: String,
    }

    #[derive(serde::Serialize, utoipa::ToSchema)]
    struct Item {
        inner: Inner,
    }

    #[test]
    fn created_json_documents_status_and_content() {
        let responses = serde_json::to_value(Created::<Json<Item>>::responses()).unwrap();

        assert_eq!(
            responses,
            json!({
                "201": {
                    "description": "Created",
                    "content": {
                        "application/json": {
                            "schema": {
                                "type": "object",
                                "required": ["inner"],
                                "properties": {
                                    "inner": { "$ref": "#/components/schemas/Inner" }
                                }
                            }
                        }
                    }
                }
            })
        );

        let mut schemas = Vec::new();
        Created::<Json<Item>>::schemas(&mut schemas);
        let names = schemas
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, ["Inner"]);
    }

    #[test]
    fn created_json_registers_referenced_schemas_in_components() {
        #[utoipa::path(post, path = "/item", responses(Created<Json<Item>>))]
        #[allow(unused)]
        async fn create_item() -> Created<Json<Item>> {
            Status(Json(Item {
                inner: Inner {
                    value: String::new(),
                },
            }))
        }

        #[derive(OpenApi)]
        #[openapi(paths(create_item))]
        struct ApiDoc;

        let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
        let reference = doc.pointer(
            "/paths/~1item/post/responses/201/content/application~1json/schema/properties/inner/$ref",
        );

        assert_eq!(reference, Some(&json!("#/components/schemas/Inner")));
        assert!(doc.pointer("/components/schemas/Inner").is_some());
    }

    #[test]
    fn status_with_constant_documents_status_and_plain_text() {
        let responses =
            serde_json::to_value(Status::<{ status::NOT_FOUND }, String>::responses()).unwrap();

        assert_eq!(
            responses,
            json!({
                "404": {
                    "description": "Not Found",
                    "content": { "text/plain": { "schema": { "type": "string" } } }
                }
            })
        );
    }

    #[test]
    fn no_content_documents_status_without_content() {
        let responses = serde_json::to_value(NoContent::responses()).unwrap();

        assert_eq!(responses, json!({ "204": { "description": "No Content" } }));
    }

    #[test]
    fn status_responds_with_status_code() {
        let created: Created<Json<String>> = Status(Json(String::from("value")));
        assert_eq!(created.into_response().status(), StatusCode::CREATED);

        assert_eq!(NoContent.into_response().status(), StatusCode::NO_CONTENT);
    }
}
