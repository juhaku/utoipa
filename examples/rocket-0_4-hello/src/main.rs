#![feature(decl_macro)]
use std::{io::Cursor, path::PathBuf, sync::Arc};

use rocket::{
    get,
    http::{Header, RawStr, Status},
    routes, Response, State,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::Config;

fn main() {
    #[derive(OpenApi)]
    #[openapi(paths(hello))]
    struct ApiDoc;

    rocket::ignite()
        .manage(Arc::new(Config::from("/api-doc/openapi.json")))
        .manage(ApiDoc::openapi())
        .mount("/", routes![hello, serve_api_doc, serve_swagger])
        .launch();
}

#[get("/swagger-ui/<tail..>")]
fn serve_swagger(tail: PathBuf, config: State<Arc<Config>>) -> Response<'static> {
    match utoipa_swagger_ui::serve(tail.as_os_str().to_str().unwrap(), config.clone()) {
        Ok(file) => file
            .map(|file| {
                Response::build()
                    .sized_body(Cursor::new(file.bytes.to_vec()))
                    .header(Header::new("Content-Type", file.content_type))
                    .finalize()
            })
            .unwrap_or_else(|| Response::build().status(Status::NotFound).finalize()),
        Err(error) => {
            let error = error.to_string();
            let len = error.len() as u64;

            Response::build()
                .raw_body(rocket::response::Body::Sized(Cursor::new(error), len))
                .status(Status::InternalServerError)
                .finalize()
        }
    }
}

#[get("/api-doc/openapi.json")]
fn serve_api_doc(openapi: State<utoipa::openapi::OpenApi>) -> Response<'static> {
    let json_string = serde_json::to_string(openapi.inner()).unwrap();
    let len = json_string.len() as u64;

    Response::build()
        .raw_body(rocket::response::Body::Sized(Cursor::new(json_string), len))
        .header(Header::new("Content-Type", "application/json"))
        .finalize()
}

#[utoipa::path(
    responses(
        (status = 200, description = "Hello response for given value", body = String, content_type = "text/plain")
    ),
    params(
        ("value" = String, description = "Say hello by value")
    )
)]
#[get("/hello/<value>")]
fn hello(value: &RawStr) -> String {
    format!("Hello {value}!")
}
