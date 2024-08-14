use std::collections::BTreeMap;
use std::convert::Infallible;
use std::sync::RwLock;

use axum::handler::Handler;
use axum::routing::{MethodFilter, MethodRouter};
use axum::{routing, Router};
use once_cell::sync::Lazy;

static GLOBAL_DATA: Lazy<RwLock<utoipa::openapi::path::Paths>> =
    once_cell::sync::Lazy::new(|| RwLock::new(utoipa::openapi::path::Paths::new()));

static OPENAPI: Lazy<RwLock<utoipa::openapi::OpenApi>> = once_cell::sync::Lazy::new(|| {
    RwLock::new(utoipa::openapi::OpenApi::new(
        utoipa::openapi::info::InfoBuilder::new().build(),
        utoipa::openapi::path::Paths::new(),
    ))
});

pub trait RouterExt<S, V> {
    fn route_with_api(self, method_router: MethodRouter<S, V>) -> Self;
    fn nest_with_api(self, path: &str, method_router: MethodRouter<S, V>) -> Self;
}

impl<S, V> RouterExt<S, V> for Router<S> {
    fn route_with_api(self, _method_router: MethodRouter<S, V>) -> Self {
        let mut api = OPENAPI.write().unwrap();

        let paths = GLOBAL_DATA.read().unwrap().clone();

        // TODO this should merged
        api.paths = paths;

        self
    }

    fn nest_with_api(self, path: &str, _method_router: MethodRouter<S, V>) -> Self {
        let mut api = OPENAPI.write().unwrap();

        let paths = GLOBAL_DATA.read().unwrap().clone();

        let paths = paths
            .paths
            .into_iter()
            .map(|(item_path, item)| {
                let path = format!("{path}{item_path}");
                (path, item)
            })
            .collect::<BTreeMap<_, _>>();

        // TODO this should merged
        let mut p = utoipa::openapi::path::Paths::new();
        p.paths = paths;
        api.paths = p;

        self
    }
}

// TODO Format the path args from `{arg} -> :arg` format
// "/api/user/{id}".replace('}', '').replace('{', ':')

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

pub trait UtoipaHandler<T, S>: Handler<T, S> {
    fn get_path_and_item(&self) -> (String, utoipa::openapi::path::PathItem);
}

// 1. required for `fn() -> impl std::future::Future<Output = ()> {tests::health_handler}` to implement `UtoipaHandler<((),), _>` [E0277]
// 2. required for `fn() -> impl std::future::Future<Output = ()> {tests::post_foo}` to implement `UtoipaHandler<((),), _>` [E0277]

// impl<O, V> UtoipaHandler<(), Infallible> for V
// where
//     O: std::future::Future<Output = ()>,
//     V: Fn() -> O,
// {
//     fn get_path_and_item(&self) -> (String, utoipa::openapi::path::PathItem) {
//         todo!()
//     }
// }

impl<T, S, P> UtoipaHandler<T, S> for P
where
    P: axum::handler::Handler<T, S> + utoipa::Path,
{
    fn get_path_and_item(&self) -> (String, utoipa::openapi::path::PathItem) {
        let path = P::path();
        let item = P::path_item();

        (path, item)
    }
}

// pub trait UtoipaPath<T, S>: utoipa::Path {
//     fn path() -> String {
//         <Self as utoipa::Path>::path()
//     }
//
//     fn path_item() -> utoipa::openapi::path::PathItem {
//         <Self as utoipa::Path>::path_item()
//     }
// }

// impl<T, S, H: Handler<T, S>> UtoipaPath<T, S> for H
// where
//     H: utoipa::Path,
// {
//     fn path() -> String {
//         todo!()
//     }
//
//     fn path_item() -> utoipa::openapi::path::PathItem {
//         todo!()
//     }
// }

pub trait UtoipaMethodRouterExt<H> {
    fn get_utoipa(self, handler: H) -> Self;
    fn post_api(self, handler: H) -> Self;
}

impl<H, S> UtoipaMethodRouterExt<H> for MethodRouter<S, Infallible>
where
    H: UtoipaHandler<Infallible, S>,
    S: Clone + Send + Sync + 'static,
    // where
    //     H: UtoipaHandler<T, S>,
    //     T: 'static,
    //     S: Clone + Send + Sync + 'static,
    //     V: utoipa::Path,
{
    fn post_api(self, handler: H) -> Self {
        let mut map = GLOBAL_DATA.write().unwrap();
        let (path, item) = handler.get_path_and_item();
        map.add_path(path, item);

        (self as MethodRouter<S, _>).on(MethodFilter::POST, handler)
    }

    fn get_utoipa(self, handler: H) -> Self {
        let mut map = GLOBAL_DATA.write().unwrap();

        let (path, item) = handler.get_path_and_item();
        map.add_path(path, item);

        // TODO add the handler
        self.on(MethodFilter::GET, handler)
    }
}

// pub trait UtoipaMethodHandler<S, V> {
//     fn post_api(&self, method_router: impl UtoipaMethodRouter<S, V>) -> Self;
// }

// impl UtoipaMethodHandler for Method

// pub fn on<H, T, S>(filter: MethodFilter, handler: H) -> MethodRouter<S, Infallible>
// where
//     H: Handler<T, S>,
//     T: 'static,
//     S: Clone + Send + Sync + 'static,
// {
//     MethodRouter::new().on(filter, handler)
// }

fn get<H, S>(handler: H) -> MethodRouter<S, Infallible>
where
    H: UtoipaHandler<Infallible, S>,
    S: Clone + Send + Sync + 'static,
{
    let mut map = GLOBAL_DATA.write().unwrap();

    let (path, item) = handler.get_path_and_item();
    map.add_path(path, item);

    // TODO add the handler
    routing::on(MethodFilter::GET, handler)
}

#[cfg(test)]
mod tests {
    use self::routing::post;

    use super::*;

    #[utoipa::path(get, path = "/")]
    async fn root() {}

    #[allow(non_camel_case_types)]
    #[doc(hidden)]
    #[derive(Clone)]
    pub struct test;
    impl<'t> utoipa::__dev::Tags<'t> for test {
        fn tags() -> Vec<&'t str> {
            [].into()
        }
    }
    impl utoipa::Path for test {
        fn path() -> String {
            "/test".replace('"', "")
        }
        fn path_item() -> utoipa::openapi::path::PathItem {
            utoipa::openapi::PathItem::new(
                utoipa::openapi::PathItemType::Post,
                utoipa::openapi::path::OperationBuilder::new()
                    .responses(utoipa::openapi::ResponsesBuilder::new().build())
                    .operation_id(Some("test")),
            )
        }
    }

    impl<T, S> Handler<T, S> for test {
        // type Future = InfallibleRouteFuture;
        type Future = std::pin::Pin<
            std::boxed::Box<
                (dyn std::future::Future<Output = axum::http::Response<axum::body::Body>>
                     + std::marker::Send
                     + 'static),
            >,
        >;

        fn call(self, req: axum::extract::Request, state: S) -> Self::Future {
            async fn test() {}
            test.call(req, state)
        }
    }
    // #[utoipa::path(post, path = "/test")]
    // async fn test() {}

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

    #[allow(non_camel_case_types)]
    #[doc(hidden)]
    #[derive(Clone)]
    pub struct search_user;
    impl<'t> utoipa::__dev::Tags<'t> for search_user {
        fn tags() -> Vec<&'t str> {
            [].into()
        }
    }
    impl utoipa::Path for search_user {
        fn path() -> String {
            "/search".replace('"', "")
        }
        fn path_item() -> utoipa::openapi::path::PathItem {
            utoipa::openapi::PathItem::new(
                utoipa::openapi::PathItemType::Get,
                utoipa::openapi::path::OperationBuilder::new()
                    .responses(utoipa::openapi::ResponsesBuilder::new().build())
                    .operation_id(Some("search_user")),
            )
        }
    }

    impl<T, S> Handler<T, S> for search_user {
        // type Future = InfallibleRouteFuture;
        type Future = std::pin::Pin<
            std::boxed::Box<
                (dyn std::future::Future<Output = axum::http::Response<axum::body::Body>>
                     + std::marker::Send
                     + 'static),
            >,
        >;

        fn call(self, req: axum::extract::Request, state: S) -> Self::Future {
            async fn search_user() {}
            search_user.call(req, state)
        }
    }
    // #[utoipa::path(get, path = "/search")]
    // async fn search_user() {}

    // --- customer

    #[utoipa::path(get, path = "/")]
    async fn get_customer() {}

    #[utoipa::path(post, path = "/")]
    async fn post_customer() {}

    #[utoipa::path(delete, path = "/")]
    async fn delete_customer() {}

    #[utoipa::path(get, path = "/search")]
    async fn search_customer() {}

    #[test]
    fn foobar_axum_route() {
        // let handler = routing::get(user_handler);
        // let ops = super::get(health_handler).post_api(|| async {});
        //
        //
        // get_user.call(req, state)
        let user_router: Router = Router::new().route_with_api(get(search_user).post_api(test));
        // .route_with_api(get(search_user).get(test))
        // .route("/", get(get_user).post(post_user).delete(delete_user))
        // .route("/search", get(search_user).get_utoipa(test));
        //
        dbg!(OPENAPI.read().unwrap().clone());

        let customer_router: Router = Router::new()
            .route(
                "/",
                self::routing::get(get_customer)
                    .post(post_customer)
                    .delete(delete_customer),
            )
            .route("/search", self::routing::get(search_customer));

        let router = Router::new()
            .nest("/api/user", user_router)
            .nest("/api/customer", customer_router)
            .route("/", self::routing::get(root))
            .route("/health", self::routing::get(health_handler))
            .route("/api/foo", post(post_foo));

        // let ops = super::get(health_handler).post_api(post_foo);
        //
        // let route: Router = Router::new().route("/", ops);

        // let router: Router = RouterExt::<_, __path_health_handler>::route_with_api(
        //     RouterExt::<_, __path_user_handler>::route_with_api(Router::new(), handler),
        //     ops,
        // );
    }
}
