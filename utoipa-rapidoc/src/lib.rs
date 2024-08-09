#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
//! This crate works as a bridge between [utoipa](https://docs.rs/utoipa/latest/utoipa/) and [RapiDoc](https://rapidocweb.com/) OpenAPI visualizer.
//!
//! Utoipa-rapidoc provides simple mechanism to transform OpenAPI spec resource to a servable HTML
//! file which can be served via [predefined framework integration][Self#examples] or used
//! [standalone][Self#using-standalone] and served manually.
//!
//! You may find fullsize examples from utoipa's Github [repository][examples].
//!
//! # Crate Features
//!
//! * **actix-web** Allows serving [`RapiDoc`] via _**`actix-web`**_.
//! * **rocket** Allows serving [`RapiDoc`] via _**`rocket`**_.
//! * **axum** Allows serving [`RapiDoc`] via _**`axum`**_.
//!
//! # Install
//!
//! Use RapiDoc only without any boiler plate implementation.
//! ```toml
//! [dependencies]
//! utoipa-rapidoc = "4"
//! ```
//!
//! Enable actix-web integration with RapiDoc.
//! ```toml
//! [dependencies]
//! utoipa-rapidoc = { version = "4", features = ["actix-web"] }
//! ```
//!
//! # Using standalone
//!
//! Utoipa-rapidoc can be used standalone as simply as creating a new [`RapiDoc`] instance and then
//! serving it by what ever means available as `text/html` from http handler in your favourite web
//! framework.
//!
//! [`RapiDoc::to_html`] method can be used to convert the [`RapiDoc`] instance to a servable html
//! file.
//! ```
//! # use utoipa_rapidoc::RapiDoc;
//! # use utoipa::OpenApi;
//! # use serde_json::json;
//! # #[derive(OpenApi)]
//! # #[openapi()]
//! # struct ApiDoc;
//! #
//! let rapidoc = RapiDoc::new("/api-docs/openapi.json");
//!
//! // Then somewhere in your application that handles http operation.
//! // Make sure you return correct content type `text/html`.
//! let rapidoc_handler = move || {
//!     rapidoc.to_html()
//! };
//! ```
//!
//! # Customization
//!
//! Utoipa-rapidoc can be customized and configured only via [`RapiDoc::custom_html`] method. This
//! method empowers users to use a custom HTML template to modify the looks of the RapiDoc UI.
//!
//! * [All allowed RapiDoc configuration options][rapidoc_api]
//! * [Default HTML template][rapidoc_quickstart]
//!
//! The template should contain _**`$specUrl`**_ variable which will be replaced with user defined
//! OpenAPI spec url provided with [`RapiDoc::new`] function when creating a new [`RapiDoc`]
//! instance. Variable will be replaced during [`RapiDoc::to_html`] function execution.
//!
//! _**Overriding the HTML template with a custom one.**_
//! ```rust
//! # use utoipa_rapidoc::RapiDoc;
//! # use utoipa::OpenApi;
//! # use serde_json::json;
//! # #[derive(OpenApi)]
//! # #[openapi()]
//! # struct ApiDoc;
//! #
//! let html = "...";
//! RapiDoc::new("/api-docs/openapi.json").custom_html(html);
//! ```
//!
//! # Examples
//!
//! _**Serve [`RapiDoc`] via `actix-web` framework.**_
//! ```no_run
//! use actix_web::App;
//! use utoipa_rapidoc::RapiDoc;
//!
//! # use utoipa::OpenApi;
//! # use std::net::Ipv4Addr;
//! # #[derive(OpenApi)]
//! # #[openapi()]
//! # struct ApiDoc;
//! App::new().service(RapiDoc::with_openapi("/rapidoc", ApiDoc::openapi()));
//! ```
//!
//! _**Serve [`RapiDoc`] via `rocket` framework.**_
//! ```no_run
//! # use rocket;
//! use utoipa_rapidoc::RapiDoc;
//!
//! # use utoipa::OpenApi;
//! # #[derive(OpenApi)]
//! # #[openapi()]
//! # struct ApiDoc;
//! rocket::build()
//!     .mount(
//!         "/",
//!         RapiDoc::with_openapi("/rapidoc", ApiDoc::openapi()),
//!     );
//! ```
//!
//! _**Serve [`RapiDoc`] via `axum` framework.**_
//!  ```no_run
//!  use axum::Router;
//!  use utoipa_rapidoc::RapiDoc;
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
//!      .merge(RapiDoc::with_openapi("/rapidoc", ApiDoc::openapi()));
//! # }
//! ```
//!
//! [rapidoc_api]: <https://rapidocweb.com/api.html>
//! [examples]: <https://github.com/juhaku/utoipa/tree/master/examples>
//! [rapidoc_quickstart]: <https://rapidocweb.com/quickstart.html>

use std::borrow::Cow;

const DEFAULT_HTML: &str = include_str!("../res/rapidoc.html");

/// Is [RapiDoc][rapidoc] UI.
///
/// This is an entry point for serving [RapiDoc][rapidoc] via predefined framework integration or
/// in standalone fashion by calling [`RapiDoc::to_html`] within custom HTTP handler handles
/// serving the [RapiDoc][rapidoc] UI. See more at [running standalone][standalone]
///
/// [rapidoc]: <https://rapidocweb.com>
/// [standalone]: index.html#using-standalone
#[non_exhaustive]
pub struct RapiDoc {
    #[allow(unused)]
    path: Cow<'static, str>,
    spec_url: Cow<'static, str>,
    html: Cow<'static, str>,
    #[cfg(any(feature = "actix-web", feature = "rocket", feature = "axum"))]
    openapi: Option<utoipa::openapi::OpenApi>,
}

impl RapiDoc {
    /// Construct a new [`RapiDoc`] that points to given `spec_url`. Spec url must be valid URL and
    /// available for RapiDoc to consume.
    ///
    /// # Examples
    ///
    /// _**Create new [`RapiDoc`].**_
    ///
    /// ```
    /// # use utoipa_rapidoc::RapiDoc;
    /// RapiDoc::new("https://petstore3.swagger.io/api/v3/openapi.json");
    /// ```
    pub fn new<U: Into<Cow<'static, str>>>(spec_url: U) -> Self {
        Self {
            path: Cow::Borrowed(""),
            spec_url: spec_url.into(),
            html: Cow::Borrowed(DEFAULT_HTML),
            #[cfg(any(feature = "actix-web", feature = "rocket", feature = "axum"))]
            openapi: None,
        }
    }

    /// Construct a new [`RapiDoc`] with given `spec_url` and `openapi`. The spec url must point to
    /// the location where the `openapi` will be served.
    ///
    /// [`RapiDoc`] is only able to create endpoint that serves the `openapi` JSON for predefined
    /// frameworks. _**For other frameworks such endpoint must be created manually.**_
    ///
    /// # Examples
    ///
    /// _**Create new [`RapiDoc`].**_
    ///
    /// ```
    /// # use utoipa_rapidoc::RapiDoc;
    /// # use utoipa::OpenApi;
    /// # #[derive(OpenApi)]
    /// # #[openapi()]
    /// # struct ApiDoc;
    /// RapiDoc::with_openapi(
    ///     "/api-docs/openapi.json",
    ///     ApiDoc::openapi()
    /// );
    /// ```
    #[cfg(any(feature = "actix-web", feature = "rocket", feature = "axum"))]
    #[cfg_attr(
        doc_cfg,
        doc(cfg(any(feature = "actix-web", feature = "rocket", feature = "axum")))
    )]
    pub fn with_openapi<U: Into<Cow<'static, str>>>(
        spec_url: U,
        openapi: utoipa::openapi::OpenApi,
    ) -> Self {
        Self {
            path: Cow::Borrowed(""),
            spec_url: spec_url.into(),
            html: Cow::Borrowed(DEFAULT_HTML),
            openapi: Some(openapi),
        }
    }

    /// Override the [default HTML template][rapidoc_quickstart] with new one. See
    /// [customization] for more details.
    ///
    /// [rapidoc_quickstart]: <https://rapidocweb.com/quickstart.html>
    /// [customization]: index.html#customization
    pub fn custom_html<H: Into<Cow<'static, str>>>(mut self, html: H) -> Self {
        self.html = html.into();

        self
    }

    /// Add `path` the [`RapiDoc`] will be served from.
    ///
    /// # Examples
    ///
    /// _**Make [`RapiDoc`] servable from `/rapidoc` path.**_
    /// ```
    /// # use utoipa_rapidoc::RapiDoc;
    ///
    /// RapiDoc::new("https://petstore3.swagger.io/api/v3/openapi.json")
    ///     .path("/rapidoc");
    /// ```
    #[cfg(any(feature = "actix-web", feature = "rocket", feature = "axum"))]
    pub fn path<U: Into<Cow<'static, str>>>(mut self, path: U) -> Self {
        self.path = path.into();

        self
    }

    /// Converts this [`RapiDoc`] instance to servable HTML file.
    ///
    /// This will replace _**`$specUrl`**_ variable placeholder with the spec
    /// url provided to the [`RapiDoc`] instance. If HTML template is not overridden with
    /// [`RapiDoc::custom_html`] then the [default HTML template][rapidoc_quickstart]
    /// will be used.
    ///
    /// See more details in [customization][customization].
    ///
    /// [rapidoc_quickstart]: <https://rapidocweb.com/quickstart.html>
    /// [customization]: index.html#customization
    pub fn to_html(&self) -> String {
        self.html.replace("$specUrl", self.spec_url.as_ref())
    }
}

mod actix {
    #![cfg(feature = "actix-web")]

    use actix_web::dev::HttpServiceFactory;
    use actix_web::guard::Get;
    use actix_web::web::Data;
    use actix_web::{HttpResponse, Resource, Responder};

    use crate::RapiDoc;

    impl HttpServiceFactory for RapiDoc {
        fn register(self, config: &mut actix_web::dev::AppService) {
            let html = self.to_html();

            async fn serve_rapidoc(rapidoc: Data<String>) -> impl Responder {
                HttpResponse::Ok()
                    .content_type("text/html")
                    .body(rapidoc.to_string())
            }

            Resource::new(self.path.as_ref())
                .guard(Get())
                .app_data(Data::new(html))
                .to(serve_rapidoc)
                .register(config);

            if let Some(openapi) = self.openapi {
                async fn serve_openapi(openapi: Data<String>) -> impl Responder {
                    HttpResponse::Ok()
                        .content_type("application/json")
                        .body(openapi.into_inner().to_string())
                }

                Resource::new(self.spec_url.as_ref())
                    .guard(Get())
                    .app_data(Data::new(
                        openapi.to_json().expect("Should serialize to JSON"),
                    ))
                    .to(serve_openapi)
                    .register(config);
            }
        }
    }
}

mod axum {
    #![cfg(feature = "axum")]

    use axum::response::Html;
    use axum::{routing, Json, Router};

    use crate::RapiDoc;

    impl<R> From<RapiDoc> for Router<R>
    where
        R: Clone + Send + Sync + 'static,
    {
        fn from(value: RapiDoc) -> Self {
            let html = value.to_html();
            let openapi = value.openapi;

            let mut router = Router::<R>::new().route(
                value.path.as_ref(),
                routing::get(move || async { Html(html) }),
            );

            if let Some(openapi) = openapi {
                router = router.route(
                    value.spec_url.as_ref(),
                    routing::get(move || async { Json(openapi) }),
                );
            }

            router
        }
    }
}

mod rocket {
    #![cfg(feature = "rocket")]

    use rocket::http::Method;
    use rocket::response::content::RawHtml;
    use rocket::route::{Handler, Outcome};
    use rocket::serde::json::Json;
    use rocket::{Data, Request, Route};

    use crate::RapiDoc;

    impl From<RapiDoc> for Vec<Route> {
        fn from(value: RapiDoc) -> Self {
            let mut routes = vec![Route::new(
                Method::Get,
                value.path.as_ref(),
                RapiDocHandler(value.to_html()),
            )];

            if let Some(openapi) = value.openapi {
                routes.push(Route::new(
                    Method::Get,
                    value.spec_url.as_ref(),
                    OpenApiHandler(openapi),
                ));
            }

            routes
        }
    }

    #[derive(Clone)]
    struct RapiDocHandler(String);

    #[rocket::async_trait]
    impl Handler for RapiDocHandler {
        async fn handle<'r>(&self, request: &'r Request<'_>, _: Data<'r>) -> Outcome<'r> {
            Outcome::from(request, RawHtml(self.0.clone()))
        }
    }

    #[derive(Clone)]
    struct OpenApiHandler(utoipa::openapi::OpenApi);

    #[rocket::async_trait]
    impl Handler for OpenApiHandler {
        async fn handle<'r>(&self, request: &'r Request<'_>, _: Data<'r>) -> Outcome<'r> {
            Outcome::from(request, Json(self.0.clone()))
        }
    }
}
