#![cfg(feature = "json")]
use std::collections::BTreeMap;

use assert_json_diff::assert_json_eq;
use paste::paste;
use serde_json::{json, Value};
use std::collections::HashMap;
use utoipa::openapi::schema::RefOr;
use utoipa::{
    openapi::{Response, ResponseBuilder, ResponsesBuilder},
    IntoParams, IntoResponses, OpenApi, ToSchema,
};

mod common;

macro_rules! test_api_fn_doc {
    ( $handler:path, operation: $operation:expr, path: $path:literal ) => {{
        use utoipa::OpenApi;
        #[derive(OpenApi, Default)]
        #[openapi(paths($handler))]
        struct ApiDoc;

        let doc = &serde_json::to_value(ApiDoc::openapi()).unwrap();
        let operation = doc
            .pointer(&format!(
                "/paths/{}/{}",
                $path.replace("/", "~1"),
                stringify!($operation)
            ))
            .unwrap_or(&serde_json::Value::Null);
        operation.clone()
    }};
}

macro_rules! test_api_fn {
    (name: $name:ident, module: $module:ident,
        operation: $operation:ident,
        path: $path:expr
        $(, params: $params:expr )?
        $(, operation_id: $operation_id:expr )?
        $(, tag: $tag:expr )?
        $(; $( #[$meta:meta] )* )? ) => {
        mod $module {
            $( $(#[$meta])* )*
            #[utoipa::path(
                $operation,
                $( operation_id = $operation_id, )*
                path = $path,
                responses(
                    (status = 200, description = "success response")
                ),
                $( params $params, )*
                $( tag = $tag, )*
            )]
            #[allow(unused)]
            async fn $name() -> String {
                "foo".to_string()
            }
        }
    };
}
macro_rules! test_path_operation {
    ( $($name:ident: $operation:ident)* ) => {
       $(paste! {
            test_api_fn! {
                name: test_operation,
                module: [<mod_ $name>],
                operation: $operation,
                path: "/foo"
            }
        }
        #[test]
        fn $name() {
            paste!{
                use utoipa::OpenApi;
                #[derive(OpenApi, Default)]
                #[openapi(paths(
                    [<mod_ $name>]::test_operation
                 ))]
                struct ApiDoc;
            }

            let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
            let operation_value = doc.pointer(&*format!("/paths/~1foo/{}", stringify!($operation))).unwrap_or(&serde_json::Value::Null);
            assert!(operation_value != &serde_json::Value::Null,
                "expected to find operation with: {}", &format!("paths./foo.{}", stringify!($operation)));
        })*
    };
}

test_path_operation! {
    derive_path_post: post
    derive_path_get: get
    derive_path_delete: delete
    derive_path_put: put
    derive_path_options: options
    derive_path_head: head
    derive_path_patch: patch
    derive_path_trace: trace
    derive_path_connect: connect
}

test_api_fn! {
    name: test_operation2,
    module: derive_path_with_all_info,
    operation: post,
    path: "/foo/bar/{id}",
    params: (("id", description = "Foo bar id")),
    operation_id: "foo_bar_id",
    tag: "custom_tag";
    /// This is test operation description
    ///
    /// Additional info in long description
    #[deprecated]
}

#[test]
fn derive_path_with_all_info_success() {
    let operation = test_api_fn_doc! {
        derive_path_with_all_info::test_operation2,
        operation: post,
        path: "/foo/bar/{id}"
    };

    common::assert_json_array_len(operation.pointer("/parameters").unwrap(), 1);
    assert_value! {operation=>
       "deprecated" = r#"true"#, "Api fn deprecated status"
       "description" = r#""This is test operation description\n\nAdditional info in long description\n""#, "Api fn description"
       "summary" = r#""This is test operation description""#, "Api fn summary"
       "operationId" = r#""foo_bar_id""#, "Api fn operation_id"
       "tags.[0]" = r#""custom_tag""#, "Api fn tag"

       "parameters.[0].deprecated" = r#"false"#, "Path parameter deprecated"
       "parameters.[0].description" = r#""Foo bar id""#, "Path parameter description"
       "parameters.[0].in" = r#""path""#, "Path parameter in"
       "parameters.[0].name" = r#""id""#, "Path parameter name"
       "parameters.[0].required" = r#"true"#, "Path parameter required"
    }
}

#[test]
fn derive_path_with_defaults_success() {
    test_api_fn! {
        name: test_operation3,
        module: derive_path_with_defaults,
        operation: post,
        path: "/foo/bar";
    }
    let operation = test_api_fn_doc! {
        derive_path_with_defaults::test_operation3,
        operation: post,
        path: "/foo/bar"
    };

    assert_value! {operation=>
       "deprecated" = r#"false"#, "Api fn deprecated status"
       "description" = r#""""#, "Api fn description"
       "operationId" = r#""test_operation3""#, "Api fn operation_id"
       "tags.[0]" = r#""derive_path_with_defaults""#, "Api fn tag"
       "parameters" = r#"null"#, "Api parameters"
    }
}

/// This is test operation
///
/// This is long description for test operation
#[utoipa::path(
    get,
    path = "/foo/{id}",
    responses(
        (status = 200, description = "success response")
    ),
    params(
        ("id" = u64, description = "Foo database id"),
        ("since" = Option<String>, Query, description = "Datetime since foo is updated")
    )
)]
#[allow(unused)]
async fn get_foos_by_id_since() -> String {
    "".to_string()
}

#[test]
fn derive_path_with_extra_attributes_without_nested_module() {
    let operation = test_api_fn_doc! {
        get_foos_by_id_since,
        operation: get,
        path: "/foo/{id}"
    };

    common::assert_json_array_len(operation.pointer("/parameters").unwrap(), 2);
    assert_value! {operation=>
        "deprecated" = r#"false"#, "Api operation deprecated"
        "description" = r#""This is test operation\n\nThis is long description for test operation\n""#, "Api operation description"
        "operationId" = r#""get_foos_by_id_since""#, "Api operation operation_id"
        "summary" = r#""This is test operation""#, "Api operation summary"
        "tags.[0]" = r#""crate""#, "Api operation tag"

        "parameters.[0].deprecated" = r#"false"#, "Parameter 0 deprecated"
        "parameters.[0].description" = r#""Foo database id""#, "Parameter 0 description"
        "parameters.[0].in" = r#""path""#, "Parameter 0 in"
        "parameters.[0].name" = r#""id""#, "Parameter 0 name"
        "parameters.[0].required" = r#"true"#, "Parameter 0 required"
        "parameters.[0].schema.format" = r#""int64""#, "Parameter 0 schema format"
        "parameters.[0].schema.type" = r#""integer""#, "Parameter 0 schema type"

        "parameters.[1].deprecated" = r#"false"#, "Parameter 1 deprecated"
        "parameters.[1].description" = r#""Datetime since foo is updated""#, "Parameter 1 description"
        "parameters.[1].in" = r#""query""#, "Parameter 1 in"
        "parameters.[1].name" = r#""since""#, "Parameter 1 name"
        "parameters.[1].required" = r#"false"#, "Parameter 1 required"
        "parameters.[1].schema.format" = r#"null"#, "Parameter 1 schema format"
        "parameters.[1].schema.type" = r#""string""#, "Parameter 1 schema type"
    }
}

#[test]
fn derive_path_with_security_requirements() {
    #[utoipa::path(
        get,
        path = "/items",
        responses(
            (status = 200, description = "success response")
        ),
        security(
            (),
            ("api_oauth" = ["read:items", "edit:items"]),
            ("jwt_token" = [])
        )
    )]
    #[allow(unused)]
    fn get_items() -> String {
        "".to_string()
    }
    let operation = test_api_fn_doc! {
        get_items,
        operation: get,
        path: "/items"
    };

    assert_value! {operation=>
        "security.[0]" = "{}", "Optional security requirement"
        "security.[1].api_oauth.[0]" = r###""read:items""###, "api_oauth first scope"
        "security.[1].api_oauth.[1]" = r###""edit:items""###, "api_oauth second scope"
        "security.[2].jwt_token" = "[]", "jwt_token auth scopes"
    }
}

#[test]
fn derive_path_with_parameter_schema() {
    #[derive(serde::Deserialize, utoipa::ToSchema)]
    struct Since {
        /// Some date
        #[allow(dead_code)]
        date: String,
        /// Some time
        #[allow(dead_code)]
        time: String,
    }

    /// This is test operation
    ///
    /// This is long description for test operation
    #[utoipa::path(
        get,
        path = "/foo/{id}",
        responses(
            (status = 200, description = "success response")
        ),
        params(
            ("id" = u64, description = "Foo database id"),
            ("since" = Option<Since>, Query, description = "Datetime since foo is updated")
        )
    )]
    #[allow(unused)]
    async fn get_foos_by_id_since() -> String {
        "".to_string()
    }

    let operation: Value = test_api_fn_doc! {
        get_foos_by_id_since,
        operation: get,
        path: "/foo/{id}"
    };

    let parameters: &Value = operation.get("parameters").unwrap();

    assert_json_eq!(
        parameters,
        json!([
            {
                "deprecated": false,
                "description": "Foo database id",
                "in": "path",
                "name": "id",
                "required": true,
                "schema": {
                    "format": "int64",
                    "type": "integer"
                }
            },
            {
                "deprecated": false,
                "description": "Datetime since foo is updated",
                "in": "query",
                "name": "since",
                "required": false,
                "schema": {
                    "$ref": "#/components/schemas/Since"
                }
            }
        ])
    );
}

#[test]
fn derive_path_with_parameter_inline_schema() {
    #[derive(serde::Deserialize, utoipa::ToSchema)]
    struct Since {
        /// Some date
        #[allow(dead_code)]
        date: String,
        /// Some time
        #[allow(dead_code)]
        time: String,
    }

    /// This is test operation
    ///
    /// This is long description for test operation
    #[utoipa::path(
        get,
        path = "/foo/{id}",
        responses(
            (status = 200, description = "success response")
        ),
        params(
            ("id" = u64, description = "Foo database id"),
            ("since" = inline(Option<Since>), Query, description = "Datetime since foo is updated")
        )
    )]
    #[allow(unused)]
    async fn get_foos_by_id_since() -> String {
        "".to_string()
    }

    let operation: Value = test_api_fn_doc! {
        get_foos_by_id_since,
        operation: get,
        path: "/foo/{id}"
    };

    let parameters: &Value = operation.get("parameters").unwrap();

    assert_json_eq!(
        parameters,
        json!([
            {
                "deprecated": false,
                "description": "Foo database id",
                "in": "path",
                "name": "id",
                "required": true,
                "schema": {
                    "format": "int64",
                    "type": "integer"
                }
            },
            {
                "deprecated": false,
                "description": "Datetime since foo is updated",
                "in": "query",
                "name": "since",
                "required": false,
                "schema": {
                    "properties": {
                        "date": {
                            "description": "Some date",
                            "type": "string"
                        },
                        "time": {
                            "description": "Some time",
                            "type": "string"
                        }
                    },
                    "required": [
                        "date",
                        "time"
                    ],
                    "type": "object"
                }
            }
        ])
    );
}

#[test]
fn derive_path_params_map() {
    #[derive(serde::Deserialize, ToSchema)]
    enum Foo {
        Bar,
        Baz,
    }

    #[derive(serde::Deserialize, IntoParams)]
    #[allow(unused)]
    struct MyParams {
        with_ref: HashMap<String, Foo>,
        with_type: HashMap<String, String>,
    }

    #[utoipa::path(
        get,
        path = "/foo",
        responses(
            (status = 200, description = "success response")
        ),
        params(
            MyParams,
        )
    )]
    #[allow(unused)]
    fn use_maps(params: MyParams) -> String {
        "".to_string()
    }

    use utoipa::OpenApi;
    #[derive(OpenApi, Default)]
    #[openapi(paths(use_maps))]
    struct ApiDoc;

    let operation: Value = test_api_fn_doc! {
        use_maps,
        operation: get,
        path: "/foo"
    };

    let parameters = operation.get("parameters").unwrap();

    assert_json_eq! {
        parameters,
        json!{[
            {
            "in": "path",
            "name": "with_ref",
            "required": true,
            "schema": {
              "additionalProperties": {
                "$ref": "#/components/schemas/Foo"
              },
              "type": "object"
            }
          },
          {
            "in": "path",
            "name": "with_type",
            "required": true,
            "schema": {
              "additionalProperties": {
                "type": "string"
              },
              "type": "object"
            }
          }
        ]}
    }
}

#[test]
fn derive_path_params_intoparams() {
    #[derive(serde::Deserialize, ToSchema)]
    #[schema(default = "foo1", example = "foo1")]
    #[serde(rename_all = "snake_case")]
    enum Foo {
        Foo1,
        Foo2,
    }

    #[derive(serde::Deserialize, IntoParams)]
    #[into_params(style = Form, parameter_in = Query)]
    struct MyParams {
        /// Foo database id.
        #[param(example = 1)]
        #[allow(unused)]
        id: u64,
        /// Datetime since foo is updated.
        #[param(example = "2020-04-12T10:23:00Z")]
        #[allow(unused)]
        since: Option<String>,
        /// A Foo item ref.
        #[allow(unused)]
        foo_ref: Foo,
        /// A Foo item inline.
        #[param(inline)]
        #[allow(unused)]
        foo_inline: Foo,
        /// An optional Foo item inline.
        #[param(inline)]
        #[allow(unused)]
        foo_inline_option: Option<Foo>,
        /// A vector of Foo item inline.
        #[param(inline)]
        #[allow(unused)]
        foo_inline_vec: Vec<Foo>,
    }

    #[utoipa::path(
        get,
        path = "/list/{id}",
        responses(
            (status = 200, description = "success response")
        ),
        params(
            MyParams,
            ("id" = i64, Path, description = "Id of some items to list")
        )
    )]
    #[allow(unused)]
    fn list(id: i64, params: MyParams) -> String {
        "".to_string()
    }

    use utoipa::OpenApi;
    #[derive(OpenApi, Default)]
    #[openapi(paths(list))]
    struct ApiDoc;

    let operation: Value = test_api_fn_doc! {
        list,
        operation: get,
        path: "/list/{id}"
    };

    let parameters = operation.get("parameters").unwrap();

    assert_json_eq!(
        parameters,
        json!([
            {
                "description": "Foo database id.",
                "example": 1,
                "in": "query",
                "name": "id",
                "required": true,
                "schema": {
                    "format": "int64",
                    "type": "integer"
                },
                "style": "form"
            },
            {
                "description": "Datetime since foo is updated.",
                "example": "2020-04-12T10:23:00Z",
                "in": "query",
                "name": "since",
                "required": false,
                "schema": {
                    "type": "string"
                },
                "style": "form"
            },
            {
                "description": "A Foo item ref.",
                "in": "query",
                "name": "foo_ref",
                "required": true,
                "schema": {
                    "$ref": "#/components/schemas/Foo"
                },
                "style": "form"
            },
            {
                "description": "A Foo item inline.",
                "in": "query",
                "name": "foo_inline",
                "required": true,
                "schema": {
                    "default": "foo1",
                    "example": "foo1",
                    "enum": ["foo1", "foo2"],
                    "type": "string",
                },
                "style": "form"
            },
            {
                "description": "An optional Foo item inline.",
                "in": "query",
                "name": "foo_inline_option",
                "required": false,
                "schema": {
                    "default": "foo1",
                    "example": "foo1",
                    "enum": ["foo1", "foo2"],
                    "type": "string",
                },
                "style": "form"
            },
            {
                "description": "A vector of Foo item inline.",
                "in": "query",
                "name": "foo_inline_vec",
                "required": true,
                "schema": {
                    "items": {
                        "default": "foo1",
                        "example": "foo1",
                        "enum": ["foo1", "foo2"],
                        "type": "string",
                    },
                    "type": "array",
                },
                "style": "form",
            },
            {
                "deprecated": false,
                "description": "Id of some items to list",
                "in": "path",
                "name": "id",
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
fn derive_path_params_into_params_with_value_type() {
    use utoipa::OpenApi;

    #[derive(ToSchema)]
    struct Foo {
        #[allow(unused)]
        value: String,
    }

    #[derive(IntoParams)]
    #[into_params(parameter_in = Query)]
    #[allow(unused)]
    struct Filter {
        #[param(value_type = i64, style = Simple)]
        id: String,
        #[param(value_type = Object)]
        another_id: String,
        #[param(value_type = Vec<Vec<String>>)]
        value1: Vec<i64>,
        #[param(value_type = Vec<String>)]
        value2: Vec<i64>,
        #[param(value_type = Option<String>)]
        value3: i64,
        #[param(value_type = Option<Object>)]
        value4: i64,
        #[param(value_type = Vec<Object>)]
        value5: i64,
        #[param(value_type = Vec<Foo>)]
        value6: i64,
    }

    #[utoipa::path(
        get,
        path = "foo",
        responses(
            (status = 200, description = "success response")
        ),
        params(
            Filter
        )
    )]
    #[allow(unused)]
    fn get_foo(query: Filter) {}

    #[derive(OpenApi, Default)]
    #[openapi(paths(get_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc.pointer("/paths/foo/get/parameters").unwrap();

    assert_json_eq!(
        parameters,
        json!([{
            "in": "query",
            "name": "id",
            "required": true,
            "style": "simple",
            "schema": {
                "format": "int64",
                "type": "integer"
            }
        },
        {
            "in": "query",
            "name": "another_id",
            "required": true,
            "schema": {
                "type": "object"
            }
        },
        {
            "in": "query",
            "name": "value1",
            "required": true,
            "schema": {
                "items": {
                    "items": {
                        "type": "string"
                    },
                    "type": "array"
                },
                "type": "array"
            }
        },
        {
            "in": "query",
            "name": "value2",
            "required": true,
            "schema": {
                "items": {
                    "type": "string"
                },
                "type": "array"
            }
        },
        {
            "in": "query",
            "name": "value3",
            "required": false,
            "schema": {
                "type": "string"
            }
        },
        {
            "in": "query",
            "name": "value4",
            "required": false,
            "schema": {
                "type": "object"
            }
        },
        {
            "in": "query",
            "name": "value5",
            "required": true,
            "schema": {
                "items": {
                    "type": "object"
                },
                "type": "array"
            }
        },
        {
            "in": "query",
            "name": "value6",
            "required": true,
            "schema": {
                "items": {
                    "$ref": "#/components/schemas/Foo"
                },
                "type": "array"
            }
        }])
    )
}

#[test]
fn derive_path_params_into_params_with_raw_identifier() {
    #[derive(IntoParams)]
    #[into_params(parameter_in = Path)]
    struct Filter {
        #[allow(unused)]
        r#in: String,
    }

    #[utoipa::path(
        get,
        path = "foo",
        responses(
            (status = 200, description = "success response")
        ),
        params(
            Filter
        )
    )]
    #[allow(unused)]
    fn get_foo(query: Filter) {}

    #[derive(OpenApi, Default)]
    #[openapi(paths(get_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc.pointer("/paths/foo/get/parameters").unwrap();

    assert_json_eq!(
        parameters,
        json!([{
            "in": "path",
            "name": "in",
            "required": true,
            "schema": {
                "type": "string"
            }
        }])
    )
}

#[test]
fn derive_path_with_into_responses() {
    #[allow(unused)]
    enum MyResponse {
        Ok,
        NotFound,
    }

    impl IntoResponses for MyResponse {
        fn responses() -> BTreeMap<String, RefOr<Response>> {
            let responses = ResponsesBuilder::new()
                .response("200", ResponseBuilder::new().description("Ok"))
                .response("404", ResponseBuilder::new().description("Not Found"))
                .build();

            responses.responses
        }
    }

    #[utoipa::path(get, path = "foo", responses(MyResponse))]
    #[allow(unused)]
    fn get_foo() {}

    #[derive(OpenApi, Default)]
    #[openapi(paths(get_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc.pointer("/paths/foo/get/responses").unwrap();

    assert_json_eq!(
        parameters,
        json!({
            "200": {
                "description": "Ok"
            },
            "404": {
                "description": "Not Found"
            }
        })
    )
}

#[cfg(feature = "uuid")]
#[test]
fn derive_path_with_uuid() {
    use uuid::Uuid;

    #[utoipa::path(
        get,
        path = "/items/{id}",
        responses(
            (status = 200, description = "success response")
        ),
        params(
            ("id" = Uuid, description = "Foo uuid"),
        )
    )]
    #[allow(unused)]
    fn get_items(id: Uuid) -> String {
        "".to_string()
    }
    let operation = test_api_fn_doc! {
        get_items,
        operation: get,
        path: "/items/{id}"
    };

    assert_value! {operation=>
        "parameters.[0].schema.type" = r#""string""#, "Parameter id type"
        "parameters.[0].schema.format" = r#""uuid""#, "Parameter id format"
        "parameters.[0].description" = r#""Foo uuid""#, "Parameter id description"
        "parameters.[0].name" = r#""id""#, "Parameter id id"
        "parameters.[0].in" = r#""path""#, "Parameter in"
    }
}
