#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
//! This crate works as a bridge between [utoipa](https://docs.rs/utoipa/latest/utoipa/) and [Scalar](https://scalar.com/) OpenAPI visualizer.
//!
//! Utoipa-scalar provides simple mechanism to transform OpenAPI spec resource to a servable HTML
//! file which can be served via [predefined framework integration][Self#examples] or used
//! [standalone][Self#using-standalone] and served manually.
//!
//! You may find fullsize examples from utoipa's Github [repository][examples].
//!
//! # Crate Features
//!
//! * **actix-web** Allows serving [`Scalar`] via _**`actix-web`**_.
//! * **rocket** Allows serving [`Scalar`] via _**`rocket`**_.
//! * **axum** Allows serving [`Scalar`] via _**`axum`**_.
//!
//! # Install
//!
//! Use Scalar only without any boiler plate implementation.
//! ```toml
//! [dependencies]
//! utoipa-scalar = "0.1"
//! ```
//!
//! Enable actix-web integration with Scalar.
//! ```toml
//! [dependencies]
//! utoipa-scalar = { version = "0.1", features = ["actix-web"] }
//! ```
//!
//! # Using standalone
//!
//! Utoipa-scalar can be used standalone as simply as creating a new [`Scalar`] instance and then
//! serving it by what ever means available as `text/html` from http handler in your favourite web
//! framework.
//!
//! [`Scalar::to_html`] method can be used to convert the [`Scalar`] instance to a servable html
//! file.
//! ```
//! # use utoipa_scalar::Scalar;
//! # use utoipa::OpenApi;
//! # use serde_json::json;
//! # #[derive(OpenApi)]
//! # #[openapi()]
//! # struct ApiDoc;
//! #
//! let scalar = Scalar::new(ApiDoc::openapi());
//!
//! // Then somewhere in your application that handles http operation.
//! // Make sure you return correct content type `text/html`.
//! let scalar_handler = move || {
//!     scalar.to_html()
//! };
//! ```
//!
//! # Examples
//!
//! _**Serve [`Scalar`] via `actix-web` framework.**_
//! ```no_run
//! use actix_web::App;
//! use utoipa_scalar::{Scalar, Servable};
//!
//! # use utoipa::OpenApi;
//! # use std::net::Ipv4Addr;
//! # #[derive(OpenApi)]
//! # #[openapi()]
//! # struct ApiDoc;
//! App::new().service(Scalar::with_url("/scalar", ApiDoc::openapi()));
//! ```
//!
//! _**Serve [`Scalar`] via `rocket` framework.**_
//! ```no_run
//! # use rocket;
//! use utoipa_scalar::{Scalar, Servable};
//!
//! # use utoipa::OpenApi;
//! # #[derive(OpenApi)]
//! # #[openapi()]
//! # struct ApiDoc;
//! rocket::build()
//!     .mount(
//!         "/",
//!         Scalar::with_url("/scalar", ApiDoc::openapi()),
//!     );
//! ```
//!
//! _**Serve [`Scalar`] via `axum` framework.**_
//!  ```no_run
//!  use axum::Router;
//!  use utoipa_scalar::{Scalar, Servable};
//!  # use utoipa::OpenApi;
//! # #[derive(OpenApi)]
//! # #[openapi()]
//! # struct ApiDoc;
//! #
//! # fn inner<S>()
//! # where
//! #     S: Clone + Send + Sync + 'static,
//! # {
//!
//!  let app = Router::<S>::new()
//!      .merge(Scalar::with_url("/scalar", ApiDoc::openapi()));
//! # }
//! ```
//!
//! _**Use [`Scalar`] to serve custom OpenAPI spec using serde's `json!()` macro.**_
//! ```rust
//! # use utoipa_scalar::Scalar;
//! # use serde_json::json;
//! Scalar::new(json!({"openapi": "3.1.0"}));
//! ```
//!
//! [examples]: <https://github.com/juhaku/utoipa/tree/master/examples>

use std::borrow::Cow;

use serde::Serialize;
use serde_json::Value;
use utoipa::openapi::OpenApi;

mod actix;
mod axum;
mod rocket;

const DEFAULT_HTML: &str = include_str!("../res/scalar.html");

/// Trait makes [`Scalar`] to accept an _`URL`_ the [Scalar][scalar] will be served via predefined
/// web server.
///
/// This is used **only** with **`actix-web`**, **`rocket`** or **`axum`** since they have implicit
/// implementation for serving the [`Scalar`] via the _`URL`_.
///
/// [scalar]: <https://scalar.com/>
#[cfg(any(feature = "actix-web", feature = "rocket", feature = "axum"))]
#[cfg_attr(
    doc_cfg,
    doc(cfg(any(feature = "actix-web", feature = "rocket", feature = "axum")))
)]
pub trait Servable<S>
where
    S: Spec,
{
    /// Construct a new [`Servable`] instance of _`openapi`_ with given _`url`_.
    ///
    /// * **url** Must point to location where the [`Servable`] is served.
    /// * **openapi** Is [`Spec`] that is served via this [`Servable`] from the _**url**_.
    fn with_url<U: Into<Cow<'static, str>>>(url: U, openapi: S) -> Self;
}

#[cfg(any(feature = "actix-web", feature = "rocket", feature = "axum"))]
impl<S: Spec> Servable<S> for Scalar<S> {
    fn with_url<U: Into<Cow<'static, str>>>(url: U, openapi: S) -> Self {
        Self {
            html: Cow::Borrowed(DEFAULT_HTML),
            url: url.into(),
            openapi,
        }
    }
}

/// Is standalone instance of [Scalar][scalar].
///
/// This can be used together with predefined web framework integration or standalone with
/// framework of your choice. [`Scalar::to_html`] method will convert this [`Scalar`] instance to
/// servable HTML file.
///
/// [scalar]: <https://scalar.com/>
#[non_exhaustive]
#[derive(Clone)]
pub struct Scalar<S: Spec> {
    #[allow(unused)]
    url: Cow<'static, str>,
    html: Cow<'static, str>,
    openapi: S,
}

impl<S: Spec> Scalar<S> {
    /// Constructs a new [`Scalar`] instance for given _`openapi`_ [`Spec`].
    ///
    /// # Examples
    ///
    /// _**Create new [`Scalar`] instance.**_
    /// ```
    /// # use utoipa_scalar::Scalar;
    /// # use serde_json::json;
    /// Scalar::new(json!({"openapi": "3.1.0"}));
    /// ```
    pub fn new(openapi: S) -> Self {
        Self {
            html: Cow::Borrowed(DEFAULT_HTML),
            url: Cow::Borrowed("/"),
            openapi,
        }
    }

    /// Converts this [`Scalar`] instance to servable HTML file.
    ///
    /// This will replace _**`$spec`**_ variable placeholder with [`Spec`] of this instance
    /// provided to this instance serializing it to JSON from the HTML template used with the
    /// [`Scalar`].
    ///
    /// At this point in time, it is not possible to customize the HTML template used by the
    /// [`Scalar`] instance.
    pub fn to_html(&self) -> String {
        self.html.replace(
            "$spec",
            &serde_json::to_string(&self.openapi).expect(
                "Invalid OpenAPI spec, expected OpenApi, String, &str or serde_json::Value",
            ),
        )
    }
}

/// Trait defines OpenAPI spec resource types supported by [`Scalar`].
///
/// By default this trait is implemented for [`utoipa::openapi::OpenApi`] and [`serde_json::Value`].
///
/// * **OpenApi** implementation allows using utoipa's OpenApi struct as a OpenAPI spec resource
///   for the [`Scalar`].
/// * **Value** implementation enables the use of arbitrary JSON values with serde's `json!()`
///   macro as a OpenAPI spec for the [`Scalar`].
///
/// # Examples
///
/// _**Use [`Scalar`] to serve utoipa's OpenApi.**_
/// ```no_run
/// # use utoipa_scalar::Scalar;
/// # use utoipa::openapi::OpenApiBuilder;
/// #
/// Scalar::new(OpenApiBuilder::new().build());
/// ```
///
/// _**Use [`Scalar`] to serve custom OpenAPI spec using serde's `json!()` macro.**_
/// ```rust
/// # use utoipa_scalar::Scalar;
/// # use serde_json::json;
/// Scalar::new(json!({"openapi": "3.1.0"}));
/// ```
pub trait Spec: Serialize {}

impl Spec for OpenApi {}

impl Spec for Value {}
