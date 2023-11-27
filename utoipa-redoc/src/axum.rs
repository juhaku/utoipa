#![cfg(feature = "axum")]

use axum::response::Html;
use axum::{routing, Router};

use crate::{Redoc, Spec};

impl<'s, 'u, S: Spec, R> From<Redoc<'s, 'u, S>> for Router<R>
where
    R: Clone + Send + Sync + 'static,
    's: 'static,
{
    fn from(value: Redoc<'s, 'u, S>) -> Self {
        let html = value.to_html();
        Router::<R>::new().route(value.url, routing::get(move || async { Html(html) }))
    }
}
