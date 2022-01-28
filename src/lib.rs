pub mod error;
pub mod openapi;
#[cfg(feature = "swagger_ui")]
pub mod swagger_ui;
pub mod types;

pub use utoipa_gen::*;

pub trait OpenApi {
    fn openapi() -> openapi::OpenApi;
}

pub trait Component {
    fn component() -> openapi::schema::Component;

    fn sub_components() -> Vec<(&'static str, openapi::schema::Component)> {
        Vec::new()
    }
}

pub trait Path {
    fn path() -> &'static str;

    fn path_item() -> openapi::path::PathItem;
}

pub trait DefaultTag {
    fn tag() -> &'static str;
}

pub trait Tag {
    fn tag() -> &'static str;
}
