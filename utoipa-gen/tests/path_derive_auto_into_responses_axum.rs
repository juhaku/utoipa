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
fn path_operation_auto_params_into_params_query() {
    use axum::extract::Query;
    use serde::Deserialize;

    #[derive(Deserialize, utoipa::IntoParams)]
    struct FindParams {
        id: i32,
    }

    #[utoipa::path(get, path = "/items", auto_params)]
    #[allow(unused)]
    async fn get_items(Query(params): Query<FindParams>) {}

    #[derive(OpenApi)]
    #[openapi(paths(get_items))]
    struct ApiDoc;

    let doc = ApiDoc::openapi();
    let value = serde_json::to_value(&doc).unwrap();
    let operation = value.pointer("/paths/~1items/get").unwrap();
    let parameters = operation.pointer("/parameters").unwrap();

    assert!(parameters.is_array());
}

#[test]
fn path_operation_auto_params_disabled() {
    use axum::extract::Query;
    use serde::Deserialize;

    #[derive(Deserialize, utoipa::IntoParams)]
    struct FindParams {
        id: i32,
    }   

    #[utoipa::path(get, path = "/items", auto_params = false)]
    #[allow(unused)]
    async fn get_items(Query(params): Query<FindParams>) {}

    #[derive(OpenApi)]
    #[openapi(paths(get_items))]
    struct ApiDoc;

    let doc = ApiDoc::openapi();
    let value = serde_json::to_value(&doc).unwrap();
    let operation = value.pointer("/paths/~1items/get").unwrap();
    let parameters = operation
        .pointer("/parameters")
        .unwrap_or(&serde_json::Value::Null);

    assert!(parameters.is_null());
}
