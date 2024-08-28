use std::net::{Ipv4Addr, SocketAddr};

use axum::{routing, Router};
use std::io::Error;
use tokio::net::TcpListener;
use utoipa::openapi::path::Operation;
use utoipa::openapi::{OpenApiBuilder, PathItem, PathsBuilder};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> Result<(), Error> {
    #[derive(OpenApi)]
    #[openapi(
        nest(
            // you can nest sub apis here
            (path = "/api/v1/ones", api = one::OneApi)
        )
    )]
    struct ApiDoc;

    #[derive(OpenApi)]
    #[openapi()]
    struct HelloApi;

    let hello_api =
        Into::<OpenApiBuilder>::into(HelloApi::openapi()).paths(PathsBuilder::new().path(
            "",
            PathItem::new(utoipa::openapi::HttpMethod::Get, Operation::new()),
        ));

    let mut doc = ApiDoc::openapi();
    doc = doc.nest("/hello", hello_api); // you can even nest programmatically apis

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", doc))
        .route("/hello", routing::get(|| async { "hello" }))
        .nest("/api/v1/ones", one::router());

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080));
    let listener = TcpListener::bind(&address).await?;
    axum::serve(listener, app.into_make_service()).await
}

mod one {
    use axum::{routing, Router};
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(paths(get_one))]
    pub(super) struct OneApi;

    pub(super) fn router() -> Router {
        Router::new().route("/one", routing::get(get_one))
    }

    #[utoipa::path(
        get,
        path = "/one",
        responses(
            (status = OK, description = "One result ok", body = str)
        )
    )]
    async fn get_one() -> &'static str {
        "one"
    }
}
