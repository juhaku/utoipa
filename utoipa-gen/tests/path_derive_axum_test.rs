#![cfg(feature = "axum_extras")]

use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use insta::assert_json_snapshot;
use serde::Deserialize;
use utoipa::{IntoParams, OpenApi};

mod common;

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

    assert_json_snapshot!(parameters);
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

    assert_json_snapshot!(parameters);
}

#[test]
fn get_todo_with_extension() {
    #[derive(utoipa::ToSchema)]
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
    async fn list_todos(Extension(store): Extension<Arc<Store>>) {}

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
    #[allow(dead_code)]
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

    assert_json_snapshot!(parameters);
}

#[test]
fn derive_path_params_with_ignored_parameter() {
    struct Auth;
    #[derive(Deserialize, IntoParams)]
    #[into_params(names("id", "name"))]
    #[allow(dead_code)]
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

    assert_json_snapshot!(parameters);
}

#[test]
fn derive_path_params_with_unnamed_struct_destructed() {
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

    assert_json_snapshot!(parameters)
}

#[test]
fn derive_path_query_params_with_named_struct_destructed() {
    #[derive(IntoParams)]
    #[allow(unused)]
    struct QueryParams<'q> {
        name: &'q str,
    }

    #[utoipa::path(get, path = "/item", params(QueryParams))]
    #[allow(unused)]
    async fn get_item(Query(QueryParams { name }): Query<QueryParams<'static>>) {}

    #[derive(OpenApi)]
    #[openapi(paths(get_item))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc.pointer("/paths/~1item/get/parameters").unwrap();

    assert_json_snapshot!(parameters);
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

    assert_json_snapshot!(&operation.pointer("/parameters").unwrap());
    assert_json_snapshot!(&operation.pointer("/requestBody"));
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

    assert_json_snapshot!(operation.pointer("/parameters"));
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

    assert_json_snapshot!(operation.pointer("/parameters"));
}

#[test]
fn path_param_single_arg_non_primitive_type() {
    #[derive(utoipa::ToSchema)]
    #[allow(dead_code)]
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

    assert_json_snapshot!(operation.pointer("/parameters"));
}

#[test]
fn path_param_single_arg_non_primitive_type_into_params() {
    #[derive(utoipa::ToSchema, utoipa::IntoParams)]
    #[into_params(names("id"))]
    #[allow(dead_code)]
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

    assert_json_snapshot!(operation.pointer("/parameters"));
}

#[test]
fn derive_path_with_validation_attributes_axum() {
    #[derive(IntoParams)]
    #[allow(dead_code)]
    struct Params {
        #[param(maximum = 10, minimum = 5, multiple_of = 2.5)]
        id: i32,

        #[param(max_length = 10, min_length = 5, pattern = "[a-z]*")]
        value: String,

        #[param(max_items = 5, min_items = 1)]
        items: Vec<String>,
    }

    #[utoipa::path(
        get,
        path = "foo/{foo_id}",
        responses(
            (status = 200, description = "success response")
        ),
        params(
            ("foo_id" = String, min_length = 1, description = "Id of Foo to get"),
            Params,
            ("name" = Option<String>, description = "Foo name", min_length = 3),
            ("nonnullable" = String, description = "Foo nonnullable", min_length = 3, max_length = 10),
            ("namequery" = Option<String>, Query, description = "Foo name", min_length = 3),
            ("nonnullablequery" = String, Query, description = "Foo nonnullable", min_length = 3, max_length = 10),
        )
    )]
    #[allow(unused)]
    async fn get_foo(path: Path<String>, query: Query<Params>) {}

    #[derive(OpenApi, Default)]
    #[openapi(paths(get_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc.pointer("/paths/foo~1{foo_id}/get/parameters").unwrap();

    assert_json_snapshot!(parameters);
}

#[test]
fn path_derive_inline_with_tuple() {
    #[derive(utoipa::ToSchema)]
    #[allow(unused)]
    pub enum ResourceType {
        Type1,
        Type2,
    }

    #[utoipa::path(
        get,
        path = "/test_2params_separated/{resource_type}/{id}",
        params(
            ("resource_type" = inline(ResourceType), Path),
            ("id" = String, Path)
        )
    )]
    #[allow(unused)]
    pub async fn inline_tuple(
        Path((resource_type, id)): axum::extract::Path<(ResourceType, String)>,
    ) {
    }

    use utoipa::Path;
    let value = __path_inline_tuple::operation();
    let value = serde_json::to_value(value).expect("operation should serialize to json");

    assert_json_snapshot!(value);
}

#[test]
fn derive_into_params_with_extensions() {
    #[derive(IntoParams)]
    #[into_params(extensions(("x-some-ext" = json!(true))))]
    #[allow(unused)]
    struct Person {
        /// Name of person
        name: String,
        /// City of residence
        #[param(extensions(("x-other-ext" = json!(1))))]
        city: Option<String>,
    }

    /// Get person by id
    #[utoipa::path(
        get,
        path = "/person",
        params(
            Person
        ),
        responses(
            (status = 200, description = "success response")
        )
    )]
    async fn get_person(person: Query<Person>) {}

    #[derive(OpenApi, Default)]
    #[openapi(paths(get_person))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc.pointer("/paths/~1person/get/parameters").unwrap();

    common::assert_json_array_len(parameters, 2);
    assert_value! {parameters=>
        "[0].in" = r#""query""#, "Parameter in"
        "[0].name" = r#""name""#, "Parameter name"
        "[0].description" = r#""Name of person""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].schema.type" = r#""string""#, "Parameter schema type"
        "[0].schema.format" = r#"null"#, "Parameter schema format"
        "[0].x-some-ext" = r#"true"#, "Parameter x-some-ext"

        "[1].in" = r#""query""#, "Parameter in"
        "[1].name" = r#""city""#, "Parameter name"
        "[1].description" = r#""City of residence""#, "Parameter description"
        "[1].required" = r#"false"#, "Parameter required"
        "[1].schema.type" = r#""string""#, "Parameter schema type"
        "[1].schema.format" = r#"null"#, "Parameter schema format"
        "[1].x-some-ext" = r#"true"#, "Parameter x-some-ext"
        "[1].x-other-ext" = r#"1"#, "Parameter x-other-ext"
    };
}
