#![cfg(feature = "axum_extras")]
#![cfg(feature = "serde_json")]

use assert_json_diff::assert_json_eq;
use axum::extract::{Path, Query};
use serde::Deserialize;
use serde_json::json;
use utoipa::{IntoParams, OpenApi};

#[test]
fn derive_path_params_into_params_axum() {
    #[derive(Deserialize, IntoParams)]
    #[allow(unused)]
    struct Person {
        /// Id of person
        id: i64,
        /// Name of person
        name: String,
    }

    pub mod custom {
        use serde::Deserialize;
        use utoipa::IntoParams;
        #[derive(Deserialize, IntoParams)]
        #[allow(unused)]
        pub(super) struct Filter {
            /// Age filter for user
            #[deprecated]
            age: Option<Vec<String>>,
        }
    }

    #[utoipa::path(
        get,
        path = "/person/{id}/{name}",
        params(Person, custom::Filter),
        responses(
            (status = 200, description = "success response")
        )
    )]
    #[allow(unused)]
    async fn get_person(person: Path<Person>, query: Query<custom::Filter>) {}

    #[derive(OpenApi)]
    #[openapi(handlers(get_person))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc
        .pointer("/paths/~1person~1{id}~1{name}/get/parameters")
        .unwrap();

    assert_json_eq!(
        parameters,
        &json!([
            {
                "description": "Id of person",
                "in": "path",
                "name": "id",
                "required": true,
                "schema": {
                    "format": "int64",
                    "type": "integer",
                },
            },
            {
                "description": "Name of person",
                "in": "path",
                "name": "name",
                "required": true,
                "schema": {
                    "type": "string",
                },
            },
            {
                "deprecated": true,
                "description":  "Age filter for user",
                "in":  "query",
                "name": "age",
                "required": false,
                "schema": {
                    "items": {
                        "type": "string",
                    },
                    "type": "array",
                }
            },
        ])
    )
}
