//! Implement `utoipa` extended [`WebServiceFactory`] for [`UtoipaHandler`].
//!
//! See usage from [`register`][fn@register].
use ntex::web::{ErrorRenderer, WebServiceFactory, dev::WebServiceConfig};
use utoipa::{
    __dev::{SchemaReferences, Tags},
    Path,
};

use crate::OpenApiFactory;

/// A wrapper that associates a handler function `H` with its corresponding
/// `utoipa::path` metadata type `P` to enable automatic OpenAPI schema generation.
///
/// This is useful when using `utoipa_ntex::UtoipaApp` or `Scope` and needing to
/// register individual handler functions that are annotated with `#[utoipa::path(...)]`.
///
/// The macro `#[utoipa::path(...)]` generates a hidden type like `__path_my_handler`,
/// which implements `utoipa::Path`, `SchemaReferences`, and `Tags`. `UtoipaHandler`
/// binds this generated type with your actual route handler so both the OpenAPI schema
/// and the actual service can be registered at once.
///
/// # Example
/// ```rust
/// use ntex::web;
/// use utoipa::OpenApi;
/// use utoipa_ntex::{AppExt, handler::UtoipaHandler, scope};
///
/// #[derive(OpenApi)]
/// #[openapi(paths(get_user), components(schemas(User)))]
/// struct ApiDoc;
///
/// #[derive(utoipa::ToSchema, serde::Serialize)]
/// struct User {
///     id: i32,
/// }
///
/// #[utoipa::path(get, path = "/user", responses((status = 200, body = User)))]
/// #[web::get("/user")]
/// async fn get_user() -> web::types::Json<User> {
///    web::types::Json(User { id: 1 })
/// }
///
/// let handler = UtoipaHandler::<_, __path_get_user>::new(get_user);
/// ```
pub struct UtoipaHandler<H, P> {
    /// The handler that is registered to Ntex service config.
    handler: H,

    /// Marker type for the generated OpenAPI metadata struct `P`.
    _phantom: std::marker::PhantomData<P>,
}

impl<H, P> UtoipaHandler<H, P> {
    /// Creates a new `UtoipaHandler` from the provided handler.
    ///
    /// # Arguments
    /// * `handler` - The Ntex-compatible handler function.
    ///
    /// # Returns
    /// A new `UtoipaHandler` that can be passed to `.service(...)`
    /// in `UtoipaApp` or `Scope` for both routing and OpenAPI generation.
    pub fn new(handler: H) -> Self {
        Self {
            handler,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<H, P, Err> WebServiceFactory<Err> for UtoipaHandler<H, P>
where
    H: WebServiceFactory<Err>,
    Err: ErrorRenderer,
{
    fn register(self, config: &mut WebServiceConfig<Err>) {
        self.handler.register(config);
    }
}

impl<'t, H, P> OpenApiFactory for UtoipaHandler<H, P>
where
    P: Path + SchemaReferences + Tags<'t>,
{
    fn paths(&self) -> utoipa::openapi::path::Paths {
        let methods = P::methods();
        methods
            .into_iter()
            .fold(
                utoipa::openapi::path::Paths::builder(),
                |mut builder, method| {
                    let mut operation = P::operation();
                    let other_tags = P::tags();
                    if !other_tags.is_empty() {
                        let tags = operation.tags.get_or_insert(Vec::new());
                        tags.extend(other_tags.into_iter().map(ToString::to_string));
                    }

                    let path_item = utoipa::openapi::PathItem::new(method, operation);
                    builder = builder.path(P::path(), path_item);
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
        P::schemas(schemas);
    }
}
