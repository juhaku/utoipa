
#[derive(utoipa::OpenApi)]
#[openapi(paths(
    get_openapi,
  )
)]
struct ApiDoc;

#[utoipa::path(
    get,
    path = "/openapi",
    responses(
      (status = 200, description = "YAML representation of this api in the OpenAPI v3.1.x format", body = str),
    ),
    extensions(
      (property = "x-amazon-apigateway-integration", value = json!({ "type": "mock" })),
    ),
)]
async fn get_openapi() -> impl actix_web::Responder {
  use utoipa::OpenApi;
  match ApiDoc::openapi().to_yaml() {
    Ok(yaml) => actix_web::HttpResponse::Ok().body(yaml),
    Err(e)   => actix_web::HttpResponse::InternalServerError().body(format!("{e:?}")),
  }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  env_logger::init();

  actix_web::HttpServer::new(move || {
    actix_web::App::new()
    .route("/openapi", actix_web::web::get().to(get_openapi))
  })
  .bind((std::net::Ipv4Addr::UNSPECIFIED, 8080))?
  .run()
  .await
}
