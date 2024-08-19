#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

//! Utoipa axum brings `utoipa` and `axum` closer together by the way of providing an ergonomic API that is extending on
//! the `axum` API. It gives a natural way to register handlers known to `axum` and also simultaneously generates OpenAPI
//! specification from the handlers.
//!
//! ## Install
//!
//! Add dependency declaration to `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! utoipa_axum = "0.1"
//! ```
//!
//! ## Examples
//!
//! _**Use [`OpenApiRouter`][router] to collect handlers with _`#[utoipa::path]`_ macro to compose service and form OpenAPI spec.**_
//!
//! ```rust
//! # use router::OpenApiRouter;
//!  #[derive(utoipa::ToSchema)]
//!  struct Todo {
//!      id: i32,
//!  }
//!  
//!  #[derive(utoipa::OpenApi)]
//!  #[openapi(components(schemas(Todo)))]
//!  struct Api;
//!  # #[utoipa::path(get, path = "/search")]
//!  # fn search_user() {}
//!  # #[utoipa::path(get, path = "")]
//!  # fn get_user() {}
//!  # #[utoipa::path(post, path = "")]
//!  # fn post_user() {}
//!  # #[utoipa::path(delete, path = "")]
//!  # fn delete_user() {}
//!  
//!  let mut router: OpenApiRouter = OpenApiRouter::with_openapi(Api::openapi())
//!      .routes(get_path(search_user))
//!      .routes(
//!          get_path(get_user)
//!              .post_path(post_user)
//!              .delete_path(delete_user),
//!      );
//!  
//!  let api = router.to_openapi();
//!  let axum_router: axum::Router = router.into();
//! ```
//!
//! [router]: router/struct.OpenApiRouter.html

pub mod router;

use std::convert::Infallible;

use axum::handler::Handler;
use axum::routing;
use axum::routing::{MethodFilter, MethodRouter};

use self::router::CURRENT_PATHS;

/// Extension trait of [`axum::handler::Handler`] that allows it to know it's OpenAPI path.
pub trait UtoipaHandler<T, S>: Handler<T, S>
where
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    /// Get path e.g. "/api/health" and path item ([`utoipa::openapi::path::PathItem`]) of the handler.
    ///
    /// Path and path item is used to construct the OpenAPI spec's path section with help of
    /// [`OpenApiRouter`][router].
    ///
    /// [router]: ./router/struct.OpenApiRouter.html
    fn get_path_and_item(&self) -> (String, utoipa::openapi::path::PathItem);
}

impl<T, S, P> UtoipaHandler<T, S> for P
where
    P: axum::handler::Handler<T, S> + utoipa::Path,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    fn get_path_and_item(&self) -> (String, utoipa::openapi::path::PathItem) {
        let path = P::path();
        let item = P::path_item();

        (path, item)
    }
}

macro_rules! chain_handle {
    ( $name:ident $method:ident) => {
        fn $name(self, handler: H) -> Self {
            let mut paths = CURRENT_PATHS.write().unwrap();

            let (path, item) = handler.get_path_and_item();
            paths.add_path(path, item);

            self.on(MethodFilter::$method, handler)
        }
    };
}

/// Extension trait of [`axum::routing::MethodRouter`] which adds _`utoipa`_ specific chainable
/// handler methods to the router.
///
/// The added methods works the same way as the axum ones but allows
/// automatic handler collection to the [`utoipa::openapi::OpenApi`] specification.
pub trait UtoipaMethodRouterExt<H, T> {
    /// Chain an additional `DELETE` requests using [`UtoipaHandler`].
    ///
    /// Using this insteand of axum routing alternative will allow automatic path collection for the
    /// [`utoipa::openapi::OpenApi`].
    ///
    /// Both the axum routing version and this can be used simulatenously but handlers registered with
    /// axum version will not get collected to the OpenAPI.
    fn delete_path(self, handler: H) -> Self;
    /// Chain an additional `GET` requests using [`UtoipaHandler`].
    ///
    /// Using this insteand of axum routing alternative will allow automatic path collection for the
    /// [`utoipa::openapi::OpenApi`].
    ///
    /// Both the axum routing version and this can be used simulatenously but handlers registered with
    /// axum version will not get collected to the OpenAPI.
    fn get_path(self, handler: H) -> Self;
    /// Chain an additional `HEAD` requests using [`UtoipaHandler`].
    ///
    /// Using this insteand of axum routing alternative will allow automatic path collection for the
    /// [`utoipa::openapi::OpenApi`].
    ///
    /// Both the axum routing version and this can be used simulatenously but handlers registered with
    /// axum version will not get collected to the OpenAPI.
    fn head_path(self, handler: H) -> Self;
    /// Chain an additional `OPTIONS` requests using [`UtoipaHandler`].
    ///
    /// Using this insteand of axum routing alternative will allow automatic path collection for the
    /// [`utoipa::openapi::OpenApi`].
    ///
    /// Both the axum routing version and this can be used simulatenously but handlers registered with
    /// axum version will not get collected to the OpenAPI.
    fn options_path(self, handler: H) -> Self;
    /// Chain an additional `PATCH` requests using [`UtoipaHandler`].
    ///
    /// Using this insteand of axum routing alternative will allow automatic path collection for the
    /// [`utoipa::openapi::OpenApi`].
    ///
    /// Both the axum routing version and this can be used simulatenously but handlers registered with
    /// axum version will not get collected to the OpenAPI.
    fn patch_path(self, handler: H) -> Self;
    /// Chain an additional `POST` requests using [`UtoipaHandler`].
    ///
    /// Using this insteand of axum routing alternative will allow automatic path collection for the
    /// [`utoipa::openapi::OpenApi`].
    ///
    /// Both the axum routing version and this can be used simulatenously but handlers registered with
    /// axum version will not get collected to the OpenAPI.
    fn post_path(self, handler: H) -> Self;
    /// Chain an additional `PUT` requests using [`UtoipaHandler`].
    ///
    /// Using this insteand of axum routing alternative will allow automatic path collection for the
    /// [`utoipa::openapi::OpenApi`].
    ///
    /// Both the axum routing version and this can be used simulatenously but handlers registered with
    /// axum version will not get collected to the OpenAPI.
    fn put_path(self, handler: H) -> Self;
    /// Chain an additional `TRACE` requests using [`UtoipaHandler`].
    ///
    /// Using this insteand of axum routing alternative will allow automatic path collection for the
    /// [`utoipa::openapi::OpenApi`].
    ///
    /// Both the axum routing version and this can be used simulatenously but handlers registered with
    /// axum version will not get collected to the OpenAPI.
    fn trace_path(self, handler: H) -> Self;
}

// routing::get
impl<H, T, S> UtoipaMethodRouterExt<H, T> for MethodRouter<S, Infallible>
where
    H: UtoipaHandler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    chain_handle!(delete_path DELETE);
    chain_handle!(get_path GET);
    chain_handle!(head_path HEAD);
    chain_handle!(options_path OPTIONS);
    chain_handle!(patch_path PATCH);
    chain_handle!(post_path POST);
    chain_handle!(put_path PUT);
    chain_handle!(trace_path TRACE);
}

macro_rules! top_level_handle {
    ( $name:ident $method:ident) => {

        #[doc = concat!("Route `", stringify!($method), "` requests to the given handler using [`UtoipaHandler`].")]
        #[doc = ""]
        #[doc = "Using this insteand of axum routing alternative will allow automatic path collection for the"]
        #[doc = "[`utoipa::openapi::OpenApi`]."]
        #[doc = ""]
        #[doc = "Both the axum routing version and this can be used simulatenously but handlers registered with"]
        #[doc = "axum version will not get collected to the OpenAPI."]
        pub fn $name<H, T, S>(handler: H) -> MethodRouter<S, Infallible>
        where
            H: UtoipaHandler<T, S>,
            T: 'static,
            S: Clone + Send + Sync + 'static,
        {
            let mut paths = CURRENT_PATHS.write().unwrap();

            let (path, item) = handler.get_path_and_item();
            paths.add_path(path, item);

            routing::on(MethodFilter::$method, handler)
        }
    };
}

top_level_handle!(delete_path DELETE);
top_level_handle!(get_path GET);
top_level_handle!(head_path HEAD);
top_level_handle!(options_path OPTIONS);
top_level_handle!(patch_path PATCH);
top_level_handle!(post_path POST);
top_level_handle!(put_path PUT);
top_level_handle!(trace_path TRACE);

#[cfg(test)]
mod tests {
    use std::marker::Send;

    use axum::extract::State;
    use utoipa::OpenApi;

    use self::router::OpenApiRouter;

    use super::*;

    #[utoipa::path(get, path = "/")]
    async fn root() {}

    #[utoipa::path(post, path = "/test")]
    async fn test() {}

    #[utoipa::path(post, path = "/health")]
    async fn health_handler() {}

    #[utoipa::path(post, path = "/api/foo")]
    async fn post_foo() {}

    // --- user

    #[utoipa::path(get, path = "/")]
    async fn get_user() {}

    #[utoipa::path(post, path = "/")]
    async fn post_user() {}

    #[utoipa::path(delete, path = "/")]
    async fn delete_user() {}

    #[utoipa::path(get, path = "/search")]
    async fn search_user() {}

    // --- customer

    #[utoipa::path(get, path = "/")]
    async fn get_customer() {}

    #[utoipa::path(post, path = "/")]
    async fn post_customer() {}

    #[utoipa::path(delete, path = "/")]
    async fn delete_customer() {}

    // test that with state handler compiles
    #[utoipa::path(get, path = "/search")]
    async fn search_customer(State(_s): State<String>) {}

    #[test]
    fn axum_router_nest_openapi_routes_compile() {
        let user_router: OpenApiRouter = OpenApiRouter::new().routes(get_path(search_user)).routes(
            get_path(get_user)
                .post_path(post_user)
                .delete_path(delete_user),
        );

        let customer_router: OpenApiRouter = OpenApiRouter::new()
            .routes(
                get_path(get_customer)
                    .post_path(post_customer)
                    .delete_path(delete_customer),
            )
            .routes(get_path(search_customer))
            .with_state(String::new());

        let router = OpenApiRouter::new()
            .nest("/api/user", user_router)
            .nest("/api/customer", customer_router)
            .route("/", get_path(root));

        let _ = router.get_openapi();
    }

    #[test]
    fn openapi_router_with_openapi() {
        #[derive(utoipa::ToSchema)]
        #[allow(unused)]
        struct Todo {
            id: i32,
        }
        #[derive(utoipa::OpenApi)]
        #[openapi(components(schemas(Todo)))]
        struct Api;

        let mut router: OpenApiRouter = OpenApiRouter::with_openapi(Api::openapi())
            .routes(get_path(search_user))
            .routes(get_path(get_user));

        let paths = router.to_openapi().paths;
        let expected_paths = utoipa::openapi::path::PathsBuilder::new()
            .path(
                "/",
                utoipa::openapi::PathItem::new(
                    utoipa::openapi::path::PathItemType::Get,
                    utoipa::openapi::path::OperationBuilder::new().operation_id(Some("get_user")),
                ),
            )
            .path(
                "/search",
                utoipa::openapi::PathItem::new(
                    utoipa::openapi::path::PathItemType::Get,
                    utoipa::openapi::path::OperationBuilder::new()
                        .operation_id(Some("search_user")),
                ),
            );
        assert_eq!(expected_paths.build(), paths);
    }

    #[test]
    fn openapi_router_nest_openapi() {
        #[derive(utoipa::ToSchema)]
        #[allow(unused)]
        struct Todo {
            id: i32,
        }
        #[derive(utoipa::OpenApi)]
        #[openapi(components(schemas(Todo)))]
        struct Api;

        let router: OpenApiRouter =
            OpenApiRouter::with_openapi(Api::openapi()).routes(get_path(search_user));

        let customer_router: OpenApiRouter = OpenApiRouter::new()
            .routes(get_path(get_customer))
            .with_state(String::new());

        let mut router = router.nest("/api/customer", customer_router);
        let paths = router.to_openapi().paths;
        let expected_paths = utoipa::openapi::path::PathsBuilder::new()
            .path(
                "/api/customer/",
                utoipa::openapi::PathItem::new(
                    utoipa::openapi::path::PathItemType::Get,
                    utoipa::openapi::path::OperationBuilder::new()
                        .operation_id(Some("get_customer")),
                ),
            )
            .path(
                "/search",
                utoipa::openapi::PathItem::new(
                    utoipa::openapi::path::PathItemType::Get,
                    utoipa::openapi::path::OperationBuilder::new()
                        .operation_id(Some("search_user")),
                ),
            );
        assert_eq!(expected_paths.build(), paths);
    }
}
