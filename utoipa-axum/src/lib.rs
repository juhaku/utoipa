use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::RwLock;

use axum::extract::Request;
use axum::handler::Handler;
use axum::response::{IntoResponse, Response};
use axum::routing::{MethodFilter, MethodRouter};
use axum::{routing, Router};
use once_cell::sync::Lazy;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

async fn handler() {}

fn foo() {
    let app: Router = Router::new().route("path", routing::get(handler));
}

static GLOBAL_DATA: Lazy<RwLock<utoipa::openapi::path::Paths>> =
    once_cell::sync::Lazy::new(|| RwLock::new(utoipa::openapi::path::Paths::new()));

pub trait RouterExt<S, V>
where
    V: utoipa::Path,
{
    fn route_with_api(self, method_router: impl UtoipaMethodRouter<S, V>) -> Self;
}

impl<S, V> RouterExt<S, V> for Router<S>
where
    V: utoipa::Path,
{
    fn route_with_api(self, method_router: impl UtoipaMethodRouter<S, V>) -> Self {
        let mut map = GLOBAL_DATA.write().unwrap();

        let (path, item) = method_router.get_api();
        map.add_path(path, item);

        self
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
    P: axum::handler::Handler<T, S>,
{
    fn get_path_and_item(&self) -> (String, utoipa::openapi::path::PathItem) {
        let path = P::path();
        let item = P::path_item();

        (path, item)
    }
}

pub trait UtoipaPath<T, S>: utoipa::Path {
    fn path() -> String {
        <Self as utoipa::Path>::path()
    }

    fn path_item() -> utoipa::openapi::path::PathItem {
        <Self as utoipa::Path>::path_item()
    }
}

impl<T, S, H: Handler<T, S>> UtoipaPath<T, S> for H
where
    H: utoipa::Path,
{
    fn path() -> String {
        todo!()
    }

    fn path_item() -> utoipa::openapi::path::PathItem {
        todo!()
    }
}

pub trait UtoipaMethodRouterExt<H, V, T> {
    fn post_api(self, handler: H) -> Self;
}

impl<H, S, T, V> UtoipaMethodRouterExt<H, V, T> for MethodRouter<S, Infallible>
where
    H: UtoipaHandler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
    V: utoipa::Path,
{
    fn post_api(self, handler: H) -> Self {
        let mut map = GLOBAL_DATA.write().unwrap();
        let (path, item) = handler.get_path_and_item();
        map.add_path(path, item);

        (self as MethodRouter<S, _>).on(MethodFilter::POST, handler)
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

fn get<H, T, S>(handler: H) -> MethodRouter<S, Infallible>
where
    H: UtoipaHandler<T, S>,
    T: 'static,
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
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[utoipa::path(get, path = "/api/user")]
    async fn user_handler() {}

    #[utoipa::path(post, path = "/api/health")]
    async fn health_handler() {}

    #[utoipa::path(post, path = "/api/foo")]
    async fn post_foo() {}

    #[test]
    fn foobar_axum_route() {
        // let handler = routing::get(user_handler);
        // let ops = super::get(health_handler).post_api(|| async {});

        let ops = super::get(health_handler).post_api(post_foo);

        let route: Router = Router::new().route("/", ops);

        // let router: Router = RouterExt::<_, __path_health_handler>::route_with_api(
        //     RouterExt::<_, __path_user_handler>::route_with_api(Router::new(), handler),
        //     ops,
        // );
    }
}
