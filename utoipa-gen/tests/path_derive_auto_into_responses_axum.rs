#![cfg(all(feature = "auto_into_responses", feature = "axum_extras"))]

use insta::assert_json_snapshot;
use utoipa::OpenApi;

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

    #[utoipa::path(get, path = "/item")]
    #[allow(unused)]
    async fn get_item() -> ItemResponse<'static> {
        ItemResponse::Success(Item { value: "super" })
    }

    #[derive(OpenApi)]
    #[openapi(paths(get_item))]
    struct ApiDoc;

    let doc = ApiDoc::openapi();
    let value = serde_json::to_value(&doc).unwrap();
    let path = value.pointer("/paths/~1item/get").unwrap();

    assert_json_snapshot!(&path.pointer("/responses").unwrap())
}

#[test]
fn path_operation_auto_types_axum_responders() {
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

    #[utoipa::path(get, path = "/status")]
    #[allow(unused)]
    async fn get_status() -> axum::http::StatusCode {
        axum::http::StatusCode::NO_CONTENT
    }

    #[utoipa::path(get, path = "/item")]
    #[allow(unused)]
    async fn get_item() -> Result<ItemResponse, axum::http::StatusCode> {
        Err(axum::http::StatusCode::NOT_FOUND)
    }

    #[derive(OpenApi)]
    #[openapi(paths(get_status, get_item))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    // axum responders cannot implement `IntoResponses`, so they document no responses
    let responses = doc.pointer("/paths/~1status/get/responses");
    assert_eq!(responses, Some(&serde_json::json!({})));

    let responses = doc.pointer("/paths/~1item/get/responses");
    assert_eq!(
        responses,
        Some(&serde_json::json!({
            "200": {
                "description": "Item found",
                "content": {
                    "application/json": { "schema": { "$ref": "#/components/schemas/Item" } }
                }
            }
        }))
    );
}

#[test]
fn path_operation_auto_types_axum_json_response() {
    use serde_json::json;

    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    struct Item {
        value: String,
    }

    #[utoipa::path(get, path = "/json")]
    #[allow(unused)]
    async fn get_json() -> axum::Json<Item> {
        axum::Json(Item {
            value: String::new(),
        })
    }

    #[utoipa::path(get, path = "/json-or-status")]
    #[allow(unused)]
    async fn get_json_or_status() -> Result<axum::Json<Item>, axum::http::StatusCode> {
        Err(axum::http::StatusCode::NOT_FOUND)
    }

    #[derive(OpenApi)]
    #[openapi(paths(get_json, get_json_or_status))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let json_item_responses = json!({
        "200": {
            "content": {
                "application/json": { "schema": { "$ref": "#/components/schemas/Item" } }
            }
        }
    });

    let responses = doc.pointer("/paths/~1json/get/responses");
    assert_eq!(responses, Some(&json_item_responses));

    let responses = doc.pointer("/paths/~1json-or-status/get/responses");
    assert_eq!(responses, Some(&json_item_responses));
}
