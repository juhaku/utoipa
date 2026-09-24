#![cfg(all(feature = "auto_into_responses", feature = "actix_extras"))]

use std::fmt::Display;

use actix_web::web::{Form, Json};
use utoipa::OpenApi;

use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{get, post, HttpResponse, Responder, ResponseError};
use insta::assert_json_snapshot;

#[test]
fn path_operation_auto_types_responses() {
    /// Test item to to return
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    struct Item<'s> {
        value: &'s str,
    }

    #[derive(utoipa::IntoResponses)]
    #[allow(unused)]
    enum ItemResponse<'s> {
        /// Item found
        #[response(status = 200)]
        Success(Item<'s>),
        /// No item found
        #[response(status = NOT_FOUND)]
        NotFound,
    }

    /// Error
    #[derive(Debug, utoipa::IntoResponses)]
    #[response(status = 500)]
    struct Error;

    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Error")
        }
    }

    impl ResponseError for Error {}

    impl Responder for ItemResponse<'static> {
        type Body = BoxBody;

        fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
            match self {
                Self::Success(item) => HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(serde_json::to_string(&item).expect("Item must serialize to json")),
                Self::NotFound => HttpResponse::NotFound().finish(),
            }
        }
    }

    #[utoipa::path]
    #[get("/item")]
    async fn get_item() -> Result<ItemResponse<'static>, Error> {
        Ok(ItemResponse::Success(Item { value: "super" }))
    }

    #[derive(OpenApi)]
    #[openapi(paths(get_item))]
    struct ApiDoc;

    let doc = ApiDoc::openapi();
    let value = serde_json::to_value(&doc).unwrap();
    let path = value.pointer("/paths/~1item/get").unwrap();

    assert_json_snapshot!(&path.pointer("/responses").unwrap());
}

#[test]
fn path_operation_auto_types_fn_parameters() {
    /// Test item to to return
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    struct Item<'s> {
        value: &'s str,
    }

    #[derive(utoipa::IntoResponses)]
    #[allow(unused)]
    enum ItemResponse<'s> {
        /// Item found
        #[response(status = 200)]
        Success(Item<'s>),
        /// No item found
        #[response(status = NOT_FOUND)]
        NotFound,
    }

    impl Responder for ItemResponse<'static> {
        type Body = BoxBody;

        fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
            match self {
                Self::Success(item) => HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(serde_json::to_string(&item).expect("Item must serialize to json")),
                Self::NotFound => HttpResponse::NotFound().finish(),
            }
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    struct ItemBody {
        value: String,
    }

    #[utoipa::path]
    #[post("/item")]
    #[allow(unused)]
    async fn post_item(item: Json<ItemBody>) -> ItemResponse<'static> {
        ItemResponse::Success(Item { value: "super" })
    }

    #[derive(OpenApi)]
    #[openapi(paths(post_item), components(schemas(ItemBody)))]
    struct ApiDoc;

    let doc = ApiDoc::openapi();
    let value = serde_json::to_value(&doc).unwrap();
    let path = value.pointer("/paths/~1item/post").unwrap();

    assert_json_snapshot!(&path.pointer("/responses").unwrap());
    assert_json_snapshot!(&path.pointer("/requestBody"));
}

#[test]
fn path_operation_optional_json_body() {
    /// Test item to to return
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    struct Item<'s> {
        value: &'s str,
    }

    #[derive(utoipa::IntoResponses)]
    #[allow(unused)]
    enum ItemResponse<'s> {
        /// Item found
        #[response(status = 200)]
        Success(Item<'s>),
        /// No item found
        #[response(status = NOT_FOUND)]
        NotFound,
    }

    impl Responder for ItemResponse<'static> {
        type Body = BoxBody;

        fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
            match self {
                Self::Success(item) => HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(serde_json::to_string(&item).expect("Item must serialize to json")),
                Self::NotFound => HttpResponse::NotFound().finish(),
            }
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    struct ItemBody {
        value: String,
    }

    #[utoipa::path]
    #[post("/item")]
    #[allow(unused)]
    async fn post_item(item: Option<Json<ItemBody>>) -> ItemResponse<'static> {
        ItemResponse::Success(Item { value: "super" })
    }

    #[derive(OpenApi)]
    #[openapi(paths(post_item), components(schemas(ItemBody)))]
    struct ApiDoc;

    let doc = ApiDoc::openapi();
    let value = serde_json::to_value(&doc).unwrap();
    let path = value.pointer("/paths/~1item/post").unwrap();

    assert_json_snapshot!(&path.pointer("/responses").unwrap());
    assert_json_snapshot!(&path.pointer("/requestBody"));
}

#[test]
fn path_operation_auto_types_tuple() {
    /// Test item to to return
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    struct Item<'s> {
        value: &'s str,
    }

    #[derive(utoipa::IntoResponses)]
    #[allow(unused)]
    enum ItemResponse<'s> {
        /// Item found
        #[response(status = 200)]
        Success(Item<'s>),
    }

    impl Responder for ItemResponse<'static> {
        type Body = BoxBody;

        fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
            match self {
                Self::Success(item) => HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(serde_json::to_string(&item).expect("Item must serialize to json")),
            }
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    struct ItemBody {
        value: String,
    }

    #[utoipa::path]
    #[post("/item")]
    #[allow(unused)]
    async fn post_item(item: Json<(ItemBody, String)>) -> ItemResponse<'static> {
        ItemResponse::Success(Item { value: "super" })
    }

    #[derive(OpenApi)]
    #[openapi(paths(post_item), components(schemas(ItemBody)))]
    struct ApiDoc;

    let doc = ApiDoc::openapi();
    let value = serde_json::to_value(&doc).unwrap();
    let path = value.pointer("/paths/~1item/post").unwrap();

    assert_json_snapshot!(&path.pointer("/requestBody"));
}

// TODO this test is currently failing to compile
//
// #[test]
// fn path_operation_request_body_bytes() {
//     /// Test item to to return
//     #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
//     struct Item<'s> {
//         value: &'s str,
//     }
//
//     #[derive(utoipa::IntoResponses)]
//     #[allow(unused)]
//     enum ItemResponse<'s> {
//         /// Item found
//         #[response(status = 200)]
//         Success(Item<'s>),
//     }
//
//     impl Responder for ItemResponse<'static> {
//         type Body = BoxBody;
//
//         fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
//             match self {
//                 Self::Success(item) => HttpResponse::Ok()
//                     .content_type(ContentType::json())
//                     .body(serde_json::to_string(&item).expect("Item must serialize to json")),
//             }
//         }
//     }
//
//     #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
//     struct ItemBody {
//         value: String,
//     }
//
//     #[utoipa::path]
//     #[post("/item")]
//     #[allow(unused)]
//     async fn post_item(item: actix_web::web::Bytes) -> ItemResponse<'static> {
//         ItemResponse::Success(Item { value: "super" })
//     }
//
//     #[derive(OpenApi)]
//     #[openapi(paths(post_item), components(schemas(ItemBody)))]
//     struct ApiDoc;
//
//     let doc = ApiDoc::openapi();
//     let value = serde_json::to_value(&doc).unwrap();
//     let path = value.pointer("/paths/~1item/post").unwrap();
//
//     assert_json_snapshot!(&path.pointer("/requestBody"));
// }

#[test]
fn path_operation_request_body_form() {
    /// Test item to to return
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    struct Item<'s> {
        value: &'s str,
    }

    #[derive(utoipa::IntoResponses)]
    #[allow(unused)]
    enum ItemResponse<'s> {
        /// Item found
        #[response(status = 200)]
        Success(Item<'s>),
    }

    impl Responder for ItemResponse<'static> {
        type Body = BoxBody;

        fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
            match self {
                Self::Success(item) => HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(serde_json::to_string(&item).expect("Item must serialize to json")),
            }
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    struct ItemBody {
        value: String,
    }

    #[utoipa::path]
    #[post("/item")]
    #[allow(unused)]
    async fn post_item(item: Form<ItemBody>) -> ItemResponse<'static> {
        ItemResponse::Success(Item { value: "super" })
    }

    #[derive(OpenApi)]
    #[openapi(paths(post_item), components(schemas(ItemBody)))]
    struct ApiDoc;

    let doc = ApiDoc::openapi();
    let value = serde_json::to_value(&doc).unwrap();
    let path = value.pointer("/paths/~1item/post").unwrap();

    assert_json_snapshot!(&path.pointer("/requestBody"))
}

#[test]
fn path_operation_auto_types_skip_actix_responders() {
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    struct Item {
        value: String,
    }

    #[utoipa::path]
    #[post("/http-response")]
    #[allow(unused)]
    async fn http_response() -> HttpResponse {
        HttpResponse::Ok().finish()
    }

    #[utoipa::path]
    #[post("/json")]
    #[allow(unused)]
    async fn json() -> Json<Item> {
        Json(Item {
            value: String::new(),
        })
    }

    mod result_alias {
        use actix_web::{post, HttpResponse, Result};

        #[utoipa::path]
        #[post("/result-alias")]
        #[allow(unused)]
        pub async fn result_alias() -> Result<HttpResponse> {
            Ok(HttpResponse::Ok().finish())
        }
    }

    #[derive(OpenApi)]
    #[openapi(paths(http_response, json, result_alias::result_alias))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    // actix-web responders cannot implement `IntoResponses`, so they are left out
    for path in [
        "/paths/~1http-response",
        "/paths/~1json",
        "/paths/~1result-alias",
    ] {
        let responses = doc.pointer(&format!("{path}/post/responses"));
        assert_eq!(responses, Some(&serde_json::json!({})), "{path}");
    }
}

#[test]
fn path_operation_auto_types_result_with_actix_types() {
    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    struct Item {
        value: String,
    }

    #[derive(utoipa::IntoResponses)]
    #[allow(unused)]
    enum ItemResponse {
        /// Item found
        #[response(status = 200)]
        Success(Item),
    }

    impl Responder for ItemResponse {
        type Body = BoxBody;

        fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
            match self {
                Self::Success(item) => HttpResponse::Ok().json(item),
            }
        }
    }

    /// Error
    #[derive(Debug, utoipa::IntoResponses)]
    #[response(status = 500)]
    struct Error;

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Error")
        }
    }

    impl actix_web::ResponseError for Error {}

    #[utoipa::path]
    #[post("/item-or-error")]
    #[allow(unused)]
    async fn item_or_error() -> Result<ItemResponse, Error> {
        Err(Error)
    }

    #[utoipa::path]
    #[post("/json-or-error")]
    #[allow(unused)]
    async fn json_or_error() -> Result<Json<Item>, Error> {
        Err(Error)
    }

    #[utoipa::path]
    #[post("/actix-result")]
    #[allow(unused)]
    async fn actix_result() -> actix_web::Result<ItemResponse> {
        Err(actix_web::error::ErrorInternalServerError("error"))
    }

    #[derive(OpenApi)]
    #[openapi(paths(item_or_error, json_or_error, actix_result))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let item_response = serde_json::json!({
        "description": "Item found",
        "content": {
            "application/json": { "schema": { "$ref": "#/components/schemas/Item" } }
        }
    });
    let error_response = serde_json::json!({ "description": "Error" });

    let responses = doc.pointer("/paths/~1item-or-error/post/responses");
    assert_eq!(
        responses,
        Some(&serde_json::json!({ "200": item_response, "500": error_response }))
    );

    // `Json<Item>` cannot implement `IntoResponses`, only the error responses are documented
    let responses = doc.pointer("/paths/~1json-or-error/post/responses");
    assert_eq!(
        responses,
        Some(&serde_json::json!({ "500": error_response }))
    );

    // `actix_web::Error` cannot implement `IntoResponses`, only the success responses are documented
    let responses = doc.pointer("/paths/~1actix-result/post/responses");
    assert_eq!(
        responses,
        Some(&serde_json::json!({ "200": item_response }))
    );
}
