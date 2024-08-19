# utoipa-axum - Bindings for Axum and utoipa

Utoipa axum brings `utoipa` and `axum` closer together by the way of providing an ergonomic API that is extending on
the `axum` API. It gives a natural way to register handlers known to `axum` and also simultaneously generates OpenAPI
specification from the handlers.

## Install

Add dependency declaration to `Cargo.toml`.

```toml
[dependencies]
utoipa_axum = "0.1"
```

## Examples

Use `OpenApiRouter` to collect handlers with `#[utoipa::path]` macro to compose service and form OpenAPI spec.

```rust
#[derive(utoipa::ToSchema)]
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
let axum_router: axum::Router = router.into();
```

## License

Licensed under either of [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) license at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate
by you, shall be dual licensed, without any additional terms or conditions.
