#![cfg(feature = "rocket")]

use std::{borrow::Cow, io::Cursor, sync::Arc};

use rocket::{
    http::{Header, Status},
    response::{status::NotFound, Responder as RocketResponder},
    route::{Handler, Outcome},
    serde::json::Json,
    Data as RocketData, Request, Response, Route,
};

use crate::{ApiDoc, Config, SwaggerFile, SwaggerUi};

impl From<SwaggerUi> for Vec<Route> {
    fn from(swagger_ui: SwaggerUi) -> Self {
        let mut routes =
            Vec::<Route>::with_capacity(swagger_ui.urls.len() + 1 + swagger_ui.external_urls.len());
        let mut api_docs =
            Vec::<Route>::with_capacity(swagger_ui.urls.len() + swagger_ui.external_urls.len());

        let urls = swagger_ui
            .urls
            .into_iter()
            .map(|(url, openapi)| (url, ApiDoc::Utoipa(openapi)))
            .chain(
                swagger_ui
                    .external_urls
                    .into_iter()
                    .map(|(url, api_doc)| (url, ApiDoc::Value(api_doc))),
            )
            .map(|(url, openapi)| {
                api_docs.push(Route::new(
                    rocket::http::Method::Get,
                    &url.url,
                    ServeApiDoc(openapi),
                ));
                url
            });

        routes.push(Route::new(
            rocket::http::Method::Get,
            swagger_ui.path.as_ref(),
            ServeSwagger(
                swagger_ui.path.clone(),
                Arc::new(if let Some(config) = swagger_ui.config {
                    if config.url.is_some() || !config.urls.is_empty() {
                        config
                    } else {
                        config.configure_defaults(urls)
                    }
                } else {
                    Config::new(urls)
                }),
            ),
        ));
        routes.extend(api_docs);

        routes
    }
}

#[derive(Clone)]
struct ServeApiDoc(ApiDoc);

#[rocket::async_trait]
impl Handler for ServeApiDoc {
    async fn handle<'r>(&self, request: &'r Request<'_>, _: RocketData<'r>) -> Outcome<'r> {
        Outcome::from(request, Json(self.0.clone()))
    }
}

#[derive(Clone)]
struct ServeSwagger(Cow<'static, str>, Arc<Config<'static>>);

#[rocket::async_trait]
impl Handler for ServeSwagger {
    async fn handle<'r>(&self, request: &'r Request<'_>, _: RocketData<'r>) -> Outcome<'r> {
        let mut base_path = self.0.as_ref();
        if let Some(index) = self.0.find('<') {
            base_path = &base_path[..index];
        }

        let request_path = request.uri().path().as_str();
        let request_path = match request_path.strip_prefix(base_path) {
            Some(stripped) => stripped,
            None => return Outcome::from(request, RedirectResponder(base_path.into())),
        };
        match super::serve(request_path, self.1.clone()) {
            Ok(swagger_file) => swagger_file
                .map(|file| Outcome::from(request, file))
                .unwrap_or_else(|| Outcome::from(request, NotFound("Swagger UI file not found"))),
            Err(error) => Outcome::from(
                request,
                rocket::response::status::Custom(Status::InternalServerError, error.to_string()),
            ),
        }
    }
}

impl<'r, 'o: 'r> RocketResponder<'r, 'o> for SwaggerFile<'o> {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'o> {
        Ok(Response::build()
            .header(Header::new("Content-Type", self.content_type))
            .sized_body(self.bytes.len(), Cursor::new(self.bytes.to_vec()))
            .status(Status::Ok)
            .finalize())
    }
}

struct RedirectResponder(String);
impl<'r, 'a: 'r> RocketResponder<'r, 'a> for RedirectResponder {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'a> {
        Response::build()
            .status(Status::Found)
            .raw_header("Location", self.0)
            .ok()
    }
}
