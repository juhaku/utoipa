#![cfg(feature = "serde_json")]
use utoipa::OpenApi;

mod common;

macro_rules! test_fn {
    ( module: $name:ident, body: $($body:tt)* ) => {
        #[allow(unused)]
        mod $name {

            struct Foo {
                name: String,
            }
            #[utoipa::path(
                                                post,
                                                path = "/foo",
                                                request_body = $($body)*,
                                                responses = [
                                                    (status = 200, description = "success response")
                                                ]
                                            )]
            fn post_foo() {}
        }
    };
}

test_fn! {
    module: derive_request_body_simple,
    body: Foo
}

#[test]
fn derive_path_request_body_simple_success() {
    #[derive(OpenApi, Default)]
    #[openapi(handlers(derive_request_body_simple::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "paths./foo.post.requestBody.content.application/json.schema.$ref" = r###""#/components/schemas/Foo""###, "Request body content object type"
        "paths./foo.post.requestBody.content.text/plain" = r###"null"###, "Request body content object type not text/plain"
        "paths./foo.post.requestBody.required" = r###"true"###, "Request body required"
        "paths./foo.post.requestBody.description" = r###"null"###, "Request body description"
    }
}

test_fn! {
    module: derive_request_body_simple_array,
    body: [Foo]
}

#[test]
fn derive_path_request_body_simple_array_success() {
    #[derive(OpenApi, Default)]
    #[openapi(handlers(derive_request_body_simple_array::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "paths./foo.post.requestBody.content.application/json.schema.$ref" = r###"null"###, "Request body content object type"
        "paths./foo.post.requestBody.content.application/json.schema.items.$ref" = r###""#/components/schemas/Foo""###, "Request body content items object type"
        "paths./foo.post.requestBody.content.application/json.schema.type" = r###""array""###, "Request body content items type"
        "paths./foo.post.requestBody.content.text/plain" = r###"null"###, "Request body content object type not text/plain"
        "paths./foo.post.requestBody.required" = r###"true"###, "Request body required"
        "paths./foo.post.requestBody.description" = r###"null"###, "Request body description"
    }
}

test_fn! {
    module: derive_request_body_option_array,
    body: Option<[Foo]>
}

#[test]
fn derive_request_body_option_array_success() {
    #[derive(OpenApi, Default)]
    #[openapi(handlers(derive_request_body_option_array::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "paths./foo.post.requestBody.content.application/json.schema.$ref" = r###"null"###, "Request body content object type"
        "paths./foo.post.requestBody.content.application/json.schema.items.$ref" = r###""#/components/schemas/Foo""###, "Request body content items object type"
        "paths./foo.post.requestBody.content.application/json.schema.type" = r###""array""###, "Request body content items type"
        "paths./foo.post.requestBody.content.text/plain" = r###"null"###, "Request body content object type not text/plain"
        "paths./foo.post.requestBody.required" = r###"false"###, "Request body required"
        "paths./foo.post.requestBody.description" = r###"null"###, "Request body description"
    }
}
test_fn! {
    module: derive_request_body_primitive_simple,
    body: String
}

#[test]
fn derive_request_body_primitive_simple_success() {
    #[derive(OpenApi, Default)]
    #[openapi(handlers(derive_request_body_primitive_simple::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "paths./foo.post.requestBody.content.application/json.schema.$ref" = r###"null"###, "Request body content object type not application/json"
        "paths./foo.post.requestBody.content.application/json.schema.items.$ref" = r###"null"###, "Request body content items object type"
        "paths./foo.post.requestBody.content.application/json.schema.type" = r###"null"###, "Request body content items type"
        "paths./foo.post.requestBody.content.text/plain.schema.type" = r###""string""###, "Request body content object type"
        "paths./foo.post.requestBody.required" = r###"true"###, "Request body required"
        "paths./foo.post.requestBody.description" = r###"null"###, "Request body description"
    }
}

test_fn! {
    module: derive_request_body_primitive_simple_array,
    body: [u64]
}

#[test]
fn derive_request_body_primitive_array_success() {
    #[derive(OpenApi, Default)]
    #[openapi(handlers(derive_request_body_primitive_simple_array::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "paths./foo.post.requestBody.content.application/json" = r###"null"###, "Request body content object type not application/json"
        "paths./foo.post.requestBody.content.text/plain.schema.type" = r###""array""###, "Request body content object item type"
        "paths./foo.post.requestBody.content.text/plain.schema.items.type" = r###""integer""###, "Request body content items object type"
        "paths./foo.post.requestBody.content.text/plain.schema.items.format" = r###""int64""###, "Request body content items object format"
        "paths./foo.post.requestBody.required" = r###"true"###, "Request body required"
        "paths./foo.post.requestBody.description" = r###"null"###, "Request body description"
    }
}

test_fn! {
    module: derive_request_body_complex,
    body: (content = Foo, description = "Create new Foo", content_type = "text/xml")
}

#[test]
fn derive_request_body_complex_success() {
    #[derive(OpenApi, Default)]
    #[openapi(handlers(derive_request_body_complex::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "paths./foo.post.requestBody.content.application/json" = r###"null"###, "Request body content object type not application/json"
        "paths./foo.post.requestBody.content.text/xml.schema.$ref" = r###""#/components/schemas/Foo""###, "Request body content object type"
        "paths./foo.post.requestBody.content.text/plain.schema.type" = r###"null"###, "Request body content object item type"
        "paths./foo.post.requestBody.content.text/plain.schema.items.type" = r###"null"###, "Request body content items object type"
        "paths./foo.post.requestBody.required" = r###"true"###, "Request body required"
        "paths./foo.post.requestBody.description" = r###""Create new Foo""###, "Request body description"
    }
}

test_fn! {
    module: derive_request_body_complex_required_explisit,
    body: (content = Option<Foo>, description = "Create new Foo", content_type = "text/xml")
}

#[test]
fn derive_request_body_complex_required_explisit_false_success() {
    #[derive(OpenApi, Default)]
    #[openapi(handlers(derive_request_body_complex_required_explisit::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "paths./foo.post.requestBody.content.application/json" = r###"null"###, "Request body content object type not application/json"
        "paths./foo.post.requestBody.content.text/xml.schema.$ref" = r###""#/components/schemas/Foo""###, "Request body content object type"
        "paths./foo.post.requestBody.content.text/plain.schema.type" = r###"null"###, "Request body content object item type"
        "paths./foo.post.requestBody.content.text/plain.schema.items.type" = r###"null"###, "Request body content items object type"
        "paths./foo.post.requestBody.required" = r###"false"###, "Request body required"
        "paths./foo.post.requestBody.description" = r###""Create new Foo""###, "Request body description"
    }
}

test_fn! {
    module: derive_request_body_complex_primitive_array,
    body: (content = [u32], description = "Create new foo references")
}

#[test]
fn derive_request_body_complex_primitive_array_success() {
    #[derive(OpenApi, Default)]
    #[openapi(handlers(derive_request_body_complex_primitive_array::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "paths./foo.post.requestBody.content.application/json" = r###"null"###, "Request body content object type not application/json"
        "paths./foo.post.requestBody.content.text/plain.schema.type" = r###""array""###, "Request body content object item type"
        "paths./foo.post.requestBody.content.text/plain.schema.items.type" = r###""integer""###, "Request body content items object type"
        "paths./foo.post.requestBody.content.text/plain.schema.items.format" = r###""int32""###, "Request body content items object format"
        "paths./foo.post.requestBody.required" = r###"true"###, "Request body required"
        "paths./foo.post.requestBody.description" = r###""Create new foo references""###, "Request body description"
    }
}
