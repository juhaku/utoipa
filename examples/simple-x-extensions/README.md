# simple-x-extensions ~ utoipa with extensions example

This is a demo `actix-web` application that defines an OpenAPI with OpenAPI extensions. The extensions are set via the `utoipa::path`
macro as well as using the `Modify` trait - for comparisons.

Just run command below to run the demo application - a `actix-web` web server - and browse to `http://localhost:8080/openapi` to view the OpenAPI in yaml.

```bash
cargo run
```

If you want to see some logging, you may prepend the command with `RUST_LOG=debug` as shown below.

```bash
RUST_LOG=debug cargo run
```
