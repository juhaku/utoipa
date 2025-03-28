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
