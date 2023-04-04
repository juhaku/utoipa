#![cfg(feature = "actix-web")]

use actix_web::{
    dev::HttpServiceFactory, guard::Get, web, web::Data, HttpResponse, Resource,
    Responder as ActixResponder,
};

use crate::{ApiDoc, Config, SwaggerUi};

impl HttpServiceFactory for SwaggerUi {
    fn register(self, config: &mut actix_web::dev::AppService) {
        let mut urls = self
            .urls
            .into_iter()
            .map(|(url, openapi)| {
                register_api_doc_url_resource(url.url.as_ref(), ApiDoc::Utoipa(openapi), config);
                url
            })
            .collect::<Vec<_>>();
        let external_api_docs = self.external_urls.into_iter().map(|(url, api_doc)| {
            register_api_doc_url_resource(url.url.as_ref(), ApiDoc::Value(api_doc), config);
            url
        });
        urls.extend(external_api_docs);

        let swagger_resource = Resource::new(self.path.as_ref())
            .guard(Get())
            .app_data(Data::new(if let Some(config) = self.config {
                if config.url.is_some() || !config.urls.is_empty() {
                    config
                } else {
                    config.configure_defaults(urls)
                }
            } else {
                Config::new(urls)
            }))
            .to(serve_swagger_ui);

        HttpServiceFactory::register(swagger_resource, config);
    }
}

fn register_api_doc_url_resource(url: &str, api: ApiDoc, config: &mut actix_web::dev::AppService) {
    async fn get_api_doc(api_doc: web::Data<ApiDoc>) -> impl ActixResponder {
        HttpResponse::Ok().json(api_doc.as_ref())
    }

    let url_resource = Resource::new(url)
        .guard(Get())
        .app_data(Data::new(api))
        .to(get_api_doc);
    HttpServiceFactory::register(url_resource, config);
}

async fn serve_swagger_ui(path: web::Path<String>, data: web::Data<Config<'_>>) -> HttpResponse {
    match super::serve(&path.into_inner(), data.into_inner()) {
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
