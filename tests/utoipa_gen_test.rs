#![cfg(feature = "actix_extras")]
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use utoipa::{Component, OpenApi};

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
        path = "/pets/{id}"
        responses = [
            (status = 200, description = "Pet found succesfully", body = Pet),
            (status = 404, description = "Pet was not found")
        ],
        params = [
            ("id" = u64, path, description = "Pet database id to get Pet for"),
        ]
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

#[derive(OpenApi, Default)]
#[openapi(handlers = [pet_api::get_pet_by_id], components = [Pet])]
struct ApiDoc;

#[test]
#[ignore = "this is just a test bed to run macros"]
fn derive_openapi() {
    println!("{}", ApiDoc::openapi().to_pretty_json().unwrap());
}

fn path() -> &'static str {
    "/pets/{id}"
}

fn path_item(default_tag: Option<&str>) -> utoipa::openapi::path::Paths {
    utoipa::openapi::Paths::new().append(
        "/pets/{id}",
        utoipa::openapi::PathItem::new(
            utoipa::openapi::PathItemType::Get,
            utoipa::openapi::path::Operation::new()
                .with_responses(
                    utoipa::openapi::Responses::new()
                        .with_response(
                            "200",
                            utoipa::openapi::Response::new("Pet found succesfully").with_content(
                                "application/json",
                                utoipa::openapi::Content::new(
                                    utoipa::openapi::Ref::from_component_name("Pet"),
                                ),
                            ),
                        )
                        .with_response("404", utoipa::openapi::Response::new("Pet was not found")),
                )
                .with_operation_id("get_pet_by_id")
                .with_deprecated(utoipa::openapi::Deprecated::False)
                .with_summary("Get pet by id")
                .with_description("Get pet by id\n\nGet pet from database by pet database id\n")
                .with_parameter(
                    utoipa::openapi::path::Parameter::new("id")
                        .with_in(utoipa::openapi::path::ParameterIn::Path)
                        .with_deprecated(utoipa::openapi::Deprecated::False)
                        .with_description("Pet database id to get Pet for")
                        .with_schema(
                            utoipa::openapi::Property::new(utoipa::openapi::ComponentType::Integer)
                                .with_format(utoipa::openapi::ComponentFormat::Int64),
                        )
                        .with_required(utoipa::openapi::Required::True),
                )
                .with_tag("pet_api"),
        ),
    )
}
