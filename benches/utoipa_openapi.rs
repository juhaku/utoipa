#![feature(test)]

use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use utoipa::{Component, OpenApi};

extern crate test;



#[cfg(test)]
mod tests {
    use test::Bencher;

    use super::*;

    #[derive(Deserialize, Serialize, Component)]
    #[component(example = json!({"name": "bob the cat", "id": 1}))]
    struct Pet {
        id: u64,
        name: String,
        age: Option<i32>,
    }

    #[derive(Component)]
    struct Status<StatusType> {
        status: StatusType,
    }

    #[derive(Component)]
    enum StatusType {
        Ok,
        NotOk,
    }

    #[derive(Component)]
    enum Random {
        Response { id: String },
        PetResponse(Pet),
        Ids(Vec<String>),
        UnitValue,
    }

    #[derive(Serialize, Deserialize, Component)]
    struct Simple {
        greeting: &'static str,
        cow: Cow<'static, str>,
    }

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
                ("id" = u64, path, description = "Pet database id to get Pet for"),
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
        components(Pet, StatusType, Random, Status<StatusType>), 
        security(
            (),
            ("my_auth" = ["read:items", "edit:items"]),
            ("token_jwt" = [])
        )
    )]
    struct ApiDoc;


    #[bench]
    fn call_utoipa_openapi(b: &mut Bencher) {
        b.iter(|| {
            ApiDoc::openapi()
        });
    }
}


