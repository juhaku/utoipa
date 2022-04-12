#![cfg(feature = "actix_extras")]
#![cfg(feature = "serde_json")]

use serde_json::Value;
use utoipa::OpenApi;

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
    #[openapi(handlers(mod_derive_path_actix::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = common::get_json_path(&doc, "paths./foo/{id}.get.parameters");

    common::assert_json_array_len(parameters, 1);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#""Foo id""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"false"#, "Parameter deprecated"
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
    #[openapi(handlers(mod_derive_path_unnamed_regex_actix::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = common::get_json_path(&doc, "paths./foo/{arg0}.get.parameters");

    common::assert_json_array_len(parameters, 1);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""arg0""#, "Parameter name"
        "[0].description" = r#""Foo path unnamed regex tail""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"false"#, "Parameter deprecated"
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
    #[openapi(handlers(mod_derive_path_named_regex_actix::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    let parameters = common::get_json_path(&doc, "paths./foo/{tail}.get.parameters");

    common::assert_json_array_len(parameters, 1);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""tail""#, "Parameter name"
        "[0].description" = r#""Foo path named regex tail""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"false"#, "Parameter deprecated"
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
    #[openapi(handlers(mod_derive_path_multiple_args::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = common::get_json_path(&doc, "paths./foo/{id}/bar/{digest}.get.parameters");

    common::assert_json_array_len(parameters, 2);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#"null"#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"false"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int64""#, "Parameter schema format"

        "[1].in" = r#""path""#, "Parameter in"
        "[1].name" = r#""digest""#, "Parameter name"
        "[1].description" = r#"null"#, "Parameter description"
        "[1].required" = r#"true"#, "Parameter required"
        "[1].deprecated" = r#"false"#, "Parameter deprecated"
        "[1].schema.type" = r#""string""#, "Parameter schema type"
        "[1].schema.format" = r#"null"#, "Parameter schema format"
    };
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
    #[openapi(handlers(mod_derive_complex_actix_path::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    dbg!(&doc);
    let parameters = common::get_json_path(&doc, "paths./foo/{id}.get.parameters");

    common::assert_json_array_len(parameters, 1);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#"null"#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"false"#, "Parameter deprecated"
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
    #[openapi(handlers(mod_derive_path_multiple_args::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = common::get_json_path(&doc, "paths./foo/{id}/bar/{digest}.get.parameters");

    common::assert_json_array_len(parameters, 2);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#""Foo id""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"false"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int64""#, "Parameter schema format"

        "[1].in" = r#""path""#, "Parameter in"
        "[1].name" = r#""digest""#, "Parameter name"
        "[1].description" = r#""Foo digest""#, "Parameter description"
        "[1].required" = r#"true"#, "Parameter required"
        "[1].deprecated" = r#"false"#, "Parameter deprecated"
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
    #[openapi(handlers(get_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let path = common::get_json_path(&doc, "paths./api/foo.get");

    assert_ne!(path, &Value::Null, "expected path with context path /api");
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
            #[openapi(handlers($mod::test_operation))]
            struct ApiDoc;

            let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

            let op_str = stringify!($operation);
            let path = format!("paths./foo.{}", op_str);
            let value = common::get_json_path(&doc, &path);
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
