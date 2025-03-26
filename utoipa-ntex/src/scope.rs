//! Implement `utoipa` extended [`Scope`] for [`ntex::web::Scope`].
//!
//! See usage from [`scope`][fn@scope].
use std::{
    cell::{Cell, RefCell},
    fmt,
};

use ntex::{
    IntoServiceFactory, ServiceFactory,
    web::{
        ErrorRenderer, Route, WebRequest, WebResponse, WebServiceFactory, guard::Guard,
        stack::WebStack,
    },
};

use crate::{OpenApiFactory, service_config::ServiceConfig};

/// Wrapper type for [`ntex::web::Scope`] and [`utoipa::openapi::OpenApi`] with additional path
/// prefix created with `scope::scope("path-prefix")` call.
///
/// See usage from [`scope`][fn@scope].
pub struct Scope<Err, M, T>(
    ntex::web::Scope<Err, M, T>,
    RefCell<utoipa::openapi::OpenApi>,
    Cell<String>,
)
where
    Err: ErrorRenderer;

impl<Err, M, T> From<ntex::web::Scope<Err, M, T>> for Scope<Err, M, T>
where
    T: ServiceFactory<
            WebRequest<Err>,
            Response = WebRequest<Err>,
            Error = Err::Container,
            InitError = (),
        >,
    Err: ErrorRenderer,
{
    fn from(value: ntex::web::Scope<Err, M, T>) -> Self {
        Self(
            value,
            RefCell::new(utoipa::openapi::OpenApiBuilder::new().build()),
            Cell::new(String::new()),
        )
    }
}

impl<'s, Err, M, T> From<&'s str> for Scope<Err, M, T>
where
    Scope<Err, M, T>: std::convert::From<ntex::web::Scope<Err>>,
    T: ServiceFactory<
            WebRequest<Err>,
            Response = WebRequest<Err>,
            Error = Err::Container,
            InitError = (),
        >,
    Err: ErrorRenderer,
{
    fn from(value: &'s str) -> Self {
        let scope = ntex::web::Scope::<Err>::new(value);
        let s: Scope<Err, M, T> = scope.into();
        Scope(s.0, s.1, Cell::new(String::from(value)))
    }
}

/// Create a new [`Scope`] with given _`scope`_ e.g. `scope("/api/v1")`.
///
/// This behaves exactly same way as [`ntex::web::Scope`] but allows automatic _`schema`_ and
/// _`path`_ collection from `service(...)` calls directly or via [`ServiceConfig::service`].
///
/// # Examples
///
/// _**Create new scoped service.**_
///
/// ```rust
/// # use ntex::web::{get, App};
/// # use utoipa_ntex::{AppExt, scope};
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
pub fn scope<Err, M, T, I>(scope: I) -> Scope<Err, M, T>
where
    I: Into<Scope<Err, M, T>>,
    T: ServiceFactory<
            WebRequest<Err>,
            Response = WebRequest<Err>,
            Error = Err::Container,
            InitError = (),
        >,
    Err: ErrorRenderer,
{
    scope.into()
}

impl<Err, M, T> Scope<Err, M, T>
where
    T: ServiceFactory<
            WebRequest<Err>,
            Response = WebRequest<Err>,
            Error = Err::Container,
            InitError = (),
        >,
    Err: ErrorRenderer,
{
    /// Passthrough implementation for [`ntex::web::Scope::guard`].
    pub fn guard<G: Guard + 'static>(self, guard: G) -> Self {
        Self(self.0.guard(guard), self.1, self.2)
    }

    /// Passthrough implementation for [`ntex::web::Scope::state`].
    pub fn state<D: 'static>(self, st: D) -> Self {
        Self(self.0.state(st), self.1, self.2)
    }

    /// Passthrough implementation for [`ntex::web::Scope::case_insensitive_routing`].
    pub fn case_insensitive_routing(self) -> Self {
        Self(self.0.case_insensitive_routing(), self.1, self.2)
    }

    /// Synonymous for [`UtoipaApp::configure`][utoipa_app_configure]
    ///
    /// [utoipa_app_configure]: ../struct.UtoipaApp.html#method.configure
    pub fn configure<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut ServiceConfig<Err>),
    {
        let mut openapi = self.1.borrow_mut();

        let scope = self.0.configure(|config| {
            let mut service_config = ServiceConfig::new(config);

            f(&mut service_config);

            let other_paths = service_config.paths.take();
            openapi.paths.merge(other_paths);
            let schemas = service_config.schemas.take();
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
        F: WebServiceFactory<Err> + OpenApiFactory + 'static,
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

    /// Passthrough implementation for [`ntex::web::Scope::route`].
    pub fn route(self, path: &str, route: Route<Err>) -> Self {
        Self(self.0.route(path, route), self.1, self.2)
    }

    /// Passthrough implementation for [`ntex::web::Scope::default_service`].
    pub fn default_service<F, S>(self, f: F) -> Self
    where
        F: IntoServiceFactory<S, WebRequest<Err>>,
        S: ServiceFactory<WebRequest<Err>, Response = WebResponse, Error = Err::Container>
            + 'static,
        S::InitError: fmt::Debug,
    {
        Self(self.0.default_service(f), self.1, self.2)
    }

    /// Passthrough implementation for [`ntex::web::Scope::filter`].
    pub fn filter<U, F>(
        self,
        filter: F,
    ) -> Scope<
        Err,
        M,
        impl ServiceFactory<
            WebRequest<Err>,
            Response = WebRequest<Err>,
            Error = Err::Container,
            InitError = (),
        >,
    >
    where
        U: ServiceFactory<WebRequest<Err>, Response = WebRequest<Err>, Error = Err::Container>,
        F: IntoServiceFactory<U, WebRequest<Err>>,
    {
        Scope(self.0.filter(filter), self.1, self.2)
    }

    /// Passthrough implementation for [`ntex::web::Scope::wrap`].
    pub fn wrap<U>(self, mw: U) -> Scope<Err, WebStack<M, U, Err>, T> {
        Scope(self.0.wrap(mw), self.1, self.2)
    }
}

impl<Err, M, T> OpenApiFactory for Scope<Err, M, T>
where
    T: ServiceFactory<
            WebRequest<Err>,
            Response = WebRequest<Err>,
            Error = Err::Container,
            InitError = (),
        >,
    Err: ErrorRenderer,
{
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
