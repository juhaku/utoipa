# utoipa-swagger-ui

This crate implements necessary boiler plate code to serve Swagger UI via web server. It
works as a bridge for serving the OpenAPI documetation created with 
[utoipa](https://docs.rs/utoipa/) libarary in the Swagger UI.

**Currently supported frameworks:**

* **actix-web**

Serving Swagger UI is framework independant thus `SwaggerUi` and `Url` of this create
could be used similarly to serve the Swagger UI in other frameworks as well.

# Features

* **actix-web** Enables actix-web integration with pre-configured SwaggerUI service factory allowing
  users to use the Swagger UI without a hazzle.

# Install

Use only the raw types without any boiler plate implementation.
```text
[dependencies]
utoipa-swagger-ui = "0.1.0.beta1"

```
Enable actix-web framework with Swagger UI you could define the dependency as follows.
```text
[dependencies]
utoipa-swagger-ui = { version = "0.1.0.beta1", features = ["actix-web"] }
```

**Note!** Also remember that you already have defined `utoipa` dependency in your `Cargo.toml`

# Examples

Serve Swagger UI with api doc via actix-web. 
```rust
HttpServer::new(move || {
    App::new()
        .service(
            SwaggerUi::new("/swagger-ui/{_:.*}")
                .with_url("/api-doc/openapi.json", ApiDoc::openapi()),
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
