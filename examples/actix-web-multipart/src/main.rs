use std::io;

use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_multipart::form::MultipartForm;
use actix_web::{post, App, HttpServer, Responder};
use utoipa::ToSchema;
use utoipa_actix_web::AppExt;
use utoipa_swagger_ui::SwaggerUi;

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .into_utoipa_app()
            .service(hello_form)
            .openapi_service(|api| {
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api/openapi.json", api)
            })
            .into_app()
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[derive(ToSchema, MultipartForm)]
struct HelloForm {
    #[multipart(limit = "10mb")]
    #[schema(value_type = String, format = Binary, content_media_type = "application/octet-stream")]
    file: TempFile,
    #[schema(value_type = String)]
    name: Text<String>,
}

#[utoipa::path(
    request_body(content = HelloForm, content_type = "multipart/form-data")
)]
#[post("/hello")]
async fn hello_form(MultipartForm(form): MultipartForm<HelloForm>) -> impl Responder {
    let name = form.name.to_string();
    let file = &form.file;
    format!(
        "Greetings: name: {name}, type: {} size: {} file_name: {}!",
        file.content_type
            .as_ref()
            .map(|mime| mime.to_string())
            .unwrap_or_default(),
        file.size,
        file.file_name.as_ref().unwrap_or(&String::new())
    )
}
