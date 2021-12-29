use actix_web::{delete, HttpResponse, Responder};
use serde_json::json;
// use utoipa::openapi_spec;
use utoipa::{path, OpenApi};

/// Delete foo entity
///
/// Delete foo entity by what
#[crate::path(delete, responses = [
    (200, "success", String),
    (400, "my bad error", u64),
    (404, "vault not found"),
    (500, "internal server error")
])]
#[delete("/foo")]
async fn foo_delete() -> impl Responder {
    HttpResponse::Ok().json(json!({"ok": "OK"}))
}

#[test]
fn derive_openapi() {
    #[derive(OpenApi, Default)]
    #[openapi(handler_files = [], handlers = [crate::foo_delete])]
    struct ApiDoc;

    println!("{:?}", ApiDoc::openapi().to_json())
}
