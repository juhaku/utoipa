#![cfg(all(feature = "auto_types", feature = "actix_extras"))]

use std::fmt::Display;
use utoipa::OpenApi;

use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{get, HttpResponse, Responder, ResponseError};
use assert_json_diff::assert_json_eq;

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

    assert_json_eq!(
        &path.pointer("/responses").unwrap(),
        serde_json::json!({
            "200": {
                "content": {
                    "application/json": {
                        "schema": {
                            "$ref": "#/components/schemas/Item"
                        }
                    }
                },
                "description": "Item found",
            },
            "404": {
                "description": "No item found"
            },
            "500": {
                "description": "Error"
            }
        })
    )
}
