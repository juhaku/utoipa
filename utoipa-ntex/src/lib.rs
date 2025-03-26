//! This crate implements necessary bindings for automatically collecting `paths` and `schemas` recursively from Actix Web
//! `App`, `Scope` and `ServiceConfig`. It provides natural API reducing duplication and support for scopes while generating
//! OpenAPI specification without the need to declare `paths` and `schemas` to `#[openapi(...)]` attribute of `OpenApi` derive.
//!
//! Currently only `service(...)` calls supports automatic collection of schemas and paths. Manual routes via `route(...)` or
//! `Route::new().to(...)` is not supported.
//!
//! ## Install
//!
//! Add dependency declaration to `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! utoipa-ntex = "0.1"
//! ```
//!
//! ## Examples
//!
//! _**Collect handlers annotated with `#[utoipa::path]` recursively from `service(...)` calls to compose OpenAPI spec.**_
//!
//! ```rust
//! use ntex::web::{Json, get, App};
//! use utoipa_ntex::{scope, AppExt};
//!
//! #[derive(utoipa::ToSchema, serde::Serialize)]
//! struct User {
//!     id: i32,
//! }
//!
//! #[utoipa::path(responses((status = OK, body = User)))]
//! #[get("/user")]
//! async fn get_user() -> Json<User> {
//!     Json(User { id: 1 })
//! }
//!
//! let (_, mut api) = App::new()
//!     .into_utoipa_app()
//!     .service(scope::scope("/api/v1").service(get_user))
//!     .split_for_parts();
//! ```

#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

pub mod scope;
pub mod service_config;

use std::fmt;

use ntex::{
    IntoServiceFactory, ServiceFactory,
    web::{ErrorRenderer, Route, WebRequest, WebResponse, WebServiceFactory, stack::WebStack},
};
use service_config::ServiceConfig;
use utoipa::{OpenApi, openapi::PathItem};

/// This trait is used to unify OpenAPI items collection from types implementing this trait.
pub trait OpenApiFactory {
    /// Get OpenAPI paths.
    fn paths(&self) -> utoipa::openapi::path::Paths;
    /// Collect schema reference and append them to the _`schemas`_.
    fn schemas(
        &self,
        schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    );
}

impl<'t, T: utoipa::Path + utoipa::__dev::SchemaReferences + utoipa::__dev::Tags<'t>> OpenApiFactory
    for T
{
    fn paths(&self) -> utoipa::openapi::path::Paths {
        let methods = T::methods();

        methods
            .into_iter()
            .fold(
                utoipa::openapi::path::Paths::builder(),
                |mut builder, method| {
                    let mut operation = T::operation();
                    let other_tags = T::tags();
                    if !other_tags.is_empty() {
                        let tags = operation.tags.get_or_insert(Vec::new());
                        tags.extend(other_tags.into_iter().map(ToString::to_string));
                    };

                    let path_item = PathItem::new(method, operation);
                    builder = builder.path(T::path(), path_item);

                    builder
                },
            )
            .build()
    }

    fn schemas(
        &self,
        schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        <T as utoipa::__dev::SchemaReferences>::schemas(schemas);
    }
}

/// Extends [`ntex::web::App`] with `utoipa` related functionality.
pub trait AppExt<M, F, Err>
where
    Err: ErrorRenderer,
{
    /// Convert's this [`ntex::web::App`] to [`UtoipaApp`].
    ///
    /// See usage from [`UtoipaApp`][struct@UtoipaApp]
    fn into_utoipa_app(self) -> UtoipaApp<M, F, Err>;
}

impl<M, F, Err> AppExt<M, F, Err> for ntex::web::App<M, F, Err>
where
    Err: ErrorRenderer,
{
    fn into_utoipa_app(self) -> UtoipaApp<M, F, Err> {
        UtoipaApp::from(self)
    }
}

/// Wrapper type for [`ntex::web::App`] and [`utoipa::openapi::OpenApi`].
///
/// [`UtoipaApp`] behaves exactly same way as [`ntex::web::App`] but allows automatic _`schema`_ and
/// _`path`_ collection from `service(...)` calls directly or via [`ServiceConfig::service`].
///
/// It exposes typical methods from [`ntex::web::App`] and provides custom [`UtoipaApp::map`]
/// method to add additional configuration options to wrapper [`ntex::web::App`].
///
/// This struct need be instantiated from [`ntex::web::App`] by calling `.into_utoipa_app()`
/// because we do not have access to _`ntex::web::App<M, F, Err>`_ generic argument and the _`App`_ does
/// not provide any default implementation.
///
/// # Examples
///
/// _**Create new [`UtoipaApp`] instance.**_
/// ```rust
/// # use utoipa_ntex::{AppExt, UtoipaApp};
/// # use ntex::web::App;
/// let utoipa_app = App::new().into_utoipa_app();
/// ```
///
/// _**Convert `ntex::web::App<M, F, Err>` to `UtoipaApp<M, F, Err>`.**_
/// ```rust
/// # use utoipa_ntex::{AppExt, UtoipaApp};
/// # use ntex::web::App;
/// let a: UtoipaApp<_> = ntex::web::App::new().into();
/// ```
pub struct UtoipaApp<M, F, Err>(ntex::web::App<M, F, Err>, utoipa::openapi::OpenApi)
where
    Err: ErrorRenderer;

impl<M, T, Err> From<ntex::web::App<M, T, Err>> for UtoipaApp<M, T, Err>
where
    Err: ErrorRenderer,
{
    fn from(value: ntex::web::App<M, T, Err>) -> Self {
        #[derive(OpenApi)]
        struct Api;
        UtoipaApp(value, Api::openapi())
    }
}

impl<M, T, Err> UtoipaApp<M, T, Err>
where
    T: ServiceFactory<
            WebRequest<Err>,
            Response = WebRequest<Err>,
            Error = Err::Container,
            InitError = (),
        >,
    Err: ErrorRenderer,
{
    /// Replace the wrapped [`utoipa::openapi::OpenApi`] with given _`openapi`_.
    ///
    /// This is useful to prepend OpenAPI doc generated with [`UtoipaApp`]
    /// with content that cannot be provided directly via [`UtoipaApp`].
    ///
    /// # Examples
    ///
    /// _**Replace wrapped [`utoipa::openapi::OpenApi`] with custom one.**_
    /// ```rust
    /// # use utoipa_ntex::web::{AppExt, UtoipaApp};
    /// # use ntex::web::App;
    /// # use utoipa::OpenApi;
    /// #[derive(OpenApi)]
    /// #[openapi(info(title = "Api title"))]
    /// struct Api;
    ///
    /// let _ = ntex::web::App::new().into_utoipa_app().openapi(Api::openapi());
    /// ```
    pub fn openapi(mut self, openapi: utoipa::openapi::OpenApi) -> Self {
        self.1 = openapi;

        self
    }

    /// Passthrough implementation for [`ntex::web::App::state`].
    pub fn state<U: 'static>(self, state: U) -> Self {
        Self(self.0.state(state), self.1)
    }

    /// Passthrough implementation for [`ntex::web::App::state_factory`].
    pub fn state_factory<F, Out, D, E>(self, state: F) -> Self
    where
        F: Fn() -> Out + 'static,
        Out: Future<Output = Result<D, E>> + 'static,
        D: 'static,
        E: fmt::Debug,
    {
        Self(self.0.state_factory(state), self.1)
    }

    /// Extended version of [`ntex::web::App::configure`] which handles _`schema`_ and _`path`_
    /// collection from [`ServiceConfig`] into the wrapped [`utoipa::openapi::OpenApi`] instance.
    pub fn configure<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut ServiceConfig<Err>),
    {
        let mut openapi = self.1;

        let app = self.0.configure(|config| {
            let mut service_config = ServiceConfig::new(config);

            f(&mut service_config);

            let paths = service_config.paths.take();
            openapi.paths.merge(paths);
            let schemas = service_config.schemas.take();
            let components = openapi
                .components
                .get_or_insert(utoipa::openapi::Components::new());
            components.schemas.extend(schemas);
        });

        Self(app, openapi)
    }

    /// Passthrough implementation for [`ntex::web::App::route`].
    pub fn route(self, path: &str, route: Route<Err>) -> Self {
        Self(self.0.route(path, route), self.1)
    }

    /// Extended version of [`ntex::web::App::service`] method which handles _`schema`_ and _`path`_
    /// collection from [`HttpServiceFactory`].
    pub fn service<F>(self, factory: F) -> Self
    where
        F: WebServiceFactory<Err> + OpenApiFactory + 'static,
    {
        let mut schemas = Vec::<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>::new();

        factory.schemas(&mut schemas);
        let paths = factory.paths();

        let mut openapi = self.1;

        openapi.paths.merge(paths);
        let components = openapi
            .components
            .get_or_insert(utoipa::openapi::Components::new());
        components.schemas.extend(schemas);

        let app = self.0.service(factory);

        Self(app, openapi)
    }

    /// Helper method to serve wrapped [`utoipa::openapi::OpenApi`] via [`HttpServiceFactory`].
    ///
    /// This method functions as a convenience to serve the wrapped OpenAPI spec alternatively to
    /// first call [`UtoipaApp::split_for_parts`] and then calling [`ntex::web::App::service`].
    pub fn openapi_service<O, F>(self, factory: F) -> Self
    where
        F: FnOnce(utoipa::openapi::OpenApi) -> O,
        O: WebServiceFactory<Err> + 'static,
    {
        let service = factory(self.1.clone());
        let app = self.0.service(service);
        Self(app, self.1)
    }

    /// Passthrough implementation for [`ntex::web::App::default_service`].
    pub fn default_service<F, U>(self, f: F) -> Self
    where
        F: IntoServiceFactory<U, WebRequest<Err>>,
        U: ServiceFactory<WebRequest<Err>, Response = WebResponse, Error = Err::Container>
            + 'static,
        U::InitError: fmt::Debug,
    {
        Self(self.0.default_service(f), self.1)
    }

    /// Passthrough implementation for [`ntex::web::App::external_resource`].
    pub fn external_resource<N, U>(self, name: N, url: U) -> Self
    where
        N: AsRef<str>,
        U: AsRef<str>,
    {
        Self(self.0.external_resource(name, url), self.1)
    }

    /// Convenience method to add custom configuration to [`ntex::web::App`] that is not directly
    /// exposed via [`UtoipaApp`]. This could for example be adding middlewares.
    ///
    /// # Examples
    ///
    /// _**Add middleware via `map` method.**_
    ///
    /// ```rust
    /// # use utoipa_btex::{AppExt, UtoipaApp};
    /// # use actix_service::Service;
    /// # use ntex::web::{App, http::header::{HeaderValue, CONTENT_TYPE}};
    ///  let _ = App::new()
    ///     .into_utoipa_app()
    ///     .map(|app| {
    ///            app.wrap_fn(|req, srv| {
    ///                let fut = srv.call(req);
    ///                async {
    ///                    let mut res = fut.await?;
    ///                    res.headers_mut()
    ///                        .insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
    ///                    Ok(res)
    ///                }
    ///            })
    ///        });
    /// ```
    pub fn map<F: FnOnce(ntex::web::App<M, T, Err>) -> ntex::web::App<M, T, Err>>(
        self,
        op: F,
    ) -> UtoipaApp<M, T, Err> {
        let app = op(self.0);
        UtoipaApp(app, self.1)
    }

    /// Passthrough implementation for [`ntex::web::App::filter`].
    pub fn filter<S, U>(
        self,
        filter: U,
    ) -> UtoipaApp<
        M,
        impl ServiceFactory<
            WebRequest<Err>,
            Response = WebRequest<Err>,
            Error = Err::Container,
            InitError = (),
        >,
        Err,
    >
    where
        S: ServiceFactory<WebRequest<Err>, Response = WebRequest<Err>, Error = Err::Container>,
        U: IntoServiceFactory<S, WebRequest<Err>>,
    {
        UtoipaApp(self.0.filter(filter), self.1)
    }

    /// Passthrough implementation for [`ntex::web::App::wrap`].
    pub fn wrap<U>(self, mw: U) -> UtoipaApp<WebStack<M, U, Err>, T, Err> {
        UtoipaApp(self.0.wrap(mw), self.1)
    }

    /// Passthrough implementation for [`ntex::web::App::case_insensitive_routing`].
    pub fn case_insensitive_routing(self) -> Self {
        Self(self.0.case_insensitive_routing(), self.1)
    }

    /// Split this [`UtoipaApp`] into parts returning tuple of [`actix_web::App`] and
    /// [`utoipa::openapi::OpenApi`] of this instance.
    pub fn split_for_parts(self) -> (ntex::web::App<M, T, Err>, utoipa::openapi::OpenApi) {
        (self.0, self.1)
    }

    /// Converts this [`UtoipaApp`] into the wrapped [`ntex::web::App`].
    pub fn into_app(self) -> ntex::web::App<M, T, Err> {
        self.0
    }
}

impl<M, F, Err> From<UtoipaApp<M, F, Err>> for ntex::web::App<M, F, Err>
where
    Err: ErrorRenderer,
{
    fn from(value: UtoipaApp<M, F, Err>) -> Self {
        value.0
    }
}
