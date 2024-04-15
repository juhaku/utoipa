#![cfg(feature = "actix-web")]

use actix_web::dev::HttpServiceFactory;
use actix_web::guard::Get;
use actix_web::web::Data;
use actix_web::{HttpResponse, Resource, Responder};

use crate::{Scalar, Spec};

impl<S: Spec> HttpServiceFactory for Scalar<S> {
    fn register(self, config: &mut actix_web::dev::AppService) {
        let html = self.to_html();

        async fn serve_scalar(scalar: Data<String>) -> impl Responder {
            HttpResponse::Ok()
                .content_type("text/html")
                .body(scalar.to_string())
        }

        Resource::new(self.url.as_ref())
            .guard(Get())
            .app_data(Data::new(html))
            .to(serve_scalar)
            .register(config);
    }
}
