# axum-nesting-vendored ~ utoipa with utoipa-swagger-ui example

This example demonstrates `axum` with programmatic and macro based nesting of OpenApis
using `utoipa-swagger-ui` for visualization.

Example uses `utoipa-swagger-ui-vendored` to demonstrate vendored version of Swagger UI.

Just run command below to run the demo application and browse to `http://localhost:8080/swagger-ui/`.

```bash
cargo run
```

## Run with Docker

You have to build the crate with `--release` or set `debug-embed` in order to embed Swagger UI.
```bash
cargo build --release --target x86_64-unknown-linux-musl
docker build -t axum-utoipa-nesting:latest .
docker run -p 8080:8080 -t axum-utoipa-nesting:latest
```
