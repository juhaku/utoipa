# utoipa-swagger-ui

[![Utoipa build](https://github.com/juhaku/utoipa/actions/workflows/build.yaml/badge.svg)](https://github.com/juhaku/utoipa/actions/workflows/build.yaml)
[![crates.io](https://img.shields.io/static/v1?label=crates.io&message=0.1.2&color=orange&logo=rust)](https://crates.io/crates/utoipa-swagger-ui/0.1.2)
[![docs.rs](https://img.shields.io/static/v1?label=docs.rs&message=utoipa-swagger-ui&color=blue&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/utoipa-swagger-ui/0.1.2/utoipa_swagger_ui/)

This crate implements necessary boiler plate code to serve Swagger UI via web server. It
works as a bridge for serving the OpenAPI documetation created with 
[utoipa](https://docs.rs/utoipa/) libarary in the Swagger UI.

**Currently supported frameworks:**

* **actix-web** `version >= 4`

Serving Swagger UI is framework independant thus `SwaggerUi` and `Url` of this create
could be used similarly to serve the Swagger UI in other frameworks as well.

# Features

* **actix-web** Enables actix-web integration with pre-configured SwaggerUI service factory allowing
  users to use the Swagger UI without a hazzle.

# Install

Use only the raw types without any boiler plate implementation.
```text
[dependencies]
utoipa-swagger-ui = "0.1.2"

```
Enable actix-web framework with Swagger UI you could define the dependency as follows.
```text
[dependencies]
utoipa-swagger-ui = { version = "0.1.2", features = ["actix-web"] }
```

**Note!** Also remember that you already have defined `utoipa` dependency in your `Cargo.toml`

# Examples

Serve Swagger UI with api doc via actix-web. 
```rust
HttpServer::new(move || {
    App::new()
        .service(
            SwaggerUi::new("/swagger-ui/{_:.*}")
                .url("/api-doc/openapi.json", ApiDoc::openapi()),
        )
  })
  .bind(format!("{}:{}", Ipv4Addr::UNSPECIFIED, 8989)).unwrap()
  .run();
```
**actix-web** feature need to be enabled.

# License

Licensed under either of [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) license at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate 
by you, shall be dual licensed, without any additional terms or conditions. 
