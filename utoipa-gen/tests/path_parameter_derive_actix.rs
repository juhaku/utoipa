#![cfg(feature = "actix_extras")]

use utoipa::OpenApi;

mod common;

mod derive_params_multiple_actix {
    use actix_web::{web, HttpResponse, Responder};
    use serde_json::json;

    /// Get foo by id
    ///
    /// Get foo by id long description
    #[utoipa::path(
        get,
        path = "/foo/{id}/{digest}",
        responses(
            (status = 200, description = "success response")
        ),
        params(
            ("id", description = "Foo id"),
            ("digest", description = "Digest of foo"),
        )
    )]
    #[allow(unused)]
    async fn get_foo_by_id(path: web::Path<(i32, String)>) -> impl Responder {
        let (id, digest) = path.into_inner();
        HttpResponse::Ok().json(json!({ "foo": format!("{:?}{:?}", &id, &digest) }))
    }
}

#[test]
fn derive_path_parameter_multiple_with_matching_names_and_types_actix_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_params_multiple_actix::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc
        .pointer("/paths/~1foo~1{id}~1{digest}/get/parameters")
        .unwrap();

    common::assert_json_array_len(parameters, 2);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#""Foo id""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"null"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int32""#, "Parameter schema format"

        "[1].in" = r#""path""#, "Parameter in"
        "[1].name" = r#""digest""#, "Parameter name"
        "[1].description" = r#""Digest of foo""#, "Parameter description"
        "[1].required" = r#"true"#, "Parameter required"
        "[1].deprecated" = r#"null"#, "Parameter deprecated"
        "[1].schema.type" = r#""string""#, "Parameter schema type"
        "[1].schema.format" = r#"null"#, "Parameter schema format"
    };
}

mod derive_parameters_multiple_no_matching_names_actix {
    use actix_web::{web, HttpResponse, Responder};
    use serde_json::json;

    /// Get foo by id
    ///
    /// Get foo by id long description
    #[utoipa::path(
        get,
        path = "/foo/{id}/{digest}",
        responses(
            (status = 200, description = "success response")
        ),
        params(
            ("id" = i32, description = "Foo id"),
            ("digest" = String, description = "Digest of foo"),
        )
    )]
    #[allow(unused)]
    async fn get_foo_by_id(info: web::Path<(i32, String)>) -> impl Responder {
        // is no matching names since the parameter name does not match to amount of types
        HttpResponse::Ok().json(json!({ "foo": format!("{:?}{:?}", &info.0, &info.1) }))
    }
}

#[test]
fn derive_path_parameter_multiple_no_matching_names_actix_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_parameters_multiple_no_matching_names_actix::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc
        .pointer("/paths/~1foo~1{id}~1{digest}/get/parameters")
        .unwrap();

    common::assert_json_array_len(parameters, 2);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#""Foo id""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"null"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int32""#, "Parameter schema format"

        "[1].in" = r#""path""#, "Parameter in"
        "[1].name" = r#""digest""#, "Parameter name"
        "[1].description" = r#""Digest of foo""#, "Parameter description"
        "[1].required" = r#"true"#, "Parameter required"
        "[1].deprecated" = r#"null"#, "Parameter deprecated"
        "[1].schema.type" = r#""string""#, "Parameter schema type"
        "[1].schema.format" = r#"null"#, "Parameter schema format"
    };
}

mod derive_params_from_method_args_actix {
    use actix_web::{web, HttpResponse, Responder};
    use serde_json::json;

    #[utoipa::path(
        get,
        path = "/foo/{id}/{digest}",
        responses(
            (status = 200, description = "success response")
        ),
    )]
    #[allow(unused)]
    async fn get_foo_by_id(path: web::Path<(i32, String)>) -> impl Responder {
        let (id, digest) = path.into_inner();
        HttpResponse::Ok().json(json!({ "foo": format!("{:?}{:?}", &id, &digest) }))
    }
}

#[test]
fn derive_params_from_method_args_actix_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_params_from_method_args_actix::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc
        .pointer("/paths/~1foo~1{id}~1{digest}/get/parameters")
        .unwrap();

    common::assert_json_array_len(parameters, 2);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#"null"#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"null"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int32""#, "Parameter schema format"

        "[1].in" = r#""path""#, "Parameter in"
        "[1].name" = r#""digest""#, "Parameter name"
        "[1].description" = r#"null"#, "Parameter description"
        "[1].required" = r#"true"#, "Parameter required"
        "[1].deprecated" = r#"null"#, "Parameter deprecated"
        "[1].schema.type" = r#""string""#, "Parameter schema type"
        "[1].schema.format" = r#"null"#, "Parameter schema format"
    };
}
