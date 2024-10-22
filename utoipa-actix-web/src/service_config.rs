//! Implement `utoipa` extended [`ServiceConfig`] for [`actix_web::web::ServiceConfig`].

use std::cell::Cell;

use actix_service::{IntoServiceFactory, ServiceFactory};
use actix_web::dev::{HttpServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::{Error, Route};

use crate::OpenApiFactory;

/// Wrapper type for [`actix_web::web::ServiceConfig`], [`utoipa::openapi::path::Paths`] and
/// vec of [`utoipa::openapi::schema::Schema`] references.
pub struct ServiceConfig<'s>(
    pub(super) &'s mut actix_web::web::ServiceConfig,
    pub(super) Cell<utoipa::openapi::path::Paths>,
    pub(super)  Cell<
        Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    >,
);

impl<'s> ServiceConfig<'s> {
    /// Construct a new [`ServiceConfig`] from given [`actix_web::web::ServiceConfig`].
    pub fn new(conf: &'s mut actix_web::web::ServiceConfig) -> ServiceConfig<'s> {
        ServiceConfig(
            conf,
            Cell::new(utoipa::openapi::path::Paths::new()),
            Cell::new(Vec::new()),
        )
    }

    /// Passthrough implementation for [`actix_web::web::ServiceConfig::app_data`].
    pub fn app_data<U: 'static>(&mut self, ext: U) -> &mut Self {
        self.0.app_data(ext);
        self
    }

    /// Passthrough implementation for [`actix_web::web::ServiceConfig::default_service`].
    pub fn default_service<F, U>(&mut self, f: F) -> &mut Self
    where
        F: IntoServiceFactory<U, ServiceRequest>,
        U: ServiceFactory<ServiceRequest, Config = (), Response = ServiceResponse, Error = Error>
            + 'static,
        U::InitError: std::fmt::Debug,
    {
        self.0.default_service(f);
        self
    }

    /// Passthrough implementation for [`actix_web::web::ServiceConfig::configure`].
    pub fn configure<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut ServiceConfig),
    {
        f(self);
        self
    }

    /// Passthrough implementation for [`actix_web::web::ServiceConfig::route`].
    pub fn route(&mut self, path: &str, route: Route) -> &mut Self {
        self.0.route(path, route);
        self
    }

    /// Counterpart for [`UtoipaApp::service`][utoipa_app_service].
    ///
    /// [utoipa_app_service]: ../struct.UtoipaApp.html#method.service
    pub fn service<F>(&mut self, factory: F) -> &mut Self
    where
        F: HttpServiceFactory + OpenApiFactory + 'static,
    {
        let mut paths = self.1.take();
        let other_paths = factory.paths();
        paths.paths.extend(other_paths.paths);
        let mut schemas = self.2.take();
        factory.schemas(&mut schemas);
        self.2.set(schemas);

        self.0.service(factory);
        self.1.set(paths);

        self
    }

    /// Passthrough implementation for [`actix_web::web::ServiceConfig::external_resource`].
    pub fn external_resource<N, U>(&mut self, name: N, url: U) -> &mut Self
    where
        N: AsRef<str>,
        U: AsRef<str>,
    {
        self.0.external_resource(name, url);
        self
    }
}
