#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
//! This crate works as a bridge between [utoipa](https://docs.rs/utoipa/latest/utoipa/) and [Redoc](https://redocly.com/) OpenAPI visualizer.
//!
//! Utoipa-redoc provides simple mechanism to transform OpenAPI spec resource to a servable HTML
//! file which can be served via [predefined framework integration][Self#examples] or used
//! [standalone][Self#using-standalone] and served manually.
//!
//! You may find fullsize examples from utoipa's Github [repository][examples].
//!
//! # Crate Features
//!
//! * **actix-web** Allows serving [`Redoc`] via _**`actix-web`**_.
//! * **rocket** Allows serving [`Redoc`] via _**`rocket`**_.
//! * **axum** Allows serving [`Redoc`] via _**`axum`**_.
//!
//! # Install
//!
//! Use Redoc only without any boiler plate implementation.
//! ```toml
//! [dependencies]
//! utoipa-redoc = "0.1"
//! ```
//!
//! Enable actix-web integration with Redoc.
//! ```toml
//! [dependencies]
//! utoipa-redoc = { version = "0.1", features = ["actix-web"] }
//! ```
//!
//! # Using standalone
//!
//! Utoipa-redoc can be used standalone as simply as creating a new [`Redoc`] instance and then
//! serving it by what ever means available as `text/html` from http handler in your favourite web
//! framework.
//!
//! [`Redoc::to_html`] method can be used to convert the [`Redoc`] instance to a servable html
//! file.
//! ```
//! # use utoipa_redoc::Redoc;
//! # use utoipa::OpenApi;
//! # use serde_json::json;
//! # #[derive(OpenApi)]
//! # #[openapi()]
//! # struct ApiDoc;
//! #
//! let redoc = Redoc::new(ApiDoc::openapi());
//!
//! // Then somewhere in your application that handles http operation.
//! // Make sure you return correct content type `text/html`.
//! let redoc_handler = move || {
//!     redoc.to_html()
//! };
//! ```
//!
//! # Customization
//!
//! Utoipa-redoc enables full customization support for [Redoc][redoc] according to what can be
//! customized by modifying the HTML template and [configuration options][Self#configuration].
//!
//! The default [HTML template][redoc_html_quickstart] can be fully overridden to ones liking with
//! [`Redoc::custom_html`] method. The HTML template **must** contain **`$spec`** and **`$config`**
//! variables which are replaced during [`Redoc::to_html`] execution.
//!
//! * **`$spec`** Will be the [`Spec`] that will be rendered via [Redoc][redoc].
//! * **`$config`** Will be the current [`Config`]. By default this is [`EmptyConfig`].
//!
//! _**Overriding the HTML template with a custom one.**_
//! ```rust
//! # use utoipa_redoc::Redoc;
//! # use utoipa::OpenApi;
//! # use serde_json::json;
//! # #[derive(OpenApi)]
//! # #[openapi()]
//! # struct ApiDoc;
//! #
//! let html = "...";
//! Redoc::new(ApiDoc::openapi()).custom_html(html);
//! ```
//!
//! # Configuration
//!
//! Redoc can be configured with JSON either inlined with the [`Redoc`] declaration or loaded from
//! user defined file with [`FileConfig`].
//!
//! * [All supported Redoc configuration options][redoc_config].
//!
//! _**Inlining the configuration.**_
//! ```rust
//! # use utoipa_redoc::Redoc;
//! # use utoipa::OpenApi;
//! # use serde_json::json;
//! # #[derive(OpenApi)]
//! # #[openapi()]
//! # struct ApiDoc;
//! #
//! Redoc::with_config(ApiDoc::openapi(), || json!({ "disableSearch": true }));
//! ```
//!
//! _**Using [`FileConfig`].**_
//! ```no_run
//! # use utoipa_redoc::{Redoc, FileConfig};
//! # use utoipa::OpenApi;
//! # use serde_json::json;
//! # #[derive(OpenApi)]
//! # #[openapi()]
//! # struct ApiDoc;
//! #
//! Redoc::with_config(ApiDoc::openapi(), FileConfig);
//! ```
//!
//! Read more details in [`Config`].
//!
//! # Examples
//!
//! _**Serve [`Redoc`] via `actix-web` framework.**_
//! ```no_run
//! use actix_web::App;
//! use utoipa_redoc::{Redoc, Servable};
//!
//! # use utoipa::OpenApi;
//! # use std::net::Ipv4Addr;
//! # #[derive(OpenApi)]
//! # #[openapi()]
//! # struct ApiDoc;
//! App::new().service(Redoc::with_url("/redoc", ApiDoc::openapi()));
//! ```
//!
//! _**Serve [`Redoc`] via `rocket` framework.**_
//! ```no_run
//! # use rocket;
//! use utoipa_redoc::{Redoc, Servable};
//!
//! # use utoipa::OpenApi;
//! # #[derive(OpenApi)]
//! # #[openapi()]
//! # struct ApiDoc;
//! rocket::build()
//!     .mount(
//!         "/",
//!         Redoc::with_url("/redoc", ApiDoc::openapi()),
//!     );
//! ```
//!
//! _**Serve [`Redoc`] via `axum` framework.**_
//!  ```no_run
//!  use axum::Router;
//!  use utoipa_redoc::{Redoc, Servable};
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
//!      .merge(Redoc::with_url("/redoc", ApiDoc::openapi()));
//! # }
//! ```
//!
//! _**Use [`Redoc`] to serve OpenAPI spec from url.**_
//! ```
//! # use utoipa_redoc::Redoc;
//! Redoc::new(
//!   "https://github.com/swagger-api/swagger-petstore/blob/master/src/main/resources/openapi.yaml");
//! ```
//!
//! _**Use [`Redoc`] to serve custom OpenAPI spec using serde's `json!()` macro.**_
//! ```rust
//! # use utoipa_redoc::Redoc;
//! # use serde_json::json;
//! Redoc::new(json!({"openapi": "3.1.0"}));
//! ```
//!
//! [redoc]: <https://redocly.com/>
//! [redoc_html_quickstart]: <https://redocly.com/docs/redoc/quickstart/>
//! [redoc_config]: <https://redocly.com/docs/api-reference-docs/configuration/functionality/#configuration-options-for-api-docs>
//! [examples]: <https://github.com/juhaku/utoipa/tree/master/examples>

use std::fs::OpenOptions;
use std::{borrow::Cow, env};

use serde::Serialize;
use serde_json::{json, Value};
use utoipa::openapi::OpenApi;

mod actix;
mod axum;
mod rocket;

const DEFAULT_HTML: &str = include_str!("../res/redoc.html");

/// Trait makes [`Redoc`] to accept an _`URL`_ the [Redoc][redoc] will be served via predefined web
/// server.
///
/// This is used **only** with **`actix-web`**, **`rocket`** or **`axum`** since they have implicit
/// implementation for serving the [`Redoc`] via the _`URL`_.
///
/// [redoc]: <https://redocly.com/>
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

    /// Construct a new [`Servable`] instance of _`openapi`_ with given _`url`_ and _`config`_.
    ///
    /// * **url** Must point to location where the [`Servable`] is served.
    /// * **openapi** Is [`Spec`] that is served via this [`Servable`] from the _**url**_.
    /// * **config** Is custom [`Config`] that is used to configure the [`Servable`].
    fn with_url_and_config<U: Into<Cow<'static, str>>, C: Config>(
        url: U,
        openapi: S,
        config: C,
    ) -> Self;
}

#[cfg(any(feature = "actix-web", feature = "rocket", feature = "axum"))]
impl<S: Spec> Servable<S> for Redoc<S> {
    fn with_url<U: Into<Cow<'static, str>>>(url: U, openapi: S) -> Self {
        Self::with_url_and_config(url, openapi, EmptyConfig)
    }

    fn with_url_and_config<U: Into<Cow<'static, str>>, C: Config>(
        url: U,
        openapi: S,
        config: C,
    ) -> Self {
        Self {
            url: url.into(),
            html: Cow::Borrowed(DEFAULT_HTML),
            openapi,
            config: config.load(),
        }
    }
}

/// Is standalone instance of [Redoc UI][redoc].
///
/// This can be used together with predefined web framework integration or standalone with
/// framework of your choice. [`Redoc::to_html`] method will convert this [`Redoc`] instance to
/// servable HTML file.
///
/// [redoc]: <https://redocly.com/>
#[non_exhaustive]
#[derive(Clone)]
pub struct Redoc<S: Spec> {
    #[allow(unused)]
    url: Cow<'static, str>,
    html: Cow<'static, str>,
    openapi: S,
    config: Value,
}

impl<S: Spec> Redoc<S> {
    /// Constructs a new [`Redoc`] instance for given _`openapi`_ [`Spec`].
    ///
    /// This will create [`Redoc`] with [`EmptyConfig`].
    ///
    /// # Examples
    ///
    /// _**Create new [`Redoc`] instance with [`EmptyConfig`].**_
    /// ```
    /// # use utoipa_redoc::Redoc;
    /// # use serde_json::json;
    /// Redoc::new(json!({"openapi": "3.1.0"}));
    /// ```
    pub fn new(openapi: S) -> Self {
        Self::with_config(openapi, EmptyConfig)
    }

    /// Constructs a new [`Redoc`] instance for given _`openapi`_ [`Spec`] and _`config`_ [`Config`] of choice.
    ///
    /// # Examples
    ///
    /// _**Create new [`Redoc`] instance with [`FileConfig`].**_
    /// ```no_run
    /// # use utoipa_redoc::{Redoc, FileConfig};
    /// # use serde_json::json;
    /// Redoc::with_config(json!({"openapi": "3.1.0"}), FileConfig);
    /// ```
    pub fn with_config<C: Config>(openapi: S, config: C) -> Self {
        Self {
            html: Cow::Borrowed(DEFAULT_HTML),
            url: Cow::Borrowed(""),
            openapi,
            config: config.load(),
        }
    }

    /// Override the [default HTML template][redoc_html_quickstart] with new one. See
    /// [customization] for more details.
    ///
    /// [redoc_html_quickstart]: <https://redocly.com/docs/redoc/quickstart/>
    /// [customization]: index.html#customization
    pub fn custom_html<H: Into<Cow<'static, str>>>(mut self, html: H) -> Self {
        self.html = html.into();

        self
    }

    /// Converts this [`Redoc`] instance to servable HTML file.
    ///
    /// This will replace _**`$config`**_ variable placeholder with [`Config`] of this instance and
    /// _**`$spec`**_ with [`Spec`] provided to this instance serializing it to JSON from the HTML
    /// template used with the [`Redoc`]. If HTML template is not overridden with
    /// [`Redoc::custom_html`] then the [default HTML template][redoc_html_quickstart] will be used.
    ///
    /// See more details in [customization][customization].
    ///
    /// [redoc_html_quickstart]: <https://redocly.com/docs/redoc/quickstart/>
    /// [customization]: index.html#customization
    pub fn to_html(&self) -> String {
        self.html
            .replace("$config", &self.config.to_string())
            .replace(
                "$spec",
                &serde_json::to_string(&self.openapi).expect(
                    "Invalid OpenAPI spec, expected OpenApi, String, &str or serde_json::Value",
                ),
            )
    }
}

/// Trait defines OpenAPI spec resource types supported by [`Redoc`].
///
/// By default this trait is implemented for [`utoipa::openapi::OpenApi`], [`String`], [`&str`] and
/// [`serde_json::Value`].
///
/// * **OpenApi** implementation allows using utoipa's OpenApi struct as a OpenAPI spec resource
/// for the [`Redoc`].
/// * **String** and **&str** implementations allows defining HTTP URL for [`Redoc`] to load the
/// OpenAPI spec from.
/// * **Value** implementation enables the use of arbitrary JSON values with serde's `json!()`
/// macro as a OpenAPI spec for the [`Redoc`].
///
/// # Examples
///
/// _**Use [`Redoc`] to serve utoipa's OpenApi.**_
/// ```no_run
/// # use utoipa_redoc::Redoc;
/// # use utoipa::openapi::OpenApiBuilder;
/// #
/// Redoc::new(OpenApiBuilder::new().build());
/// ```
///
/// _**Use [`Redoc`] to serve OpenAPI spec from url.**_
/// ```
/// # use utoipa_redoc::Redoc;
/// Redoc::new(
///   "https://github.com/swagger-api/swagger-petstore/blob/master/src/main/resources/openapi.yaml");
/// ```
///
/// _**Use [`Redoc`] to serve custom OpenAPI spec using serde's `json!()` macro.**_
/// ```rust
/// # use utoipa_redoc::Redoc;
/// # use serde_json::json;
/// Redoc::new(json!({"openapi": "3.1.0"}));
/// ```
pub trait Spec: Serialize {}

impl Spec for OpenApi {}

impl Spec for String {}

impl Spec for &str {}

impl Spec for Value {}

/// Trait defines configuration options for [`Redoc`].
///
/// There are 3 configuration methods [`EmptyConfig`], [`FileConfig`] and [`FnOnce`] closure
/// config. The [`Config`] must be able to load and serialize valid JSON.
///
/// * **EmptyConfig** is the default config and serializes to empty JSON object _`{}`_.
/// * **FileConfig** Allows [`Redoc`] to be configured via user defined file which serializes to
///   JSON.
/// * **FnOnce** closure config allows inlining JSON serializable config directly to [`Redoc`]
///   declaration.
///
/// Configuration format and allowed options can be found from Redocly's own API documentation.
///
/// * [All supported Redoc configuration options][redoc_config].
///
/// **Note!** There is no validity check for configuration options and all options provided are
/// serialized as is to the [Redoc][redoc]. It is users own responsibility to check for possible
/// misspelled configuration options against the valid configuration options.
///
/// # Examples
///
/// _**Using [`FnOnce`] closure config.**_
/// ```rust
/// # use utoipa_redoc::Redoc;
/// # use utoipa::OpenApi;
/// # use serde_json::json;
/// # #[derive(OpenApi)]
/// # #[openapi()]
/// # struct ApiDoc;
/// #
/// Redoc::with_config(ApiDoc::openapi(), || json!({ "disableSearch": true }));
/// ```
///
/// _**Using [`FileConfig`].**_
/// ```no_run
/// # use utoipa_redoc::{Redoc, FileConfig};
/// # use utoipa::OpenApi;
/// # use serde_json::json;
/// # #[derive(OpenApi)]
/// # #[openapi()]
/// # struct ApiDoc;
/// #
/// Redoc::with_config(ApiDoc::openapi(), FileConfig);
/// ```
///
/// [redoc]: <https://redocly.com/>
/// [redoc_config]: <https://redocly.com/docs/api-reference-docs/configuration/functionality/#configuration-options-for-api-docs>
pub trait Config {
    /// Implementor must implement the logic which loads the configuration of choice and converts it
    /// to serde's [`serde_json::Value`].
    fn load(self) -> Value;
}

impl<S: Serialize, F: FnOnce() -> S> Config for F {
    fn load(self) -> Value {
        json!(self())
    }
}

/// Makes [`Redoc`] load it's configuration from a user defined file.
///
/// The config file must be defined via _**`UTOIPA_REDOC_CONFIG_FILE`**_ env variable for your
/// application. It can either be defined in runtime before the [`Redoc`] declaration or before
/// application startup or at compile time via `build.rs` file.
///
/// The file must be located relative to your application runtime directory.
///
/// The file must be loadable via [`Config`] and it must return a JSON object representing the
/// [Redoc configuration][redoc_config].
///
/// # Examples
///
/// _**Using a `build.rs` file to define the config file.**_
/// ```rust
/// # fn main() {
/// println!("cargo:rustc-env=UTOIPA_REDOC_CONFIG_FILE=redoc.config.json");
/// # }
/// ```
///
/// _**Defining config file at application startup.**_
/// ```bash
/// UTOIPA_REDOC_CONFIG_FILE=redoc.config.json cargo run
/// ```
///
/// [redoc_config]: <https://redocly.com/docs/api-reference-docs/configuration/functionality/#configuration-options-for-api-docs>
pub struct FileConfig;

impl Config for FileConfig {
    fn load(self) -> Value {
        let path = env::var("UTOIPA_REDOC_CONFIG_FILE")
            .expect("Missing `UTOIPA_REDOC_CONFIG_FILE` env variable, cannot load file config.");

        let file = OpenOptions::new()
            .read(true)
            .open(&path)
            .unwrap_or_else(|_| panic!("File `{path}` is not readable or does not exist."));
        serde_json::from_reader(file).expect("Config file cannot be parsed to JSON")
    }
}

/// Is the default configuration and serializes to empty JSON object _`{}`_.
pub struct EmptyConfig;

impl Config for EmptyConfig {
    fn load(self) -> Value {
        json!({})
    }
}
