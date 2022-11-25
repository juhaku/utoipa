#![cfg(feature = "axum")]

use std::sync::Arc;

use axum::{
    body::HttpBody, extract::Path, http::StatusCode, response::IntoResponse, routing, Extension,
    Json, Router,
};

use crate::{Config, SwaggerUi, Url};

impl<S, B> From<SwaggerUi> for Router<S, B>
where
    S: Clone + Send + Sync + 'static,
    B: HttpBody + Send + 'static,
{
    fn from(swagger_ui: SwaggerUi) -> Self {
        let urls_capacity = swagger_ui.urls.len();

        let (router, urls) = swagger_ui.urls.into_iter().fold(
            (
                Router::<S, B>::new(),
                Vec::<Url>::with_capacity(urls_capacity),
            ),
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
            config.configure_defaults(urls)
        } else {
            Config::new(urls)
        };

        let handler = routing::get(serve_swagger_ui).layer(Extension(Arc::new(config)));
        let path: &str = swagger_ui.path.as_ref();
        let slash_path = format!("{}/", path);

        router
            .route(
                path,
                routing::get(|| async move { axum::response::Redirect::to(&slash_path) }),
            )
            .route(&format!("{}/", path), handler.clone())
            .route(&format!("{}/*rest", path), handler)
    }
}

async fn serve_swagger_ui(
    path: Option<Path<String>>,
    Extension(state): Extension<Arc<Config<'static>>>,
) -> impl IntoResponse {
    let tail = match path.as_ref() {
        Some(tail) => tail,
        None => "",
    };

    match super::serve(tail, state) {
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
