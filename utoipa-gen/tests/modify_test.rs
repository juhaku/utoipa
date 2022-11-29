use utoipa::{
    openapi::{
        self,
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    },
    Modify, OpenApi,
};

mod common;

#[test]
fn modify_openapi_add_security_schema() {
    #[derive(Default, OpenApi)]
    #[openapi(modifiers(&SecurityAddon))]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut openapi::OpenApi) {
            openapi.components = Some(
                utoipa::openapi::ComponentsBuilder::new()
                    .security_scheme(
                        "api_jwt_token",
                        SecurityScheme::Http(
                            HttpBuilder::new()
                                .scheme(HttpAuthScheme::Bearer)
                                .bearer_format("JWT")
                                .build(),
                        ),
                    )
                    .build(),
            )
        }
    }

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "components.securitySchemes.api_jwt_token.scheme" = r###""bearer""###, "api_jwt_token scheme"
        "components.securitySchemes.api_jwt_token.type" = r###""http""###, "api_jwt_token type"
        "components.securitySchemes.api_jwt_token.bearerFormat" = r###""JWT""###, "api_jwt_token bearerFormat"
    }
}
