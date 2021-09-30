// use utoipa::openapi_spec;
use utoipa::OpenApi;

// fn foo() {}

// #[test]
// fn expand_openapi_spec_macro() {
//     openapi_spec!("tests/utoipa_gen_test.rs");

//     get_pkg_info()
// }

#[test]
fn test_derive_openapi() {
    #[derive(OpenApi, Default)]
    #[openapi(handler_files = ["tests/utoipa_gen_test.rs"])]
    struct ApiDoc;

    let doc = ApiDoc {};

    println!("{:?}", ApiDoc::openapi().to_json())
}
