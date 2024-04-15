# todo-rocket ~ utoipa with utoipa-swagger-ui, utoipa-redoc and utoipa-rapidoc example

This is a demo `rocket` application with in-memory storage to manage Todo items. The API
demonstrates `utoipa` with `utoipa-swagger-ui` functionalities.

For security restricted endpoints the super secret API key is: `utoipa-rocks`.

Just run command below to run the demo application and browse to `http://localhost:8000/swagger-ui/`.

If you prefer Redoc just head to `http://localhost:8000/redoc` and view the Open API.

RapiDoc can be found from `http://localhost:8000/redoc`.

Scalar can be reached on `http://localhost:8000/scalar`.

```bash
cargo run
```

If you want to see some logging, you may prepend the command with `RUST_LOG=debug` as shown below.

```bash
RUST_LOG=debug cargo run
```
