use std::collections::BTreeMap;

use assert_json_diff::{assert_json_eq, assert_json_matches, CompareMode, Config, NumericMode};
use paste::paste;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use utoipa::openapi::RefOr;
use utoipa::openapi::{Object, ObjectBuilder};
use utoipa::Path;
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
}

macro_rules! api_fn_doc_with_params {
    ( $method:ident: $path:literal => $( #[$attr:meta] )* $key:ident $name:ident $body:tt ) => {{
        #[allow(dead_code)]
        #[derive(serde::Deserialize, utoipa::IntoParams)]
        $(#[$attr])*
        $key $name $body

        #[utoipa::path(
                $method,
                path = $path,
                responses(
                    (status = 200, description = "success response")
                ),
                params(
                    $name,
                )
            )]
        #[allow(unused)]
        async fn my_operation(params: MyParams) -> String {
            "".to_string()
        }

        let operation: Value = test_api_fn_doc! {
            my_operation,
            operation: $method,
            path: $path
        };

        operation
    }};
}

test_api_fn! {
    name: test_operation2,
    module: derive_path_with_all_info,
    operation: post,
    path: "/foo/bar/{id}",
    params: (("id", description = "Foo bar id")),
    operation_id: "foo_bar_id",
    tag: "custom_tag";
    /// This is test operation long multiline
    /// summary. That need to be correctly split.
    ///
    /// Additional info in long description
    ///
    /// With more info on separate lines
    /// containing markdown:
    /// - A
    ///   Indented.
    /// - B
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
       "description" = r#""Additional info in long description\n\nWith more info on separate lines\ncontaining markdown:\n- A\n  Indented.\n- B""#, "Api fn description"
       "summary" = r#""This is test operation long multiline\nsummary. That need to be correctly split.""#, "Api fn summary"
       "operationId" = r#""foo_bar_id""#, "Api fn operation_id"
       "tags.[0]" = r#""custom_tag""#, "Api fn tag"

       "parameters.[0].deprecated" = r#"null"#, "Path parameter deprecated"
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
       "deprecated" = r#"null"#, "Api fn deprecated status"
       "operationId" = r#""test_operation3""#, "Api fn operation_id"
       "tags.[0]" = r#""derive_path_with_defaults""#, "Api fn tag"
       "parameters" = r#"null"#, "Api parameters"
    }
}

#[test]
fn derive_path_with_extra_attributes_without_nested_module() {
    /// This is test operation
    ///
    /// This is long description for test operation
    #[utoipa::path(
        get,
        path = "/foo/{id}",
        responses(
            (
                status = 200, description = "success response")
            ),
            params(
                ("id" = i64, deprecated = false, description = "Foo database id"),
                ("since" = Option<String>, Query, deprecated = false, description = "Datetime since foo is updated")
            )
    )]
    #[allow(unused)]
    async fn get_foos_by_id_since() -> String {
        "".to_string()
    }

    let operation = test_api_fn_doc! {
        get_foos_by_id_since,
        operation: get,
        path: "/foo/{id}"
    };

    common::assert_json_array_len(operation.pointer("/parameters").unwrap(), 2);
    assert_value! {operation=>
        "deprecated" = r#"null"#, "Api operation deprecated"
        "description" = r#""This is long description for test operation""#, "Api operation description"
        "operationId" = r#""get_foos_by_id_since""#, "Api operation operation_id"
        "summary" = r#""This is test operation""#, "Api operation summary"
        "tags.[0]" = r#"null"#, "Api operation tag"

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
        "parameters.[1].schema.allOf.[0].format" = r#"null"#, "Parameter 1 schema format"
        "parameters.[1].schema.allOf.[0].type" = r#"null"#, "Parameter 1 schema type"
        "parameters.[1].schema.allOf.nullable" = r#"null"#, "Parameter 1 schema type"
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
fn derive_path_with_datetime_format_query_parameter() {
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
        path = "/foo/{id}/{start}",
        responses(
            (status = 200, description = "success response")
        ),
        params(
            ("id" = i64, Path, description = "Foo database id"),
            ("start" = String, Path, description = "Datetime since foo is updated", format = DateTime)
        )
    )]
    #[allow(unused)]
    async fn get_foos_by_id_date() -> String {
        "".to_string()
    }

    let operation: Value = test_api_fn_doc! {
        get_foos_by_id_date,
        operation: get,
        path: "/foo/{id}/{start}"
    };

    let parameters: &Value = operation.get("parameters").unwrap();

    assert_json_eq!(
        parameters,
        json!([
            {
                "description": "Foo database id",
                "in": "path",
                "name": "id",
                "required": true,
                "schema": {
                    "format": "int64",
                    "type": "integer",
                }
            },
            {
                "description": "Datetime since foo is updated",
                "in": "path",
                "name": "start",
                "required": true,
                "schema": {
                    "format": "date-time",
                    "type": "string",
                }
            }
        ])
    );
}

#[test]
fn derive_path_with_datetime_format_path_parameter() {
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
            ("id" = i64, description = "Foo database id"),
            ("start" = String, Query, description = "Datetime since foo is updated", format = DateTime)
        )
    )]
    #[allow(unused)]
    async fn get_foos_by_id_date() -> String {
        "".to_string()
    }

    let operation: Value = test_api_fn_doc! {
        get_foos_by_id_date,
        operation: get,
        path: "/foo/{id}"
    };

    let parameters: &Value = operation.get("parameters").unwrap();

    assert_json_eq!(
        parameters,
        json!([
            {
                "description": "Foo database id",
                "in": "path",
                "name": "id",
                "required": true,
                "schema": {
                    "format": "int64",
                    "type": "integer",
                }
            },
            {
                "description": "Datetime since foo is updated",
                "in": "query",
                "name": "start",
                "required": true,
                "schema": {
                    "format": "date-time",
                    "type": "string",
                }
            }
        ])
    );
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
            ("id" = i64, description = "Foo database id"),
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
                "description": "Foo database id",
                "in": "path",
                "name": "id",
                "required": true,
                "schema": {
                    "format": "int64",
                    "type": "integer",
                }
            },
            {
                "description": "Datetime since foo is updated",
                "in": "query",
                "name": "since",
                "required": false,
                "schema": {
                    "allOf": [
                        {
                            "type": "null"
                        },
                        {
                            "$ref": "#/components/schemas/Since"
                        }
                    ],
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
            ("id" = i64, description = "Foo database id"),
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
                "description": "Foo database id",
                "in": "path",
                "name": "id",
                "required": true,
                "schema": {
                    "format": "int64",
                    "type": "integer",
                }
            },
            {
                "description": "Datetime since foo is updated",
                "in": "query",
                "name": "since",
                "required": false,
                "schema": {
                    "allOf": [
                        {
                            "type": "null"
                        },
                        {
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
                    ],
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
                "propertyNames": {
                    "type": "string"
                },
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
                "propertyNames": {
                    "type": "string"
                },
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
fn derive_path_params_with_examples() {
    let operation = api_fn_doc_with_params! {get: "/foo" =>
        struct MyParams {
            #[param(example = json!({"key": "value"}))]
            map: HashMap<String, String>,
            #[param(example = json!(["value1", "value2"]))]
            vec: Vec<String>,
        }
    };
    let parameters = operation.get("parameters").unwrap();

    assert_json_eq! {
        parameters,
        json!{[
            {
            "in": "path",
            "name": "map",
            "required": true,
            "example": {
                "key": "value"
            },
            "schema": {
                "propertyNames": {
                    "type": "string"
                },
                "additionalProperties": {
                    "type": "string"
                },
                "type": "object"
            }
          },
          {
            "in": "path",
            "name": "vec",
            "required": true,
            "example": ["value1", "value2"],
            "schema": {
              "items": {
                "type": "string"
              },
              "type": "array"
            }
          }
        ]}
    }
}

#[test]
fn path_parameters_with_free_form_properties() {
    let operation = api_fn_doc_with_params! {get: "/foo" =>
        struct MyParams {
            #[param(additional_properties)]
            map: HashMap<String, String>,
        }
    };
    let parameters = operation.get("parameters").unwrap();

    assert_json_eq! {
        parameters,
        json!{[
            {
            "in": "path",
            "name": "map",
            "required": true,
            "schema": {
              "additionalProperties": true,
              "type": "object"
            }
          }
        ]}
    }
}

#[test]
fn derive_path_query_params_with_schema_features() {
    let operation = api_fn_doc_with_params! {get: "/foo" =>
        #[into_params(parameter_in = Query)]
        struct MyParams {
            #[serde(default)]
            #[param(write_only, read_only, default = "value", nullable, xml(name = "xml_value"))]
            value: String,
            #[param(value_type = String, format = Binary)]
            int: i64,
        }
    };
    let parameters = operation.get("parameters").unwrap();

    assert_json_eq! {
        parameters,
        json!{[
            {
            "in": "query",
            "name": "value",
            "required": false,
            "schema": {
                "default": "value",
                "type": ["string", "null"],
                "readOnly": true,
                "writeOnly": true,
                "xml": {
                    "name": "xml_value"
                }
            }
          },
          {
            "in": "query",
            "name": "int",
            "required": true,
            "schema": {
              "type": "string",
              "format": "binary"
            }
          }
        ]}
    }
}

#[test]
fn derive_path_params_always_required() {
    let operation = api_fn_doc_with_params! {get: "/foo" =>
        #[into_params(parameter_in = Path)]
        struct MyParams {
            #[serde(default)]
            value: String,
        }
    };
    let parameters = operation.get("parameters").unwrap();

    assert_json_eq! {
        parameters,
        json!{[
            {
            "in": "path",
            "name": "value",
            "required": true,
            "schema": {
                "type": "string",
            }
          }
        ]}
    }
}

#[test]
fn derive_required_path_params() {
    let operation = api_fn_doc_with_params! {get: "/list/{id}" =>
        #[into_params(parameter_in = Query)]
        struct MyParams {
            #[serde(default)]
            vec_default: Option<Vec<String>>,

            #[serde(default)]
            string_default: Option<String>,

            #[serde(default)]
            vec_default_required: Vec<String>,

            #[serde(default)]
            string_default_required: String,

            vec_option: Option<Vec<String>>,

            string_option: Option<String>,

            vec: Vec<String>,

            string: String,
        }
    };

    let parameters = operation.get("parameters").unwrap();

    assert_json_eq!(
        parameters,
        json!([
            {
                "in": "query",
                "name": "vec_default",
                "required": false,
                "schema": {
                    "type": ["array", "null"],
                    "items": {
                        "type": "string"
                    }
                },
            },
            {
                "in": "query",
                "name": "string_default",
                "required": false,
                "schema": {
                    "type": ["string", "null"]
                }
            },
            {
                "in": "query",
                "name": "vec_default_required",
                "required": false,
                "schema": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    }
                },
            },
            {
                "in": "query",
                "name": "string_default_required",
                "required": false,
                "schema": {
                    "type": "string"
                },
            },
            {
                "in": "query",
                "name": "vec_option",
                "required": false,
                "schema": {
                    "items": {
                        "type": "string"
                    },
                    "type": ["array", "null"],
                },
            },
            {
                "in": "query",
                "name": "string_option",
                "required": false,
                "schema": {
                    "type": ["string", "null"]
                }
            },
            {
                "in": "query",
                "name": "vec",
                "required": true,
                "schema": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    }
                }
            },
            {
                "in": "query",
                "name": "string",
                "required": true,
                "schema": {
                    "type": "string"
                }
            }
        ])
    )
}

#[test]
fn derive_path_params_with_serde_and_custom_rename() {
    let operation = api_fn_doc_with_params! {get: "/list/{id}" =>
        #[into_params(parameter_in = Query)]
        #[serde(rename_all = "camelCase")]
        struct MyParams {
            vec_default: Option<Vec<String>>,

            #[serde(default, rename = "STRING")]
            string_default: Option<String>,

            #[serde(default, rename = "VEC")]
            #[param(rename = "vec2")]
            vec_default_required: Vec<String>,

            #[serde(default)]
            #[param(rename = "string_r2")]
            string_default_required: String,

            string: String,
        }
    };
    let parameters = operation.get("parameters").unwrap();

    assert_json_eq!(
        parameters,
        json!([
            {
                "in": "query",
                "name": "vecDefault",
                "required": false,
                "schema": {
                    "type": ["array", "null"],
                    "items": {
                        "type": "string"
                    }
                },
            },
            {
                "in": "query",
                "name": "STRING",
                "required": false,
                "schema": {
                    "type": ["string", "null"]
                }
            },
            {
                "in": "query",
                "name": "VEC",
                "required": false,
                "schema": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    }
                },
            },
            {
                "in": "query",
                "name": "string_r2",
                "required": false,
                "schema": {
                    "type": "string"
                },
            },
            {
                "in": "query",
                "name": "string",
                "required": true,
                "schema": {
                    "type": "string"
                }
            }
        ])
    )
}

#[test]
fn derive_path_params_custom_rename_all() {
    let operation = api_fn_doc_with_params! {get: "/list/{id}" =>
        #[into_params(rename_all = "camelCase", parameter_in = Query)]
        struct MyParams {
            vec_default: Option<Vec<String>>,
        }
    };
    let parameters = operation.get("parameters").unwrap();

    assert_json_eq!(
        parameters,
        json!([
            {
                "in": "query",
                "name": "vecDefault",
                "required": false,
                "schema": {
                    "type": ["array", "null"],
                    "items": {
                        "type": "string"
                    }
                },
            },
        ])
    )
}

#[test]
fn derive_path_params_custom_rename_all_serde_will_override() {
    let operation = api_fn_doc_with_params! {get: "/list/{id}" =>
        #[into_params(rename_all = "camelCase", parameter_in = Query)]
        #[serde(rename_all = "UPPERCASE")]
        struct MyParams {
            vec_default: Option<Vec<String>>,
        }
    };
    let parameters = operation.get("parameters").unwrap();

    assert_json_eq!(
        parameters,
        json!([
            {
                "in": "query",
                "name": "VEC_DEFAULT",
                "required": false,
                "schema": {
                    "type": ["array", "null"],
                    "items": {
                        "type": "string"
                    }
                },
            },
        ])
    )
}

#[test]
fn derive_path_parameters_container_level_default() {
    let operation = api_fn_doc_with_params! {get: "/list/{id}" =>
        #[derive(Default)]
        #[into_params(parameter_in = Query)]
        #[serde(default)]
        struct MyParams {
            vec_default: Vec<String>,
            string: String,
        }
    };
    let parameters = operation.get("parameters").unwrap();

    assert_json_eq!(
        parameters,
        json!([
            {
                "in": "query",
                "name": "vec_default",
                "required": false,
                "schema": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    }
                },
            },
            {
                "in": "query",
                "name": "string",
                "required": false,
                "schema": {
                    "type": "string"
                },
            }
        ])
    )
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
        id: i64,
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
                    "type": "integer",
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
                    "type": ["string", "null"]
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
                    "allOf": [
                        {
                            "type": "null",
                        },
                        {
                            "default": "foo1",
                            "example": "foo1",
                            "enum": ["foo1", "foo2"],
                            "type": "string",
                        }
                    ],
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
    #[allow(dead_code)]
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
                "type": ["string", "null"]
            }
        },
        {
            "in": "query",
            "name": "value4",
            "required": false,
            "schema": {
                "type": ["object", "null"]
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
fn derive_path_params_into_params_with_unit_type() {
    #[derive(IntoParams)]
    #[into_params(parameter_in = Path)]
    struct Filter {
        #[allow(unused)]
        r#in: (),
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
                "default": null,
            }
        }])
    )
}

#[test]
fn arbitrary_expr_in_operation_id() {
    #[utoipa::path(
        get,
        path = "foo",
        operation_id=format!("{}", 3+5),
        responses(
            (status = 200, description = "success response")
        ),
    )]
    #[allow(unused)]
    fn get_foo() {}

    #[derive(OpenApi, Default)]
    #[openapi(paths(get_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let operation_id = doc.pointer("/paths/foo/get/operationId").unwrap();

    assert_json_eq!(operation_id, json!("8"))
}

#[test]
fn derive_path_with_validation_attributes() {
    #[derive(IntoParams)]
    #[allow(dead_code)]
    struct Query {
        #[param(maximum = 10, minimum = 5, multiple_of = 2.5)]
        id: i32,

        #[param(max_length = 10, min_length = 5, pattern = "[a-z]*")]
        value: String,

        #[param(max_items = 5, min_items = 1)]
        items: Vec<String>,
    }

    #[utoipa::path(
        get,
        path = "foo",
        responses(
            (status = 200, description = "success response")
        ),
        params(
            Query
        )
    )]
    #[allow(unused)]
    fn get_foo(query: Query) {}

    #[derive(OpenApi, Default)]
    #[openapi(paths(get_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc.pointer("/paths/foo/get/parameters").unwrap();

    let config = Config::new(CompareMode::Strict).numeric_mode(NumericMode::AssumeFloat);

    assert_json_matches!(
        parameters,
        json!([
            {
                "schema": {
                    "format": "int32",
                    "type": "integer",
                    "maximum": 10.0,
                    "minimum": 5.0,
                    "multipleOf": 2.5,
                },
                "required": true,
                "name": "id",
                "in": "path"
            },
            {
                "schema": {
                    "type": "string",
                    "maxLength": 10,
                    "minLength": 5,
                    "pattern": "[a-z]*"
                },
                "required": true,
                "name": "value",
                "in": "path"
            },
            {
                "schema": {
                    "type": "array",
                    "items": {
                        "type": "string",
                    },
                    "maxItems": 5,
                    "minItems": 1,
                },
                "required": true,
                "name": "items",
                "in": "path"
            }
        ]),
        config
    );
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

#[cfg(feature = "ulid")]
#[test]
fn derive_path_with_ulid() {
    use ulid::Ulid;

    #[utoipa::path(
        get,
        path = "/items/{id}",
        responses(
            (status = 200, description = "success response")
        ),
        params(
            ("id" = Ulid, description = "Foo ulid"),
        )
    )]
    #[allow(unused)]
    fn get_items(id: Ulid) -> String {
        "".to_string()
    }
    let operation = test_api_fn_doc! {
        get_items,
        operation: get,
        path: "/items/{id}"
    };

    assert_value! {operation=>
        "parameters.[0].schema.type" = r#""string""#, "Parameter id type"
        "parameters.[0].schema.format" = r#""ulid""#, "Parameter id format"
        "parameters.[0].description" = r#""Foo ulid""#, "Parameter id description"
        "parameters.[0].name" = r#""id""#, "Parameter id id"
        "parameters.[0].in" = r#""path""#, "Parameter in"
    }
}

#[test]
fn derive_path_with_into_params_custom_schema() {
    fn custom_type() -> Object {
        ObjectBuilder::new()
            .schema_type(utoipa::openapi::Type::String)
            .format(Some(utoipa::openapi::SchemaFormat::Custom(
                "email".to_string(),
            )))
            .description(Some("this is the description"))
            .build()
    }

    #[derive(IntoParams)]
    #[into_params(parameter_in = Query)]
    #[allow(unused)]
    struct Query {
        #[param(schema_with = custom_type)]
        email: String,
    }

    #[utoipa::path(
        get,
        path = "/items",
        responses(
            (status = 200, description = "success response")
        ),
        params(
            Query
        )
    )]
    #[allow(unused)]
    fn get_items(query: Query) -> String {
        "".to_string()
    }
    let operation = test_api_fn_doc! {
        get_items,
        operation: get,
        path: "/items"
    };

    let value = operation.pointer("/parameters");

    assert_json_eq!(
        value,
        json!([
            {
                "in": "query",
                "name": "email",
                "required": false,
                "schema": {
                    "description": "this is the description",
                    "type": "string",
                    "format": "email"
                }
            }
        ])
    )
}

#[test]
fn derive_into_params_required() {
    #[derive(IntoParams)]
    #[into_params(parameter_in = Query)]
    #[allow(unused)]
    struct Params {
        name: String,
        name2: Option<String>,
        #[param(required)]
        name3: Option<String>,
    }

    #[utoipa::path(get, path = "/params", params(Params))]
    #[allow(unused)]
    fn get_params() {}
    let operation = test_api_fn_doc! {
        get_params,
        operation: get,
        path: "/params"
    };

    let value = operation.pointer("/parameters");

    assert_json_eq!(
        value,
        json!([
          {
              "in": "query",
              "name": "name",
              "required": true,
              "schema": {
                  "type": "string",
              },
          },
          {
              "in": "query",
              "name": "name2",
              "required": false,
              "schema": {
                  "type": ["string", "null"],
              },
          },
          {
              "in": "query",
              "name": "name3",
              "required": true,
              "schema": {
                  "type": ["string", "null"],
              },
          },
        ])
    )
}

#[test]
fn derive_into_params_with_serde_skip() {
    #[derive(IntoParams, Serialize)]
    #[into_params(parameter_in = Query)]
    #[allow(unused)]
    struct Params {
        name: String,
        name2: Option<String>,
        #[serde(skip)]
        name3: Option<String>,
    }

    #[utoipa::path(get, path = "/params", params(Params))]
    #[allow(unused)]
    fn get_params() {}
    let operation = test_api_fn_doc! {
        get_params,
        operation: get,
        path: "/params"
    };

    let value = operation.pointer("/parameters");

    assert_json_eq!(
        value,
        json!([
          {
              "in": "query",
              "name": "name",
              "required": true,
              "schema": {
                  "type": "string",
              },
          },
          {
              "in": "query",
              "name": "name2",
              "required": false,
              "schema": {
                  "type": ["string", "null"],
              },
          },
        ])
    )
}

// TODO: IntoParams seems not to follow Option<T> is automatically nullable rule!

#[test]
fn derive_into_params_with_serde_skip_deserializing() {
    #[derive(IntoParams, Serialize)]
    #[into_params(parameter_in = Query)]
    #[allow(unused)]
    struct Params {
        name: String,
        name2: Option<String>,
        #[serde(skip_deserializing)]
        name3: Option<String>,
    }

    #[utoipa::path(get, path = "/params", params(Params))]
    #[allow(unused)]
    fn get_params() {}
    let operation = test_api_fn_doc! {
        get_params,
        operation: get,
        path: "/params"
    };

    let value = operation.pointer("/parameters");

    assert_json_eq!(
        value,
        json!([
          {
              "in": "query",
              "name": "name",
              "required": true,
              "schema": {
                  "type": "string",
              },
          },
          {
              "in": "query",
              "name": "name2",
              "required": false,
              "schema": {
                  "type": ["string", "null"],
              },
          },
        ])
    )
}

#[test]
fn derive_into_params_with_serde_skip_serializing() {
    #[derive(IntoParams, Serialize)]
    #[into_params(parameter_in = Query)]
    #[allow(unused)]
    struct Params {
        name: String,
        name2: Option<String>,
        #[serde(skip_serializing)]
        name3: Option<String>,
    }

    #[utoipa::path(get, path = "/params", params(Params))]
    #[allow(unused)]
    fn get_params() {}
    let operation = test_api_fn_doc! {
        get_params,
        operation: get,
        path: "/params"
    };

    let value = operation.pointer("/parameters");

    assert_json_eq!(
        value,
        json!([
          {
              "in": "query",
              "name": "name",
              "required": true,
              "schema": {
                  "type": "string",
              },
          },
          {
              "in": "query",
              "name": "name2",
              "required": false,
              "schema": {
                  "type": ["string", "null"],
              },
          },
        ])
    )
}

#[test]
fn derive_path_with_const_expression_context_path() {
    const FOOBAR: &str = "/api/v1/prefix";

    #[utoipa::path(
        get,
        context_path = FOOBAR,
        path = "/items",
        responses(
            (status = 200, description = "success response")
        ),
    )]
    #[allow(unused)]
    fn get_items() -> String {
        "".to_string()
    }

    let operation = test_api_fn_doc! {
        get_items,
        operation: get,
        path: "/api/v1/prefix/items"
    };

    assert_ne!(operation, Value::Null);
}

#[test]
fn derive_path_with_const_expression_reference_context_path() {
    const FOOBAR: &str = "/api/v1/prefix";

    #[utoipa::path(
        get,
        context_path = &FOOBAR,
        path = "/items",
        responses(
            (status = 200, description = "success response")
        ),
    )]
    #[allow(unused)]
    fn get_items() -> String {
        "".to_string()
    }

    let operation = test_api_fn_doc! {
        get_items,
        operation: get,
        path: "/api/v1/prefix/items"
    };

    assert_ne!(operation, Value::Null);
}

#[test]
fn derive_path_with_const_expression() {
    const FOOBAR: &str = "/items";

    #[utoipa::path(
        get,
        path = FOOBAR,
        responses(
            (status = 200, description = "success response")
        ),
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

    assert_ne!(operation, Value::Null);
}

#[test]
fn derive_path_with_tag_constant() {
    const TAG: &str = "mytag";

    #[utoipa::path(
        get,
        tag = TAG,
        path = "/items",
        responses(
            (status = 200, description = "success response")
        ),
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

    assert_ne!(operation, Value::Null);
    assert_json_eq!(
        &operation,
        json!({
            "operationId": "get_items",
            "responses": {
                "200": {
                    "description": "success response",
                },
            },
            "tags": ["mytag"]
        })
    );
}

#[test]
fn derive_path_with_multiple_tags() {
    #[allow(dead_code)]
    const TAG: &str = "mytag";
    const ANOTHER: &str = "another";

    #[utoipa::path(
        get,
        tag = TAG,
        tags = ["one", "two", ANOTHER],
        path = "/items",
        responses(
            (status = 200, description = "success response")
        ),
    )]
    #[allow(unused)]
    async fn get_items() -> String {
        "".to_string()
    }

    let operation = test_api_fn_doc! {
        get_items,
        operation: get,
        path: "/items"
    };

    assert_ne!(operation, Value::Null);
    assert_json_eq!(
        &operation,
        json!({
            "operationId": "get_items",
            "responses": {
                "200": {
                    "description": "success response",
                },
            },
            "tags": ["mytag", "one", "two","another"]
        })
    );
}

#[test]
fn derive_path_with_description_and_summary_override() {
    const SUMMARY: &str = "This is summary override that is
split to multiple lines";
    /// This is long summary
    /// split to multiple lines
    ///
    /// This is description
    /// split to multiple lines
    #[allow(dead_code)]
    #[utoipa::path(
        get,
        path = "/test-description",
        summary = SUMMARY,
        description = "This is description override",
        responses(
            (status = 200, description = "success response")
        ),
    )]
    #[allow(unused)]
    async fn test_description_summary() -> &'static str {
        ""
    }

    let operation = test_api_fn_doc! {
        test_description_summary,
        operation: get,
        path: "/test-description"
    };

    assert_json_eq!(
        &operation,
        json!({
            "description": "This is description override",
            "operationId": "test_description_summary",
            "responses": {
                "200": {
                    "description": "success response",
                },
            },
            "summary": "This is summary override that is\nsplit to multiple lines",
            "tags": []
        })
    );
}

#[test]
fn derive_path_include_str_description() {
    #[allow(dead_code)]
    #[utoipa::path(
        get,
        path = "/test-description",
        description = include_str!("./testdata/description_override"),
        responses(
            (status = 200, description = "success response")
        ),
    )]
    #[allow(unused)]
    async fn test_description_summary() -> &'static str {
        ""
    }

    let operation = test_api_fn_doc! {
        test_description_summary,
        operation: get,
        path: "/test-description"
    };

    assert_json_eq!(
        &operation,
        json!({
            "description": "This is description from include_str!\n",
            "operationId": "test_description_summary",
            "responses": {
                "200": {
                    "description": "success response",
                },
            },
            "tags": []
        })
    );
}

#[test]
fn path_and_nest_with_default_tags_from_path() {
    mod test_path {
        #[allow(dead_code)]
        #[utoipa::path(get, path = "/test")]
        #[allow(unused)]
        fn test_path() -> &'static str {
            ""
        }
    }

    mod test_nest {
        #[derive(utoipa::OpenApi)]
        #[openapi(paths(test_path_nested))]
        pub struct NestApi;

        #[allow(dead_code)]
        #[utoipa::path(get, path = "/test")]
        #[allow(unused)]
        fn test_path_nested() -> &'static str {
            ""
        }
    }

    #[derive(utoipa::OpenApi)]
    #[openapi(
        paths(test_path::test_path),
        nest(
            (path = "/api/nest", api = test_nest::NestApi)
        )
    )]
    struct ApiDoc;
    let value = serde_json::to_value(ApiDoc::openapi()).expect("should be able to serialize json");
    let paths = value
        .pointer("/paths")
        .expect("should find /paths from the OpenAPI spec");

    assert_json_eq!(
        &paths,
        json!({
            "/api/nest/test": {
                "get": {
                    "operationId": "test_path_nested",
                    "responses": {},
                    "tags": ["test_nest"]
                }
            },
            "/test": {
                "get": {
                    "operationId": "test_path",
                    "responses": {},
                    "tags": ["test_path"]
                }
            }
        })
    );
}

#[test]
fn path_and_nest_with_additional_tags() {
    mod test_path {
        #[allow(dead_code)]
        #[utoipa::path(get, path = "/test", tag = "this_is_tag", tags = ["additional"])]
        #[allow(unused)]
        fn test_path() -> &'static str {
            ""
        }
    }

    mod test_nest {
        #[derive(utoipa::OpenApi)]
        #[openapi(paths(test_path_nested))]
        pub struct NestApi;

        #[allow(dead_code)]
        #[utoipa::path(get, path = "/test", tag = "this_is_tag:nest", tags = ["additional:nest"])]
        #[allow(unused)]
        fn test_path_nested() -> &'static str {
            ""
        }
    }

    #[derive(utoipa::OpenApi)]
    #[openapi(
        paths(test_path::test_path),
        nest(
            (path = "/api/nest", api = test_nest::NestApi)
        )
    )]
    struct ApiDoc;
    let value = serde_json::to_value(ApiDoc::openapi()).expect("should be able to serialize json");
    let paths = value
        .pointer("/paths")
        .expect("should find /paths from the OpenAPI spec");

    assert_json_eq!(
        &paths,
        json!({
            "/api/nest/test": {
                "get": {
                    "operationId": "test_path_nested",
                    "responses": {},
                    "tags": ["this_is_tag:nest", "additional:nest"]
                },
            },
            "/test": {
                "get": {
                    "operationId": "test_path",
                    "responses": {},
                    "tags": ["this_is_tag", "additional"]
                },
            }
        })
    );
}

#[test]
fn path_nest_without_any_tags() {
    mod test_path {
        #[allow(dead_code)]
        #[utoipa::path(get, path = "/test")]
        #[allow(unused)]
        pub fn test_path() -> &'static str {
            ""
        }
    }

    mod test_nest {
        #[derive(utoipa::OpenApi)]
        #[openapi(paths(test_path_nested))]
        pub struct NestApi;

        #[allow(dead_code)]
        #[utoipa::path(get, path = "/test")]
        #[allow(unused)]
        fn test_path_nested() -> &'static str {
            ""
        }
    }

    use test_nest::NestApi;
    use test_path::__path_test_path;
    #[derive(utoipa::OpenApi)]
    #[openapi(
        paths(test_path),
        nest(
            (path = "/api/nest", api = NestApi)
        )
    )]
    struct ApiDoc;
    let value = serde_json::to_value(ApiDoc::openapi()).expect("should be able to serialize json");
    let paths = value
        .pointer("/paths")
        .expect("should find /paths from the OpenAPI spec");

    assert_json_eq!(
        &paths,
        json!({
            "/api/nest/test": {
                "get": {
                    "operationId": "test_path_nested",
                    "responses": {},
                    "tags": []
                },
            },
            "/test": {
                "get": {
                    "operationId": "test_path",
                    "responses": {},
                    "tags": []
                },
            }
        })
    );
}

#[test]
fn derive_path_with_multiple_methods() {
    #[allow(dead_code)]
    #[utoipa::path(
        method(head, get),
        path = "/test-multiple",
        responses(
            (status = 200, description = "success response")
        ),
    )]
    #[allow(unused)]
    async fn test_multiple() -> &'static str {
        ""
    }
    use utoipa::OpenApi;
    #[derive(OpenApi, Default)]
    #[openapi(paths(test_multiple))]
    struct ApiDoc;

    let doc = &serde_json::to_value(ApiDoc::openapi()).unwrap();
    let paths = doc.pointer("/paths").expect("OpenApi must have paths");

    assert_json_eq!(
        &paths,
        json!({
            "/test-multiple": {
                "get": {
                    "operationId": "test_multiple",
                    "responses": {
                        "200": {
                            "description": "success response",
                        },
                    },
                    "tags": []
                },
                "head": {
                    "operationId": "test_multiple",
                    "responses": {
                        "200": {
                            "description": "success response",
                        },
                    },
                    "tags": []
                }
            }
        })
    );
}

#[test]
fn derive_path_with_response_links() {
    #![allow(dead_code)]

    #[utoipa::path(
        get,
        path = "/test-links",
        responses(
            (status = 200, description = "success response", 
                links(
                    ("getFoo" = (
                        operation_id = "test_links", 
                        parameters(("key" = "value"), ("json_value" = json!(1))), 
                        request_body = "this is body", 
                        server(url = "http://localhost") 
                    )),
                    ("getBar" = (
                        operation_ref = "this is ref"
                    ))
                )
            )
        ),
    )]
    #[allow(unused)]
    async fn test_links() -> &'static str {
        ""
    }
    use utoipa::OpenApi;
    #[derive(OpenApi, Default)]
    #[openapi(paths(test_links))]
    struct ApiDoc;

    let doc = &serde_json::to_value(ApiDoc::openapi()).unwrap();
    let paths = doc.pointer("/paths").expect("OpenApi must have paths");

    assert_json_eq!(
        &paths,
        json!({
            "/test-links": {
                "get": {
                    "operationId": "test_links",
                    "responses": {
                        "200": {
                            "description": "success response",
                            "links": {
                                "getFoo": {
                                    "operation_id": "test_links",
                                    "parameters": {
                                        "json_value": 1,
                                        "key": "value"
                                    },
                                    "request_body": "this is body",
                                    "server": {
                                        "url": "http://localhost"
                                    }
                                },
                                "getBar": {
                                    "operation_ref": "this is ref"
                                }
                            },
                        },
                    },
                    "tags": []
                },
            }
        })
    );
}

#[test]
fn derive_path_test_collect_request_body() {
    #![allow(dead_code)]

    #[derive(ToSchema)]
    struct Account {
        id: i32,
    }

    #[derive(ToSchema)]
    struct Person {
        name: String,
        account: Account,
    }

    #[utoipa::path(
        post,
        request_body = Person,
        path = "/test-collect-schemas",
        responses(
            (status = 200, description = "success response")
        ),
    )]
    async fn test_collect_schemas(_body: Person) -> &'static str {
        ""
    }

    use utoipa::OpenApi;
    #[derive(OpenApi)]
    #[openapi(paths(test_collect_schemas))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let schemas = doc
        .pointer("/components/schemas")
        .expect("OpenApi must have schemas");

    assert_json_eq!(
        &schemas,
        json!({
            "Person": {
                "properties": {
                    "name": {
                        "type": "string",
                    },
                    "account": {
                        "$ref": "#/components/schemas/Account"
                    }
                },
                "required": ["name", "account"],
                "type": "object"
            },
            "Account": {
                "properties": {
                    "id": {
                        "type": "integer",
                        "format":  "int32",
                    },
                },
                "required": ["id"],
                "type": "object"
            }
        })
    );
}

#[test]
fn derive_path_test_collect_generic_array_request_body() {
    #![allow(dead_code)]

    #[derive(ToSchema)]
    struct Account {
        id: i32,
    }

    #[derive(ToSchema)]
    struct Person {
        name: String,
        account: Account,
    }

    #[derive(ToSchema)]
    struct CreateRequest<T> {
        value: T,
    }

    #[utoipa::path(
        post,
        request_body = [ CreateRequest<Person> ],
        path = "/test-collect-schemas",
        responses(
            (status = 200, description = "success response")
        ),
    )]
    async fn test_collect_schemas(_body: Person) -> &'static str {
        ""
    }

    use utoipa::OpenApi;
    #[derive(OpenApi)]
    #[openapi(paths(test_collect_schemas))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let schemas = doc
        .pointer("/components/schemas")
        .expect("OpenApi must have schemas");

    assert_json_eq!(
        &schemas,
        json!({
            "CreateRequest_Person": {
                "properties": {
                    "value": {
                        "properties": {
                            "name": {
                                "type": "string",
                            },
                            "account": {
                                "$ref": "#/components/schemas/Account"
                            }
                        },
                        "required": ["name", "account"],
                        "type": "object"
                    }
                },
                "required": ["value"],
                "type": "object"
            },
            "Person": {
                "properties": {
                    "name": {
                        "type": "string",
                    },
                    "account": {
                        "$ref": "#/components/schemas/Account"
                    }
                },
                "required": ["name", "account"],
                "type": "object"
            },
            "Account": {
                "properties": {
                    "id": {
                        "type": "integer",
                        "format":  "int32",
                    },
                },
                "required": ["id"],
                "type": "object"
            }
        })
    );
}

#[test]
fn derive_path_test_collect_generic_request_body() {
    #![allow(dead_code)]

    #[derive(ToSchema)]
    struct Account {
        id: i32,
    }

    #[derive(ToSchema)]
    struct Person {
        name: String,
        account: Account,
    }

    #[derive(ToSchema)]
    struct CreateRequest<T> {
        value: T,
    }

    #[utoipa::path(
        post,
        request_body = CreateRequest<Person>,
        path = "/test-collect-schemas",
        responses(
            (status = 200, description = "success response")
        ),
    )]
    async fn test_collect_schemas(_body: Person) -> &'static str {
        ""
    }

    use utoipa::OpenApi;
    #[derive(OpenApi)]
    #[openapi(paths(test_collect_schemas))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let schemas = doc
        .pointer("/components/schemas")
        .expect("OpenApi must have schemas");

    assert_json_eq!(
        &schemas,
        json!({
            "CreateRequest_Person": {
                "properties": {
                    "value": {
                        "properties": {
                            "name": {
                                "type": "string",
                            },
                            "account": {
                                "$ref": "#/components/schemas/Account"
                            }
                        },
                        "required": ["name", "account"],
                        "type": "object"
                    }
                },
                "required": ["value"],
                "type": "object"
            },
            "Person": {
                "properties": {
                    "name": {
                        "type": "string",
                    },
                    "account": {
                        "$ref": "#/components/schemas/Account"
                    }
                },
                "required": ["name", "account"],
                "type": "object"
            },
            "Account": {
                "properties": {
                    "id": {
                        "type": "integer",
                        "format":  "int32",
                    },
                },
                "required": ["id"],
                "type": "object"
            }
        })
    );
}

#[test]
fn path_derive_with_body_ref_using_as_attribute_schema() {
    #![allow(unused)]

    #[derive(Serialize, serde::Deserialize, Debug, Clone, ToSchema)]
    #[schema(as = types::calculation::calculation_assembly_cost::v1::CalculationAssemblyCostResponse)]
    pub struct CalculationAssemblyCostResponse {
        #[schema(value_type = uuid::Uuid)]
        pub id: String,
    }

    #[utoipa::path(
        get,
        path = "/calculations/assembly-costs",
        responses(
            (status = 200, description = "Get calculated cost of an assembly.",
                body = CalculationAssemblyCostResponse)
        ),
    )]
    async fn handler() {}

    let operation = __path_handler::operation();
    let operation = serde_json::to_value(&operation).expect("operation is JSON serializable");

    assert_json_eq!(
        operation,
        json!({
            "operationId": "handler",
            "responses": {
                "200": {
                    "description": "Get calculated cost of an assembly.",
                    "content": {
                        "application/json": {
                            "schema": {
                                "$ref": "#/components/schemas/types.calculation.calculation_assembly_cost.v1.CalculationAssemblyCostResponse"
                            },
                        }
                    }
                }
            }
        })
    );
}
