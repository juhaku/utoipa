#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

//! Utoipa axum brings `utoipa` and `axum` closer together by the way of providing an ergonomic API that is extending on
//! the `axum` API. It gives a natural way to register handlers known to `axum` and also simultaneously generates OpenAPI
//! specification from the handlers.
//!
//! ## Crate features
//!
//! - **`debug`**: Implement debug traits for types.
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
//! # use utoipa::OpenApi;
//! # use utoipa_axum::{routes, PathItemExt, router::OpenApiRouter};
//!  #[derive(utoipa::ToSchema)]
//!  struct Todo {
//!      id: i32,
//!  }
//!  
//!  #[derive(utoipa::OpenApi)]
//!  #[openapi(components(schemas(Todo)))]
//!  struct Api;
//!  # #[utoipa::path(get, path = "/search")]
//!  # async fn search_user() {}
//!  # #[utoipa::path(get, path = "")]
//!  # async fn get_user() {}
//!  # #[utoipa::path(post, path = "")]
//!  # async fn post_user() {}
//!  # #[utoipa::path(delete, path = "")]
//!  # async fn delete_user() {}
//!  
//!  let mut router: OpenApiRouter = OpenApiRouter::with_openapi(Api::openapi())
//!      .routes(routes!(search_user))
//!      .routes(routes!(get_user, post_user, delete_user));
//!  
//!  let api = router.to_openapi();
//!  let axum_router: axum::Router = router.into();
//! ```
//!
//! [router]: router/struct.OpenApiRouter.html

pub mod router;

use axum::routing::MethodFilter;
use utoipa::openapi::HttpMethod;

/// Extends [`utoipa::openapi::path::PathItem`] by providing conversion methods to convert this
/// path item type to a [`axum::routing::MethodFilter`].
pub trait PathItemExt {
    /// Convert this path item type ot a [`axum::routing::MethodFilter`].
    ///
    /// Method filter is used with handler registration on [`axum::routing::MethodRouter`].
    fn to_method_filter(&self) -> MethodFilter;
}

impl PathItemExt for HttpMethod {
    fn to_method_filter(&self) -> MethodFilter {
        match self {
            HttpMethod::Get => MethodFilter::GET,
            HttpMethod::Put => MethodFilter::PUT,
            HttpMethod::Post => MethodFilter::POST,
            HttpMethod::Head => MethodFilter::HEAD,
            HttpMethod::Patch => MethodFilter::PATCH,
            HttpMethod::Trace => MethodFilter::TRACE,
            HttpMethod::Delete => MethodFilter::DELETE,
            HttpMethod::Options => MethodFilter::OPTIONS,
        }
    }
}

/// re-export paste so users do not need to add the dependency.
#[doc(hidden)]
pub use paste::paste;

/// Collect axum handlers annotated with [`utoipa::path`] to [`router::UtoipaMethodRouter`].
///
/// [`routes`] macro will return [`router::UtoipaMethodRouter`] which contains an
/// [`axum::routing::MethodRouter`] and currenty registered paths. The output of this macro is
/// meant to be used together with [`router::OpenApiRouter`] which combines the paths and axum
/// routers to a single entity.
///
/// Only handlers collected with [`routes`] macro will get registered to the OpenApi.
///
/// # Panics
///
/// Routes registered via [`routes`] macro or via `axum::routing::*` operations are bound to same
/// rules where only one one HTTP method can can be registered once per call. This means that the
/// following will produce runtime panic from axum code.
///
/// ```rust,no_run
/// # use utoipa_axum::{routes, router::UtoipaMethodRouter};
/// # use utoipa::path;
///  #[utoipa::path(get, path = "/search")]
///  async fn search_user() {}
///
///  #[utoipa::path(get, path = "")]
///  async fn get_user() {}
///
///  let _: UtoipaMethodRouter = routes!(get_user, search_user);
/// ```
/// Since the _`axum`_ does not support method filter for `CONNECT` requests, using this macro with
/// handler having request method type `CONNECT` `#[utoipa::path(connect, path = "")]` will panic at
/// runtime.
///
/// # Examples
///
/// _**Create new `OpenApiRouter` with `get_user` and `post_user` paths.**_
/// ```rust
/// # use utoipa_axum::{routes, router::{OpenApiRouter, UtoipaMethodRouter}};
/// # use utoipa::path;
///  #[utoipa::path(get, path = "")]
///  async fn get_user() {}
///
///  #[utoipa::path(post, path = "")]
///  async fn post_user() {}
///
///  let _: OpenApiRouter = OpenApiRouter::new().routes(routes!(get_user, post_user));
/// ```
#[macro_export]
macro_rules! routes {
    ( $handler:ident $(, $tail:tt)* ) => {
        {
            use $crate::PathItemExt;
            let mut paths = utoipa::openapi::path::Paths::new();
            let (path, item, types) = routes!(@resolve_types $handler);
            #[allow(unused_mut)]
            let mut method_router = types.iter().by_ref().fold(axum::routing::MethodRouter::new(), |router, path_type| {
                router.on(path_type.to_method_filter(), $handler)
            });
            for method_type in types {
                paths.add_path(&path, utoipa::openapi::path::PathItem::new(method_type, item.clone()));
            }
            $( method_router = routes!( method_router: paths: $tail ); )*
            (paths, method_router)
        }
    };
    ( $router:ident: $paths:ident: $handler:ident $(, $tail:tt)* ) => {
        {
            let (path, item, types) = routes!(@resolve_types $handler);
            let router = types.iter().by_ref().fold($router, |router, path_type| {
                router.on(path_type.to_method_filter(), $handler)
            });
            for method_type in types {
                $paths.add_path(&path, utoipa::openapi::path::PathItem::new(method_type, item.clone()));
            }
            router
        }
    };
    ( @resolve_types $handler:ident ) => {
        {
            use utoipa::{Path, __dev::Tags};
            $crate::paste! {
                let path = [<__path_ $handler>]::path();
                let mut operation = [<__path_ $handler>]::operation();
                let types = [<__path_ $handler>]::methods();
                let tags = [< __path_ $handler>]::tags();
                if !tags.is_empty() {
                    let operation_tags = operation.tags.get_or_insert(Vec::new());
                    operation_tags.extend(tags.iter().map(ToString::to_string));
                }
                (path, operation, types)
            }
        }
    };
    ( ) => {};
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use router::*;

    #[utoipa::path(get, path = "/")]
    async fn root() {}

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
        let user_router: OpenApiRouter = OpenApiRouter::new()
            .routes(routes!(search_user))
            .routes(routes!(get_user, post_user, delete_user));

        let customer_router: OpenApiRouter = OpenApiRouter::new()
            .routes(routes!(get_customer, post_customer, delete_customer))
            .routes(routes!(search_customer))
            .with_state(String::new());

        let router = OpenApiRouter::new()
            .nest("/api/user", user_router)
            .nest("/api/customer", customer_router)
            .route("/", axum::routing::get(root));

        let _ = router.get_openapi();
    }

    #[test]
    fn openapi_router_with_openapi() {
        use utoipa::OpenApi;

        #[derive(utoipa::ToSchema)]
        #[allow(unused)]
        struct Todo {
            id: i32,
        }
        #[derive(utoipa::OpenApi)]
        #[openapi(components(schemas(Todo)))]
        struct Api;

        let mut router: OpenApiRouter = OpenApiRouter::with_openapi(Api::openapi())
            .routes(routes!(search_user))
            .routes(routes!(get_user));

        let paths = router.to_openapi().paths;
        let expected_paths = utoipa::openapi::path::PathsBuilder::new()
            .path(
                "/",
                utoipa::openapi::PathItem::new(
                    utoipa::openapi::path::HttpMethod::Get,
                    utoipa::openapi::path::OperationBuilder::new().operation_id(Some("get_user")),
                ),
            )
            .path(
                "/search",
                utoipa::openapi::PathItem::new(
                    utoipa::openapi::path::HttpMethod::Get,
                    utoipa::openapi::path::OperationBuilder::new()
                        .operation_id(Some("search_user")),
                ),
            );
        assert_eq!(expected_paths.build(), paths);
    }

    #[test]
    fn openapi_router_nest_openapi() {
        use utoipa::OpenApi;

        #[derive(utoipa::ToSchema)]
        #[allow(unused)]
        struct Todo {
            id: i32,
        }
        #[derive(utoipa::OpenApi)]
        #[openapi(components(schemas(Todo)))]
        struct Api;

        let router: router::OpenApiRouter =
            router::OpenApiRouter::with_openapi(Api::openapi()).routes(routes!(search_user));

        let customer_router: router::OpenApiRouter = router::OpenApiRouter::new()
            .routes(routes!(get_customer))
            .with_state(String::new());

        let mut router = router.nest("/api/customer", customer_router);
        let paths = router.to_openapi().paths;
        let expected_paths = utoipa::openapi::path::PathsBuilder::new()
            .path(
                "/api/customer/",
                utoipa::openapi::PathItem::new(
                    utoipa::openapi::path::HttpMethod::Get,
                    utoipa::openapi::path::OperationBuilder::new()
                        .operation_id(Some("get_customer")),
                ),
            )
            .path(
                "/search",
                utoipa::openapi::PathItem::new(
                    utoipa::openapi::path::HttpMethod::Get,
                    utoipa::openapi::path::OperationBuilder::new()
                        .operation_id(Some("search_user")),
                ),
            );
        assert_eq!(expected_paths.build(), paths);
    }
}
