use serde_json::Value;
use utoipa::OpenApi;

mod common;

mod derive_params_all_options {
    use actix_web::{web, HttpResponse, Responder};
    use serde_json::json;

    /// Get foo by id
    ///
    /// Get foo by id long description
    #[utoipa::path(
        get,
        path = "/foo/{id}",
        responses = [
            (200, "success", String),
        ],
        params = [
            ("id" = i32, path, required, deprecated, description = "Search foos by ids"),
        ]
    )]
    #[allow(unused)]
    async fn get_foo_by_id(web::Path(id): web::Path<i32>) -> impl Responder {
        HttpResponse::Ok().json(json!({ "foo": id }))
    }
}

#[test]
fn derive_path_parameters_with_all_options_success() {
    #[derive(OpenApi, Default)]
    #[openapi(handler_files = [], handlers = [derive_params_all_options::get_foo_by_id])]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = common::get_json_path(&doc, "paths./foo/{id}.get.parameters");

    match parameters {
        Value::Array(array) => assert_eq!(1, array.len()),
        _ => unreachable!(),
    };
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#""Search foos by ids""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"true"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int32""#, "Parameter schema format"
    };
}

mod derive_params_minimal {
    use actix_web::{web, HttpResponse, Responder};
    use serde_json::json;

    /// Get foo by id
    ///
    /// Get foo by id long description
    #[utoipa::path(
        get,
        path = "/foo/{id}",
        responses = [
            (200, "success", String),
        ],
        params = [
            ("id" = i32, description = "Search foos by ids"),
        ]
    )]
    #[allow(unused)]
    async fn get_foo_by_id(web::Path(id): web::Path<i32>) -> impl Responder {
        HttpResponse::Ok().json(json!({ "foo": id }))
    }
}

#[test]
fn derive_path_parameters_minimal_success() {
    #[derive(OpenApi, Default)]
    #[openapi(handler_files = [], handlers = [derive_params_minimal::get_foo_by_id])]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = common::get_json_path(&doc, "paths./foo/{id}.get.parameters");

    match parameters {
        Value::Array(array) => assert_eq!(1, array.len()),
        _ => unreachable!(),
    };
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#""Search foos by ids""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"false"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int32""#, "Parameter schema format"
    };
}

mod derive_params_multiple {
    use actix_web::{web, HttpResponse, Responder};
    use serde_json::json;

    /// Get foo by id
    ///
    /// Get foo by id long description
    #[utoipa::path(
        get,
        path = "/foo/{id}/{digest}",
        responses = [
            (200, "success", String),
        ],
        params = [
            ("id" = i32, description = "Foo id"),
            ("digest" = String, description = "Digest of foo"),
        ]
    )]
    #[allow(unused)]
    async fn get_foo_by_id(web::Path((id, digest)): web::Path<(i32, String)>) -> impl Responder {
        HttpResponse::Ok().json(json!({ "foo": format!("{:?}{:?}", &id, &digest) }))
    }
}

#[test]
fn derive_path_parameter_multiple_success() {
    #[derive(OpenApi, Default)]
    #[openapi(handler_files = [], handlers = [derive_params_multiple::get_foo_by_id])]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = common::get_json_path(&doc, "paths./foo/{id}/{digest}.get.parameters");

    match parameters {
        Value::Array(array) => assert_eq!(
            2,
            array.len(),
            "wrong amount of parameters {} != {}",
            2,
            array.len()
        ),
        _ => unreachable!(),
    };
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#""Foo id""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"false"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int32""#, "Parameter schema format"

        "[1].in" = r#""path""#, "Parameter in"
        "[1].name" = r#""digest""#, "Parameter name"
        "[1].description" = r#""Digest of foo""#, "Parameter description"
        "[1].required" = r#"true"#, "Parameter required"
        "[1].deprecated" = r#"false"#, "Parameter deprecated"
        "[1].schema.type" = r#""string""#, "Parameter schema type"
        "[1].schema.format" = r#"null"#, "Parameter schema format"
    };
}

mod derive_parameters_multiple_no_matching_names {
    use actix_web::{web, HttpResponse, Responder};
    use serde_json::json;

    /// Get foo by id
    ///
    /// Get foo by id long description
    #[utoipa::path(
        get,
        path = "/foo/{id}/{digest}",
        responses = [
            (200, "success", String),
        ],
        params = [
            ("id" = i32, description = "Foo id"),
            ("digest" = String, description = "Digest of foo"),
        ]
    )]
    #[allow(unused)]
    async fn get_foo_by_id(info: web::Path<(i32, String)>) -> impl Responder {
        // is no matching names since the parameter name does not match to amount of types
        HttpResponse::Ok().json(json!({ "foo": format!("{:?}{:?}", &info.0, &info.1) }))
    }
}

#[test]
fn derive_path_parameter_multiple_no_matching_names() {
    #[derive(OpenApi, Default)]
    #[openapi(handler_files = [], handlers = [derive_parameters_multiple_no_matching_names::get_foo_by_id])]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = common::get_json_path(&doc, "paths./foo/{id}/{digest}.get.parameters");

    match parameters {
        Value::Array(array) => assert_eq!(
            2,
            array.len(),
            "wrong amount of parameters {} != {}",
            2,
            array.len()
        ),
        _ => unreachable!(),
    };
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#""Foo id""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"false"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int32""#, "Parameter schema format"

        "[1].in" = r#""path""#, "Parameter in"
        "[1].name" = r#""digest""#, "Parameter name"
        "[1].description" = r#""Digest of foo""#, "Parameter description"
        "[1].required" = r#"true"#, "Parameter required"
        "[1].deprecated" = r#"false"#, "Parameter deprecated"
        "[1].schema.type" = r#""string""#, "Parameter schema type"
        "[1].schema.format" = r#"null"#, "Parameter schema format"
    };
}
