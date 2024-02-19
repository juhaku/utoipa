use std::net::SocketAddr;

use axum::{routing::get, Json};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(openapi))]
struct ApiDoc;

/// Return JSON version of an OpenAPI schema
#[utoipa::path(
    get,
    path = "/api-docs/openapi.json",
    responses(
        (status = 200, description = "JSON file", body = ())
    )
)]
async fn openapi() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[tokio::main]
async fn main() {
    let socket_address: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(socket_address).await.unwrap();

    let app = axum::Router::new().route("/api-docs/openapi.json", get(openapi));

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap()
}
