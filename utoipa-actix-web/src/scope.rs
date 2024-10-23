//! Implement `utoipa` extended [`Scope`] for [`actix_web::Scope`].
//!
//! See usage from [`scope`][fn@scope].

use core::fmt;
use std::cell::{Cell, RefCell};

use actix_service::{IntoServiceFactory, ServiceFactory};
use actix_web::body::MessageBody;
use actix_web::dev::{AppService, HttpServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::guard::Guard;
use actix_web::{Error, Route};

use crate::service_config::ServiceConfig;
use crate::OpenApiFactory;

/// Wrapper type for [`actix_web::Scope`] and [`utoipa::openapi::OpenApi`] with additional path
/// prefix created with `scope::scope("path-prefix")` call.
///
/// See usage from [`scope`][fn@scope].
pub struct Scope<T>(
    actix_web::Scope<T>,
    RefCell<utoipa::openapi::OpenApi>,
    Cell<String>,
);

impl<T> From<actix_web::Scope<T>> for Scope<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    fn from(value: actix_web::Scope<T>) -> Self {
        Self(
            value,
            RefCell::new(utoipa::openapi::OpenApiBuilder::new().build()),
            Cell::new(String::new()),
        )
    }
}

impl<'s, T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>>
    From<&'s str> for Scope<T>
where
    Scope<T>: std::convert::From<actix_web::Scope>,
{
    fn from(value: &'s str) -> Self {
        let scope = actix_web::Scope::new(value);
        let s: Scope<T> = scope.into();
        Scope(s.0, s.1, Cell::new(String::from(value)))
    }
}

/// Create a new [`Scope`] with given _`scope`_ e.g. `scope("/api/v1")`.
///
/// This behaves exactly same way as [`actix_web::Scope`] but allows automatic _`schema`_ and
/// _`path`_ collection from `service(...)` calls directly or via [`ServiceConfig::service`].
///
/// # Examples
///
/// _**Create new scoped service.**_
///
/// ```rust
/// # use actix_web::{get, App};
/// # use utoipa_actix_web::{AppExt, scope};
/// #
///  #[utoipa::path()]
///  #[get("/handler")]
///  pub async fn handler() -> &'static str {
///      "OK"
///  }
/// let _ = App::new()
///     .into_utoipa_app()
///     .service(scope::scope("/api/v1/inner").configure(|cfg| {
///         cfg.service(handler);
///     }));
/// ```
pub fn scope<
    I: Into<Scope<T>>,
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
>(
    scope: I,
) -> Scope<T> {
    scope.into()
}

impl<T> Scope<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    /// Passthrough implementation for [`actix_web::Scope::guard`].
    pub fn guard<G: Guard + 'static>(self, guard: G) -> Self {
        let scope = self.0.guard(guard);
        Self(scope, self.1, self.2)
    }

    /// Passthrough implementation for [`actix_web::Scope::app_data`].
    pub fn app_data<U: 'static>(self, data: U) -> Self {
        Self(self.0.app_data(data), self.1, self.2)
    }

    /// Synonymous for [`UtoipaApp::configure`][utoipa_app_configure]
    ///
    /// [utoipa_app_configure]: ../struct.UtoipaApp.html#method.configure
    pub fn configure<F>(self, cfg_fn: F) -> Self
    where
        F: FnOnce(&mut ServiceConfig),
    {
        let mut openapi = self.1.borrow_mut();

        let scope = self.0.configure(|config| {
            let mut service_config = ServiceConfig::new(config);

            cfg_fn(&mut service_config);

            let other_paths = service_config.1.take();
            openapi.paths.merge(other_paths);
            let schemas = service_config.2.take();
            let components = openapi
                .components
                .get_or_insert(utoipa::openapi::Components::new());
            components.schemas.extend(schemas);
        });
        drop(openapi);

        Self(scope, self.1, self.2)
    }

    /// Synonymous for [`UtoipaApp::service`][utoipa_app_service]
    ///
    /// [utoipa_app_service]: ../struct.UtoipaApp.html#method.service
    pub fn service<F>(self, factory: F) -> Self
    where
        F: HttpServiceFactory + OpenApiFactory + 'static,
    {
        let mut schemas = Vec::<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>::new();
        {
            let mut openapi = self.1.borrow_mut();
            let other_paths = factory.paths();
            factory.schemas(&mut schemas);
            openapi.paths.merge(other_paths);
            let components = openapi
                .components
                .get_or_insert(utoipa::openapi::Components::new());
            components.schemas.extend(schemas);
        }

        let app = self.0.service(factory);

        Self(app, self.1, self.2)
    }

    /// Passthrough implementation for [`actix_web::Scope::route`].
    pub fn route(self, path: &str, route: Route) -> Self {
        Self(self.0.route(path, route), self.1, self.2)
    }

    /// Passthrough implementation for [`actix_web::Scope::default_service`].
    pub fn default_service<F, U>(self, f: F) -> Self
    where
        F: IntoServiceFactory<U, ServiceRequest>,
        U: ServiceFactory<
                ServiceRequest,
                Config = (),
                Response = ServiceResponse,
                Error = actix_web::Error,
            > + 'static,
        U::InitError: fmt::Debug,
    {
        Self(self.0.default_service(f), self.1, self.2)
    }

    /// Synonymous for [`UtoipaApp::map`][utoipa_app_map]
    ///
    /// [utoipa_app_map]: ../struct.UtoipaApp.html#method.map
    pub fn map<
        F: FnOnce(actix_web::Scope<T>) -> actix_web::Scope<NF>,
        NF: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
    >(
        self,
        op: F,
    ) -> Scope<NF> {
        let scope = op(self.0);
        Scope(scope, self.1, self.2)
    }
}

impl<T, B> HttpServiceFactory for Scope<T>
where
    T: ServiceFactory<
            ServiceRequest,
            Config = (),
            Response = ServiceResponse<B>,
            Error = Error,
            InitError = (),
        > + 'static,
    B: MessageBody + 'static,
{
    fn register(self, config: &mut AppService) {
        let Scope(scope, ..) = self;
        scope.register(config);
    }
}

impl<T> OpenApiFactory for Scope<T> {
    fn paths(&self) -> utoipa::openapi::path::Paths {
        let prefix = self.2.take();
        let mut openapi = self.1.borrow_mut();
        let mut paths = std::mem::take(&mut openapi.paths);

        let prefixed_paths = paths
            .paths
            .into_iter()
            .map(|(path, item)| {
                let path = format!("{prefix}{path}");

                (path, item)
            })
            .collect::<utoipa::openapi::path::PathsMap<_, _>>();
        paths.paths = prefixed_paths;

        paths
    }

    fn schemas(
        &self,
        schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        let mut api = self.1.borrow_mut();
        if let Some(components) = &mut api.components {
            schemas.extend(std::mem::take(&mut components.schemas));
        }
    }
}
