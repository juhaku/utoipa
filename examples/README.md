# utoipa examples

This is folder contain a set of examples of utoipa library which should help people to get started
with the library.

All examples have their own `README.md`, and can be seen using two steps:

1. Run `cargo run`
2. Browse to `http://localhost:8080/swagger-ui/` or `http://localhost:8080/redoc` or `http://localhost:8080/rapidoc`.

`todo-actix`, `todo-axum` and `rocket-todo` have Swagger UI, Redoc, RapiDoc, and Scalar setup, others have Swagger UI 
if not explicitly stated otherwise.

Even if there is no example for your favourite framework, `utoipa` can be used with any
web framework which supports decorating functions with macros similarly to the **warp** and **tide** examples.

## Community examples

- **[graphul](https://github.com/graphul-rs/graphul/tree/main/examples/utoipa-swagger-ui)**
- **[salvo](https://github.com/salvo-rs/salvo/tree/main/examples/todos-utoipa)**
- **[viz](https://github.com/viz-rs/viz/tree/main/examples/routing/openapi)**
- **[ntex](https://github.com/leon3s/ntex-rest-api-example)**

