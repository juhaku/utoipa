use hyperlane::*;
use utoipa::{OpenApi, ToSchema};
use utoipa_rapidoc::RapiDoc;
use serde::Serialize;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Serialize, ToSchema)]
struct User {
    name: String,
    age: usize,
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(User)),
    info(title = "Hello World", version = "1.0.0"),
    paths(openapi_json, swagger_handler)
)]
struct ApiDoc;

#[utoipa::path(
    get,
    path = "/api/openapi.json",   
    responses(
        (status = 200, description = "Response docs", body = User)
    )
)]
async fn openapi_json(ctx: Context) {
    ctx.send_response(200, ApiDoc::openapi().to_json().unwrap())
        .await
        .unwrap();
}

#[utoipa::path(
    get,
    path = "/{file}",   
    responses(
        (status = 200, description = "Response docs", body = User)
    )
)]
async fn swagger_handler(ctx: Context) {
    SwaggerUi::new("/{file}").url("/openapi.json", ApiDoc::openapi());
    let res: String = RapiDoc::with_openapi("/api/openapi.json", ApiDoc::openapi()).to_html();
    ctx.set_response_header(CONTENT_TYPE, TEXT_HTML)
        .await
        .send_response(200, res)
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    let server: Server = Server::new();
    server.route("/api/openapi.json", openapi_json).await;
    server.route("/{file}", swagger_handler).await;
    server.run().await.unwrap();
}
