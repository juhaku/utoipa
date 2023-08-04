#![cfg(feature = "axum")]

use axum::body::HttpBody;
use axum::response::Html;
use axum::{routing, Router};

use crate::{Redoc, Spec};

impl<'s, 'u, S: Spec, R, B> From<Redoc<'s, 'u, S>> for Router<R, B>
where
    R: Clone + Send + Sync + 'static,
    B: HttpBody + Send + 'static,
    's: 'static,
{
    fn from(value: Redoc<'s, 'u, S>) -> Self {
        let html = value.to_html();
        Router::<R, B>::new().route(value.url, routing::get(move || async { Html(html) }))
    }
}
