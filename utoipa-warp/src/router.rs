//! Implements Router for composing handlers and collecting OpenAPI information.

use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

/// Wrapper type for [`utoipa::openapi::path::Paths`], collected schemas, and a
/// [`warp::filters::BoxedFilter`].
///
/// This is used with [`OpenApiRouter::routes`] method to register current _`paths`_ to the
/// [`utoipa::openapi::OpenApi`] of [`OpenApiRouter`] instance.
///
/// See [`routes`][routes] for usage.
///
/// [routes]: ../macro.routes.html
pub type UtoipaMethodRouter = (
    Vec<(
        String,
        utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
    )>,
    utoipa::openapi::path::Paths,
    BoxedFilter<(Box<dyn Reply>,)>,
);

/// Compose two boxed filters extracting `(Box<dyn Reply>,)` into one using `.or().unify()`.
fn compose_filters(
    a: BoxedFilter<(Box<dyn Reply>,)>,
    b: BoxedFilter<(Box<dyn Reply>,)>,
) -> BoxedFilter<(Box<dyn Reply>,)> {
    a.or(b).unify().boxed()
}

/// A wrapper struct for a composed [`warp::filters::BoxedFilter`] and
/// [`utoipa::openapi::OpenApi`] for composing handlers and collecting OpenAPI information.
///
/// This struct provides an API similar to [`utoipa-axum`]'s `OpenApiRouter` but adapted for
/// warp's filter-based architecture. Routes are composed internally using `.or()` on boxed
/// filters.
///
/// # Examples
///
/// _**Create new [`OpenApiRouter`] with default values populated from cargo environment variables.**_
/// ```rust
/// # use utoipa_warp::router::OpenApiRouter;
/// let _: OpenApiRouter = OpenApiRouter::new();
/// ```
///
/// _**Instantiate a new [`OpenApiRouter`] with new empty [`utoipa::openapi::OpenApi`].**_
/// ```rust
/// # use utoipa_warp::router::OpenApiRouter;
/// let _: OpenApiRouter = OpenApiRouter::default();
/// ```
#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct OpenApiRouter {
    filter: Option<BoxedFilter<(Box<dyn Reply>,)>>,
    openapi: utoipa::openapi::OpenApi,
}

impl OpenApiRouter {
    /// Instantiate a new [`OpenApiRouter`] with default values populated from cargo environment
    /// variables. This creates an `OpenApi` similar of creating a new `OpenApi` via
    /// `#[derive(OpenApi)]`
    ///
    /// If you want to create [`OpenApiRouter`] with completely empty [`utoipa::openapi::OpenApi`]
    /// instance, use [`OpenApiRouter::default()`].
    pub fn new() -> OpenApiRouter {
        use utoipa::OpenApi;
        #[derive(OpenApi)]
        struct Api;

        Self::with_openapi(Api::openapi())
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
    /// # use utoipa_warp::router::OpenApiRouter;
    /// #[derive(utoipa::ToSchema)]
    /// struct Todo {
    ///     id: i32,
    /// }
    /// #[derive(utoipa::OpenApi)]
    /// #[openapi(components(schemas(Todo)))]
    /// struct Api;
    ///
    /// let router: OpenApiRouter = OpenApiRouter::with_openapi(Api::openapi());
    /// ```
    pub fn with_openapi(openapi: utoipa::openapi::OpenApi) -> Self {
        Self {
            filter: None,
            openapi,
        }
    }

    /// Register [`UtoipaMethodRouter`] content created with [`routes`][routes] macro to `self`.
    ///
    /// Paths of the [`UtoipaMethodRouter`] will be extended to [`utoipa::openapi::OpenApi`] and
    /// the [`warp::filters::BoxedFilter`] will be composed into the internal filter chain.
    ///
    /// [routes]: ../macro.routes.html
    pub fn routes(mut self, (schemas, paths, filter): UtoipaMethodRouter) -> Self {
        // Merge paths into the OpenApi
        for (path, item) in paths.paths {
            if let Some(it) = self.openapi.paths.paths.get_mut(&path) {
                it.merge_operations(item);
            } else {
                self.openapi.paths.paths.insert(path, item);
            }
        }

        // Merge schemas
        let components = self
            .openapi
            .components
            .get_or_insert(utoipa::openapi::Components::new());
        components.schemas.extend(schemas);

        // Compose filters with .or().unify()
        self.filter = Some(match self.filter {
            Some(existing) => compose_filters(existing, filter),
            None => filter,
        });

        self
    }

    /// Nest `router` to `self` under given `path`. The child router's OpenAPI paths will be
    /// prefixed with the given path, and the child's warp filter will be nested under a
    /// `warp::path` prefix.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use utoipa_warp::router::OpenApiRouter;
    /// let todo_router = OpenApiRouter::new();
    /// let router = OpenApiRouter::new()
    ///     .nest("/api/todos", todo_router);
    /// ```
    pub fn nest(mut self, path: &str, router: OpenApiRouter) -> Self {
        // Prefix all child paths in OpenAPI
        let prefix = path.trim_end_matches('/');
        for (child_path, item) in router.openapi.paths.paths {
            let full_path = if child_path == "/" || child_path.is_empty() {
                prefix.to_string()
            } else {
                format!("{prefix}{child_path}")
            };
            if let Some(it) = self.openapi.paths.paths.get_mut(&full_path) {
                it.merge_operations(item);
            } else {
                self.openapi.paths.paths.insert(full_path, item);
            }
        }

        // Merge schemas from child
        if let Some(child_components) = router.openapi.components {
            let components = self
                .openapi
                .components
                .get_or_insert(utoipa::openapi::Components::new());
            components.schemas.extend(child_components.schemas);
        }

        // Nest the warp filter under the path prefix
        if let Some(child_filter) = router.filter {
            let segments: Vec<String> = path
                .trim_matches('/')
                .split('/')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();

            let nested: BoxedFilter<(Box<dyn Reply>,)> = if segments.is_empty() {
                child_filter
            } else {
                let mut base = warp::path(segments[0].clone()).boxed();
                for seg in &segments[1..] {
                    base = base.and(warp::path(seg.clone())).boxed();
                }
                base.and(child_filter).boxed()
            };

            self.filter = Some(match self.filter {
                Some(existing) => compose_filters(existing, nested),
                None => nested,
            });
        }

        self
    }

    /// Merge another [`OpenApiRouter`]'s paths and filters into `self` without adding a prefix.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use utoipa_warp::router::OpenApiRouter;
    /// let search_router = OpenApiRouter::new();
    /// let router = OpenApiRouter::new()
    ///     .merge(search_router);
    /// ```
    pub fn merge(mut self, router: OpenApiRouter) -> Self {
        // Merge OpenAPI paths
        for (path, item) in router.openapi.paths.paths {
            if let Some(it) = self.openapi.paths.paths.get_mut(&path) {
                it.merge_operations(item);
            } else {
                self.openapi.paths.paths.insert(path, item);
            }
        }

        // Merge schemas
        if let Some(child_components) = router.openapi.components {
            let components = self
                .openapi
                .components
                .get_or_insert(utoipa::openapi::Components::new());
            components.schemas.extend(child_components.schemas);
        }

        // Merge filters
        if let Some(child_filter) = router.filter {
            self.filter = Some(match self.filter {
                Some(existing) => compose_filters(existing, child_filter),
                None => child_filter,
            });
        }

        self
    }

    /// Split the content of the [`OpenApiRouter`] to parts. Method will return a tuple of
    /// the composed [`warp::filters::BoxedFilter`] and [`utoipa::openapi::OpenApi`].
    ///
    /// If no routes have been registered, the filter will reject all requests.
    pub fn split_for_parts(self) -> (BoxedFilter<(Box<dyn Reply>,)>, utoipa::openapi::OpenApi) {
        let filter = self.filter.unwrap_or_else(|| {
            warp::any()
                .and_then(|| async { Err::<Box<dyn Reply>, _>(warp::reject::not_found()) })
                .boxed()
        });
        (filter, self.openapi)
    }

    /// Consume `self` returning the [`utoipa::openapi::OpenApi`] instance of the
    /// [`OpenApiRouter`].
    pub fn into_openapi(self) -> utoipa::openapi::OpenApi {
        self.openapi
    }

    /// Take the [`utoipa::openapi::OpenApi`] instance without consuming the [`OpenApiRouter`].
    pub fn to_openapi(&mut self) -> utoipa::openapi::OpenApi {
        std::mem::take(&mut self.openapi)
    }

    /// Get reference to the [`utoipa::openapi::OpenApi`] instance of the router.
    pub fn get_openapi(&self) -> &utoipa::openapi::OpenApi {
        &self.openapi
    }

    /// Get mutable reference to the [`utoipa::openapi::OpenApi`] instance of the router.
    pub fn get_openapi_mut(&mut self) -> &mut utoipa::openapi::OpenApi {
        &mut self.openapi
    }
}

impl Default for OpenApiRouter {
    fn default() -> Self {
        Self::with_openapi(utoipa::openapi::OpenApiBuilder::new().build())
    }
}
