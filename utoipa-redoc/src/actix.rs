#![cfg(feature = "actix-web")]

use actix_web::dev::HttpServiceFactory;
use actix_web::guard::Get;
use actix_web::web::Data;
use actix_web::{HttpResponse, Resource, Responder};

use crate::{Redoc, Spec};

impl<'s, 'u, S: Spec> HttpServiceFactory for Redoc<'s, 'u, S> {
    fn register(self, config: &mut actix_web::dev::AppService) {
        let html = self.to_html();

        async fn serve_redoc(redoc: Data<String>) -> impl Responder {
            HttpResponse::Ok()
                .content_type("text/html")
                .body(redoc.to_string())
        }

        Resource::new(self.url)
            .guard(Get())
            .app_data(Data::new(html))
            .to(serve_redoc)
            .register(config);
    }
}
