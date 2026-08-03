use std::net::SocketAddr;

use axum::{routing::get, Json};
use utoipa::{IntoParams, OpenApi, ToSchema};

#[derive(ToSchema)]
enum Bird {
    Sparrow,
    Pigeon,
    Crow,
    Coot,
}

#[derive(ToSchema)]
enum Beetle {
    Stag,
    Longhorn,
    Weevil,
}

#[derive(IntoParams, ToSchema)]
struct QueryParams<AnimalType: ToSchema> {
    animal: AnimalType,
}

/// Return JSON version of an OpenAPI schema
#[utoipa::path(
    get,
    path = "/api-docs/birds.json",
    responses(
        (status = 200, description = "JSON file", body = ())
    ),
    params(QueryParams<Bird>),
)]
async fn birds() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

/// Return JSON version of an OpenAPI schema
#[utoipa::path(
    get,
    path = "/api-docs/beetles.json",
    responses(
        (status = 200, description = "JSON file", body = ())
    ),
    params(QueryParams<Beetle>),
)]
async fn beetles() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[derive(OpenApi)]
#[openapi(paths(beetles, birds))]
struct ApiDoc {}

#[tokio::main]
async fn main() {
    let socket_address: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(socket_address).await.unwrap();

    let app = axum::Router::new()
        .route("/api-docs/beetles.json", get(beetles))
        .route("/api-docs/birds.json", get(birds));

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap()
}
