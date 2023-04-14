use std::{error::Error, net::Ipv4Addr};

use actix_web::{
    middleware::Logger, patch, App, HttpResponse, HttpServer, Responder, Result, web::Json,
};
use serde_json::Value;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[utoipa::path(
    request_body = Value,
    responses(
        (status = 200, description = "Patch completed"),
        (status = 406, description = "Not accepted"),
    ),
    security(
        ("api_key" = [])
    ),
)]
#[patch("/patch_raw")]
pub async fn patch_raw(body: Json<Value>) -> Result<impl Responder> {
    let value: Value = body.into_inner();
    eprintln!("body = {:?}", value);
    Ok(HttpResponse::Ok())
}

#[actix_web::main]
async fn main() -> Result<(), impl Error> {
    env_logger::init();

    #[derive(OpenApi)]
    #[openapi(paths(patch_raw))]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(patch_raw)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await
}
