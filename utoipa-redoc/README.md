# utoipa-redoc

[![Utoipa build](https://github.com/juhaku/utoipa/actions/workflows/build.yaml/badge.svg)](https://github.com/juhaku/utoipa/actions/workflows/build.yaml)
[![crates.io](https://img.shields.io/crates/v/utoipa-redoc.svg?label=crates.io&color=orange&logo=rust)](https://crates.io/crates/utoipa-redoc)
[![docs.rs](https://img.shields.io/static/v1?label=docs.rs&message=utoipa-redoc&color=blue&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/utoipa-redoc/latest/)
![rustc](https://img.shields.io/static/v1?label=rustc&message=1.60%2B&color=orange&logo=rust)

This crate works as a bridge between [utoipa](https://docs.rs/utoipa/latest/utoipa/) and [Redoc](https://redocly.com/) OpenAPI visualizer.

Utoipa-redoc provides simple mechanism to transform OpenAPI spec resource to a servable HTML
file which can be served via [predefined framework integration](#examples) or used
[standalone](#using-standalone) and served manually.

You may find fullsize examples from utoipa's Github [repository][examples].

# Crate Features

* **actix-web** Allows serving `Redoc` via _**`actix-web`**_. `version >= 4`
* **rocket** Allows serving `Redoc` via _**`rocket`**_. `version >=0.5.0-rc.3`
* **axum** Allows serving `Redoc` via _**`axum`**_. `version >=0.6`

# Install

Use Redoc only without any boiler plate implementation.
```toml
[dependencies]
utoipa-redoc = "0.1"
```

Enable actix-web integration with Redoc.
```toml
[dependencies]
utoipa-redoc = { version = "0.1", features = ["actix-web"] }
```

# Using standalone

Utoipa-redoc can be used standalone as simply as creating a new `Redoc` instance and then
serving it by what ever means available as `text/html` from http handler in your favourite web
framework.

`Redoc::to_html` method can be used to convert the `Redoc` instance to a servable html
file.
```rust
let redoc = Redoc::new(ApiDoc::openapi());

// Then somewhere in your application that handles http operation.
// Make sure you return correct content type `text/html`.
let redoc_handler = move || async {
    redoc.to_html()
};
```

# Customization

Utoipa-redoc enables full customizaton support for [Redoc][redoc] according to what can be
customized by modifying the HTML template and [configuration options](#configuration).

The default [HTML template][redoc_html_quickstart] can be fully overridden to ones liking with
`Redoc::custom_html` method. The HTML template **must** contain **`$spec`** and **`$config`**
variables which are replaced during `Redoc::to_html` execution.

* **`$spec`** Will be the `Spec` that will be rendered via [Redoc][redoc].
* **`$config`** Will be the current `Config`. By default this is `EmptyConfig`.

_**Overiding the HTML template with a custom one.**_
```rust
let html = "...";
Redoc::new(ApiDoc::openapi()).custom_html(html);
```

# Configuration

Redoc can be configured with JSON either inlined with the `Redoc` declaration or loaded from
user defined file with `FileConfig`.

* [All supported Redoc configuration options][redoc_config].

_**Inlining the configuration.**_
```rust
Redoc::with_config(ApiDoc::openapi(), || json!({ "disableSearch": true }));
```

_**Using `FileConfig`.**_
```rust
Redoc::with_config(ApiDoc::openapi(), FileConfig);
```

Read more details in `Config`.

# Examples

_**Serve `Redoc` via `actix-web` framework.**_
```rust
use actix_web::App;
use utoipa_redoc::{Redoc, Servable};

App::new().service(Redoc::with_url("/redoc", ApiDoc::openapi()));
```

_**Serve `Redoc` via `rocket` framework.**_
```rust
use utoipa_redoc::{Redoc, Servable};

rocket::build()
    .mount(
        "/",
        Redoc::with_url("/redoc", ApiDoc::openapi()),
    );
```

_**Serve `Redoc` via `axum` framework.**_
 ```rust
 use axum::{Router, body::HttpBody};
 use utoipa_redoc::{Redoc, Servable};

 let app = Router::<S, B>::new()
     .merge(Redoc::with_url("/redoc", ApiDoc::openapi()));
```

_**Use `Redoc` to serve OpenAPI spec from url.**_
```rust
Redoc::new(
  "https://github.com/swagger-api/swagger-petstore/blob/master/src/main/resources/openapi.yaml")
```

_**Use `Redoc` to serve custom OpenAPI spec using serde's `json!()` macro.**_
```rust
Redoc::new(json!({"openapi": "3.1.0"}));
```

# License

Licensed under either of [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) license at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate
by you, shall be dual licensed, without any additional terms or conditions.

[redoc]: <https://redocly.com/>
[redoc_html_quickstart]: <https://redocly.com/docs/redoc/quickstart/>
[redoc_config]: <https://redocly.com/docs/api-reference-docs/configuration/functionality/#configuration-options-for-api-docs>
[examples]: <https://github.com/juhaku/utoipa/tree/master/examples>
