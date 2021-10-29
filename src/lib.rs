pub mod error;
pub mod openapi;
#[cfg(feature = "swagger_ui")]
pub mod swagger_ui;
pub mod types;

pub use utoipa_gen::*;

pub trait OpenApi {
    fn openapi() -> openapi::OpenApi;
}
