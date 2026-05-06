#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
//! This crate works as a bridge between [utoipa](https://docs.rs/utoipa/latest/utoipa/) and [Scalar](https://scalar.com/) OpenAPI visualizer.
//!
//! Utoipa-scalar provides a simple mechanism to transform OpenAPI spec resource to a servable HTML
//! file which can be served via [predefined framework integration][Self#examples] or used
//! [standalone][Self#using-standalone] and served manually.
//!
//! You may find fullsize examples from utoipa's GitHub [repository][examples].
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
//! utoipa-scalar = "0.4"
//! ```
//!
//! Enable actix-web integration with Scalar.
//! ```toml
//! [dependencies]
//! utoipa-scalar = { version = "0.4", features = ["actix-web"] }
//! ```
//!
//! # Using standalone
//!
//! Utoipa-scalar can be used standalone as simply as creating a new [`Scalar`] instance and then
//! serving it by what ever means available as `text/html` from http handler in your favourite web
//! framework.
//!
//! [`Scalar::to_html`] method can be used to convert the [`Scalar`] instance to a servable HTML
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
//! # Configuration
//!
//! Scalar supports extensive configuration via [`ScalarConfig`] using the builder pattern powered
//! by the [Bon](https://github.com/elastio/bon) crate. This allows you to customize themes, layout,
//! behavior, and much more.
//!
//!
//! ## Using ScalarConfig Builder
//!
//! ```rust
//! # use utoipa_scalar::{Scalar, ScalarConfig, ScalarLayout, ScalarTheme};
//! # use utoipa::OpenApi;
//! # use serde_json::json;
//! # #[derive(OpenApi)]
//! # #[openapi()]
//! # struct ApiDoc;
//! #
//! let config = ScalarConfig::builder()
//!     .theme(ScalarTheme::Moon)
//!     .dark_mode(true)
//!     .show_sidebar(false)
//!     .layout(ScalarLayout::Classic)
//!     .custom_css("body { background-color: #1a1a1a; }".to_string())
//!     .build();
//!
//! let scalar = Scalar::with_config(ApiDoc::openapi(), config);
//! ```
//!
//! # Customization
//!
//! The HTML template can be overridden via [`Scalar::custom_html`] method while preserving
//! configuration capabilities.
//!
//! The HTML template must contain **`$spec`** variable which will be replaced with the complete
//! Scalar configuration during [`Scalar::to_html`] execution.
//!
//! * **`$spec`** Will be the complete configuration object including the content of the openapi spec and all scalar configuration options.
//!
//! _**Overriding the HTML template with a custom one.**_
//! Override the HTML template while preserving configuration capabilities:
//! ```rust
//! # use utoipa::OpenApi;
//! # use utoipa_scalar::{Scalar, ScalarConfig, ScalarTheme};
//!
//! # #[derive(OpenApi)]
//! # #[openapi()]
//! # struct ApiDoc;
//! #
//!
//! let custom_html = r#"
//! <!doctype html>
//! <html>
//!     <head>
//!         <title>My Custom API Docs</title>
//!         <meta charset="utf-8" />
//!         <meta name="viewport" content="width=device-width, initial-scale=1" />
//!         <style>
//!             body { font-family: 'Roboto', sans-serif; }
//!         </style>
//!     </head>
//!     <body>
//!         <div id="app"></div>
//!
//!         <script src="https://!cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
//!
//!         <script>
//!             const config = $spec;
//!             Scalar.createApiReference('#app', config);
//!         </script>
//!     </body>
//! </html>
//! "#;
//! let config = ScalarConfig::builder()
//!     .theme(ScalarTheme::Moon)
//!     .build();
//!
//! let scalar = Scalar::with_config(ApiDoc::openapi(), config).custom_html(custom_html);
//! ```

//! **Note:** The HTML template must contain the **`$spec`** variable placeholder, which will be replaced with the complete Scalar configuration including your OpenAPI spec and all configuration options.
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
//! let _ = rocket::build()
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
//! [scalar]: <https://scalar.com/>
//! [configuration]: <https://github.com/scalar/scalar/blob/main/documentation/configuration.md>
//! [themes]: <https://github.com/scalar/scalar/blob/main/documentation/themes.md>
//! [html]: <https://github.com/scalar/scalar/blob/main/documentation/integrations/html.md>

use std::borrow::Cow;

use bon::Builder;
use serde::Serialize;
use serde_json::Value;
use serde_with::skip_serializing_none;
use utoipa::openapi::OpenApi;

mod actix;
mod axum;
mod rocket;

const DEFAULT_HTML: &str = include_str!("../res/scalar.html");

/// Predefined themes available for Scalar API documentation.
///
/// Scalar supports multiple built-in themes for customizing the appearance
/// of the API documentation interface.
/// Use `Other` for custom themes.
///
/// # Examples
///
/// ```rust
/// # use utoipa_scalar::{ScalarTheme, ScalarConfig};
/// let config = ScalarConfig::builder()
///     .theme(ScalarTheme::Moon)
///     .build();
///
/// // Using a custom theme
/// let custom_config = ScalarConfig::builder()
///     .theme(ScalarTheme::Other("my-custom-theme".to_string()))
///     .build();
/// ```
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ScalarTheme {
    /// Default Scalar theme
    Default,
    /// Alternative color scheme
    Alternate,
    /// Dark theme with blue accents
    Moon,
    /// Purple-themed interface
    Purple,
    /// Solarized color scheme
    Solarized,
    /// Blue planet theme
    BluePlanet,
    /// Dark space theme
    DeepSpace,
    /// Saturn-inspired theme
    Saturn,
    /// Kepler theme
    Kepler,
    /// Mars-themed interface
    Mars,
    /// Retro laser wave theme
    Laserwave,
    /// Custom theme name
    Other(String),
}

/// Configuration options for Scalar API documentation interface.
///
/// This struct provides comprehensive configuration options for customizing the Scalar
/// API documentation appearance and behavior.
/// Use the builder pattern to configure:
///
/// ```rust
/// # use utoipa_scalar::{ScalarConfig, ScalarTheme};
/// let config = ScalarConfig::builder()
///     .theme(ScalarTheme::Moon)
///     .dark_mode(true)
///     .show_sidebar(false)
///     .build();
/// ```
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Default)]
#[builder(derive(Clone), on(String, into))]
#[serde(rename_all = "camelCase")]
pub struct ScalarConfig {
    /// The visual theme to apply.
    /// Use predefined themes from `ScalarTheme` enum
    /// or `ScalarTheme::Other("custom")` for custom themes.
    #[builder(into)]
    pub theme: Option<ScalarTheme>,

    /// Whether to start in dark mode initially. If not set, follows system preference.
    pub dark_mode: Option<bool>,

    /// Force a specific dark/light mode state, overriding user preferences.
    /// Options: "dark", "light"
    #[builder(into)]
    pub force_dark_mode_state: Option<ScalarForceDarkModeState>,

    /// Hide the dark mode toggle button from the interface.
    pub hide_dark_mode_toggle: Option<bool>,

    /// Whether to show the sidebar navigation.
    pub show_sidebar: Option<bool>,

    /// Hide the search bar in the sidebar.
    pub hide_search: Option<bool>,

    /// The layout style to use. Options: "modern", "classic"
    #[builder(into)]
    pub layout: Option<ScalarLayout>,

    /// Custom CSS to inject into the interface.
    pub custom_css: Option<String>,

    /// Proxy URL for API requests to avoid CORS issues.
    pub proxy_url: Option<String>,

    /// Keyboard shortcut key for opening search (used with Ctrl/Cmd).
    pub search_hot_key: Option<char>,

    /// Whether to open all tags by default instead of just the relevant one.
    pub default_open_all_tags: Option<bool>,

    /// Whether to load default fonts from Scalar CDN.
    pub with_default_fonts: Option<bool>,

    /// Override servers from the OpenAPI specification.
    pub servers: Option<Vec<ScalarServer>>,
}

/// Represents the state of dark mode in Scalar API documentation.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum ScalarForceDarkModeState {
    /// Dark mode is enabled.
    Dark,
    /// Light mode is enabled.
    Light,
}

/// Layout options for the Scalar API documentation interface.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum ScalarLayout {
    /// Modern layout with a sidebar.
    Modern,
    /// Classic layout without a sidebar.
    Classic,
}

/// Server configuration for overriding OpenAPI servers in Scalar.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, PartialEq, Builder)]
pub struct ScalarServer {
    /// The server URL.
    pub url: String,
    /// Optional description of the server.
    pub description: Option<String>,
    /// Optional server variables.
    pub variables: Option<serde_json::Map<String, Value>>,
}

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
        Self::with_url_and_config(url, openapi, ScalarConfig::default())
    }
}

#[cfg(any(feature = "actix-web", feature = "rocket", feature = "axum"))]
impl<S: Spec> Scalar<S> {
    /// Constructs a new [`Scalar`] instance for serving at the given URL with custom configuration.
    ///
    /// # Examples
    ///
    /// _**Create [`Scalar`] with URL and configuration.**_
    /// ```
    /// # use utoipa_scalar::{Scalar, ScalarConfig, ScalarTheme, Servable};
    /// # use serde_json::json;
    /// let config = ScalarConfig::builder()
    ///     .theme(ScalarTheme::Moon)
    ///     .dark_mode(true)
    ///     .build();
    ///
    /// let scalar = Scalar::with_url_and_config("/docs", json!({"openapi": "3.1.0"}), config);
    /// ```
    pub fn with_url_and_config<U: Into<Cow<'static, str>>>(
        url: U,
        openapi: S,
        config: ScalarConfig,
    ) -> Self {
        Self {
            html: Cow::Borrowed(DEFAULT_HTML),
            url: url.into(),
            openapi,
            config,
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
    config: ScalarConfig,
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
        Self::with_config(openapi, ScalarConfig::default())
    }

    /// _**Create new [`Scalar`] instance with custom configuration.**_
    /// ```
    /// # use utoipa_scalar::{Scalar, ScalarConfig, ScalarTheme};
    /// # use serde_json::json;
    /// let config = ScalarConfig::builder()
    ///     .theme(ScalarTheme::Moon)
    ///     .dark_mode(true)
    ///     .show_sidebar(false)
    ///     .build();
    ///
    /// Scalar::with_config(json!({"openapi": "3.1.0"}), config);
    /// ```
    pub fn with_config(openapi: S, config: ScalarConfig) -> Self {
        Self {
            html: Cow::Borrowed(DEFAULT_HTML),
            url: Cow::Borrowed("/"),
            openapi,
            config,
        }
    }

    /// Converts this [`Scalar`] instance to servable HTML file.
    ///
    /// This will replace _**`$spec`**_ variable placeholder with the complete Scalar
    /// configuration including the [`Spec`] and any configuration options.
    pub fn to_html(&self) -> String {
        // Create the full configuration object that includes both the spec and config
        let mut full_config = serde_json::Map::new();

        // Add the OpenAPI spec as 'content' field
        let spec_content = serde_json::to_value(&self.openapi)
            .expect("Invalid OpenAPI spec, expected valid JSON serializable value");
        full_config.insert("content".to_string(), spec_content);

        // Merge in the configuration options
        if let Ok(config_value) = serde_json::to_value(&self.config) {
            if let Some(config_obj) = config_value.as_object() {
                for (key, value) in config_obj {
                    full_config.insert(key.clone(), value.clone());
                }
            }
        }

        let config_json =
            serde_json::to_string(&full_config).expect("Failed to serialize Scalar configuration");

        self.html.replace("$spec", &config_json)
    }

    /// Override the [default HTML template][scalar_html_quickstart] with new one. Refer to
    /// [customization] for more comprehensive guide for customization options.
    ///
    /// [customization]: <index.html#customization>
    /// [scalar_html_quickstart]: <https://github.com/scalar/scalar?tab=readme-ov-file#quickstart>
    pub fn custom_html<H: Into<Cow<'static, str>>>(mut self, html: H) -> Self {
        self.html = html.into();

        self
    }
}

/// Trait defines OpenAPI spec resource types supported by [`Scalar`].
///
/// By default this trait is implemented for [`OpenApi`] and [`Value`].
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
