#![cfg(feature = "actix_extras")]

use std::{fmt::Display, future::Ready};

use actix_web::{
    get, post,
    web::{Json, Path, Query},
    FromRequest, ResponseError,
};
use assert_json_diff::assert_json_eq;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::{
    openapi::{
        path::{Parameter, ParameterBuilder, ParameterIn},
        Array, KnownFormat, ObjectBuilder, SchemaFormat,
    },
    IntoParams, OpenApi, ToSchema,
};
use uuid::Uuid;

mod common;

mod mod_derive_path_actix {
    use actix_web::{get, web, HttpResponse, Responder};
    use serde_json::json;

    /// Get foo by id
    ///
    /// Get foo by id long description
    #[utoipa::path(
        responses(
            (status = 200, description = "success response")
        ),
        params(
            ("id", description = "Foo id"),
        )
    )]
    #[get("/foo/{id}")]
    #[allow(unused)]
    async fn get_foo_by_id(id: web::Path<i32>) -> impl Responder {
        HttpResponse::Ok().json(json!({ "foo": format!("{:?}", &id.into_inner()) }))
    }
}

#[test]
fn derive_path_one_value_actix_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(mod_derive_path_actix::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc.pointer("/paths/~1foo~1{id}/get/parameters").unwrap();

    common::assert_json_array_len(parameters, 1);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#""Foo id""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"null"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int32""#, "Parameter schema format"
    };
}

mod mod_derive_path_unnamed_regex_actix {
    use actix_web::{get, web, HttpResponse, Responder};
    use serde_json::json;

    /// Get foo by id
    ///
    /// Get foo by id long description
    #[utoipa::path(
        responses(
            (status = 200, description = "success"),
        ),
        params(
            ("arg0", description = "Foo path unnamed regex tail")
        )
    )]
    #[get("/foo/{_:.*}")]
    #[allow(unused)]
    async fn get_foo_by_id(arg0: web::Path<String>) -> impl Responder {
        HttpResponse::Ok().json(json!({ "foo": &format!("{:?}", arg0.into_inner()) }))
    }
}

#[test]
fn derive_path_with_unnamed_regex_actix_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(mod_derive_path_unnamed_regex_actix::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc.pointer("/paths/~1foo~1{arg0}/get/parameters").unwrap();

    common::assert_json_array_len(parameters, 1);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""arg0""#, "Parameter name"
        "[0].description" = r#""Foo path unnamed regex tail""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"null"#, "Parameter deprecated"
        "[0].schema.type" = r#""string""#, "Parameter schema type"
        "[0].schema.format" = r#"null"#, "Parameter schema format"
    };
}

mod mod_derive_path_named_regex_actix {
    use actix_web::{get, web, HttpResponse, Responder};
    use serde_json::json;

    /// Get foo by id
    ///
    /// Get foo by id long description
    #[utoipa::path(
        responses(
            (status = 200, description = "success response")
        ),
        params(
            ("tail", description = "Foo path named regex tail")
        )
    )]
    #[get("/foo/{tail:.*}")]
    #[allow(unused)]
    async fn get_foo_by_id(tail: web::Path<String>) -> impl Responder {
        HttpResponse::Ok().json(json!({ "foo": &format!("{:?}", tail.into_inner()) }))
    }
}

#[test]
fn derive_path_with_named_regex_actix_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(mod_derive_path_named_regex_actix::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    let parameters = doc.pointer("/paths/~1foo~1{tail}/get/parameters").unwrap();

    common::assert_json_array_len(parameters, 1);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""tail""#, "Parameter name"
        "[0].description" = r#""Foo path named regex tail""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"null"#, "Parameter deprecated"
        "[0].schema.type" = r#""string""#, "Parameter schema type"
        "[0].schema.format" = r#"null"#, "Parameter schema format"
    };
}

#[test]
fn derive_path_with_multiple_args() {
    mod mod_derive_path_multiple_args {
        use actix_web::{get, web, HttpResponse, Responder};
        use serde_json::json;

        #[utoipa::path(
            responses(
                (status = 200, description = "success response")
            ),
        )]
        #[get("/foo/{id}/bar/{digest}")]
        #[allow(unused)]
        async fn get_foo_by_id(path: web::Path<(i64, String)>) -> impl Responder {
            let (id, digest) = path.into_inner();
            HttpResponse::Ok().json(json!({ "id": &format!("{:?} {:?}", id, digest) }))
        }
    }

    #[derive(OpenApi, Default)]
    #[openapi(paths(mod_derive_path_multiple_args::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc
        .pointer("/paths/~1foo~1{id}~1bar~1{digest}/get/parameters")
        .unwrap();

    common::assert_json_array_len(parameters, 2);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#"null"#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"null"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int64""#, "Parameter schema format"

        "[1].in" = r#""path""#, "Parameter in"
        "[1].name" = r#""digest""#, "Parameter name"
        "[1].description" = r#"null"#, "Parameter description"
        "[1].required" = r#"true"#, "Parameter required"
        "[1].deprecated" = r#"null"#, "Parameter deprecated"
        "[1].schema.type" = r#""string""#, "Parameter schema type"
        "[1].schema.format" = r#"null"#, "Parameter schema format"
    };
}

#[test]
fn derive_path_with_dyn_trait_compiles() {
    use actix_web::{get, web, HttpResponse, Responder};
    use serde_json::json;

    trait Store {}

    #[utoipa::path(
        responses(
            (status = 200, description = "success response")
        ),
    )]
    #[get("/foo/{id}/bar/{digest}")]
    #[allow(unused)]
    async fn get_foo_by_id(
        path: web::Path<(i64, String)>,
        data: web::Data<&dyn Store>,
    ) -> impl Responder {
        let (id, digest) = path.into_inner();
        HttpResponse::Ok().json(json!({ "id": &format!("{:?} {:?}", id, digest) }))
    }
}

#[test]
fn derive_complex_actix_web_path() {
    mod mod_derive_complex_actix_path {
        use actix_web::{get, web, HttpResponse, Responder};
        use serde_json::json;

        #[utoipa::path(
            responses(
                (status = 200, description = "success response")
            ),
        )]
        #[get("/foo/{id}", name = "api_name")]
        #[allow(unused)]
        async fn get_foo_by_id(path: web::Path<i64>) -> impl Responder {
            let id = path.into_inner();
            HttpResponse::Ok().json(json!({ "id": &format!("{}", id) }))
        }
    }

    #[derive(OpenApi, Default)]
    #[openapi(paths(mod_derive_complex_actix_path::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc.pointer("/paths/~1foo~1{id}/get/parameters").unwrap();

    common::assert_json_array_len(parameters, 1);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#"null"#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"null"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int64""#, "Parameter schema format"
    };
}

#[test]
fn derive_path_with_multiple_args_with_descriptions() {
    mod mod_derive_path_multiple_args {
        use actix_web::{get, web, HttpResponse, Responder};
        use serde_json::json;

        #[utoipa::path(
            responses(
                (status = 200, description = "success response")
            ),
            params(
                ("id", description = "Foo id"),
                ("digest", description = "Foo digest")
            )
        )]
        #[get("/foo/{id}/bar/{digest}")]
        #[allow(unused)]
        async fn get_foo_by_id(path: web::Path<(i64, String)>) -> impl Responder {
            let (id, digest) = path.into_inner();
            HttpResponse::Ok().json(json!({ "id": &format!("{:?} {:?}", id, digest) }))
        }
    }

    #[derive(OpenApi, Default)]
    #[openapi(paths(mod_derive_path_multiple_args::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc
        .pointer("/paths/~1foo~1{id}~1bar~1{digest}/get/parameters")
        .unwrap();

    common::assert_json_array_len(parameters, 2);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#""Foo id""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"null"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int64""#, "Parameter schema format"

        "[1].in" = r#""path""#, "Parameter in"
        "[1].name" = r#""digest""#, "Parameter name"
        "[1].description" = r#""Foo digest""#, "Parameter description"
        "[1].required" = r#"true"#, "Parameter required"
        "[1].deprecated" = r#"null"#, "Parameter deprecated"
        "[1].schema.type" = r#""string""#, "Parameter schema type"
        "[1].schema.format" = r#"null"#, "Parameter schema format"
    };
}

#[test]
fn derive_path_with_context_path() {
    use actix_web::{get, HttpResponse, Responder};
    use serde_json::json;

    #[utoipa::path(
        context_path = "/api",
        responses(
            (status = 200, description = "success response")
        )
    )]
    #[get("/foo")]
    #[allow(unused)]
    async fn get_foo() -> impl Responder {
        HttpResponse::Ok().json(json!({ "id": "foo" }))
    }

    #[derive(OpenApi, Default)]
    #[openapi(paths(get_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let path = doc.pointer("/paths/~1api~1foo/get").unwrap();

    assert_ne!(path, &Value::Null, "expected path with context path /api");
}

#[test]
fn path_with_struct_variables_with_into_params() {
    use actix_web::{get, HttpResponse, Responder};
    use serde_json::json;

    #[derive(Deserialize)]
    #[allow(unused)]
    struct Person {
        id: i64,
        name: String,
    }

    impl IntoParams for Person {
        fn into_params(
            _: impl Fn() -> Option<utoipa::openapi::path::ParameterIn>,
        ) -> Vec<Parameter> {
            vec![
                ParameterBuilder::new()
                    .name("name")
                    .schema(Some(
                        ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::String),
                    ))
                    .parameter_in(ParameterIn::Path)
                    .build(),
                ParameterBuilder::new()
                    .name("id")
                    .schema(Some(
                        ObjectBuilder::new()
                            .schema_type(utoipa::openapi::SchemaType::Integer)
                            .format(Some(SchemaFormat::KnownFormat(KnownFormat::Int64))),
                    ))
                    .parameter_in(ParameterIn::Path)
                    .build(),
            ]
        }
    }

    #[derive(Deserialize)]
    #[allow(unused)]
    struct Filter {
        age: Vec<String>,
    }

    impl IntoParams for Filter {
        fn into_params(
            _: impl Fn() -> Option<utoipa::openapi::path::ParameterIn>,
        ) -> Vec<Parameter> {
            vec![ParameterBuilder::new()
                .name("age")
                .schema(Some(Array::new(
                    ObjectBuilder::new().schema_type(utoipa::openapi::SchemaType::String),
                )))
                .parameter_in(ParameterIn::Query)
                .build()]
        }
    }

    #[utoipa::path(
        params(
            Person,
            Filter
        ),
        responses(
            (status = 200, description = "success response")
        )
    )]
    #[get("/foo/{id}/{name}")]
    #[allow(unused)]
    async fn get_foo(person: Path<Person>, query: Query<Filter>) -> impl Responder {
        HttpResponse::Ok().json(json!({ "id": "foo" }))
    }

    #[derive(OpenApi, Default)]
    #[openapi(paths(get_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc
        .pointer("/paths/~1foo~1{id}~1{name}/get/parameters")
        .unwrap();

    common::assert_json_array_len(parameters, 3);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""name""#, "Parameter name"
        "[0].required" = r#"false"#, "Parameter required"
        "[0].schema.type" = r#""string""#, "Parameter schema type"
        "[0].schema.format" = r#"null"#, "Parameter schema format"

        "[1].in" = r#""path""#, "Parameter in"
        "[1].name" = r#""id""#, "Parameter name"
        "[1].required" = r#"false"#, "Parameter required"
        "[1].schema.type" = r#""integer""#, "Parameter schema type"
        "[1].schema.format" = r#""int64""#, "Parameter schema format"

        "[2].in" = r#""query""#, "Parameter in"
        "[2].name" = r#""age""#, "Parameter name"
        "[2].required" = r#"false"#, "Parameter required"
        "[2].schema.type" = r#""array""#, "Parameter schema type"
        "[2].schema.items.type" = r#""string""#, "Parameter items schema type"
    }
}

#[test]
fn derive_path_with_struct_variables_with_into_params() {
    use actix_web::{get, HttpResponse, Responder};
    use serde_json::json;

    #[derive(Deserialize, IntoParams)]
    #[allow(unused)]
    struct Person {
        /// Id of person
        id: i64,
        /// Name of person
        name: String,
    }

    #[derive(Deserialize, IntoParams)]
    #[allow(unused)]
    struct Filter {
        /// Age filter for user
        #[deprecated]
        age: Option<Vec<String>>,
    }

    #[utoipa::path(
        params(
            Person,
            Filter
        ),
        responses(
            (status = 200, description = "success response")
        )
    )]
    #[get("/foo/{id}/{name}")]
    #[allow(unused)]
    async fn get_foo(person: Path<Person>, query: Query<Filter>) -> impl Responder {
        HttpResponse::Ok().json(json!({ "id": "foo" }))
    }

    #[derive(OpenApi, Default)]
    #[openapi(paths(get_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc
        .pointer("/paths/~1foo~1{id}~1{name}/get/parameters")
        .unwrap();

    common::assert_json_array_len(parameters, 3);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#""Id of person""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"null"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int64""#, "Parameter schema format"

        "[1].in" = r#""path""#, "Parameter in"
        "[1].name" = r#""name""#, "Parameter name"
        "[1].description" = r#""Name of person""#, "Parameter description"
        "[1].required" = r#"true"#, "Parameter required"
        "[1].deprecated" = r#"null"#, "Parameter deprecated"
        "[1].schema.type" = r#""string""#, "Parameter schema type"
        "[1].schema.format" = r#"null"#, "Parameter schema format"

        "[2].in" = r#""query""#, "Parameter in"
        "[2].name" = r#""age""#, "Parameter name"
        "[2].description" = r#""Age filter for user""#, "Parameter description"
        "[2].required" = r#"false"#, "Parameter required"
        "[2].deprecated" = r#"true"#, "Parameter deprecated"
        "[2].schema.type" = r#""array""#, "Parameter schema type"
        "[2].schema.items.type" = r#""string""#, "Parameter items schema type"
    }
}

#[test]
fn derive_path_with_multiple_instances_same_path_params() {
    use actix_web::{delete, get, HttpResponse, Responder};
    use serde_json::json;

    #[derive(Deserialize, Serialize, ToSchema, IntoParams)]
    #[into_params(names("id"))]
    struct Id(u64);

    #[utoipa::path(
        params(
            Id
        ),
        responses(
            (status = 200, description = "success response")
        )
    )]
    #[get("/foo/{id}")]
    #[allow(unused)]
    async fn get_foo(id: Path<Id>) -> impl Responder {
        HttpResponse::Ok().json(json!({ "id": "foo" }))
    }

    #[utoipa::path(
        params(
            Id
        ),
        responses(
            (status = 200, description = "success response")
        )
    )]
    #[delete("/foo/{id}")]
    #[allow(unused)]
    async fn delete_foo(id: Path<Id>) -> impl Responder {
        HttpResponse::Ok().json(json!({ "id": "foo" }))
    }

    #[derive(OpenApi, Default)]
    #[openapi(paths(get_foo, delete_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    for operation in ["get", "delete"] {
        let parameters = doc
            .pointer(&format!("/paths/~1foo~1{{id}}/{operation}/parameters"))
            .unwrap();

        common::assert_json_array_len(parameters, 1);
        assert_value! {parameters=>
            "[0].in" = r#""path""#, "Parameter in"
            "[0].name" = r#""id""#, "Parameter name"
            "[0].required" = r#"true"#, "Parameter required"
            "[0].deprecated" = r#"null"#, "Parameter deprecated"
            "[0].schema.type" = r#""integer""#, "Parameter schema type"
            "[0].schema.format" = r#""int64""#, "Parameter schema format"
        }
    }
}

#[test]
fn derive_path_with_multiple_into_params_names() {
    use actix_web::{get, HttpResponse, Responder};

    #[derive(Deserialize, Serialize, IntoParams)]
    #[into_params(names("id", "name"))]
    struct IdAndName(u64, String);

    #[utoipa::path(
        params(IdAndName),
        responses(
            (status = 200, description = "success response")
        )
    )]
    #[get("/foo/{id}/{name}")]
    #[allow(unused)]
    async fn get_foo(path: Path<IdAndName>) -> impl Responder {
        HttpResponse::Ok()
    }

    #[derive(OpenApi, Default)]
    #[openapi(paths(get_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    let parameters = doc
        .pointer("/paths/~1foo~1{id}~1{name}/get/parameters")
        .unwrap();

    common::assert_json_array_len(parameters, 2);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"null"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int64""#, "Parameter schema format"

        "[1].in" = r#""path""#, "Parameter in"
        "[1].name" = r#""name""#, "Parameter name"
        "[1].required" = r#"true"#, "Parameter required"
        "[1].deprecated" = r#"null"#, "Parameter deprecated"
        "[1].schema.type" = r#""string""#, "Parameter schema type"
        "[1].schema.format" = r#"null"#, "Parameter schema format"
    }
}

#[test]
fn derive_into_params_with_custom_attributes() {
    use actix_web::{get, HttpResponse, Responder};
    use serde_json::json;

    #[derive(Deserialize, IntoParams)]
    #[allow(unused)]
    struct Person {
        /// Id of person
        id: i64,
        /// Name of person
        #[param(style = Simple, example = "John")]
        name: String,
    }

    #[derive(Deserialize, IntoParams)]
    #[allow(unused)]
    struct Filter {
        /// Age filter for user
        #[param(style = Form, explode, allow_reserved, example = json!(["10"]))]
        age: Option<Vec<String>>,
        sort: Sort,
    }

    #[derive(Deserialize, ToSchema)]
    enum Sort {
        Asc,
        Desc,
    }

    #[utoipa::path(
        params(
            Person,
            Filter
        ),
        responses(
            (status = 200, description = "success response")
        )
    )]
    #[get("/foo/{id}/{name}")]
    #[allow(unused)]
    async fn get_foo(person: Path<Person>, query: Query<Filter>) -> impl Responder {
        HttpResponse::Ok().json(json!({ "id": "foo" }))
    }

    #[derive(OpenApi, Default)]
    #[openapi(paths(get_foo), components(schemas(Sort)))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc
        .pointer("/paths/~1foo~1{id}~1{name}/get/parameters")
        .unwrap();

    common::assert_json_array_len(parameters, 4);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#""Id of person""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"null"#, "Parameter deprecated"
        "[0].style" = r#"null"#, "Parameter style"
        "[0].example" = r#"null"#, "Parameter example"
        "[0].allowReserved" = r#"null"#, "Parameter allowReserved"
        "[0].explode" = r#"null"#, "Parameter explode"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int64""#, "Parameter schema format"

        "[1].in" = r#""path""#, "Parameter in"
        "[1].name" = r#""name""#, "Parameter name"
        "[1].description" = r#""Name of person""#, "Parameter description"
        "[1].required" = r#"true"#, "Parameter required"
        "[1].deprecated" = r#"null"#, "Parameter deprecated"
        "[1].style" = r#""simple""#, "Parameter style"
        "[1].allowReserved" = r#"null"#, "Parameter allowReserved"
        "[1].explode" = r#"null"#, "Parameter explode"
        "[1].example" = r#""John""#, "Parameter example"
        "[1].schema.type" = r#""string""#, "Parameter schema type"
        "[1].schema.format" = r#"null"#, "Parameter schema format"

        "[2].in" = r#""query""#, "Parameter in"
        "[2].name" = r#""age""#, "Parameter name"
        "[2].description" = r#""Age filter for user""#, "Parameter description"
        "[2].required" = r#"false"#, "Parameter required"
        "[2].deprecated" = r#"null"#, "Parameter deprecated"
        "[2].style" = r#""form""#, "Parameter style"
        "[2].example" = r#"["10"]"#, "Parameter example"
        "[2].allowReserved" = r#"true"#, "Parameter allowReserved"
        "[2].explode" = r#"true"#, "Parameter explode"
        "[2].schema.type" = r#""array""#, "Parameter schema type"
        "[2].schema.items.type" = r#""string""#, "Parameter items schema type"

        "[3].in" = r#""query""#, "Parameter in"
        "[3].name" = r#""sort""#, "Parameter name"
        "[3].description" = r#"null"#, "Parameter description"
        "[3].required" = r#"true"#, "Parameter required"
        "[3].deprecated" = r#"null"#, "Parameter deprecated"
        "[3].schema.$ref" = r###""#/components/schemas/Sort""###, "Parameter schema type"
    }
}

#[test]
fn derive_into_params_in_another_module() {
    use actix_web::{get, HttpResponse, Responder};
    use utoipa::OpenApi;
    pub mod params {
        use serde::Deserialize;
        use utoipa::IntoParams;

        #[derive(Deserialize, IntoParams)]
        pub struct FooParams {
            pub id: String,
        }
    }

    /// Foo test
    #[utoipa::path(
        params(
            params::FooParams,
        ),
        responses(
            (status = 200, description = "Todo foo operation success"),
        )
    )]
    #[get("/todo/foo/{id}")]
    pub async fn foo_todos(_path: Path<params::FooParams>) -> impl Responder {
        HttpResponse::Ok()
    }

    #[derive(OpenApi, Default)]
    #[openapi(paths(foo_todos))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc
        .pointer("/paths/~1todo~1foo~1{id}/get/parameters")
        .unwrap();

    common::assert_json_array_len(parameters, 1);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
    }
}

#[test]
fn path_with_all_args() {
    #[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
    struct Item(String);

    /// Error
    #[derive(Debug)]
    struct Error;

    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Error")
        }
    }

    impl ResponseError for Error {}

    #[derive(serde::Serialize, serde::Deserialize, IntoParams)]
    struct Filter {
        age: i32,
        status: String,
    }

    // NOTE! temporarily disable automatic parameter recognition
    #[utoipa::path(params(Filter))]
    #[post("/item/{id}/{name}")]
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
fn path_with_all_args_using_uuid() {
    #[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
    struct Item(String);

    /// Error
    #[derive(Debug)]
    struct Error;

    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Error")
        }
    }

    impl ResponseError for Error {}

    #[utoipa::path]
    #[post("/item/{uuid}")]
    async fn post_item(_path: Path<uuid::Uuid>, _body: Json<Item>) -> Result<Json<Item>, Error> {
        Ok(Json(Item(String::new())))
    }

    #[derive(utoipa::OpenApi)]
    #[openapi(paths(post_item))]
    struct Doc;

    let doc = serde_json::to_value(Doc::openapi()).unwrap();
    let operation = doc.pointer("/paths/~1item~1{uuid}/post").unwrap();

    assert_json_eq!(
        &operation.pointer("/parameters").unwrap(),
        json!([
              {
                  "in": "path",
                  "name": "uuid",
                  "required": true,
                  "schema": {
                      "format": "uuid",
                      "type": "string"
                  }
              },
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
fn path_with_all_args_using_custom_uuid() {
    #[derive(utoipa::ToSchema, serde::Serialize, serde::Deserialize)]
    struct Item(String);

    /// Error
    #[derive(Debug)]
    struct Error;

    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Error")
        }
    }

    impl ResponseError for Error {}

    #[derive(Serialize, Deserialize, IntoParams)]
    #[into_params(names("custom_uuid"))]
    struct Id(Uuid);

    impl FromRequest for Id {
        type Error = Error;

        type Future = Ready<Result<Self, Self::Error>>;

        fn from_request(
            _: &actix_web::HttpRequest,
            _: &mut actix_web::dev::Payload,
        ) -> Self::Future {
            todo!()
        }
    }

    // NOTE! temporarily disable automatic parameter recognition
    #[utoipa::path(params(Id))]
    #[post("/item/{custom_uuid}")]
    async fn post_item(_path: Path<Id>, _body: Json<Item>) -> Result<Json<Item>, Error> {
        Ok(Json(Item(String::new())))
    }

    #[derive(utoipa::OpenApi)]
    #[openapi(paths(post_item))]
    struct Doc;

    let doc = serde_json::to_value(Doc::openapi()).unwrap();
    let operation = doc.pointer("/paths/~1item~1{custom_uuid}/post").unwrap();

    assert_json_eq!(
        &operation.pointer("/parameters").unwrap(),
        json!([
              {
                  "in": "path",
                  "name": "custom_uuid",
                  "required": true,
                  "schema": {
                      "format": "uuid",
                      "type": "string"
                  }
              },
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

macro_rules! test_derive_path_operations {
    ( $( $name:ident, $mod:ident: $operation:ident)* ) => {
        $(
           mod $mod {
            use actix_web::{$operation, HttpResponse, Responder};
            use serde_json::json;

            #[utoipa::path(
                responses(
                    (status = 200, description = "success response")
                )
            )]
            #[$operation("/foo")]
            #[allow(unused)]
            async fn test_operation() -> impl Responder {
                HttpResponse::Ok().json(json!({ "foo": "".to_string() }))
            }
        }

        #[test]
        fn $name() {
            #[derive(OpenApi, Default)]
            #[openapi(paths($mod::test_operation))]
            struct ApiDoc;

            let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

            let op_str = stringify!($operation);
            let path = format!("/paths/~1foo/{}", op_str);
            let value = doc.pointer(&path).unwrap_or(&serde_json::Value::Null);
            assert!(value != &Value::Null, "expected to find operation with: {}", path);
        }
        )*
    };
}

test_derive_path_operations! {
    derive_path_operation_post, mod_test_post: post
    derive_path_operation_get, mod_test_get: get
    derive_path_operation_delete, mod_test_delete: delete
    derive_path_operation_put, mod_test_put: put
    derive_path_operation_head, mod_test_head: head
    derive_path_operation_connect, mod_test_connect: connect
    derive_path_operation_options, mod_test_options: options
    derive_path_operation_trace, mod_test_trace: trace
    derive_path_operation_patch, mod_test_patch: patch
}
