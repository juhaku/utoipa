pub mod router;

use std::collections::{BTreeMap, HashMap};
use std::convert::Infallible;
use std::sync::RwLock;

use axum::handler::Handler;
use axum::routing::{MethodFilter, MethodRouter};
use axum::{routing, Router};
use once_cell::sync::Lazy;

use self::router::CURRENT_PATHS;

// const ROUTER_ID: AtomicI64 = AtomicI64::new(0);

// static ROUTES: Lazy<RwLock<HashMap<i64, Router<S>>>> =
//     once_cell::sync::Lazy::new(|| RwLock::new(HashMap::new()));

// static APIS: Lazy<RwLock<HashMap<u64, utoipa::openapi::OpenApi>>> =
//     once_cell::sync::Lazy::new(|| RwLock::new(HashMap::new()));

// static CURRENT_PATHS: Lazy<RwLock<utoipa::openapi::path::Paths>> =
//     once_cell::sync::Lazy::new(|| RwLock::new(utoipa::openapi::path::Paths::new()));

static OPENAPI: Lazy<RwLock<utoipa::openapi::OpenApi>> = once_cell::sync::Lazy::new(|| {
    RwLock::new(utoipa::openapi::OpenApi::new(
        utoipa::openapi::info::InfoBuilder::new().build(),
        utoipa::openapi::path::Paths::new(),
    ))
});

// #[inline]
// fn colonized_params<S: AsRef<str>>(path: S) -> String
// where
//     String: From<S>,
// {
//     String::from(path).replace('}', "").replace('{', ":")
// }

// pub type UtoipaRouter<S = ()> = (i64, Router<S>);

pub trait RouterExt<S> {
    fn route_paths(self, method_router: MethodRouter<S>) -> Self;
    fn nest_paths(self, path: &str, method_router: Router<S>) -> Self;
    fn make_openapi(&self) -> utoipa::openapi::OpenApi;
}

// let (id, user_router): (i64, Router) = Router::with_openapi()
//     .route_paths(get(search_user))
//     .route_paths(get(get_user).post_path(post_user).delete_path(delete_user));

// let user_router: Router = Router::new()
//     .route_paths(get(search_user))
//     .route_paths(get(get_user).post_path(post_user).delete_path(delete_user));
//
// let customer_router: Router = Router::new()
//     .route_paths(
//         get(get_customer)
//             .post_path(post_customer)
//             .delete_path(delete_customer),
//     )
//     .route_paths(get(search_customer));
//
// let router = Router::new()
//     .nest_paths("/api/user" id, user_router)
//     .nest_paths("/api/customer", customer_router)
//     .route("/", get(root));

impl<S> RouterExt<S> for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn route_paths(self, _method_router: MethodRouter<S>) -> Self {
        // let mut api = OPENAPI.write().unwrap();
        let mut paths = CURRENT_PATHS
            .write()
            .expect("write CURRENT_PATHS lock poisoned");

        // TODO this should merged
        // api.paths.paths.extend(paths.paths);
        // let this = paths.paths.iter().fold(self, |this, (path, _)| {
        //     this.route(&colonized_params(path), method_router.clone())
        // });

        // paths.paths = BTreeMap::new();

        // this
        // self.route(&path, method_router)
        self
    }

    fn nest_paths(self, path: &str, _router: Router<S>) -> Self {
        let mut api = OPENAPI.write().unwrap();

        let paths = CURRENT_PATHS.read().unwrap().clone();

        let paths = paths
            .paths
            .into_iter()
            .map(|(item_path, item)| {
                let path = format!("{path}{item_path}");
                (path, item)
            })
            .collect::<BTreeMap<_, _>>();

        // TODO this should merged
        // let mut p = utoipa::openapi::path::Paths::new();
        // p.paths = paths;
        api.paths.paths.extend(paths);

        // self.nest(&colonized_params(path), router)
        self
    }

    fn make_openapi(&self) -> utoipa::openapi::OpenApi {
        OPENAPI.read().expect("read OPENAPI lock poisoned").clone()
    }
}

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
    T: 'static,
    S: Clone + Send + Sync + 'static,
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
    // fn delete_path(self, handler: H) -> Self {
    //     todo!()
    // }
    //
    // fn get_path(self, handler: H) -> Self {
    //     let mut paths = CURRENT_PATHS.write().unwrap();
    //
    //     let (path, item) = handler.get_path_and_item();
    //     paths.add_path(colonized_params(path), item);
    //
    //     self.on(MethodFilter::GET, handler)
    // }
    //
    // fn head_path(self, handler: H) -> Self {
    //     todo!()
    // }
    //
    // fn options_path(self, handler: H) -> Self {
    //     todo!()
    // }
    //
    // fn patch_path(self, handler: H) -> Self {
    //     todo!()
    // }
    //
    // fn post_path(self, handler: H) -> Self {
    //     let mut paths = CURRENT_PATHS.write().unwrap();
    //     let (path, item) = handler.get_path_and_item();
    //     paths.add_path(colonized_params(path), item);
    //
    //     (self as MethodRouter<S, _>).on(MethodFilter::POST, handler)
    // }
    //
    // fn put_path(self, handler: H) -> Self {
    //     todo!()
    // }
    //
    // fn trace_path(self, handler: H) -> Self {
    //     todo!()
    // }
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

// pub fn get<H, T, S>(handler: H) -> MethodRouter<S, Infallible>
// where
//     H: Handler<T, S>,
//     T: 'static,
//     S: Clone + Send + Sync + 'static,
// {
//     on(MethodFilter::GET, handler)
// }

macro_rules! top_level_handle {
    ( $name:ident $method:ident) => {
        pub fn $name<H, T, S>(handler: H) -> MethodRouter<S, Infallible>
        where
            H: UtoipaHandler<T, S>,
            T: 'static,
            S: Clone + Send + Sync + 'static,
        {
            let mut paths = CURRENT_PATHS.write().unwrap();
            // // clear the map if not empty to start another `Router` paths
            // if !paths.paths.is_empty() {
            //     paths.paths = BTreeMap::new();
            // }

            let (path, item) = handler.get_path_and_item();
            paths.add_path(path, item);

            // TODO add the handler
            routing::on(MethodFilter::$method, handler)
        }
    };
}

// fn get<H, T, S>(handler: H) -> MethodRouter<S, Infallible>
// where
//     H: UtoipaHandler<T, S>,
//     T: 'static,
//     S: Clone + Send + Sync + 'static,
// {
//     let mut paths = CURRENT_PATHS.write().unwrap();
//     // clear the map if not empty to start another `Router` paths
//     if !paths.paths.is_empty() {
//         paths.paths = BTreeMap::new();
//     }
//
//     let (path, item) = handler.get_path_and_item();
//     paths.add_path(colonized_params(path), item);
//
//     // TODO add the handler
//     routing::on(MethodFilter::GET, handler)
// }

top_level_handle!(delete DELETE);
top_level_handle!(get GET);
top_level_handle!(head HEAD);
top_level_handle!(options OPTIONS);
top_level_handle!(patch PATCH);
top_level_handle!(post POST);
top_level_handle!(put PUT);
top_level_handle!(trace TRACE);

#[cfg(test)]
mod tests {
    use std::marker::Send;
    use std::vec;

    use axum::extract::State;

    use self::router::{OpenApiRouter, CURRENT_PATHS};

    use super::*;

    #[utoipa::path(get, path = "/")]
    async fn root() {}

    // #[allow(non_camel_case_types)]
    // #[doc(hidden)]
    // #[derive(Clone)]
    // pub struct test;
    // impl<'t> utoipa::__dev::Tags<'t> for test {
    //     fn tags() -> Vec<&'t str> {
    //         [].into()
    //     }
    // }
    // impl utoipa::Path for test {
    //     fn path() -> String {
    //         "/test".replace('"', "")
    //     }
    //     fn path_item() -> utoipa::openapi::path::PathItem {
    //         utoipa::openapi::PathItem::new(
    //             utoipa::openapi::PathItemType::Post,
    //             utoipa::openapi::path::OperationBuilder::new()
    //                 .responses(utoipa::openapi::ResponsesBuilder::new().build())
    //                 .operation_id(Some("test")),
    //         )
    //     }
    // }
    //
    // impl<T, S> Handler<T, S> for test {
    //     // type Future = InfallibleRouteFuture;
    //     type Future = std::pin::Pin<
    //         std::boxed::Box<
    //             (dyn std::future::Future<Output = axum::http::Response<axum::body::Body>>
    //                  + std::marker::Send
    //                  + 'static),
    //         >,
    //     >;
    //
    //     fn call(self, req: axum::extract::Request, state: S) -> Self::Future {
    //         async fn test() {}
    //         test.call(req, state)
    //     }
    // }
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

    // #[allow(non_camel_case_types)]
    // #[doc(hidden)]
    // #[derive(Clone)]
    // pub struct search_user;
    // impl<'t> utoipa::__dev::Tags<'t> for search_user {
    //     fn tags() -> Vec<&'t str> {
    //         [].into()
    //     }
    // }
    // impl utoipa::Path for search_user {
    //     fn path() -> String {
    //         "/search".replace('"', "")
    //     }
    //     fn path_item() -> utoipa::openapi::path::PathItem {
    //         utoipa::openapi::PathItem::new(
    //             utoipa::openapi::PathItemType::Get,
    //             utoipa::openapi::path::OperationBuilder::new()
    //                 .responses(utoipa::openapi::ResponsesBuilder::new().build())
    //                 .operation_id(Some("search_user")),
    //         )
    //     }
    // }
    // impl<S> Handler<Infallible, S> for search_user
    // where
    //     S: Clone + Send + Sync + 'static,
    // {
    //     type Future = Pin<Box<(dyn Future<Output = Response<Body>> + Send + 'static)>>;
    //
    //     fn call(self, req: axum::extract::Request, state: S) -> Self::Future {
    //         async fn search_user() {}
    //         search_user.call(req, state)
    //     }
    // }
    #[utoipa::path(get, path = "/search")]
    async fn search_user() {}

    // --- customer

    #[utoipa::path(get, path = "/")]
    async fn get_customer() {}

    #[utoipa::path(post, path = "/")]
    async fn post_customer() {}

    #[utoipa::path(delete, path = "/")]
    async fn delete_customer() {}

    #[utoipa::path(get, path = "/search")]
    async fn search_customer(State(json): State<String>) {}

    #[test]
    fn foobar_axum_route() {
        let user_router: OpenApiRouter = OpenApiRouter::new()
            .routes(get(search_user))
            .routes(get(get_user).post_path(post_user).delete_path(delete_user));
        dbg!(&CURRENT_PATHS.read().unwrap());
        let api = user_router.get_openapi();
        dbg!(&api);

        let customer_router: OpenApiRouter = OpenApiRouter::new()
            .routes(
                get(get_customer)
                    .post_path(post_customer)
                    .delete_path(delete_customer),
            )
            .routes(get(search_customer));
        let api = customer_router.get_openapi();
        dbg!(&api);

        let router = OpenApiRouter::new()
            .nest("/api/user", user_router)
            .nest("/api/customer", customer_router)
            .route("/", get(root));
        // .route_paths(post(health_handler));

        let api = router.get_openapi();
        dbg!(&api);

        // .route("/api/foo", post(post_foo));

        // let ops = super::get(health_handler).post_api(post_foo);
        //
        // let route: Router = Router::new().route("/", ops);

        // let router: Router = RouterExt::<_, __path_health_handler>::route_with_api(
        //     RouterExt::<_, __path_user_handler>::route_with_api(Router::new(), handler),
        //     ops,
        // );
    }

    // macro_rules! expand {
    //     ( $vec:expr ) => {
    //         {
    //             let ops = $vec.len();
    //
    //             expand!( @internal $vec, ops)
    //
    //         }
    //     };
    //     ( @internal $vec:expr, $len:expr ) => {
    //         {
    //             const _: &str = stringify!( $index $len );
    //             if ${index(0)} < $len {
    //                 // vec.get($index).unwrap();
    //                 expand!( @internal $vec, $index + 1, $len )
    //             }
    //         }
    //     }
    // }

    // #[test]
    // fn test_expand() {
    //     let str = vec!["a", "b", "c"];
    //
    //     expand!(str)
    // }
}
