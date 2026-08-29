//! Implement `utoipa` extended [`ServiceConfig`] for [`ntex::web::ServiceConfig`].

use std::cell::Cell;

use ntex::web::{ErrorRenderer, Route, WebServiceFactory};

use crate::OpenApiFactory;

/// Wrapper type for [`ntex::web::ServiceConfig`], [`utoipa::openapi::path::Paths`] and
/// vec of [`utoipa::openapi::schema::Schema`] references.
pub struct ServiceConfig<'s, Err> {
    pub(super) service_config: &'s mut ntex::web::ServiceConfig<Err>,
    pub(super) paths: Cell<utoipa::openapi::path::Paths>,
    pub(super) schemas: Cell<
        Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    >,
}

impl<'s, Err: ErrorRenderer> ServiceConfig<'s, Err> {
    /// Construct a new [`ServiceConfig`] from given [`ntex::web::ServiceConfig`].
    pub fn new(service_config: &'s mut ntex::web::ServiceConfig<Err>) -> ServiceConfig<'s, Err> {
        ServiceConfig {
            service_config,
            paths: Cell::new(utoipa::openapi::path::Paths::new()),
            schemas: Cell::new(Vec::new()),
        }
    }

    /// Passthrough implementation for [`ntex::web::ServiceConfig::state`].
    pub fn state<S: 'static>(&mut self, st: S) -> &mut Self {
        self.service_config.state(st);
        self
    }

    /// Passthrough implementation for [`ntex::web::ServiceConfig::route`].
    pub fn route(&mut self, path: &str, route: Route<Err>) -> &mut Self {
        self.service_config.route(path, route);
        self
    }

    /// Counterpart for [`UtoipaApp::service`][utoipa_app_service].
    ///
    /// [utoipa_app_service]: ../struct.UtoipaApp.html#method.service
    pub fn service<F>(&mut self, factory: F) -> &mut Self
    where
        F: WebServiceFactory<Err> + OpenApiFactory + 'static,
    {
        let mut paths = self.paths.take();
        let other_paths = factory.paths();
        paths.merge(other_paths);

        let mut schemas = self.schemas.take();
        factory.schemas(&mut schemas);
        self.schemas.set(schemas);

        self.service_config.service(factory);
        self.paths.set(paths);

        self
    }

    /// Passthrough implementation for [`ntex::web::ServiceConfig::external_resource`].
    pub fn external_resource<N, U>(&mut self, name: N, url: U) -> &mut Self
    where
        N: AsRef<str>,
        U: AsRef<str>,
    {
        self.service_config.external_resource(name, url);
        self
    }

    /// Synonymous for [`UtoipaApp::map`][utoipa_app_map]
    ///
    /// [utoipa_app_map]: ../struct.UtoipaApp.html#method.map
    pub fn map<
        F: FnOnce(&mut ntex::web::ServiceConfig<Err>) -> &mut ntex::web::ServiceConfig<Err>,
    >(
        &mut self,
        op: F,
    ) -> &mut Self {
        op(self.service_config);

        self
    }
}
