# hello rocket 0.4.x version ~ utoipa with utoipa-swagger-ui example

This is a hello world `rocket 0.4.x` version example, using `utoipa` and `utoipa-swagger-ui`.

If you are looking for rocket `0.5.x` example look for [rocket-todo](../rocket-todo).

Just run command below to run the demo application and browse to `http://localhost:8000/swagger-ui/index.html`.

```bash
cargo run
```

If you want to see some logging, you may prepend the command with `RUST_LOG=debug` as shown below.

```bash
RUST_LOG=debug cargo run
```

Rocket `0.4` needs nightly toolchain, so if you encounter error, remove `Cargo.lock` and run `cargo clean`
and then try to re-build / re-run the project.
