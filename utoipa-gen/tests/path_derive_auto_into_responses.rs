#![cfg(feature = "auto_into_responses")]

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
fn path_operation_auto_types_default_response_type() {
    #[utoipa::path(get, path = "/item")]
    #[allow(unused)]
    async fn post_item() {}

    #[derive(OpenApi)]
    #[openapi(paths(post_item))]
    struct ApiDoc;

    let doc = ApiDoc::openapi();
    let value = serde_json::to_value(&doc).unwrap();
    let path = value.pointer("/paths/~1item/get").unwrap();

    assert_json_snapshot!(&path.pointer("/responses").unwrap())
}

#[test]
fn path_operation_auto_types_without_into_responses() {
    #[utoipa::path(get, path = "/string")]
    #[allow(unused)]
    async fn get_string() -> String {
        String::new()
    }

    #[derive(OpenApi)]
    #[openapi(paths(get_string))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let responses = doc.pointer("/paths/~1string/get/responses");

    // A return type without `IntoResponses` documents no responses instead of failing to compile
    assert_eq!(responses, Some(&serde_json::json!({})));
}

#[test]
fn path_operation_auto_types_result_alias() {
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

    /// Error
    #[derive(utoipa::IntoResponses)]
    #[response(status = 500)]
    struct Error;

    type Result<T> = std::result::Result<T, Error>;

    #[utoipa::path(get, path = "/item")]
    #[allow(unused)]
    async fn get_item() -> Result<ItemResponse> {
        Err(Error)
    }

    #[derive(OpenApi)]
    #[openapi(paths(get_item))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let responses = doc.pointer("/paths/~1item/get/responses");

    // The error type of the alias is resolved through the `IntoResponses` implementation of `Result`
    assert_eq!(
        responses,
        Some(&serde_json::json!({
            "200": {
                "description": "Item found",
                "content": {
                    "application/json": { "schema": { "$ref": "#/components/schemas/Item" } }
                }
            },
            "500": { "description": "Error" }
        }))
    );
}

#[test]
fn path_operation_auto_types_json_without_framework_extras() {
    #[derive(serde::Serialize, utoipa::ToSchema)]
    struct Item {
        value: String,
    }

    struct Json<T>(T);

    #[utoipa::path(get, path = "/json")]
    #[allow(unused)]
    async fn get_json() -> Json<Item> {
        Json(Item {
            value: String::new(),
        })
    }

    #[derive(OpenApi)]
    #[openapi(paths(get_json))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let responses = doc.pointer("/paths/~1json/get/responses");

    // Without framework extras a type named `Json` is not a JSON response of a web framework
    assert_eq!(responses, Some(&serde_json::json!({})));
}
