use axum::extract::Multipart;
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let (router, api) = OpenApiRouter::new()
        .routes(routes!(hello_form, foo_bar))
        .split_for_parts();

    let router = router.merge(SwaggerUi::new("/swagger-ui").url("/api/openapi.json", api));

    let app = router.into_make_service();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await
}

/// Just a schema for axum native multipart
#[derive(Deserialize, ToSchema)]
#[allow(unused)]
struct HelloForm {
    name: String,
    #[schema(format = Binary, content_media_type = "application/octet-stream")]
    file: String,
}

#[utoipa::path(
    post,
    path = "/foobar",
)]
async fn foo_bar() -> String {
    String::from("Foo Bar!")
}

#[utoipa::path(
    post,
    path = "/hello",
    request_body(content = HelloForm, content_type = "multipart/form-data")
)]
async fn hello_form(mut multipart: Multipart) -> String {
    let mut name: Option<String> = None;

    let mut content_type: Option<String> = None;
    let mut size: usize = 0;
    let mut file_name: Option<String> = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let field_name = field.name();

        match &field_name {
            Some("name") => {
                name = Some(field.text().await.expect("should be text for name field"));
            }
            Some("file") => {
                file_name = field.file_name().map(ToString::to_string);
                content_type = field.content_type().map(ToString::to_string);
                let bytes = field.bytes().await.expect("should be bytes for file field");
                size = bytes.len();
            }
            _ => (),
        };
    }
    format!(
        "name: {}, content_type: {}, size: {}, file_name: {}",
        name.unwrap_or_default(),
        content_type.unwrap_or_default(),
        size,
        file_name.unwrap_or_default()
    )
}
