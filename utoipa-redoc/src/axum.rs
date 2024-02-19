#![cfg(feature = "axum")]

use axum::response::Html;
use axum::{routing, Router};

use crate::{Redoc, Spec};

impl<S: Spec, R> From<Redoc<S>> for Router<R>
where
    R: Clone + Send + Sync + 'static,
{
    fn from(value: Redoc<S>) -> Self {
        let html = value.to_html();
        Router::<R>::new().route(
            value.url.as_ref(),
            routing::get(move || async { Html(html) }),
        )
    }
}
