# raw-json-actix

This is demo `actix-web` application showing using raw JSON in endpoints.
The API demonstrates `utoipa` with `utoipa-swagger-ui` functionalities.

Just run command below to run the demo application and browse to `http://localhost:8080/swagger-ui/`.
```bash
cargo run
```

In the swagger UI:

1. Send body `"string"` and the console will show the body was a `serde_json::String`.
2. Send body `1` and the console will show the body was a `serde_json::Number`.
3. Send body `[1, 2]` and the console will show the body was a `serde_json::Array`.