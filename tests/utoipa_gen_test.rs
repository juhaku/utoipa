use actix_web::{delete, get, web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
// use utoipa::openapi_spec;
use utoipa::{path, OpenApi};

#[derive(Deserialize)]
struct Foo {
    ids: Vec<i32>,
}

// mod api {
//     use super::*;

/// Delete foo entity
///
/// Delete foo entity by what
#[crate::path(
    responses = [
        (200, "success", String),
        (400, "my bad error", u64),
        (404, "vault not found"),
        (500, "internal server error")
    ],
     params = [
        ("ids" = [i32], query, description = "Search foos by ids"),
   ]
)]
#[get("/foo")]
// #[deprecated = "this is deprecated"]
// web::Path(id): web::Path<i32>
async fn foo_delete(web::Query(foo): web::Query<Foo>) -> impl Responder {
    let ids = foo.ids;
    HttpResponse::Ok().json(json!({ "searched": ids }))
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
