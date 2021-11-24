use std::collections::HashMap;

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
    #[derive(Component)]
    enum Mode {
        Mode1,
        Mode2,
    }

    #[derive(Component)]
    struct Book {
        name: String,
    }

    /// This is user component
    ///
    /// User component is being used to store user information
    #[derive(Component)]
    // #[component()]
    struct User {
        id: u64,
        username: String,
        roles: Vec<String>,
        foobar: HashMap<String, i64>,
        enabled: Option<bool>,
        random: Option<Vec<String>>,
        mode: Option<Mode>,
        book: Book,
    }

    #[derive(OpenApi, Default)]
    #[openapi(handler_files = [], components = [User, Book])]
    struct ApiDoc;

    println!("{}", ApiDoc::openapi().to_json().unwrap());
}
