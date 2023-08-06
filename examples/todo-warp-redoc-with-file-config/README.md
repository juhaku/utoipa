# todo-warp-redoc-with-file-config ~ utoipa with utoipa-redoc example

This is a demo `warp` application with in-memory storage to manage Todo items.

This example is more bare minimum compared to `todo-actix`, since similarly same macro syntax is
supported, no matter the framework.


This how `utoipa-redoc` can be used as standalone without pre-existing framework integration with additional
file configuration for the Redoc UI. The configuration is applicable in any other `utoipa-redoc` setup as well.

See the `build.rs` file that defines the Redoc config file and `redoc.json` where the [configuration options](https://redocly.com/docs/api-reference-docs/configuration/functionality/#configuration-options-for-api-docs)
are defined. 

For security restricted endpoints the super secret API key is: `utoipa-rocks`.

Just run command below to run the demo application and browse to `http://localhost:8080/redoc`.

```bash
cargo run
```

If you want to see some logging, you may prepend the command with `RUST_LOG=debug` as shown below.

```bash
RUST_LOG=debug cargo run
```
