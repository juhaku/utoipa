use std::{error::Error, net::Ipv4Addr};

use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

use crate::todo::{ErrorResponse, Todo, TodoStore, TodoUpdateRequest};

mod todo;

#[actix_web::main]
async fn main() -> Result<(), impl Error> {
    env_logger::init();

    #[derive(OpenApi)]
    #[openapi(
        handlers(
            todo::get_todos,
            todo::create_todo,
            todo::delete_todo,
            todo::get_todo_by_id,
            todo::update_todo
        ),
        components(Todo, TodoUpdateRequest, ErrorResponse),
        tags(
            (name = "todo", description = "Todo management endpoints.")
        ),
        modifiers(&SecurityAddon)
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
            )
        }
    }

    let store = Data::new(TodoStore::default());

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .configure(todo::configure(store.clone()))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", ApiDoc::openapi()),
            )
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await
}
