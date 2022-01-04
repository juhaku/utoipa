use actix_web::{delete, web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
// use utoipa::openapi_spec;
use utoipa::{path, OpenApi};

#[derive(Deserialize)]
struct Foo {
    id: i32,
}

// mod api {
//     use super::*;

/// Delete foo entity
///
/// Delete foo entity by what
#[crate::path(responses = [
    (200, "success", String),
    (400, "my bad error", u64),
    (404, "vault not found"),
    (500, "internal server error")
])]
#[delete("/foo/{id}")]
// #[deprecated = "this is deprecated"]
// web::Path(id): web::Path<i32>
async fn foo_delete(id: web::Path<Foo>) -> impl Responder {
    let id = id.id;
    HttpResponse::Ok().json(json!({ "deleted": id }))
}
// }

#[test]
fn derive_openapi() {
    // use crate::api::__path_foo_delete;
    #[derive(OpenApi, Default)]
    #[openapi(handler_files = [], handlers = [foo_delete])]
    struct ApiDoc;

    println!("{:?}", ApiDoc::openapi().to_json())
}
