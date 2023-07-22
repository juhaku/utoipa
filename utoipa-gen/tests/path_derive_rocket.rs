#![cfg(feature = "rocket_extras")]

use std::io::Error;

use assert_json_diff::assert_json_eq;
use rocket::post;
use rocket::request::FromParam;
use rocket::serde::json::Json;
use serde_json::{json, Value};
use utoipa::openapi::path::ParameterBuilder;
use utoipa::{IntoParams, OpenApi, ToSchema};
use utoipa_gen::schema;

mod common;

#[test]
fn resolve_route_with_simple_url() {
    mod rocket_route_operation {
        use rocket::route;

        #[utoipa::path(responses(
                (status = 200, description = "Hello from server")
            ))]
        #[route(GET, uri = "/hello")]
        #[allow(unused)]
        fn hello() -> String {
            "Hello".to_string()
        }
    }

    #[derive(OpenApi)]
    #[openapi(paths(rocket_route_operation::hello))]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();
    let value = &serde_json::to_value(&openapi).unwrap();
    let operation = value.pointer("/paths/~1hello/get").unwrap();

    assert_ne!(operation, &Value::Null, "expected paths.hello.get not null");
}

#[test]
fn resolve_get_with_multiple_args() {
    mod rocket_get_operation {
        use rocket::get;

        #[utoipa::path(responses(
                (status = 200, description = "Hello from server")
            ))]
        #[get("/hello/<id>/<name>?<colors>")]
        #[allow(unused)]
        fn hello(id: i32, name: &str, colors: Vec<&str>) -> String {
            "Hello".to_string()
        }
    }

    #[derive(OpenApi)]
    #[openapi(paths(rocket_get_operation::hello))]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();
    let value = &serde_json::to_value(&openapi).unwrap();
    let parameters = value
        .pointer("/paths/~1hello~1{id}~1{name}/get/parameters")
        .unwrap();

    common::assert_json_array_len(parameters, 3);
    assert_ne!(
        parameters,
        &Value::Null,
        "expected paths.hello.{{id}}.name.get.parameters not null"
    );
    assert_value! {parameters=>
        "[0].schema.type" = r#""array""#, "Query parameter type"
        "[0].schema.format" = r#"null"#, "Query parameter format"
        "[0].schema.items.type" = r#""string""#, "Query items parameter type"
        "[0].schema.items.format" = r#"null"#, "Query items parameter format"
        "[0].name" = r#""colors""#, "Query parameter name"
        "[0].required" = r#"true"#, "Query parameter required"
        "[0].deprecated" = r#"null"#, "Query parameter required"
        "[0].in" = r#""query""#, "Query parameter in"

        "[1].schema.type" = r#""integer""#, "Id parameter type"
        "[1].schema.format" = r#""int32""#, "Id parameter format"
        "[1].name" = r#""id""#, "Id parameter name"
        "[1].required" = r#"true"#, "Id parameter required"
        "[1].deprecated" = r#"null"#, "Id parameter required"
        "[1].in" = r#""path""#, "Id parameter in"

        "[2].schema.type" = r#""string""#, "Name parameter type"
        "[2].schema.format" = r#"null"#, "Name parameter format"
        "[2].name" = r#""name""#, "Name parameter name"
        "[2].required" = r#"true"#, "Name parameter required"
        "[2].deprecated" = r#"null"#, "Name parameter required"
        "[2].in" = r#""path""#, "Name parameter in"
    }
}

#[test]
fn resolve_get_with_optional_query_args() {
    mod rocket_get_operation {
        use rocket::get;

        #[utoipa::path(responses(
                (status = 200, description = "Hello from server")
            ))]
        #[get("/hello?<colors>")]
        #[allow(unused)]
        fn hello(colors: Option<Vec<&str>>) -> String {
            "Hello".to_string()
        }
    }

    #[derive(OpenApi)]
    #[openapi(paths(rocket_get_operation::hello))]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();
    let value = &serde_json::to_value(&openapi).unwrap();
    let parameters = value.pointer("/paths/~1hello/get/parameters").unwrap();

    common::assert_json_array_len(parameters, 1);
    assert_ne!(
        parameters,
        &Value::Null,
        "expected paths.hello.get.parameters not null"
    );

    assert_json_eq!(
        parameters,
        json!([
            {
                "in": "query",
                "name": "colors",
                "required": false,
                "schema": {
                    "items": {
                        "type": "string",
                    },
                    "type": "array",
                    "nullable": true,
                }
            }
        ])
    );
}

#[test]
fn resolve_path_arguments_not_same_order() {
    mod rocket_get_operation {
        use rocket::get;

        #[utoipa::path(responses(
                (status = 200, description = "Hello from server")
            ))]
        #[get("/hello/<id>/<name>")]
        #[allow(unused)]
        fn hello(name: &str, id: i64) -> String {
            "Hello".to_string()
        }
    }

    #[derive(OpenApi)]
    #[openapi(paths(rocket_get_operation::hello))]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();
    let value = &serde_json::to_value(&openapi).unwrap();
    let parameters = value
        .pointer("/paths/~1hello~1{id}~1{name}/get/parameters")
        .unwrap();

    common::assert_json_array_len(parameters, 2);
    assert_ne!(
        parameters,
        &Value::Null,
        r"expected paths.hello/{{id}}/{{name}}.get.parameters not null"
    );

    assert_value! {parameters=>
        "[0].schema.type" = r#""integer""#, "Id parameter type"
        "[0].schema.format" = r#""int64""#, "Id parameter format"
        "[0].name" = r#""id""#, "Id parameter name"
        "[0].required" = r#"true"#, "Id parameter required"
        "[0].deprecated" = r#"null"#, "Id parameter required"
        "[0].in" = r#""path""#, "Id parameter in"

        "[1].schema.type" = r#""string""#, "Name parameter type"
        "[1].schema.format" = r#"null"#, "Name parameter format"
        "[1].name" = r#""name""#, "Name parameter name"
        "[1].required" = r#"true"#, "Name parameter required"
        "[1].deprecated" = r#"null"#, "Name parameter required"
        "[1].in" = r#""path""#, "Name parameter in"
    }
}

#[test]
fn resolve_get_path_with_anonymous_parts() {
    mod rocket_get_operation {
        use rocket::get;

        #[utoipa::path(responses(
                (status = 200, description = "Hello from server")
            ))]
        #[get("/hello/<_>/<_>/<id>")]
        #[allow(unused)]
        fn hello(id: i64) -> String {
            "Hello".to_string()
        }
    }

    #[derive(OpenApi)]
    #[openapi(paths(rocket_get_operation::hello))]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();
    let value = &serde_json::to_value(&openapi).unwrap();
    let parameters = value
        .pointer("/paths/~1hello~1{arg0}~1{arg1}~1{id}/get/parameters")
        .unwrap();

    common::assert_json_array_len(parameters, 3);
    assert_ne!(
        parameters,
        &Value::Null,
        r"expected paths.hello/{{arg0}}/{{arg1}}/{{id}}.get.parameters not null"
    );

    assert_value! {parameters=>
        "[0].schema.type" = r#""integer""#, "Id parameter type"
        "[0].schema.format" = r#""int64""#, "Id parameter format"
        "[0].name" = r#""id""#, "Id parameter name"
        "[0].required" = r#"true"#, "Id parameter required"
        "[0].deprecated" = r#"null"#, "Id parameter required"
        "[0].in" = r#""path""#, "Id parameter in"

        "[1].schema.type" = r#"null"#, "Arg0 parameter type"
        "[1].schema.format" = r#"null"#, "Arg0 parameter format"
        "[1].name" = r#""arg0""#, "Arg0 parameter name"
        "[1].required" = r#"true"#, "Arg0 parameter required"
        "[1].deprecated" = r#"null"#, "Arg0 parameter required"
        "[1].in" = r#""path""#, "Arg0 parameter in"

        "[2].schema.type" = r#"null"#, "Arg1 parameter type"
        "[2].schema.format" = r#"null"#, "Arg1 parameter format"
        "[2].name" = r#""arg1""#, "Arg1 parameter name"
        "[2].required" = r#"true"#, "Arg1 parameter required"
        "[2].deprecated" = r#"null"#, "Arg1 parameter required"
        "[2].in" = r#""path""#, "Arg1 parameter in"
    }
}

#[test]
fn resolve_get_path_with_tail() {
    mod rocket_get_operation {
        use std::path::PathBuf;

        use rocket::get;

        #[utoipa::path(responses(
                (status = 200, description = "Hello from server")
            ))]
        #[get("/hello/<tail..>")]
        #[allow(unused)]
        fn hello(tail: PathBuf) -> String {
            "Hello".to_string()
        }
    }

    #[derive(OpenApi)]
    #[openapi(paths(rocket_get_operation::hello))]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();
    let value = &serde_json::to_value(&openapi).unwrap();
    let parameters = value
        .pointer("/paths/~1hello~1{tail}/get/parameters")
        .unwrap();

    common::assert_json_array_len(parameters, 1);
    assert_ne!(
        parameters,
        &Value::Null,
        r"expected paths.hello/{{tail}}.get.parameters not null"
    );

    assert_value! {parameters=>
        "[0].schema.type" = r#""string""#, "Tail parameter type"
        "[0].schema.format" = r#"null"#, "Tail parameter format"
        "[0].name" = r#""tail""#, "Tail parameter name"
        "[0].required" = r#"true"#, "Tail parameter required"
        "[0].deprecated" = r#"null"#, "Tail parameter required"
        "[0].in" = r#""path""#, "Tail parameter in"
    }
}

#[test]
fn resolve_get_path_and_update_params() {
    mod rocket_get_operation {
        use rocket::get;

        #[utoipa::path(
            responses(
                (status = 200, description = "Hello from server")
            ),
            params(
                ("id", description = "Hello id")
            )
        )]
        #[get("/hello/<id>/<name>")]
        #[allow(unused)]
        fn hello(id: i32, name: String) -> String {
            "Hello".to_string()
        }
    }

    #[derive(OpenApi)]
    #[openapi(paths(rocket_get_operation::hello))]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();
    let value = &serde_json::to_value(&openapi).unwrap();
    let parameters = value
        .pointer("/paths/~1hello~1{id}~1{name}/get/parameters")
        .unwrap();

    common::assert_json_array_len(parameters, 2);
    assert_ne!(
        parameters,
        &Value::Null,
        r"expected paths.hello/{{id}}/{{name}}.get.parameters not null"
    );

    assert_value! {parameters=>
        "[0].schema.type" = r#""integer""#, "Id parameter type"
        "[0].schema.format" = r#""int32""#, "Id parameter format"
        "[0].description" = r#""Hello id""#, "Id parameter format"
        "[0].name" = r#""id""#, "Id parameter name"
        "[0].required" = r#"true"#, "Id parameter required"
        "[0].deprecated" = r#"null"#, "Id parameter required"
        "[0].in" = r#""path""#, "Id parameter in"

        "[1].schema.type" = r#""string""#, "Name parameter type"
        "[1].schema.format" = r#"null"#, "Name parameter format"
        "[1].description" = r#"null"#, "Name parameter format"
        "[1].name" = r#""name""#, "Name parameter name"
        "[1].required" = r#"true"#, "Name parameter required"
        "[1].deprecated" = r#"null"#, "Name parameter required"
        "[1].in" = r#""path""#, "Name parameter in"
    }
}

#[test]
fn resolve_path_query_params_from_form() {
    mod rocket_get_operation {
        use rocket::{get, FromForm};
        use utoipa::IntoParams;

        #[derive(serde::Deserialize, FromForm, IntoParams)]
        #[allow(unused)]
        struct QueryParams {
            foo: String,
            bar: i64,
        }

        #[utoipa::path(
            responses(
                (status = 200, description = "Hello from server")
            ),
            params(
                ("id", description = "Hello id"),
                QueryParams
            )
        )]
        #[get("/hello/<id>?<rest..>")]
        #[allow(unused)]
        fn hello(id: i32, rest: QueryParams) -> String {
            "Hello".to_string()
        }
    }

    #[derive(OpenApi)]
    #[openapi(paths(rocket_get_operation::hello))]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();
    let value = &serde_json::to_value(&openapi).unwrap();
    let parameters = value
        .pointer("/paths/~1hello~1{id}/get/parameters")
        .unwrap();

    assert_json_eq!(
        parameters,
        json!([
            {
                "description": "Hello id",
                "in": "path",
                "name": "id",
                "required": true,
                "schema": {
                    "format": "int32",
                    "type": "integer"
                }
            },
            {
                "in": "query",
                "name": "foo",
                "required": true,
                "schema": {
                    "type": "string"
                }
            },
            {
                "in": "query",
                "name": "bar",
                "required": true,
                "schema": {
                    "format": "int64",
                    "type": "integer"
                }
            }
        ])
    )
}

#[test]
fn path_with_all_args_and_body() {
    use rocket::FromForm;
    use utoipa::IntoParams;

    #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    struct Hello<'a> {
        message: &'a str,
    }

    #[derive(serde::Deserialize, FromForm, IntoParams)]
    #[allow(unused)]
    struct QueryParams {
        foo: String,
        bar: i64,
    }

    // NOTE! temporarily disable automatic parameter recognition
    #[utoipa::path(
    responses(
        (
            status = 200, description = "Hello from server")
        ),
        params(
            ("id", description = "Hello id"),
            QueryParams
        )
    )]
    #[post("/hello/<id>/<name>?<colors>&<rest..>", data = "<hello>")]
    #[allow(unused)]
    fn post_hello(
        id: i32,
        name: &str,
        colors: Vec<&str>,
        rest: QueryParams,
        hello: Json<Hello>,
    ) -> String {
        "Hello".to_string()
    }

    #[derive(OpenApi)]
    #[openapi(paths(post_hello))]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();
    let value = &serde_json::to_value(&openapi).unwrap();
    let operation = value.pointer("/paths/~1hello~1{id}~1{name}/post").unwrap();

    assert_json_eq!(
        operation.pointer("/parameters"),
        json!([
            {
                "description": "Hello id",
                "in": "path",
                "name": "id",
                "required": true,
                "schema": {
                    "format": "int32",
                    "type": "integer"
                }
            },
            {
                "in": "query",
                "name": "foo",
                "required": true,
                "schema": {
                    "type": "string"
                }
            },
            {
                "in": "query",
                "name": "bar",
                "required": true,
                "schema": {
                    "format": "int64",
                    "type": "integer"
                }
            },
            {
                "in": "query",
                "name": "colors",
                "required": true,
                "schema": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    }
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
            "content": {
                "application/json": {
                    "schema": {
                        "$ref": "#/components/schemas/Hello"
                    }
                }
            },
            "description": "",
            "required": true
        })
    );
}

#[test]
fn path_with_enum_path_param() {
    #[derive(ToSchema)]
    #[allow(unused)]
    enum ApiVersion {
        V1,
    }

    impl IntoParams for ApiVersion {
        fn into_params(
            _: impl Fn() -> Option<utoipa::openapi::path::ParameterIn>,
        ) -> Vec<utoipa::openapi::path::Parameter> {
            vec![ParameterBuilder::new()
                .description(Some(""))
                .name("api_version")
                .required(utoipa::openapi::Required::True)
                .parameter_in(utoipa::openapi::path::ParameterIn::Path)
                .schema(Some(schema!(
                    #[inline]
                    ApiVersion
                )))
                .build()]
        }
    }

    impl<'a> FromParam<'a> for ApiVersion {
        type Error = Error;

        fn from_param(_param: &'a str) -> Result<Self, Self::Error> {
            todo!()
        }
    }

    // NOTE! temporarily disable automatic parameter recognition
    #[utoipa::path(
        post,
        path = "/item",
        params(
            ApiVersion
        ),
        responses(
            (status = 201, description = "Item created successfully"),
        ),
    )]
    #[post("/<api_version>/item", format = "json")]
    #[allow(unused)]
    async fn create_item(api_version: ApiVersion) -> String {
        todo!()
    }

    #[derive(OpenApi)]
    #[openapi(paths(create_item))]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();
    let value = &serde_json::to_value(&openapi).unwrap();
    let operation = value.pointer("/paths/~1item/post").unwrap();

    assert_json_eq!(
        operation.pointer("/parameters"),
        json!([
            {
                "description": "",
                "in": "path",
                "name": "api_version",
                "required": true,
                "schema": {
                    "type": "string",
                    "enum": [
                        "V1"
                    ]
                }
            }
        ])
    )
}

macro_rules! test_derive_path_operations {
    ( $($name:ident: $operation:ident)* ) => {
        $(
            #[test]
            fn $name() {
                mod rocket_operation {
                    use rocket::$operation;

                    #[utoipa::path(
                                                responses(
                                                    (status = 200, description = "Hello from server")
                                                )
                                            )]
                    #[$operation("/hello")]
                    #[allow(unused)]
                    fn hello() -> String {
                        "Hello".to_string()
                    }
                }

                #[derive(OpenApi)]
                #[openapi(paths(rocket_operation::hello))]
                struct ApiDoc;

                let openapi = ApiDoc::openapi();
                let value = &serde_json::to_value(&openapi).unwrap();
                let op = value
                    .pointer(&*format!("/paths/~1hello/{}", stringify!($operation)))
                    .unwrap();

                assert_ne!(
                    op,
                    &Value::Null,
                    "expected paths./hello.{}", stringify!($operation)
                );
            }
        )*
    };
}

test_derive_path_operations! {
    derive_path_get: get
    derive_path_post: post
    derive_path_put: put
    derive_path_delete: delete
    derive_path_head: head
    derive_path_options: options
    derive_path_patch: patch
}
