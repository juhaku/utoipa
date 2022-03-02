#![cfg(feature = "json")]

use utoipa::OpenApi;

mod common;

#[test]
fn derive_openapi_with_security_requirement() {
    #[derive(Default, OpenApi)]
    #[openapi(security = [
            (),
            ("my_auth" = ["read:items", "edit:items"]),
            ("token_jwt" = [])
        ])]
    struct ApiDoc;

    let doc_value = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc_value=>
        "security.[0]" = "{}", "Optional security requirement"
        "security.[1].my_auth.[0]" = r###""read:items""###, "api_oauth first scope"
        "security.[1].my_auth.[1]" = r###""edit:items""###, "api_oauth second scope"
        "security.[2].token_jwt" = "[]", "jwt_token auth scopes"
    }
}
