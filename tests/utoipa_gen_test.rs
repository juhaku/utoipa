use std::{collections::HashMap, fmt::Display};

// use utoipa::openapi_spec;
use utoipa::{api_operation, Component, OpenApi};

/// Delete foo entity
///
/// Delete foo entity by what
#[api_operation(delete, responses = [
    (200, "success", String),
    (400, "my bad error", u64),
    (404, "vault not found"),
    (500, "internal server error")
])]
fn foo_delete() {}

#[test]
fn derive_openapi() {
    #[derive(OpenApi, Default)]
    #[openapi(handler_files = ["tests/utoipa_gen_test.rs"])]
    struct ApiDoc;

    println!("{:?}", ApiDoc::openapi().to_json())
}

#[test]
fn derive_component_struct() {
    /// Mode defines user type
    #[derive(Component)]
    #[component(default = "Mode1", example = "Mode2")]
    enum Mode {
        /// Mode1 is admin user
        Mode1,
        Mode2,
        // Mode3(usize),
        // Mode4 { x: String },
        // Mode5(usize, String),
    }

    #[derive(Component)]
    struct Book {
        #[component(default = "crate::random_default", example = "Book 1")]
        name: String,

        #[component(
            default = "test",
            example = "base64 text",
            format = "ComponentFormat::Byte"
        )]
        hash: String,
    }

    /// This is user component
    ///
    /// User component is being used to store user information
    #[derive(Component)]
    // #[component()]
    struct User {
        /// This is a database id of a user
        #[component(default = 1)]
        id: u64,
        // username: String,
        /// User authenticated roles
        roles: Vec<String>,
        /// Foobar hashmap
        foobar: HashMap<String, i64>,
        /// Optional value is user enabled
        enabled: Option<bool>,
        // random: Option<Vec<String>>,
        // mode: Option<Mode>,
        // book: Book,
        long_property: String,
    }

    #[derive(OpenApi, Default)]
    #[openapi(handler_files = [], components = [User, Book, Mode])]
    struct ApiDoc;

    println!("{}", ApiDoc::openapi().to_json().unwrap());
}

fn random_default() -> String {
    "random".to_string()
}
