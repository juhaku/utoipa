pub mod router;

use std::convert::Infallible;

use axum::handler::Handler;
use axum::routing;
use axum::routing::{MethodFilter, MethodRouter};

use self::router::CURRENT_PATHS;

pub trait UtoipaMethodRouter<S, V> {
    fn get_api(&self) -> (String, utoipa::openapi::path::PathItem);
}

impl<S, V: utoipa::Path> UtoipaMethodRouter<S, V> for MethodRouter<S> {
    fn get_api(&self) -> (String, utoipa::openapi::path::PathItem) {
        let path = V::path();
        let item = V::path_item();

        (path, item)
    }
}

pub trait UtoipaHandler<T, S>: Handler<T, S>
where
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
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

pub trait UtoipaMethodRouterExt<H, T> {
    fn delete_path(self, handler: H) -> Self;
    fn get_path(self, handler: H) -> Self;
    fn head_path(self, handler: H) -> Self;
    fn options_path(self, handler: H) -> Self;
    fn patch_path(self, handler: H) -> Self;
    fn post_path(self, handler: H) -> Self;
    fn put_path(self, handler: H) -> Self;
    fn trace_path(self, handler: H) -> Self;
}

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
    fn axum_router_nest_openapi_routes() {
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

        let api = router.get_openapi();
        dbg!(&api);
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
            .routes(
                get_path(get_user)
                    .post_path(post_user)
                    .delete_path(delete_user),
            );

        let api = router.to_openapi();
        dbg!(&api);
    }
}
