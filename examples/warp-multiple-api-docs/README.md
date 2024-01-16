# warp-multiple-api-docs ~ utoipa with utoipa-swagger-ui example

This is a demo `warp` application with multiple API docs to demonstrate splitting APIs with `utoipa` and `utoipa-swagger-ui`.

Just run command below to run the demo application and browse to `http://localhost:8080/swagger-ui/`.

```bash
cargo run
```

On the Swagger-UI will be a drop-down labelled "Select a definition", containing `/api-doc1.json` and `/api-doc2.json`.

Alternatively, they can be loaded directly using

- api1: http://localhost:8080/swagger-ui/?urls.primaryName=%2Fapi-doc1.json
- api2: http://localhost:8080/swagger-ui/?urls.primaryName=%2Fapi-doc2.json

If you want to see some logging, you may prepend the command with `RUST_LOG=debug` as shown below.

```bash
RUST_LOG=debug cargo run
```
