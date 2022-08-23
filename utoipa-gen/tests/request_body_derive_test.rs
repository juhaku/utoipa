#![cfg(feature = "json")]
use assert_json_diff::assert_json_eq;
use serde_json::{json, Value};
use utoipa::OpenApi;

mod common;

macro_rules! test_fn {
    ( module: $name:ident, body: $($body:tt)* ) => {
        #[allow(unused)]
        mod $name {
            #[derive(utoipa::ToSchema)]
            /// Some struct
            pub struct Foo {
                /// Some name
                name: String,
            }
            #[utoipa::path(
                post,
                path = "/foo",
                request_body $($body)*,
                responses(
                    (status = 200, description = "success response")
                )
            )]
            fn post_foo() {}
        }
    };
}

test_fn! {
    module: derive_request_body_simple,
    body: = Foo
}

#[test]
fn derive_path_request_body_simple_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_request_body_simple::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "paths.~1foo.post.requestBody.content.application~1json.schema.$ref" = r###""#/components/schemas/Foo""###, "Request body content object type"
        "paths.~1foo.post.requestBody.content.text~1plain" = r###"null"###, "Request body content object type not text/plain"
        "paths.~1foo.post.requestBody.required" = r###"true"###, "Request body required"
        "paths.~1foo.post.requestBody.description" = r###"null"###, "Request body description"
    }
}

test_fn! {
    module: derive_request_body_simple_array,
    body: = [Foo]
}

#[test]
fn derive_path_request_body_simple_array_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_request_body_simple_array::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "paths.~1foo.post.requestBody.content.application~1json.schema.$ref" = r###"null"###, "Request body content object type"
        "paths.~1foo.post.requestBody.content.application~1json.schema.items.$ref" = r###""#/components/schemas/Foo""###, "Request body content items object type"
        "paths.~1foo.post.requestBody.content.application~1json.schema.type" = r###""array""###, "Request body content items type"
        "paths.~1foo.post.requestBody.content.text~1plain" = r###"null"###, "Request body content object type not text/plain"
        "paths.~1foo.post.requestBody.required" = r###"true"###, "Request body required"
        "paths.~1foo.post.requestBody.description" = r###"null"###, "Request body description"
    }
}

test_fn! {
    module: derive_request_body_option_array,
    body: = Option<[Foo]>
}

#[test]
fn derive_request_body_option_array_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_request_body_option_array::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "paths.~1foo.post.requestBody.content.application~1json.schema.$ref" = r###"null"###, "Request body content object type"
        "paths.~1foo.post.requestBody.content.application~1json.schema.items.$ref" = r###""#/components/schemas/Foo""###, "Request body content items object type"
        "paths.~1foo.post.requestBody.content.application~1json.schema.type" = r###""array""###, "Request body content items type"
        "paths.~1foo.post.requestBody.content.text~1plain" = r###"null"###, "Request body content object type not text/plain"
        "paths.~1foo.post.requestBody.required" = r###"false"###, "Request body required"
        "paths.~1foo.post.requestBody.description" = r###"null"###, "Request body description"
    }
}

test_fn! {
    module: derive_request_body_primitive_simple,
    body: = String
}

#[test]
fn derive_request_body_primitive_simple_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_request_body_primitive_simple::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "paths.~1foo.post.requestBody.content.application~1json.schema.$ref" = r###"null"###, "Request body content object type not application/json"
        "paths.~1foo.post.requestBody.content.application~1json.schema.items.$ref" = r###"null"###, "Request body content items object type"
        "paths.~1foo.post.requestBody.content.application~1json.schema.type" = r###"null"###, "Request body content items type"
        "paths.~1foo.post.requestBody.content.text~1plain.schema.type" = r###""string""###, "Request body content object type"
        "paths.~1foo.post.requestBody.required" = r###"true"###, "Request body required"
        "paths.~1foo.post.requestBody.description" = r###"null"###, "Request body description"
    }
}

test_fn! {
    module: derive_request_body_primitive_simple_array,
    body: = [u64]
}

#[test]
fn derive_request_body_primitive_array_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_request_body_primitive_simple_array::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "paths.~1foo.post.requestBody.content.application~1json" = r###"null"###, "Request body content object type not application/json"
        "paths.~1foo.post.requestBody.content.text~1plain.schema.type" = r###""array""###, "Request body content object item type"
        "paths.~1foo.post.requestBody.content.text~1plain.schema.items.type" = r###""integer""###, "Request body content items object type"
        "paths.~1foo.post.requestBody.content.text~1plain.schema.items.format" = r###""int64""###, "Request body content items object format"
        "paths.~1foo.post.requestBody.required" = r###"true"###, "Request body required"
        "paths.~1foo.post.requestBody.description" = r###"null"###, "Request body description"
    }
}

test_fn! {
    module: derive_request_body_complex,
    body: (content = Foo, description = "Create new Foo", content_type = "text/xml")
}

#[test]
fn derive_request_body_complex_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_request_body_complex::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    let request_body: &Value = doc.pointer("/paths/~1foo/post/requestBody").unwrap();

    assert_json_eq!(
        request_body,
        json!({
            "content": {
                "text/xml": {
                    "schema": {
                        "$ref": "#/components/schemas/Foo"
                    }
                }
            },
            "description": "Create new Foo",
            "required": true
        })
    );
}

test_fn! {
    module: derive_request_body_complex_inline,
    body: (content = inline(Foo), description = "Create new Foo", content_type = "text/xml")
}

#[test]
fn derive_request_body_complex_success_inline() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_request_body_complex_inline::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    let request_body: &Value = doc.pointer("/paths/~1foo/post/requestBody").unwrap();

    assert_json_eq!(
        request_body,
        json!({
            "content": {
                "text/xml": {
                    "schema": {
                        "description": "Some struct",
                        "properties": {
                            "name": {
                                "description": "Some name",
                                "type": "string"
                            }
                        },
                        "required": [
                            "name"
                        ],
                        "type": "object"
                    }
                }
            },
            "description": "Create new Foo",
            "required": true
        })
    );
}

test_fn! {
    module: derive_request_body_complex_array,
    body: (content = [Foo], description = "Create new Foo", content_type = "text/xml")
}

#[test]
fn derive_request_body_complex_success_array() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_request_body_complex_array::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    let request_body: &Value = doc.pointer("/paths/~1foo/post/requestBody").unwrap();

    assert_json_eq!(
        request_body,
        json!({
            "content": {
                "text/xml": {
                    "schema": {
                        "items": {
                            "$ref": "#/components/schemas/Foo"
                        },
                        "type": "array"
                    }
                }
            },
            "description": "Create new Foo",
            "required": true
        })
    );
}

test_fn! {
    module: derive_request_body_complex_inline_array,
    body: (content = inline([Foo]), description = "Create new Foo", content_type = "text/xml")
}

#[test]
fn derive_request_body_complex_success_inline_array() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_request_body_complex_inline_array::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    let request_body: &Value = doc.pointer("/paths/~1foo/post/requestBody").unwrap();

    assert_json_eq!(
        request_body,
        json!({
            "content": {
                "text/xml": {
                    "schema": {
                        "items": {
                            "description": "Some struct",
                            "properties": {
                                "name": {
                                    "description": "Some name",
                                    "type": "string"
                                }
                            },
                            "required": [
                                "name"
                            ],
                            "type": "object"
                        },
                        "type": "array"
                    }
                }
            },
            "description": "Create new Foo",
            "required": true
        })
    );
}

test_fn! {
    module: derive_request_body_simple_inline,
    body: = inline(Foo)
}

#[test]
fn derive_request_body_simple_inline_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_request_body_simple_inline::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();
    let request_body: &Value = doc.pointer("/paths/~1foo/post/requestBody").unwrap();

    assert_json_eq!(
        request_body,
        json!({
            "content": {
                "application/json": {
                    "schema": {
                        "description": "Some struct",
                        "properties": {
                            "name": {
                                "description": "Some name",
                                "type": "string"
                            }
                        },
                        "required": [
                            "name"
                        ],
                        "type": "object"
                    }
                }
            },
            "required": true
        })
    );
}

test_fn! {
    module: derive_request_body_complex_required_explisit,
    body: (content = Option<Foo>, description = "Create new Foo", content_type = "text/xml")
}

#[test]
fn derive_request_body_complex_required_explisit_false_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_request_body_complex_required_explisit::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "paths.~1foo.post.requestBody.content.application~1json" = r###"null"###, "Request body content object type not application/json"
        "paths.~1foo.post.requestBody.content.text~1xml.schema.$ref" = r###""#/components/schemas/Foo""###, "Request body content object type"
        "paths.~1foo.post.requestBody.content.text~1plain.schema.type" = r###"null"###, "Request body content object item type"
        "paths.~1foo.post.requestBody.content.text~1plain.schema.items.type" = r###"null"###, "Request body content items object type"
        "paths.~1foo.post.requestBody.required" = r###"false"###, "Request body required"
        "paths.~1foo.post.requestBody.description" = r###""Create new Foo""###, "Request body description"
    }
}

test_fn! {
    module: derive_request_body_complex_primitive_array,
    body: (content = [u32], description = "Create new foo references")
}

#[test]
fn derive_request_body_complex_primitive_array_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_request_body_complex_primitive_array::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "paths.~1foo.post.requestBody.content.application~1json" = r###"null"###, "Request body content object type not application/json"
        "paths.~1foo.post.requestBody.content.text~1plain.schema.type" = r###""array""###, "Request body content object item type"
        "paths.~1foo.post.requestBody.content.text~1plain.schema.items.type" = r###""integer""###, "Request body content items object type"
        "paths.~1foo.post.requestBody.content.text~1plain.schema.items.format" = r###""int32""###, "Request body content items object format"
        "paths.~1foo.post.requestBody.required" = r###"true"###, "Request body required"
        "paths.~1foo.post.requestBody.description" = r###""Create new foo references""###, "Request body description"
    }
}

test_fn! {
    module: derive_request_body_primitive_ref_path,
    body: = path::to::Foo
}

#[test]
fn derive_request_body_primitive_ref_path_success() {
    #[derive(OpenApi, Default)]
    #[openapi(
        paths(derive_request_body_primitive_ref_path::post_foo),
        components(schemas(derive_request_body_primitive_ref_path::Foo as path::to::Foo))
    )]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();
    let schemas = doc.pointer("/components/schemas").unwrap();
    assert!(schemas.get("path.to.Foo").is_some());

    let component_ref: &str = doc
        .pointer("/paths/~1foo/post/requestBody/content/application~1json/schema/$ref")
        .unwrap()
        .as_str()
        .unwrap();
    assert_eq!(component_ref, "#/components/schemas/path.to.Foo");
}
