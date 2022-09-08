#![cfg(feature = "axum")]

use std::sync::Arc;

use axum::{
    body::HttpBody, extract::Path, http::StatusCode, response::IntoResponse, routing, Extension,
    Json, Router,
};

use crate::{Config, SwaggerUi, Url};

impl<B> From<SwaggerUi> for Router<B>
where
    B: HttpBody + Send + 'static,
{
    fn from(swagger_ui: SwaggerUi) -> Self {
        let urls_capacity = swagger_ui.urls.len();

        let (router, urls) = swagger_ui.urls.into_iter().fold(
            (Router::<B>::new(), Vec::<Url>::with_capacity(urls_capacity)),
            |(router, mut urls), url| {
                let (url, openapi) = url;
                (
                    router.route(
                        url.url.as_ref(),
                        routing::get(move || async { Json(openapi) }),
                    ),
                    {
                        urls.push(url);
                        urls
                    },
                )
            },
        );

        let config = if let Some(config) = swagger_ui.config {
            if config.url.is_some() || !config.urls.is_empty() {
                config
            } else {
                config.configure_defaults(urls)
            }
        } else {
            Config::new(urls)
        };

        router.route(
            swagger_ui.path.as_ref(),
            routing::get(serve_swagger_ui).layer(Extension(Arc::new(config))),
        )
    }
}

async fn serve_swagger_ui(
    Path(tail): Path<String>,
    Extension(state): Extension<Arc<Config<'static>>>,
) -> impl IntoResponse {
    match super::serve(&tail[1..], state) {
        Ok(file) => file
            .map(|file| {
                (
                    StatusCode::OK,
                    [("Content-Type", file.content_type)],
                    file.bytes,
                )
                    .into_response()
            })
            .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response()),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}
