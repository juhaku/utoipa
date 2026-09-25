# utoipa-warp - Bindings for Warp and utoipa

[![Utoipa build](https://github.com/juhaku/utoipa/actions/workflows/build.yaml/badge.svg)](https://github.com/juhaku/utoipa/actions/workflows/build.yaml)
[![crates.io](https://img.shields.io/crates/v/utoipa-warp.svg?label=crates.io&color=orange&logo=rust)](https://crates.io/crates/utoipa-warp)
[![docs.rs](https://img.shields.io/static/v1?label=docs.rs&message=utoipa-warp&color=blue&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/utoipa-warp/latest/)
![rustc](https://img.shields.io/static/v1?label=rustc&message=1.75&color=orange&logo=rust)

Utoipa warp brings `utoipa` and `warp` closer together by providing an ergonomic API that extends warp's
filter-based routing. It gives a natural way to register handlers known to `warp` and simultaneously generates
OpenAPI specification from the handlers.

## Crate features

- **`debug`**: Implement debug traits for types.
- **`swagger-ui`**: Enable Swagger UI serving via convenience filters.

## Install

Add dependency declaration to `Cargo.toml`.

```toml
[dependencies]
utoipa-warp = "0.1"
```

## Examples

Use `OpenApiRouter` to collect handlers with `#[utoipa::path]` macro to compose service and form OpenAPI spec.

```rust
use utoipa_warp::{routes, router::OpenApiRouter};
use warp::Filter;

#[derive(utoipa::ToSchema, serde::Serialize)]
struct User {
    id: i32,
}

#[utoipa::path(get, path = "/user", responses((status = OK, body = User)))]
async fn get_user() -> warp::reply::Json {
    warp::reply::json(&User { id: 1 })
}

let get_user_filter = warp::path("user")
    .and(warp::get())
    .and(warp::path::end())
    .and_then(|| async { Ok::<_, warp::Rejection>(get_user().await) });

let (filter, api) = OpenApiRouter::new()
    .routes(routes!(get_user; filter = get_user_filter))
    .split_for_parts();
```

## License

Licensed under either of [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) license at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate
by you, shall be dual licensed, without any additional terms or conditions.
