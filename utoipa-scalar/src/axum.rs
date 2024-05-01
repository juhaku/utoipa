#![cfg(feature = "axum")]

use axum::response::Html;
use axum::{routing, Router};

use crate::{Scalar, Spec};

impl<S: Spec, R> From<Scalar<S>> for Router<R>
where
    R: Clone + Send + Sync + 'static,
{
    fn from(value: Scalar<S>) -> Self {
        let html = value.to_html();
        Router::<R>::new().route(
            value.url.as_ref(),
            routing::get(move || async { Html(html) }),
        )
    }
}
