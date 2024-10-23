use std::{error::Error, net::Ipv4Addr};

use actix_web::{middleware::Logger, App, HttpServer};
use utoipa_actix_web::{scope, AppExt};
use utoipa_swagger_ui::SwaggerUi;

#[actix_web::main]
async fn main() -> Result<(), impl Error> {
    env_logger::init();

    HttpServer::new(move || {
        let (app, api) = App::new()
            .into_utoipa_app()
            .map(|app| app.wrap(Logger::default()))
            .service(
                scope::scope("/api")
                    .service(scope::scope("/v1").service(api1::hello1))
                    .service(scope::scope("/v2").service(api2::hello2)),
            )
            .split_for_parts();

        app.service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", api))
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await
}

mod api1 {
    use actix_web::get;

    #[utoipa::path(
        responses(
            (status = 200, description = "Hello from api 1", body = str)
        )
    )]
    #[get("/hello")]
    pub(super) async fn hello1() -> &'static str {
        "hello from api 1"
    }
}

mod api2 {
    use actix_web::get;

    #[utoipa::path(
        responses(
            (status = 200, description = "Hello from api 2", body = str)
        )
    )]
    #[get("/hello")]
    pub(super) async fn hello2() -> &'static str {
        "hello from api 2"
    }
}
