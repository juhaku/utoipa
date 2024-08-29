//! Implements Router for composing handlers and collecting OpenAPI information.
use std::convert::Infallible;

use axum::extract::Request;
use axum::handler::Handler;
use axum::response::IntoResponse;
use axum::routing::{MethodRouter, Route, RouterAsService};
use axum::Router;
use tower_layer::Layer;
use tower_service::Service;

#[inline]
fn colonized_params<S: AsRef<str>>(path: S) -> String
where
    String: From<S>,
{
    String::from(path).replace('}', "").replace('{', ":")
}

/// Wrapper type for [`utoipa::openapi::path::Paths`] and [`axum::routing::MethodRouter`].
///
/// This is used with [`OpenApiRouter::routes`] method to register current _`paths`_ to the
/// [`utoipa::openapi::OpenApi`] of [`OpenApiRouter`] instance.
///
/// See [`routes`][routes] for usage.
///
/// [routes]: ../macro.routes.html
pub type UtoipaMethodRouter<S = ()> =
    (utoipa::openapi::path::Paths, axum::routing::MethodRouter<S>);

/// A wrapper struct for [`axum::Router`] and [`utoipa::openapi::OpenApi`] for composing handlers
/// and services with collecting OpenAPI information from the handlers.
///
/// This struct provides pass through implementation for most of the [`axum::Router`] methods and
/// extends capabilities for few to collect the OpenAPI information. Methods that are not
/// implemented can be easily called after converting this router to [`axum::Router`] by
/// [`Into::into`].
#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct OpenApiRouter<S = ()>(Router<S>, utoipa::openapi::OpenApi);

impl<S> OpenApiRouter<S>
where
    S: Send + Sync + Clone + 'static,
{
    /// Instantiate a new [`OpenApiRouter`] with new empty [`utoipa::openapi::OpenApi`].
    ///
    /// This is essentially same as calling
    /// _`OpenApiRouter::with_openapi(utoipa::openapi::OpenApiBuilder::new().build())`_.
    pub fn new() -> OpenApiRouter<S> {
        Self::with_openapi(utoipa::openapi::OpenApiBuilder::new().build())
    }

    /// Instantiates a new [`OpenApiRouter`] with given _`openapi`_ instance.
    ///
    /// This function allows using existing [`utoipa::openapi::OpenApi`] as source for this router.
    ///
    /// # Examples
    ///
    /// _**Use derived [`utoipa::openapi::OpenApi`] as source for [`OpenApiRouter`].**_
    /// ```rust
    /// # use utoipa::OpenApi;
    /// # use utoipa_axum::router::OpenApiRouter;
    /// #[derive(utoipa::ToSchema)]
    /// struct Todo {
    ///     id: i32,
    /// }
    /// #[derive(utoipa::OpenApi)]
    /// #[openapi(components(schemas(Todo)))]
    /// struct Api;
    ///
    /// let mut router: OpenApiRouter = OpenApiRouter::with_openapi(Api::openapi());
    /// ```
    pub fn with_openapi(openapi: utoipa::openapi::OpenApi) -> Self {
        Self(Router::new(), openapi)
    }

    /// Pass through method for [`axum::Router::as_service`].
    pub fn as_service<B>(&mut self) -> RouterAsService<'_, B, S> {
        self.0.as_service()
    }

    /// Pass through method for [`axum::Router::fallback`].
    pub fn fallback<H, T>(self, handler: H) -> Self
    where
        H: Handler<T, S>,
        T: 'static,
    {
        Self(self.0.fallback(handler), self.1)
    }

    /// Pass through method for [`axum::Router::fallback_service`].
    pub fn fallback_service<T>(self, service: T) -> Self
    where
        T: Service<Request, Error = Infallible> + Clone + Send + 'static,
        T::Response: IntoResponse,
        T::Future: Send + 'static,
    {
        Self(self.0.fallback_service(service), self.1)
    }

    /// Pass through method for [`axum::Router::layer`].
    pub fn layer<L>(self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + 'static,
        L::Service: Service<Request> + Clone + Send + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        Self(self.0.layer(layer), self.1)
    }

    /// Register [`UtoipaMethodRouter`] content created with [`routes`][routes] macro to `self`.
    ///
    /// Paths of the [`UtoipaMethodRouter`] will be extended to [`utoipa::openapi::OpenApi`] and
    /// [`axum::routing::MethodRouter`] will be added to the [`axum::Router`].
    ///
    /// [routes]: ../macro.routes.html
    pub fn routes(mut self, (mut paths, method_router): UtoipaMethodRouter<S>) -> Self {
        let router = if paths.paths.len() == 1 {
            let first_entry = &paths.paths.first_entry();
            let path = first_entry.as_ref().map(|path| path.key());
            let Some(path) = path else {
                unreachable!("Whoopsie, I thought there was one Path entry");
            };
            let path = if path.is_empty() { "/" } else { path };

            self.0.route(&colonized_params(path), method_router)
        } else {
            paths.paths.iter().fold(self.0, |this, (path, _)| {
                let path = if path.is_empty() { "/" } else { path };
                this.route(&colonized_params(path), method_router.clone())
            })
        };

        // add current paths to the OpenApi
        self.1.paths.paths.extend(paths.paths.clone());

        Self(router, self.1)
    }

    /// Pass through method for [`axum::Router<S>::route`].
    pub fn route(self, path: &str, method_router: MethodRouter<S>) -> Self {
        Self(self.0.route(&colonized_params(path), method_router), self.1)
    }

    /// Pass through method for [`axum::Router::route_layer`].
    pub fn route_layer<L>(self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + 'static,
        L::Service: Service<Request> + Clone + Send + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        Self(self.0.route_layer(layer), self.1)
    }

    /// Pass through method for [`axum::Router<S>::route_service`].
    pub fn route_service<T>(self, path: &str, service: T) -> Self
    where
        T: Service<Request, Error = Infallible> + Clone + Send + 'static,
        T::Response: IntoResponse,
        T::Future: Send + 'static,
    {
        Self(self.0.route_service(path, service), self.1)
    }

    /// Nest `router` to `self` under given `path`. Router routes will be nestsed with
    /// [`axum::Router::nest`].
    ///
    /// This method expects [`OpenApiRouter`] instance in order to nest OpenApi paths and router
    /// routes. If you wish to use [`axum::Router::nest`] you need to first convert this instance
    /// to [`axum::Router`] _(`let _: Router = OpenApiRouter::new().into()`)_.
    ///
    /// # Examples
    ///
    /// _**Nest two routers.**_
    /// ```rust
    /// # use utoipa_axum::{routes, PathItemExt, router::OpenApiRouter};
    /// #[utoipa::path(get, path = "/search")]
    /// async fn search() {}
    ///
    /// let search_router = OpenApiRouter::new()
    ///     .routes(utoipa_axum::routes!(search));
    ///
    /// let router: OpenApiRouter = OpenApiRouter::new()
    ///     .nest("/api", search_router);
    /// ```
    pub fn nest(self, path: &str, router: OpenApiRouter<S>) -> Self {
        let api = self.1.nest(path, router.1);
        let path = if path.is_empty() { "/" } else { path };
        let router = self.0.nest(&colonized_params(path), router.0);

        Self(router, api)
    }

    /// Pass through method for [`axum::Router::nest_service`]. _**This does nothing for OpenApi paths.**_
    pub fn nest_service<T>(self, path: &str, service: T) -> Self
    where
        T: Service<Request, Error = Infallible> + Clone + Send + 'static,
        T::Response: IntoResponse,
        T::Future: Send + 'static,
    {
        Self(self.0.nest_service(path, service), self.1)
    }

    /// Merge [`utoipa::openapi::path::Paths`] from `router` to `self` and merge [`Router`] routes
    /// and fallback with [`axum::Router::merge`].
    ///
    /// This method expects [`OpenApiRouter`] instance in order to merge OpenApi paths and router
    /// routes. If you wish to use [`axum::Router::merge`] you need to first convert this instance
    /// to [`axum::Router`] _(`let _: Router = OpenApiRouter::new().into()`)_.
    ///
    /// # Examples
    ///
    /// _**Merge two routers.**_
    /// ```rust
    /// # use utoipa_axum::{routes, PathItemExt, router::OpenApiRouter};
    /// #[utoipa::path(get, path = "/search")]
    /// async fn search() {}
    ///
    /// let search_router = OpenApiRouter::new()
    ///     .routes(utoipa_axum::routes!(search));
    ///
    /// let router: OpenApiRouter = OpenApiRouter::new()
    ///     .merge(search_router);
    /// ```
    pub fn merge(mut self, router: OpenApiRouter<S>) -> Self {
        self.1.merge(router.1);

        Self(self.0.merge(router.0), self.1)
    }

    /// Pass through method for [`axum::Router::with_state`].
    pub fn with_state<S2>(self, state: S) -> OpenApiRouter<S2> {
        OpenApiRouter(self.0.with_state(state), self.1)
    }

    /// Consume `self` returning the [`utoipa::openapi::OpenApi`] instance of the
    /// [`OpenApiRouter`].
    pub fn into_openapi(self) -> utoipa::openapi::OpenApi {
        self.1
    }

    /// Take the [`utoipa::openapi::OpenApi`] instance without consuming the [`OpenApiRouter`].
    pub fn to_openapi(&mut self) -> utoipa::openapi::OpenApi {
        std::mem::take(&mut self.1)
    }

    /// Get reference to the [`utoipa::openapi::OpenApi`] instance of the router.
    pub fn get_openapi(&self) -> &utoipa::openapi::OpenApi {
        &self.1
    }

    /// Split the content of the [`OpenApiRouter`] to parts. Method will return a tuple of
    /// inner [`axum::Router`] and [`utoipa::openapi::OpenApi`].
    pub fn split_for_parts(self) -> (axum::Router<S>, utoipa::openapi::OpenApi) {
        (self.0, self.1)
    }
}

impl<S> Default for OpenApiRouter<S>
where
    S: Send + Sync + Clone + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> From<OpenApiRouter<S>> for Router<S> {
    fn from(value: OpenApiRouter<S>) -> Self {
        value.0
    }
}

impl<S> From<Router<S>> for OpenApiRouter<S> {
    fn from(value: Router<S>) -> Self {
        OpenApiRouter(value, utoipa::openapi::OpenApiBuilder::new().build())
    }
}
