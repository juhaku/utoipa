#![cfg(all(feature = "auto_into_responses", feature = "actix_extras"))]

use actix_web::web::{Form, Json};
use utoipa::OpenApi;

use actix_web::body::BoxBody;
use actix_web::http::header::ContentType;
use actix_web::{post, HttpResponse, Responder};
use insta::assert_json_snapshot;

// TODO this test is currently failing to compile
//
// #[test]
// fn path_operation_auto_types_responses() {
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
//         /// No item found
//         #[response(status = NOT_FOUND)]
//         NotFound,
//     }
//
//     /// Error
//     #[derive(Debug, utoipa::IntoResponses)]
//     #[response(status = 500)]
//     struct Error;
//
//     impl Display for Error {
//         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//             write!(f, "Error")
//         }
//     }
//
//     impl ResponseError for Error {}
//
//     impl Responder for ItemResponse<'static> {
//         type Body = BoxBody;
//
//         fn respond_to(self, _: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
//             match self {
//                 Self::Success(item) => HttpResponse::Ok()
//                     .content_type(ContentType::json())
//                     .body(serde_json::to_string(&item).expect("Item must serialize to json")),
//                 Self::NotFound => HttpResponse::NotFound().finish(),
//             }
//         }
//     }
//
//     #[utoipa::path]
//     #[get("/item")]
//     async fn get_item() -> Result<ItemResponse<'static>, Error> {
//         Ok(ItemResponse::Success(Item { value: "super" }))
//     }
//
//     #[derive(OpenApi)]
//     #[openapi(paths(get_item))]
//     struct ApiDoc;
//
//     let doc = ApiDoc::openapi();
//     let value = serde_json::to_value(&doc).unwrap();
//     let path = value.pointer("/paths/~1item/get").unwrap();
//
//     assert_json_snapshot!(&path.pointer("/responses").unwrap());
// }

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
