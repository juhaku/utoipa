use insta::assert_json_snapshot;
use serde_json::Value;
use utoipa::{OpenApi, Path};
use utoipa_gen::ToSchema;

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

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "paths.~1foo.post.requestBody.content.application~1json.schema.$ref" = r###""#/components/schemas/Foo""###, "Request body content object type"
        "paths.~1foo.post.requestBody.content.text~1plain" = r###"null"###, "Request body content object type not text/plain"
        "paths.~1foo.post.requestBody.required" = r###"true"###, "Request body required"
        "paths.~1foo.post.requestBody.description" = r###"null"###, "Request body description"
    }
}

#[test]
fn derive_path_request_body_simple_array_success() {
    #![allow(unused)]

    #[derive(utoipa::ToSchema)]
    /// Some struct
    pub struct Foo {
        /// Some name
        name: String,
    }
    #[utoipa::path(
        post,
        path = "/foo",
        request_body = [Foo],
        responses(
            (status = 200, description = "success response")
        )
    )]
    fn post_foo() {}
    #[derive(OpenApi, Default)]
    #[openapi(paths(post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

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

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let body = doc.pointer("/paths/~1foo/post/requestBody").unwrap();

    assert_json_snapshot!(body);
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

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "paths.~1foo.post.requestBody.content.application~1json.schema.$ref" = r###"null"###, "Request body content object type not application/json"
        "paths.~1foo.post.requestBody.content.application~1json.schema.items.$ref" = r###"null"###, "Request body content items object type"
        "paths.~1foo.post.requestBody.content.application~1json.schema.type" = r###"null"###, "Request body content items type"
        "paths.~1foo.post.requestBody.content.text~1plain.schema.type" = r###""string""###, "Request body content object type"
        "paths.~1foo.post.requestBody.required" = r###"true"###, "Request body required"
        "paths.~1foo.post.requestBody.description" = r###"null"###, "Request body description"
    }
}

#[test]
fn request_body_with_only_single_content_type() {
    #![allow(unused)]

    #[derive(utoipa::ToSchema)]
    /// Some struct
    pub struct Foo {
        /// Some name
        name: String,
    }
    #[utoipa::path(post, path = "/foo", request_body(content_type = "application/json"))]
    fn post_foo() {}

    #[derive(OpenApi, Default)]
    #[openapi(paths(post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let content = doc
        .pointer("/paths/~1foo/post/requestBody/content")
        .unwrap();

    assert_json_snapshot!(content);
}

test_fn! {
    module: derive_request_body_primitive_simple_array,
    body: = [i64]
}

#[test]
fn derive_request_body_primitive_array_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_request_body_primitive_simple_array::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let content = doc
        .pointer("/paths/~1foo/post/requestBody/content")
        .unwrap();

    assert_json_snapshot!(content);
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

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    let request_body: &Value = doc.pointer("/paths/~1foo/post/requestBody").unwrap();

    assert_json_snapshot!(request_body);
}

#[test]
fn derive_request_body_complex_multi_content_type_success() {
    #![allow(unused)]

    #[derive(utoipa::ToSchema)]
    /// Some struct
    pub struct Foo {
        /// Some name
        name: String,
    }

    #[utoipa::path(
        post,
        path = "/foo",
        request_body(content( ( Foo = "application/json" ), ( Foo = "text/xml") ), description = "Create new Foo" ),
        responses(
            (status = 200, description = "success response")
        )
    )]
    fn post_foo() {}

    let operation = serde_json::to_value(__path_post_foo::operation()).unwrap();
    let request_body: &Value = operation.pointer("/requestBody").unwrap();

    assert_json_snapshot!(request_body);
}

#[test]
fn derive_request_body_with_multiple_content_type_guess_default_content_type() {
    #![allow(unused)]

    #[derive(utoipa::ToSchema)]
    /// Some struct
    pub struct Foo {
        /// Some name
        name: String,
    }

    #[utoipa::path(
        post,
        path = "/foo",
        request_body(content( ( Foo ), ( Foo = "text/xml") ), description = "Create new Foo" ),
        responses(
            (status = 200, description = "success response")
        )
    )]
    fn post_foo() {}

    let operation = serde_json::to_value(__path_post_foo::operation()).unwrap();
    let request_body: &Value = operation.pointer("/requestBody").unwrap();

    assert_json_snapshot!(request_body);
}

#[test]
fn multiple_request_body_with_only_content_type() {
    #![allow(unused)]

    #[derive(utoipa::ToSchema)]
    /// Some struct
    pub struct Foo {
        /// Some name
        name: String,
    }

    #[utoipa::path(
        post,
        path = "/foo",
        request_body(content( ( "application/json" ), ( Foo = "text/xml") ), description = "Create new Foo" ),
        responses(
            (status = 200, description = "success response")
        )
    )]
    fn post_foo() {}

    let operation = serde_json::to_value(__path_post_foo::operation()).unwrap();
    let request_body = operation.pointer("/requestBody").unwrap();

    assert_json_snapshot!(request_body);
}

#[test]
fn multiple_content_with_examples() {
    #![allow(unused)]

    #[derive(utoipa::ToSchema)]
    /// Some struct
    pub struct Foo {
        /// Some name
        name: String,
    }

    #[utoipa::path(
        post,
        path = "/foo",
        request_body(
            description = "Create new Foo",
            content(
                ( Foo, examples(
                    ("example1" = (value = json!("Foo name"), description = "Foo name example")  ),
                    ("example2" = (value = json!("example value"), description = "example value") ),
                    ),
                ),
                ( Foo = "text/xml", example = "Value" )
            ),
        ),
        responses(
            (status = 200, description = "success response")
        )
    )]
    fn post_foo() {}

    let operation = serde_json::to_value(__path_post_foo::operation()).unwrap();
    let request_body = operation.pointer("/requestBody").unwrap();

    assert_json_snapshot!(request_body);
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

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    let request_body: &Value = doc.pointer("/paths/~1foo/post/requestBody").unwrap();

    assert_json_snapshot!(request_body);
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

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    let request_body: &Value = doc.pointer("/paths/~1foo/post/requestBody").unwrap();

    assert_json_snapshot!(request_body);
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

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    let request_body: &Value = doc.pointer("/paths/~1foo/post/requestBody").unwrap();

    assert_json_snapshot!(request_body);
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

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let request_body: &Value = doc.pointer("/paths/~1foo/post/requestBody").unwrap();

    assert_json_snapshot!(request_body);
}

#[test]
fn derive_request_body_complex_required_explicit_false_success() {
    #![allow(unused)]

    #[derive(utoipa::ToSchema)]
    /// Some struct
    pub struct Foo {
        /// Some name
        name: String,
    }
    #[utoipa::path(
        post,
        path = "/foo",
        request_body(content = Option<Foo>, description = "Create new Foo", content_type = "text/xml"),
        responses(
            (status = 200, description = "success response")
        )
    )]
    fn post_foo() {}
    #[derive(OpenApi, Default)]
    #[openapi(paths(post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let body = doc.pointer("/paths/~1foo/post/requestBody").unwrap();

    assert_json_snapshot!(body);
}

test_fn! {
    module: derive_request_body_complex_primitive_array,
    body: (content = [i32], description = "Create new foo references")
}

#[test]
fn derive_request_body_complex_primitive_array_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_request_body_complex_primitive_array::post_foo))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let content = doc
        .pointer("/paths/~1foo/post/requestBody/content")
        .unwrap();
    assert_json_snapshot!(content);
}

#[test]
fn derive_request_body_ref_path_success() {
    /// Some struct
    #[derive(ToSchema)]
    #[schema(as = path::to::Foo)]
    #[allow(unused)]
    pub struct Foo {
        /// Some name
        name: String,
    }

    #[utoipa::path(
            post,
            path = "/foo",
            request_body = Foo,
            responses(
                (status = 200, description = "success response")
            )
        )]
    #[allow(unused)]
    fn post_foo() {}

    #[derive(OpenApi, Default)]
    #[openapi(paths(post_foo), components(schemas(Foo)))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let schemas = doc.pointer("/components/schemas").unwrap();
    assert!(schemas.get("path.to.Foo").is_some());

    let component_ref: &str = doc
        .pointer("/paths/~1foo/post/requestBody/content/application~1json/schema/$ref")
        .unwrap()
        .as_str()
        .unwrap();
    assert_eq!(component_ref, "#/components/schemas/path.to.Foo");
}

#[test]
fn unit_type_request_body() {
    #[utoipa::path(
        post,
        path = "/unit_type_test",
        request_body = ()
    )]
    #[allow(unused)]
    fn unit_type_test() {}

    #[derive(OpenApi)]
    #[openapi(paths(unit_type_test))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let request_body = doc
        .pointer("/paths/~1unit_type_test/post/requestBody")
        .unwrap();

    assert_json_snapshot!(request_body);
}

#[test]
fn request_body_with_example() {
    #[derive(ToSchema)]
    #[allow(unused)]
    struct Foo<'v> {
        value: &'v str,
    }

    #[utoipa::path(get, path = "/item", request_body(content = Foo, example = json!({"value": "this is value"})))]
    #[allow(dead_code)]
    fn get_item() {}

    #[derive(OpenApi)]
    #[openapi(components(schemas(Foo)), paths(get_item))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    let content = doc
        .pointer("/paths/~1item/get/requestBody/content")
        .unwrap();
    assert_json_snapshot!(content);
}

#[test]
fn request_body_with_examples() {
    #[derive(ToSchema)]
    #[allow(unused)]
    struct Foo<'v> {
        value: &'v str,
    }

    #[utoipa::path(
        get,
        path = "/item",
        request_body(content = Foo,
            examples(
                ("Value1" = (value = json!({"value": "this is value"}) ) ),
                ("Value2" = (value = json!({"value": "this is value2"}) ) )
            )
        )
    )]
    #[allow(dead_code)]
    fn get_item() {}

    #[derive(OpenApi)]
    #[openapi(components(schemas(Foo)), paths(get_item))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    let content = doc
        .pointer("/paths/~1item/get/requestBody/content")
        .unwrap();
    assert_json_snapshot!(content);
}

#[test]
fn request_body_with_binary() {
    #[utoipa::path(get, path = "/item", request_body(content = [u8]))]
    #[allow(dead_code)]
    fn get_item() {}

    #[derive(OpenApi)]
    #[openapi(paths(get_item))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    let content = doc
        .pointer("/paths/~1item/get/requestBody/content")
        .unwrap();

    assert_json_snapshot!(content);
}

#[test]
fn request_body_with_external_ref() {
    #[utoipa::path(get, path = "/item", request_body(content = ref("./MyUser.json")))]
    #[allow(dead_code)]
    fn get_item() {}

    #[derive(utoipa::OpenApi)]
    #[openapi(paths(get_item))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    let content = doc
        .pointer("/paths/~1item/get/requestBody/content")
        .unwrap();
    assert_json_snapshot!(content);
}

#[test]
fn request_body_with_extensions() {
  #[utoipa::path(get, path = "/pets",
    request_body(
      extensions(
        ("x-request-body-ext1" = json!( { "type": "request_body" }))
      ),
      content(
        ( "text/plain",
          extensions(
            ("x-request-body-ext2" = json!( { "type": "request_body/text/plain" }) )
          )
        )
      )
    )
  )]
  #[allow(unused)]
  fn get_pets() {}
  let operation = __path_get_pets::operation();
  let value = serde_json::to_value(operation).expect("operation is JSON serializable");
  let request_body = value.pointer("/requestBody").unwrap();
  assert_json_snapshot!(request_body);
}
