# actix-web-multiple-api-docs-with-scopes ~ utoipa with utoipa-swagger-ui example

This is a demo `actix-web` application with multiple API docs with scope and context path.

Just run command below to run the demo application and browse to `http://localhost:8080/swagger-ui/`.

```bash
cargo run
```

On the Swagger-UI will be a drop-down labelled "Select a definition", containing "api1" and "api2".

Alternatively, they can be loaded directly using

- api1: http://localhost:8080/swagger-ui/?urls.primaryName=api1
- api2: http://localhost:8080/swagger-ui/?urls.primaryName=api1

If you want to see some logging, you may prepend the command with `RUST_LOG=debug` as shown below.

```bash
RUST_LOG=debug cargo run
```
