#![cfg(feature = "serde_json")]
use paste::paste;

mod common;

macro_rules! test_api_fn_doc {
    ( $handler:path, operation: $operation:expr, path: $path:literal ) => {{
        use utoipa::OpenApi;
        #[derive(OpenApi, Default)]
        #[openapi(handlers = [$handler])]
        struct ApiDoc;

        let doc = &serde_json::to_value(ApiDoc::openapi()).unwrap();
        let operation =
            common::get_json_path(doc, &format!("paths.{}.{}", $path, stringify!($operation)));
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
                responses = [
                    (status = 200, description = "success response")
                ],
                $( params = $params, )*
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
                #[openapi(handlers = [
                    [<mod_ $name>]::test_operation
                ])]
                struct ApiDoc;
            }

            let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
            let operation_value = common::get_json_path(&doc, &format!("paths./foo.{}", stringify!($operation)));
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
    params: [("id", description = "Foo bar id")],
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

    common::assert_json_array_len(common::get_json_path(&operation, "parameters"), 1);
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

test_api_fn! {
    name: test_operation3,
    module: derive_path_with_defaults,
    operation: post,
    path: "/foo/bar";
}

#[test]
fn derive_path_with_defaults_success() {
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
    responses = [
        (status = 200, description = "success response")
    ],
    params = [
        ("id" = u64, description = "Foo database id"),
        ("since" = Option<String>, query, description = "Datetime since foo is updated")
    ]
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

    common::assert_json_array_len(common::get_json_path(&operation, "parameters"), 2);
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
