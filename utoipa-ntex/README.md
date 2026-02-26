# utoipa-ntex - Bindings for Ntex and utoipa

[![Utoipa build](https://github.com/juhaku/utoipa/actions/workflows/build.yaml/badge.svg)](https://github.com/juhaku/utoipa/actions/workflows/build.yaml)
[![crates.io](https://img.shields.io/crates/v/utoipa-ntex.svg?label=crates.io&color=orange&logo=rust)](https://crates.io/crates/utoipa-ntex)
[![docs.rs](https://img.shields.io/static/v1?label=docs.rs&message=utoipa-ntex&color=blue&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/utoipa-ntex/latest/)
![rustc](https://img.shields.io/static/v1?label=rustc&message=1.75&color=orange&logo=rust)

This crate implements necessary bindings for automatically collecting `paths` and `schemas` recursively from Ntex
`App`, `Scope` and `ServiceConfig`. It provides natural API reducing duplication and support for scopes while generating
OpenAPI specification without the need to declare `paths` and `schemas` to `#[openapi(...)]` attribute of `OpenApi` derive.

Currently only `service(...)` calls supports automatic collection of schemas and paths. Manual routes via `route(...)` or
`Route::new().to(...)` is not supported.

## Install

Add dependency declaration to `Cargo.toml`.

```toml
[dependencies]
utoipa-ntex = "0.1"
```

## Examples

Collect handlers annotated with `#[utoipa::path]` recursively from `service(...)` calls to compose OpenAPI spec.

```rust
use ntex::web::{get, App};
use utoipa_ntex::{scope, AppExt};

#[derive(utoipa::ToSchema)]
struct User {
    id: i32,
}

#[utoipa::path(responses((status = OK, body = User)))]
#[get("/user")]
async fn get_user() -> Json<User> {
    Json(User { id: 1 })
}

let (_, mut api) = App::new()
    .into_utoipa_app()
    .service(scope::scope("/api/v1").service(get_user))
    .split_for_parts();
```

## License

Licensed under either of [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) license at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate
by you, shall be dual licensed, without any additional terms or conditions.
