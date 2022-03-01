#![cfg(feature = "json")]

use utoipa::{
    openapi::{
        self,
        security::{Http, HttpAuthenticationType, SecuritySchema},
    },
    Modify, OpenApi,
};

mod common;

#[test]
fn modify_openapi_add_security_schema() {
    #[derive(Default, OpenApi)]
    #[openapi(modifiers = [&SecurityAddon])]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut openapi::OpenApi) {
            if let Some(schema) = openapi.components.as_mut() {
                schema.add_security_schema(
                    "api_jwt_token",
                    SecuritySchema::Http(
                        Http::new(HttpAuthenticationType::Bearer).with_bearer_format("JWT"),
                    ),
                )
            }
        }
    }

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "components.securitySchemas.api_jwt_token.scheme" = r###""bearer""###, "api_jwt_token scheme"
        "components.securitySchemas.api_jwt_token.type" = r###""http""###, "api_jwt_token type"
        "components.securitySchemas.api_jwt_token.bearerFormat" = r###""JWT""###, "api_jwt_token bearerFormat"
    }
}
