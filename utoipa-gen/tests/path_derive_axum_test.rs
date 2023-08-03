#![cfg(feature = "axum_extras")]

use std::sync::{Arc, Mutex};

use assert_json_diff::{assert_json_eq, assert_json_matches, CompareMode, Config, NumericMode};
use axum::{
    extract::{Path, Query},
    Extension, Json,
};
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
    #[openapi(paths(get_person))]
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
                    "nullable": true,
                    "type": "array",
                }
            },
        ])
    )
}

#[test]
fn get_todo_with_path_tuple() {
    #[utoipa::path(
        get,
        path = "/person/{id}/{name}",
        params(
            ("id", description = "Person id"),
            ("name", description = "Person name")
        ),
        responses(
            (status = 200, description = "success response")
        )
    )]
    #[allow(unused)]
    async fn get_person(Path((id, name)): Path<(String, String)>) {}

    #[derive(OpenApi)]
    #[openapi(paths(get_person))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc
        .pointer("/paths/~1person~1{id}~1{name}/get/parameters")
        .unwrap();

    assert_json_eq!(
        parameters,
        &json!([
            {
                "description": "Person id",
                "in": "path",
                "name": "id",
                "required": true,
                "schema": {
                    "type": "string"
                },
            },
            {
                "description": "Person name",
                "in": "path",
                "name": "name",
                "required": true,
                "schema": {
                    "type": "string",
                },
            },
        ])
    )
}

#[test]
fn get_todo_with_extension() {
    struct Todo {
        #[allow(unused)]
        id: i32,
    }
    /// In-memory todo store
    type Store = Mutex<Vec<Todo>>;
    /// List all Todo items

    ///
    /// List all Todo items from in-memory storage.
    #[utoipa::path(
        get,
        path = "/todo",
        responses(
            (status = 200, description = "List all todos successfully", body = [Todo])
        )
    )]
    #[allow(unused)]
    fn list_todos(Extension(store): Extension<Arc<Store>>) {}

    #[derive(OpenApi)]
    #[openapi(paths(list_todos))]
    struct ApiDoc;

    serde_json::to_value(ApiDoc::openapi())
        .unwrap()
        .pointer("/paths/~1todo/get")
        .expect("Expected to find /paths/todo/get");
}

#[test]
fn derive_path_params_into_params_unnamed() {
    #[derive(Deserialize, IntoParams)]
    #[into_params(names("id", "name"))]
    struct IdAndName(u64, String);

    #[utoipa::path(
        get,
        path = "/person/{id}/{name}",
        params(IdAndName),
        responses(
            (status = 200, description = "success response")
        )
    )]
    #[allow(unused)]
    async fn get_person(person: Path<IdAndName>) {}

    #[derive(OpenApi)]
    #[openapi(paths(get_person))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc
        .pointer("/paths/~1person~1{id}~1{name}/get/parameters")
        .unwrap();

    let config = Config::new(CompareMode::Strict).numeric_mode(NumericMode::AssumeFloat);

    assert_json_matches!(
        parameters,
        &json!([
            {
                "in": "path",
                "name": "id",
                "required": true,
                "schema": {
                    "format": "int64",
                    "type": "integer",
                    "minimum": 0.0,
                },
            },
            {
                "in": "path",
                "name": "name",
                "required": true,
                "schema": {
                    "type": "string",
                },
            },
        ]),
        config
    )
}

#[test]
fn derive_path_params_with_ignored_parameter() {
    struct Auth;
    #[derive(Deserialize, IntoParams)]
    #[into_params(names("id", "name"))]
    struct IdAndName(u64, String);

    #[utoipa::path(
        get,
        path = "/person/{id}/{name}",
        params(IdAndName),
        responses(
            (status = 200, description = "success response")
        )
    )]
    #[allow(unused)]
    async fn get_person(_: Auth, person: Path<IdAndName>) {}

    #[derive(OpenApi)]
    #[openapi(paths(get_person))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc
        .pointer("/paths/~1person~1{id}~1{name}/get/parameters")
        .unwrap();

    let config = Config::new(CompareMode::Strict).numeric_mode(NumericMode::AssumeFloat);

    assert_json_matches!(
        parameters,
        &json!([
            {
                "in": "path",
                "name": "id",
                "required": true,
                "schema": {
                    "format": "int64",
                    "type": "integer",
                    "minimum": 0.0,
                },
            },
            {
                "in": "path",
                "name": "name",
                "required": true,
                "schema": {
                    "type": "string",
                },
            },
        ]),
        config
    )
}

#[test]
fn derive_path_params_with_unnamed_struct_desctructed() {
    #[derive(Deserialize, IntoParams)]
    #[into_params(names("id", "name"))]
    struct IdAndName(u64, String);

    #[utoipa::path(
        get,
        path = "/person/{id}/{name}",
        params(IdAndName),
        responses(
            (status = 200, description = "success response")
        )
    )]
    #[allow(unused)]
    async fn get_person(Path(IdAndName(id, name)): Path<IdAndName>) {}

    #[derive(OpenApi)]
    #[openapi(paths(get_person))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc
        .pointer("/paths/~1person~1{id}~1{name}/get/parameters")
        .unwrap();

    let config = Config::new(CompareMode::Strict).numeric_mode(NumericMode::AssumeFloat);
    assert_json_matches!(
        parameters,
        &json!([
            {
                "in": "path",
                "name": "id",
                "required": true,
                "schema": {
                    "format": "int64",
                    "type": "integer",
                    "minimum": 0.0,
                },
            },
            {
                "in": "path",
                "name": "name",
                "required": true,
                "schema": {
                    "type": "string",
                },
            },
        ]),
        config
    )
}

#[test]
fn derive_path_query_params_with_named_struct_destructed() {
    #[derive(IntoParams)]
    #[allow(unused)]
    struct QueryParmas<'q> {
        name: &'q str,
    }

    #[utoipa::path(get, path = "/item", params(QueryParmas))]
    #[allow(unused)]
    async fn get_item(Query(QueryParmas { name }): Query<QueryParmas<'static>>) {}

    #[derive(OpenApi)]
    #[openapi(paths(get_item))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc.pointer("/paths/~1item/get/parameters").unwrap();

    assert_json_eq!(
        parameters,
        &json!([
            {
                "in": "query",
                "name": "name",
                "required": true,
                "schema": {
                    "type": "string",
                },
            },
        ])
    )
}

#[test]
fn path_with_path_query_body_resolved() {
    #[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
    struct Item(String);

    #[allow(unused)]
    struct Error;

    #[derive(serde::Serialize, serde::Deserialize, IntoParams)]
    struct Filter {
        age: i32,
        status: String,
    }

    #[utoipa::path(path = "/item/{id}/{name}", params(Filter), post)]
    #[allow(unused)]
    async fn post_item(
        _path: Path<(i32, String)>,
        _query: Query<Filter>,
        _body: Json<Item>,
    ) -> Result<Json<Item>, Error> {
        Ok(Json(Item(String::new())))
    }

    #[derive(utoipa::OpenApi)]
    #[openapi(paths(post_item))]
    struct Doc;

    let doc = serde_json::to_value(Doc::openapi()).unwrap();
    let operation = doc.pointer("/paths/~1item~1{id}~1{name}/post").unwrap();

    assert_json_eq!(
        &operation.pointer("/parameters").unwrap(),
        json!([
              {
                  "in": "query",
                  "name": "age",
                  "required": true,
                  "schema": {
                      "format": "int32",
                      "type": "integer"
                  }
              },
              {
                  "in": "query",
                  "name": "status",
                  "required": true,
                  "schema": {
                      "type": "string"
                  }
              },
              {
                  "in": "path",
                  "name": "id",
                  "required": true,
                  "schema": {
                      "format": "int32",
                      "type": "integer"
                  }
              },
              {
                  "in": "path",
                  "name": "name",
                  "required": true,
                  "schema": {
                      "type": "string"
                  }
              }
        ])
    );
    assert_json_eq!(
        &operation.pointer("/requestBody"),
        json!({
            "description": "",
            "content": {
                "application/json": {
                    "schema": {
                        "$ref": "#/components/schemas/Item"
                    }
                }
            },
            "required": true,
        })
    )
}

#[test]
fn test_into_params_for_option_query_type() {
    #[utoipa::path(
        get,
        path = "/items",
        params(("id" = u32, Query, description = "")),
        responses(
            (status = 200, description = "success response")
        )
    )]
    #[allow(unused)]
    async fn get_item(id: Option<Query<u32>>) {}

    #[derive(OpenApi)]
    #[openapi(paths(get_item))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let operation = doc.pointer("/paths/~1items/get").unwrap();

    assert_json_eq!(
        operation.pointer("/parameters"),
        json!([
            {
                "description": "",
                "in": "query",
                "name": "id",
                "required": true,
                "schema": {
                    "format": "int32",
                    "type": "integer",
                    "minimum": 0
                }
            }
        ])
    )
}

#[test]
fn path_param_single_arg_primitive_type() {
    #[utoipa::path(
        get,
        path = "/items/{id}",
        params(("id" = u32, Path, description = "")),
        responses(
            (status = 200, description = "success response")
        )
    )]
    #[allow(unused)]
    async fn get_item(id: Path<u32>) {}

    #[derive(OpenApi)]
    #[openapi(paths(get_item))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let operation = doc.pointer("/paths/~1items~1{id}/get").unwrap();

    assert_json_eq!(
        operation.pointer("/parameters"),
        json!([
            {
                "description": "",
                "in": "path",
                "name": "id",
                "required": true,
                "schema": {
                    "format": "int32",
                    "type": "integer",
                    "minimum": 0
                }
            }
        ])
    )
}

#[test]
fn path_param_single_arg_non_primitive_type() {
    #[derive(utoipa::ToSchema)]
    struct Id(String);

    #[utoipa::path(
        get,
        path = "/items/{id}",
        params(("id" = inline(Id), Path, description = "")),
        responses(
            (status = 200, description = "success response")
        )
    )]
    #[allow(unused)]
    async fn get_item(id: Path<Id>) {}

    #[derive(OpenApi)]
    #[openapi(paths(get_item))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let operation = doc.pointer("/paths/~1items~1{id}/get").unwrap();

    assert_json_eq!(
        operation.pointer("/parameters"),
        json!([
            {
                "description": "",
                "in": "path",
                "name": "id",
                "required": true,
                "schema": {
                    "type": "string",
                }
            }
        ])
    )
}

#[test]
fn path_param_single_arg_non_primitive_type_into_params() {
    #[derive(utoipa::ToSchema, utoipa::IntoParams)]
    #[into_params(names("id"))]
    struct Id(String);

    #[utoipa::path(
        get,
        path = "/items/{id}",
        params(Id),
        responses(
            (status = 200, description = "success response")
        )
    )]
    #[allow(unused)]
    async fn get_item(id: Path<Id>) {}

    #[derive(OpenApi)]
    #[openapi(paths(get_item))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let operation = doc.pointer("/paths/~1items~1{id}/get").unwrap();

    assert_json_eq!(
        operation.pointer("/parameters"),
        json!([
            {
                "in": "path",
                "name": "id",
                "required": true,
                "schema": {
                    "type": "string",
                }
            }
        ])
    )
}
