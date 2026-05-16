#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use utoipa::{
    openapi::{
        self,
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
        server::{ServerBuilder, ServerVariableBuilder},
    },
    Modify, OpenApi, ToSchema,
};

#[derive(Deserialize, Serialize, ToSchema)]
#[schema(example = json!({"name": "bob the cat", "id": 1}))]
struct Pet {
    id: u64,
    name: String,
    age: Option<i32>,
}

// #[derive(ToSchema)]
// struct Status<StatusType> {
//     status: StatusType,
// }

// #[derive(ToSchema)]
// enum StatusType {
//     Ok,
//     NotOk,
// }

// #[derive(ToSchema)]
// enum Random {
//     Response { id: String },
//     PetResponse(Pet),
//     Ids(Vec<String>),
//     UnitValue,
// }

// #[derive(Serialize, Deserialize, ToSchema)]
// struct Simple {
//     greeting: &'static str,
//     cow: Cow<'static, str>,
// }

mod pet_api {
    use super::*;

    const ID: &str = "get_pet";

    /// Get pet by id
    ///
    /// Get pet from database by pet database id
    #[utoipa::path(
        get,
        operation_id = ID,
        path = "/pets/{id}",
        responses(
            (status = 200, description = "Pet found successfully", body = Pet),
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
    paths(pet_api::get_pet_by_id),
    components(schemas(Pet, C<A, B>, C<B, A>)),
    modifiers(&Foo),
    security(
        (),
        ("my_auth" = ["read:items", "edit:items"]),
        ("token_jwt" = [])
    )
)]
struct ApiDoc;

macro_rules! build_foo {
    ($type: ident, $d: ty, $r: ty) => {
        #[derive(Debug, Serialize, ToSchema)]
        struct $type {
            data: $d,
            resources: $r,
        }
    };
}

#[derive(Deserialize, Serialize, ToSchema)]
struct A {
    a: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
struct B {
    b: i64,
}

#[derive(Deserialize, Serialize, ToSchema)]
struct C<T, R> {
    field_1: R,
    field_2: T,
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

#[derive(Debug, Serialize, ToSchema)]
struct Foo;

#[derive(Debug, Serialize, ToSchema)]
struct FooResources;

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
fn derive_openapi_with_security_display_types() {
    use std::fmt::Display;

    #[derive(Debug)]
    enum AuthScope {
        Read,
        Write,
        Admin,
    }

    impl Display for AuthScope {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                AuthScope::Read => write!(f, "read:all"),
                AuthScope::Write => write!(f, "write:all"),
                AuthScope::Admin => write!(f, "admin:all"),
            }
        }
    }

    const CUSTOM_SCOPE: &str = "custom:scope";

    #[derive(Default, OpenApi)]
    #[openapi(
        security(
            (),
            ("oauth2" = [AuthScope::Read.to_string(), AuthScope::Write.to_string()]),
            ("api_key" = []),
            ("mixed" = [CUSTOM_SCOPE, AuthScope::Admin.to_string()])
        )
    )]
    struct ApiDocWithDisplay;

    let api = ApiDocWithDisplay::openapi();
    let json = api.to_json().unwrap();
    let security = serde_json::from_str::<serde_json::Value>(&json).unwrap()["security"].clone();

    assert_eq!(security[0], serde_json::json!({}));
    assert_eq!(
        security[1]["oauth2"],
        serde_json::json!(["read:all", "write:all"])
    );
    assert_eq!(security[2]["api_key"], serde_json::json!([]));
    assert_eq!(
        security[3]["mixed"],
        serde_json::json!(["custom:scope", "admin:all"])
    );
}
