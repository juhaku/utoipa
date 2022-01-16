mod common;

macro_rules! test_fn {
    ( module: $name:ident, responses: $responses:expr ) => {
        #[allow(unused)]
        mod $name {
            #[utoipa::path(get,path = "/foo",responses = $responses)]
            fn get_foo() {}
        }
    };
}

macro_rules! api_doc {
    ( module: $module:expr ) => {{
        use utoipa::OpenApi;
        #[derive(OpenApi, Default)]
        #[openapi(handler_files = [], handlers = [$module::get_foo])]
        struct ApiDoc;

        let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();
        common::get_json_path(&doc, "paths./foo.get").clone()
    }};
}

test_fn! {
    module: simple_success_response,
    responses: [
        (status = 200, description = "success")
    ]
}

#[test]
fn derive_path_with_simple_success_response() {
    let doc = api_doc!(module: simple_success_response);

    assert_value! {doc=>
        "responses.200.description" = r#""success""#, "Response description"
        "responses.200.content" = r#"null"#, "Response content"
        "responses.200.headers" = r#"null"#, "Response headers"
    }
}

test_fn! {
    module: multiple_simple_responses,
    responses: [
        (status = 200, description = "success"),
        (status = 401, description = "unauthorized"),
        (status = 404, description = "not found"),
        (status = 500, description = "server error")
    ]
}

#[test]
fn derive_path_with_multiple_simple_responses() {
    let doc = api_doc!(module: multiple_simple_responses);

    assert_value! {doc=>
        "responses.200.description" = r#""success""#, "Response description"
        "responses.200.content" = r#"null"#, "Response content"
        "responses.200.headers" = r#"null"#, "Response headers"
        "responses.401.description" = r#""unauthorized""#, "Response description"
        "responses.401.content" = r#"null"#, "Response content"
        "responses.401.headers" = r#"null"#, "Response headers"
        "responses.404.description" = r#""not found""#, "Response description"
        "responses.404.content" = r#"null"#, "Response content"
        "responses.404.headers" = r#"null"#, "Response headers"
        "responses.500.description" = r#""server error""#, "Response description"
        "responses.500.content" = r#"null"#, "Response content"
        "responses.500.headers" = r#"null"#, "Response headers"
    }
}

macro_rules! test_response_types {
    ( $( $name:ident=> $(body: $expected:expr,)? $( $content_type:literal, )? $( headers: $headers:expr, )?
        assert: $( $path:literal = $expection:literal, $comment:literal )* )* ) => {
        $(
            paste::paste! {
                test_fn! {
                    module: [<mod_ $name>],
                    responses: [
                        (status = 200, description = "success",
                            $(body = $expected ,)*
                            $(content_type = $content_type,)*
                            $(headers = $headers)*),
                    ]
                }
            }

            #[test]
            fn $name() {
                paste::paste! {
                    let doc = api_doc!(module: [<mod_ $name>]);
                }

                assert_value! {doc=>
                    "responses.200.description" = r#""success""#, "Response description"
                    $($path = $expection, $comment)*
                }
            }
        )*
    };
}

#[allow(unused)]
struct Foo {
    name: String,
}

test_response_types! {
primitive_string_body => body: String, assert:
    "responses.200.content.text/plain.schema.type" = r#""string""#, "Response content type"
    "responses.200.headers" = r###"null"###, "Response headers"
primitive_string_sclice_body => body: [String], assert:
    "responses.200.content.text/plain.schema.items.type" = r#""string""#, "Response content items type"
    "responses.200.content.text/plain.schema.type" = r#""array""#, "Response content type"
    "responses.200.headers" = r###"null"###, "Response headers"
primitive_integer_slice_body => body: [i32], assert:
    "responses.200.content.text/plain.schema.items.type" = r#""integer""#, "Response content items type"
    "responses.200.content.text/plain.schema.items.format" = r#""int32""#, "Response content items format"
    "responses.200.content.text/plain.schema.type" = r#""array""#, "Response content type"
    "responses.200.headers" = r###"null"###, "Response headers"
primitive_integer_body => body: i64, assert:
    "responses.200.content.text/plain.schema.type" = r#""integer""#, "Response content type"
    "responses.200.content.text/plain.schema.format" = r#""int64""#, "Response content format"
    "responses.200.headers" = r###"null"###, "Response headers"
primitive_big_integer_body => body: u128, assert:
    "responses.200.content.text/plain.schema.type" = r#""integer""#, "Response content type"
    "responses.200.content.text/plain.schema.format" = r#"null"#, "Response content format"
    "responses.200.headers" = r###"null"###, "Response headers"
primitive_bool_body => body: bool, assert:
    "responses.200.content.text/plain.schema.type" = r#""boolean""#, "Response content type"
    "responses.200.headers" = r###"null"###, "Response headers"
object_body => body: Foo, assert:
    "responses.200.content.application/json.schema.$ref" = r###""#/components/schemas/Foo""###, "Response content type"
    "responses.200.headers" = r###"null"###, "Response headers"
object_slice_body => body: [Foo], assert:
    "responses.200.content.application/json.schema.type" = r###""array""###, "Response content type"
    "responses.200.content.application/json.schema.items.$ref" = r###""#/components/schemas/Foo""###, "Response content items type"
    "responses.200.headers" = r###"null"###, "Response headers"
object_body_override_content_type_to_xml => body: Foo, "text/xml", assert:
    "responses.200.content.application/json.schema.$ref" = r###"null"###, "Response content type"
    "responses.200.content.text/xml.schema.$ref" = r###""#/components/schemas/Foo""###, "Response content type"
    "responses.200.headers" = r###"null"###, "Response headers"
object_body_with_simple_header => body: Foo, headers: [
    ("xsrf-token")
], assert:
    "responses.200.content.application/json.schema.$ref" = r###""#/components/schemas/Foo""###, "Response content type"
    "responses.200.headers.xsrf-token.schema.type" = r###""string""###, "xsrf-token header type"
    "responses.200.headers.xsrf-token.description" = r###"null"###, "xsrf-token header description"
object_body_with_multiple_headers => body: Foo, headers: [
    ("xsrf-token"),
    ("another-header")
], assert:
    "responses.200.content.application/json.schema.$ref" = r###""#/components/schemas/Foo""###, "Response content type"
    "responses.200.headers.xsrf-token.schema.type" = r###""string""###, "xsrf-token header type"
    "responses.200.headers.xsrf-token.description" = r###"null"###, "xsrf-token header description"
    "responses.200.headers.another-header.schema.type" = r###""string""###, "another-header header type"
    "responses.200.headers.another-header.description" = r###"null"###, "another-header header description"
object_body_with_header_with_type => body: Foo, headers: [
    ("random-digits" = [u64]),
], assert:
    "responses.200.content.application/json.schema.$ref" = r###""#/components/schemas/Foo""###, "Response content type"
    "responses.200.headers.random-digits.schema.type" = r###""array""###, "random-digits header type"
    "responses.200.headers.random-digits.description" = r###"null"###, "random-digits header description"
    "responses.200.headers.random-digits.schema.items.type" = r###""integer""###, "random-digits header items type"
    "responses.200.headers.random-digits.schema.items.format" = r###""int64""###, "random-digits header items format"
response_no_body_with_complex_header_with_description => headers: [
    ("random-digits" = [u64], description = "Random digits response header"),
], assert:
    "responses.200.content" = r###"null"###, "Response content type"
    "responses.200.headers.random-digits.description" = r###""Random digits response header""###, "random-digits header description"
    "responses.200.headers.random-digits.schema.type" = r###""array""###, "random-digits header type"
    "responses.200.headers.random-digits.schema.items.type" = r###""integer""###, "random-digits header items type"
    "responses.200.headers.random-digits.schema.items.format" = r###""int64""###, "random-digits header items format"
}
