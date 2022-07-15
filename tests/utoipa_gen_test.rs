#![cfg(feature = "yaml")]
#![cfg(feature = "json")]
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use utoipa::{
    openapi::{
        self,
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
        server::{ServerBuilder, ServerVariableBuilder},
    },
    Component, Modify, OpenApi,
};

#[derive(Deserialize, Serialize, Component)]
#[component(example = json!({"name": "bob the cat", "id": 1}))]
struct Pet {
    id: u64,
    name: String,
    age: Option<i32>,
}

// #[derive(Component)]
// struct Status<StatusType> {
//     status: StatusType,
// }

// #[derive(Component)]
// enum StatusType {
//     Ok,
//     NotOk,
// }

// #[derive(Component)]
// enum Random {
//     Response { id: String },
//     PetResponse(Pet),
//     Ids(Vec<String>),
//     UnitValue,
// }

// #[derive(Serialize, Deserialize, Component)]
// struct Simple {
//     greeting: &'static str,
//     cow: Cow<'static, str>,
// }

mod pet_api {
    use super::*;

    /// Get pet by id
    ///
    /// Get pet from database by pet database id
    #[utoipa::path(
        get,
        path = "/pets/{id}",
        responses(
            (status = 200, description = "Pet found succesfully", body = Pet),
            (status = 404, description = "Pet was not found")
        ),
        params(
            ("id" = u64, Path, description = "Pet database id to get Pet for"),
        ),
        security(
            (),
            ("my_auth" = ["read:items", "edit:items"]),
            ("token_jwt" = [])
        )
    )]
    #[allow(unused)]
    async fn get_pet_by_id(pet_id: u64) -> Pet {
        Pet {
            id: pet_id,
            age: None,
            name: "lightning".to_string(),
        }
    }
}

#[derive(Default, OpenApi)]
#[openapi(
    handlers(pet_api::get_pet_by_id),
    components(Pet, GenericC, GenericD),
    modifiers(&Foo),
    security(
        (),
        ("my_auth" = ["read:items", "edit:items"]),
        ("token_jwt" = [])
    )
)]
struct ApiDoc;

macro_rules! build_foo {
    ($typ: ident, $d: ty, $r: ty) => {
        #[derive(Debug, Serialize, Component)]
        struct $typ {
            data: $d,
            resources: $r,
        }
    };
}

#[derive(Deserialize, Serialize, Component)]
struct A {
    a: String,
}

#[derive(Deserialize, Serialize, Component)]
struct B {
    b: i64,
}

#[derive(Deserialize, Serialize, Component)]
#[aliases(GenericC = C<A, B>, GenericD = C<B, A>)]
struct C<T, R> {
    field_1: R,
    field_2: T,
}

#[test]
#[ignore = "this is just a test bed to run macros"]
fn derive_openapi() {
    utoipa::openapi::OpenApi::new(
        utoipa::openapi::Info::new("my application", "0.1.0"),
        utoipa::openapi::Paths::new(),
    );
    println!("{}", ApiDoc::openapi().to_pretty_json().unwrap());

    build_foo!(GetFooBody, Foo, FooResources);
}

#[test]
fn stable_yaml() {
    let left = ApiDoc::openapi().to_yaml().unwrap();
    let right = ApiDoc::openapi().to_yaml().unwrap();
    assert_eq!(left, right);
}

#[test]
fn stable_json() {
    let left = ApiDoc::openapi().to_json().unwrap();
    let right = ApiDoc::openapi().to_json().unwrap();
    assert_eq!(left, right);
}

impl Modify for Foo {
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        if let Some(schema) = openapi.components.as_mut() {
            schema.add_security_scheme(
                "token_jwt",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }

        openapi.servers = Some(vec![ServerBuilder::new()
            .url("/api/bar/{username}")
            .description(Some("this is description of the server"))
            .parameter(
                "username",
                ServerVariableBuilder::new()
                    .default_value("the_user")
                    .description(Some("this is user")),
            )
            .build()]);
    }
}

#[derive(Debug, Serialize)]
struct Foo;

#[derive(Debug, Serialize)]
struct FooResources;
