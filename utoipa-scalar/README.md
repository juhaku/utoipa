# utoipa-scalar

[![Utoipa build](https://github.com/juhaku/utoipa/actions/workflows/build.yaml/badge.svg)](https://github.com/juhaku/utoipa/actions/workflows/build.yaml)
[![crates.io](https://img.shields.io/crates/v/utoipa-scalar.svg?label=crates.io&color=orange&logo=rust)](https://crates.io/crates/utoipa-scalar)
[![docs.rs](https://img.shields.io/static/v1?label=docs.rs&message=utoipa-scalar&color=blue&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/utoipa-scalar/latest/)
![rustc](https://img.shields.io/static/v1?label=rustc&message=1.60%2B&color=orange&logo=rust)

This crate works as a bridge between [utoipa](https://docs.rs/utoipa/latest/utoipa/) and [Scalar](https://scalar.com/) OpenAPI visualizer.

Utoipa-scalar provides simple mechanism to transform OpenAPI spec resource to a servable HTML
file which can be served via [predefined framework integration](#examples) or used
[standalone](#using-standalone) and served manually.

You may find fullsize examples from utoipa's Github [repository][examples].

# Crate Features

* **actix-web** Allows serving `Scalar` via _**`actix-web`**_. `version >= 4`
* **rocket** Allows serving `Scalar` via _**`rocket`**_. `version >=0.5`
* **axum** Allows serving `Scalar` via _**`axum`**_. `version >=0.7`

# Install

Use Scalar only without any boiler plate implementation.
```toml
[dependencies]
utoipa-scalar = "3"
```

Enable actix-web integration with Scalar.
```toml
[dependencies]
utoipa-scalar = { version = "3", features = ["actix-web"] }
```

# Using standalone

Utoipa-scalar can be used standalone as simply as creating a new `Scalar` instance and then
serving it by what ever means available as `text/html` from http handler in your favourite web
framework.

`Scalar::to_html` method can be used to convert the `Scalar` instance to a servable html
file.
```rust
let scalar = Scalar::new(ApiDoc::openapi());

// Then somewhere in your application that handles http operation.
// Make sure you return correct content type `text/html`.
let scalar = move || async {
    scalar.to_html()
};
```

# Examples

_**Serve `Scalar` via `actix-web` framework.**_
```rust
use actix_web::App;
use utoipa_scalar::{Scalar, Servable};

App::new().service(Scalar::with_url("/scalar", ApiDoc::openapi()));
```

_**Serve `Scalar` via `rocket` framework.**_
```rust
use utoipa_scalar::{Scalar, Servable};

rocket::build()
    .mount(
        "/",
        Scalar::with_url("/scalar", ApiDoc::openapi()),
    );
```

_**Serve `Scalar` via `axum` framework.**_
 ```rust
 use axum::Router;
 use utoipa_scalar::{Scalar, Servable};

 let app = Router::<S>::new()
     .merge(Scalar::with_url("/scalar", ApiDoc::openapi()));
```

_**Use `Scalar` to serve OpenAPI spec from url.**_
```rust
Scalar::new(
  "https://github.com/swagger-api/swagger-petstore/blob/master/src/main/resources/openapi.yaml")
```

_**Use `Scalar` to serve custom OpenAPI spec using serde's `json!()` macro.**_
```rust
Scalar::new(json!({"openapi": "3.1.0"}));
```

# License

Licensed under either of [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) license at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate
by you, shall be dual licensed, without any additional terms or conditions.

[examples]: <https://github.com/juhaku/utoipa/tree/master/examples>
