#![cfg(feature = "actix_extras")]
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
// use utoipa::openapi_spec;
use utoipa::{Component, OpenApi};

#[derive(Deserialize, Serialize, Component)]
struct Foo {
    ids: Vec<i32>,
}

// mod api {
//     use super::*;

/// Delete foo entity
///
/// Delete foo entity by what
#[utoipa::path(
    request_body = (content = Foo, required, description = "foobar", content_type = "text/xml"), 
    responses = [
        (status = 200, description = "success response", body = [Foo], headers = [("xrfs-token" = u64)])
        // (400, "my bad error", u64),
        // (404, "vault not found"),
        // (500, "internal server error")
    ],
     params = [
        ("ids" = [i32], query, description = "Search foos by ids"),
   ]
)]
#[get("/foo/{_:.*}")]
// #[deprecated = "this is deprecated"]
// web::Path(id): web::Path<i32>
async fn foo_delete(web::Query(foo): web::Query<Foo>) -> impl Responder {
    let ids = foo.ids;
    HttpResponse::Ok().json(json!({ "searched": ids }))
}
// }

#[test]
#[ignore = "this is just a test bed to run macros"]
fn derive_openapi() {
    // use crate::api::__path_foo_delete;
    #[derive(OpenApi, Default)]
    #[openapi(handlers = [foo_delete], components = [Foo])]
    struct ApiDoc;

    println!("{}", ApiDoc::openapi().to_pretty_json().unwrap());
}
