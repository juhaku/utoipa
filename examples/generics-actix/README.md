# generics-actix

This is demo `actix-web` application showing using external `geo-types`, which uses generics, in endpoints.
The API demonstrates `utoipa` with `utoipa-swagger-ui` functionalities.

Just run command below to run the demo application and browse to `http://localhost:8080/swagger-ui/`.

```bash
cargo run
```

In the swagger UI:

1. Send `x=1`, `y=2` to endpoint `coord_u64` to see an integer `x`,`y` coord object returned.
2. Send `x=1.1`, `y=2.2` to endpoint `coord_f64` to see a float `x`,`y` coord object returned.

If you want to see some logging, you may prepend the command with `RUST_LOG=debug` as shown below.

```bash
RUST_LOG=debug cargo run
```
