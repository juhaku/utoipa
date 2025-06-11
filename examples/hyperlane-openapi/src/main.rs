use hyperlane::*;
use serde::Serialize;
use serde_json;
use utoipa::{OpenApi, ToSchema};
use utoipa_rapidoc::RapiDoc;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Serialize, ToSchema)]
struct User {
    name: String,
    age: usize,
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(User)),
    info(title = "Hyperlane", version = "1.0.0"),
    paths(index, user, openapi_json, swagger)
)]
struct ApiDoc;

async fn request_middleware(ctx: Context) {
    ctx.set_response_status_code(200).await;
}

#[utoipa::path(
    get,
    path = "/openapi.json",   
    responses(
        (status = 200, description = "Openapi docs", body = String)
    )
)]
async fn openapi_json(ctx: Context) {
    ctx.set_response_body(ApiDoc::openapi().to_json().unwrap())
        .await
        .send()
        .await
        .unwrap();
}

#[utoipa::path(
    get,
    path = "/{file}",   
    responses(
        (status = 200, description = "Openapi json", body = String)
    )
)]
async fn swagger(ctx: Context) {
    SwaggerUi::new("/{file}").url("/openapi.json", ApiDoc::openapi());
    let res: String = RapiDoc::with_openapi("/openapi.json", ApiDoc::openapi()).to_html();
    ctx.set_response_header(CONTENT_TYPE, TEXT_HTML)
        .await
        .set_response_body(res)
        .await
        .send()
        .await
        .unwrap();
}

#[utoipa::path(
    get,
    path = "/",   
    responses(
        (status = 302, description = "Redirect to index.html")
    )
)]
async fn index(ctx: Context) {
    ctx.set_response_header(LOCATION, "/index.html")
        .await
        .set_response_body(vec![])
        .await
        .send()
        .await
        .unwrap();
}

#[utoipa::path(
    get,
    path = "/user/{name}",   
    responses(
        (status = 200, description = "User", body = User)
    )
)]
async fn user(ctx: Context) {
    let name: String = ctx.get_route_param("name").await.unwrap();
    let user: User = User { name, age: 0 };
    ctx.set_response_body(serde_json::to_vec(&user).unwrap())
        .await
        .send()
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    let server: Server = Server::new();
    server.request_middleware(request_middleware).await;
    server.route("/", index).await;
    server.route("/user/{name}", user).await;
    server.route("/openapi.json", openapi_json).await;
    server.route("/{file}", swagger).await;
    server.run().await.unwrap();
}
