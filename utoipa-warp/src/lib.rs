#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

//! Utoipa warp brings `utoipa` and `warp` closer together by providing an ergonomic API that
//! extends warp's filter-based routing. It gives a natural way to register handlers known to
//! `warp` and simultaneously generates OpenAPI specification from the handlers.
//!
//! ## Crate features
//!
//! - **`debug`**: Implement debug traits for types.
//! - **`swagger-ui`**: Enable Swagger UI serving via convenience filters.
//!
//! ## Install
//!
//! Add dependency declaration to `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! utoipa-warp = "0.1"
//! ```
//!
//! ## Examples
//!
//! _**Use [`OpenApiRouter`][router] to collect handlers with `#[utoipa::path]` macro to compose
//! service and form OpenAPI spec.**_
//!
//! ```rust
//! # use utoipa_warp::{routes, router::OpenApiRouter};
//! # use warp::{Filter, Reply, reply::Json};
//! #[derive(utoipa::ToSchema, serde::Serialize)]
//! struct User {
//!     id: i32,
//! }
//!
//! #[utoipa::path(get, path = "/user", responses((status = OK, body = User)))]
//! async fn get_user() -> Json {
//!     warp::reply::json(&User { id: 1 })
//! }
//!
//! let get_user_filter = warp::path("user")
//!     .and(warp::get())
//!     .and(warp::path::end())
//!     .and_then(|| async { Ok::<_, warp::Rejection>(get_user().await) });
//!
//! let (filter, api) = OpenApiRouter::new()
//!     .routes(routes!(get_user; filter = get_user_filter))
//!     .split_for_parts();
//! ```
//!
//! [router]: router/struct.OpenApiRouter.html

pub mod router;
#[cfg(feature = "swagger-ui")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "swagger-ui")))]
pub mod serving;

/// re-export paste so users do not need to add the dependency.
#[doc(hidden)]
pub use paste::paste;

use std::sync::Arc;

/// Create a warp filter that serves the OpenAPI specification as JSON.
///
/// # Arguments
///
/// * `path` - The path segment to serve the spec at (e.g., `"api-doc.json"`)
/// * `openapi` - The OpenAPI specification to serve
///
/// # Examples
///
/// ```rust
/// # use utoipa_warp::openapi_json_filter;
/// # use utoipa::openapi::OpenApiBuilder;
/// let openapi = OpenApiBuilder::new().build();
/// let filter = openapi_json_filter("api-doc.json", openapi);
/// ```
pub fn openapi_json_filter(
    path: &str,
    openapi: utoipa::openapi::OpenApi,
) -> warp::filters::BoxedFilter<(Box<dyn warp::Reply>,)> {
    use warp::Filter;
    let openapi = Arc::new(openapi);
    warp::path(path.to_string())
        .and(warp::get())
        .and(warp::path::end())
        .map(move || -> Box<dyn warp::Reply> { Box::new(warp::reply::json(openapi.as_ref())) })
        .boxed()
}

/// Collect warp handlers annotated with [`utoipa::path`] to [`router::UtoipaMethodRouter`].
///
/// The [`routes`] macro collects OpenAPI metadata from handlers annotated with
/// `#[utoipa::path]` and pairs them with user-provided warp filters.
///
/// Unlike `utoipa-axum`, warp handlers are filter chains rather than plain async functions,
/// so the macro requires you to provide the warp filter alongside the handler metadata.
///
/// The macro returns a [`router::UtoipaMethodRouter`] which contains collected schemas,
/// paths, and a composed [`warp::filters::BoxedFilter`]. The output is meant to be used with
/// [`router::OpenApiRouter`].
///
/// # Syntax
///
/// ```text
/// routes!(handler; filter = warp_filter)
/// routes!(handler1, handler2, ...; filter = warp_filter)
/// ```
///
/// - `handler`: A function annotated with `#[utoipa::path]` whose metadata will be collected
/// - `filter`: A warp filter that handles the actual HTTP requests for these routes
/// - The `;` separator is required between handler list and the filter assignment
///
/// # Examples
///
/// _**Create a router with a single handler.**_
/// ```rust
/// # use utoipa_warp::{routes, router::OpenApiRouter};
/// # use warp::{Filter, Reply};
/// #[utoipa::path(get, path = "/")]
/// async fn root() -> &'static str { "hello" }
///
/// let root_filter = warp::get()
///     .and(warp::path::end())
///     .and_then(|| async { Ok::<_, warp::Rejection>(root().await) });
///
/// let _: OpenApiRouter = OpenApiRouter::new()
///     .routes(routes!(root; filter = root_filter));
/// ```
///
/// _**Create a router with multiple handlers sharing a filter.**_
/// ```rust
/// # use utoipa_warp::{routes, router::OpenApiRouter};
/// # use warp::{Filter, Reply};
/// #[utoipa::path(get, path = "/")]
/// async fn get_item() -> &'static str { "get" }
///
/// #[utoipa::path(post, path = "/")]
/// async fn post_item() -> &'static str { "post" }
///
/// let items_filter = warp::path::end().and(
///     warp::get().and_then(|| async { Ok::<_, warp::Rejection>(get_item().await) })
///     .or(warp::post().and_then(|| async { Ok::<_, warp::Rejection>(post_item().await) }))
///     .unify()
/// );
///
/// let _: OpenApiRouter = OpenApiRouter::new()
///     .routes(routes!(get_item, post_item; filter = items_filter));
/// ```
#[macro_export]
macro_rules! routes {
    ( $handler:path ; filter = $filter:expr $(,)? ) => {
        {
            let mut paths = utoipa::openapi::path::Paths::new();
            let mut schemas = Vec::<(String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>::new();
            let (path, item, types) = $crate::routes!(@resolve_types $handler : schemas);
            paths.add_path_operation(&path, types, item);
            let boxed: warp::filters::BoxedFilter<(Box<dyn warp::Reply>,)> = {
                use warp::Filter;
                $filter
                    .map(|r| -> Box<dyn warp::Reply> { Box::new(r) })
                    .boxed()
            };
            (schemas, paths, boxed)
        }
    };
    ( $handler:path $(, $tail:path)+ ; filter = $filter:expr $(,)? ) => {
        {
            let mut paths = utoipa::openapi::path::Paths::new();
            let mut schemas = Vec::<(String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>::new();
            let (path, item, types) = $crate::routes!(@resolve_types $handler : schemas);
            paths.add_path_operation(&path, types, item);
            $(
                let (path, item, types) = $crate::routes!(@resolve_types $tail : schemas);
                paths.add_path_operation(&path, types, item);
            )+
            let boxed: warp::filters::BoxedFilter<(Box<dyn warp::Reply>,)> = {
                use warp::Filter;
                $filter
                    .map(|r| -> Box<dyn warp::Reply> { Box::new(r) })
                    .boxed()
            };
            (schemas, paths, boxed)
        }
    };
    ( @resolve_types $handler:path : $schemas:tt ) => {
        {
            $crate::paste! {
                let path = $crate::routes!( @path [path()] of $handler );
                let mut operation = $crate::routes!( @path [operation()] of $handler );
                let types = $crate::routes!( @path [methods()] of $handler );
                let tags = $crate::routes!( @path [tags()] of $handler );
                $crate::routes!( @path [schemas(&mut $schemas)] of $handler );
                if !tags.is_empty() {
                    let operation_tags = operation.tags.get_or_insert(Vec::new());
                    operation_tags.extend(tags.iter().map(ToString::to_string));
                }
                (path, operation, types)
            }
        }
    };
    ( @path $op:tt of $part:ident $( :: $tt:tt )* ) => {
        $crate::routes!( $op : [ $part $( $tt )*] )
    };
    ( $op:tt : [ $first:tt $( $rest:tt )* ] $( $rev:tt )* ) => {
        $crate::routes!( $op : [ $( $rest )* ] $first $( $rev)* )
    };
    ( $op:tt : [] $first:tt $( $rest:tt )* ) => {
        $crate::routes!( @inverse $op : $first $( $rest )* )
    };
    ( @inverse $op:tt : $tt:tt $( $rest:tt )* ) => {
        $crate::routes!( @rev $op : $tt [$($rest)*] )
    };
    ( @rev $op:tt : $tt:tt [ $first:tt $( $rest:tt)* ] $( $reversed:tt )* ) => {
        $crate::routes!( @rev $op : $tt [ $( $rest )* ] $first $( $reversed )* )
    };
    ( @rev [$op:ident $( $args:tt )* ] : $handler:tt [] $($tt:tt)* ) => {
        {
            #[allow(unused_imports)]
            use utoipa::{Path, __dev::{Tags, SchemaReferences}};
            $crate::paste! {
                $( $tt :: )* [<__path_ $handler>]::$op $( $args )*
            }
        }
    };
    ( ) => {};
}

#[cfg(test)]
mod tests {
    use super::*;
    use router::*;
    use warp::Filter;

    #[utoipa::path(get, path = "/")]
    async fn root() -> &'static str {
        "hello"
    }

    #[utoipa::path(get, path = "/")]
    async fn get_user() -> &'static str {
        "get_user"
    }

    #[utoipa::path(post, path = "/")]
    async fn post_user() -> &'static str {
        "post_user"
    }

    #[utoipa::path(get, path = "/search")]
    async fn search_user() -> &'static str {
        "search"
    }

    #[test]
    fn openapi_router_new_has_default_info() {
        let router = OpenApiRouter::new();
        let openapi = router.get_openapi();
        assert!(!openapi.info.title.is_empty());
    }

    #[test]
    fn openapi_router_default_has_empty_info() {
        let router = OpenApiRouter::default();
        let openapi = router.get_openapi();
        assert!(openapi.info.title.is_empty());
    }

    #[test]
    fn openapi_router_routes_collect_paths() {
        let get_filter = warp::get()
            .and(warp::path::end())
            .and_then(|| async { Ok::<_, warp::Rejection>(get_user().await) });

        let mut router = OpenApiRouter::new().routes(routes!(get_user; filter = get_filter));

        let openapi = router.to_openapi();
        assert!(openapi.paths.paths.contains_key("/"));
        let path_item = &openapi.paths.paths["/"];
        assert!(path_item.get.is_some());
    }

    #[test]
    fn openapi_router_multiple_handlers_in_routes() {
        let users_filter = warp::path::end().and(
            warp::get()
                .and_then(|| async { Ok::<_, warp::Rejection>(get_user().await) })
                .or(warp::post().and_then(|| async { Ok::<_, warp::Rejection>(post_user().await) }))
                .unify(),
        );

        let mut router =
            OpenApiRouter::new().routes(routes!(get_user, post_user; filter = users_filter));

        let openapi = router.to_openapi();
        let path_item = &openapi.paths.paths["/"];
        assert!(path_item.get.is_some());
        assert!(path_item.post.is_some());
    }

    #[test]
    fn openapi_router_nest_prefixes_paths() {
        let search_filter = warp::path("search")
            .and(warp::get())
            .and(warp::path::end())
            .and_then(|| async { Ok::<_, warp::Rejection>(search_user().await) });

        let child = OpenApiRouter::new().routes(routes!(search_user; filter = search_filter));
        let mut router = OpenApiRouter::new().nest("/api", child);

        let openapi = router.to_openapi();
        assert!(
            openapi.paths.paths.contains_key("/api/search"),
            "Expected /api/search in paths, got: {:?}",
            openapi.paths.paths.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn openapi_router_merge_combines_paths() {
        let root_filter = warp::get()
            .and(warp::path::end())
            .and_then(|| async { Ok::<_, warp::Rejection>(root().await) });

        let search_filter = warp::path("search")
            .and(warp::get())
            .and(warp::path::end())
            .and_then(|| async { Ok::<_, warp::Rejection>(search_user().await) });

        let router_a = OpenApiRouter::new().routes(routes!(root; filter = root_filter));
        let router_b = OpenApiRouter::new().routes(routes!(search_user; filter = search_filter));

        let mut router = router_a.merge(router_b);
        let openapi = router.to_openapi();
        assert!(openapi.paths.paths.contains_key("/"));
        assert!(openapi.paths.paths.contains_key("/search"));
    }

    #[test]
    fn openapi_router_split_for_parts() {
        let root_filter = warp::get()
            .and(warp::path::end())
            .and_then(|| async { Ok::<_, warp::Rejection>(root().await) });

        let router = OpenApiRouter::new().routes(routes!(root; filter = root_filter));

        let (_filter, openapi) = router.split_for_parts();
        assert!(openapi.paths.paths.contains_key("/"));
    }

    #[test]
    fn openapi_with_auto_collected_schemas() {
        #[derive(utoipa::ToSchema, serde::Serialize)]
        #[allow(unused)]
        struct Todo {
            id: i32,
        }

        #[utoipa::path(get, path = "/todo", responses((status = 200, body = Todo)))]
        async fn get_todo() -> String {
            String::new()
        }

        let todo_filter = warp::path("todo")
            .and(warp::get())
            .and(warp::path::end())
            .and_then(|| async { Ok::<_, warp::Rejection>(get_todo().await) });

        let mut router = OpenApiRouter::new().routes(routes!(get_todo; filter = todo_filter));

        let openapi = router.to_openapi();
        assert!(openapi.paths.paths.contains_key("/todo"));
        let schemas = openapi
            .components
            .expect("Router must have auto collected schemas")
            .schemas;
        assert!(schemas.contains_key("Todo"));
    }

    #[tokio::test]
    async fn test_warp_filter_responds() {
        let root_filter = warp::get()
            .and(warp::path::end())
            .and_then(|| async { Ok::<_, warp::Rejection>("hello world") });

        let (filter, _openapi) = OpenApiRouter::new()
            .routes(routes!(root; filter = root_filter))
            .split_for_parts();

        let response = warp::test::request()
            .method("GET")
            .path("/")
            .reply(&filter)
            .await;

        assert_eq!(response.status(), 200);
        assert_eq!(response.body(), "hello world");
    }

    mod pets {
        #[utoipa::path(get, path = "/")]
        pub async fn get_pet() -> &'static str {
            "pet"
        }
    }

    #[test]
    fn openapi_routes_from_another_module() {
        let pet_filter = warp::get()
            .and(warp::path::end())
            .and_then(|| async { Ok::<_, warp::Rejection>(pets::get_pet().await) });

        let mut router = OpenApiRouter::new().routes(routes!(pets::get_pet; filter = pet_filter));

        let openapi = router.to_openapi();
        assert!(openapi.paths.paths.contains_key("/"));
    }
}
