#![cfg(not(feature = "json"))]

macro_rules! test_fn {
    ( module: $name:ident, responses: $($responses:tt)* ) => {
        #[allow(unused)]
        mod $name {
            #[utoipa::path(get,path = "/foo",responses $($responses)*)]
            fn get_foo() {}
        }
    }
}

test_fn! {
    module: response_with_string_example,
    responses: (
        (status = 200, description = "success", body = Foo, example = r#"{"foo": "bar"}"#)
    )
}

#[test]
fn derive_response_with_string_example_compiles_success() {
    use utoipa::OpenApi;

    #[derive(OpenApi, Default)]
    #[openapi(paths(response_with_string_example::get_foo))]
    struct ApiDoc;
    ApiDoc::openapi();
}
