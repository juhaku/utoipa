//! This crate holds the [Swagger UI](https://github.com/swagger-api/swagger-ui) zip archive re-packaged as
//! Rust crate. The crate serves as a build dependency for `utoipa-swagger-ui` and is used to serve the
//! Swagger UI when `vendored` crate feature is enabled for `utoipa-swagger-ui` crate.
//!
//! Vendored Swagger UI provides the means to serve Swagger UI in sandboxed environments where network access or
//! even other means to provide Swagger UI is not possible.
//!
//! **Swagger UI version: `5.17.14`**
//!
//! ## License
//!
//! Licensed under either of [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) license at your option.
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate
//! by you, shall be dual licensed, without any additional terms or conditions.

/// Swagger UI zip bytes
#[doc(hidden)]
pub const SWAGGER_UI_VENDORED: &[u8] = std::include_bytes!("../res/v5.17.14.zip");
