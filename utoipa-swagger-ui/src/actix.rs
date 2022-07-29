#![cfg(feature = "actix-web")]

use actix_web::{
    dev::HttpServiceFactory, guard::Get, web, web::Data, HttpResponse, Resource,
    Responder as ActixResponder,
};

use utoipa::openapi::OpenApi;

use crate::{Config, SwaggerUi};

impl HttpServiceFactory for SwaggerUi {
    fn register(self, config: &mut actix_web::dev::AppService) {
        let urls = self
            .urls
            .into_iter()
            .map(|url| {
                let (url, openapi) = url;
                register_api_doc_url_resource(url.url.as_ref(), openapi, config);
                url
            })
            .collect::<Vec<_>>();

        let swagger_resource = Resource::new(self.path.as_ref())
            .guard(Get())
            .app_data(Data::new(if let Some(config) = self.config {
                config.configure_defaults(urls)
            } else {
                Config::new(urls)
            }))
            .to(serve_swagger_ui);

        HttpServiceFactory::register(swagger_resource, config);
    }
}

fn register_api_doc_url_resource(url: &str, api: OpenApi, config: &mut actix_web::dev::AppService) {
    pub async fn get_api_doc(api_doc: web::Data<OpenApi>) -> impl ActixResponder {
        HttpResponse::Ok().json(api_doc.as_ref())
    }

    let url_resource = Resource::new(url)
        .guard(Get())
        .app_data(Data::new(api))
        .to(get_api_doc);
    HttpServiceFactory::register(url_resource, config);
}

async fn serve_swagger_ui(path: web::Path<String>, data: web::Data<Config<'_>>) -> HttpResponse {
    match super::serve(&*path.into_inner(), data.into_inner()) {
        Ok(swagger_file) => swagger_file
            .map(|file| {
                HttpResponse::Ok()
                    .content_type(file.content_type)
                    .body(file.bytes.to_vec())
            })
            .unwrap_or_else(|| HttpResponse::NotFound().finish()),
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}
