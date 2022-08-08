#![cfg(feature = "json")]

use assert_json_diff::assert_json_eq;
use serde_json::{json, Value};
use utoipa::{
    openapi::{Response, ResponseBuilder},
    OpenApi, ToResponse,
};

mod common;

#[test]
fn derive_openapi_with_security_requirement() {
    #[derive(Default, OpenApi)]
    #[openapi(security(
            (),
            ("my_auth" = ["read:items", "edit:items"]),
            ("token_jwt" = [])
        ))]
    struct ApiDoc;

    let doc_value = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc_value=>
        "security.[0]" = "{}", "Optional security requirement"
        "security.[1].my_auth.[0]" = r###""read:items""###, "api_oauth first scope"
        "security.[1].my_auth.[1]" = r###""edit:items""###, "api_oauth second scope"
        "security.[2].token_jwt" = "[]", "jwt_token auth scopes"
    }
}

#[test]
fn derive_openapi_tags() {
    #[derive(OpenApi)]
    #[openapi(tags(
        (name = "random::api", description = "this is random api description"),
        (name = "pets::api", description = "api all about pets", external_docs(
            url = "http://localhost", description = "Find more about pets")
        )
    ))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "tags.[0].name" = r###""random::api""###, "Tags random_api name"
        "tags.[0].description" = r###""this is random api description""###, "Tags random_api description"
        "tags.[0].externalDocs" = r###"null"###, "Tags random_api external docs"
        "tags.[1].name" = r###""pets::api""###, "Tags pets_api name"
        "tags.[1].description" = r###""api all about pets""###, "Tags pets_api description"
        "tags.[1].externalDocs.url" = r###""http://localhost""###, "Tags pets_api external docs url"
        "tags.[1].externalDocs.description" = r###""Find more about pets""###, "Tags pets_api external docs description"
    }
}

#[test]
fn derive_openapi_with_external_docs() {
    #[derive(OpenApi)]
    #[openapi(external_docs(
        url = "http://localhost.more.about.api",
        description = "Find out more"
    ))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "externalDocs.url" = r###""http://localhost.more.about.api""###, "External docs url"
        "externalDocs.description" = r###""Find out more""###, "External docs description"
    }
}

#[test]
fn derive_openapi_with_external_docs_only_url() {
    #[derive(OpenApi)]
    #[openapi(external_docs(url = "http://localhost.more.about.api"))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "externalDocs.url" = r###""http://localhost.more.about.api""###, "External docs url"
        "externalDocs.description" = r###"null"###, "External docs description"
    }
}

#[test]
fn derive_openapi_with_components_in_different_module() {
    mod custom {
        use utoipa::ToSchema;

        #[derive(ToSchema)]
        #[allow(unused)]
        pub(super) struct Todo {
            name: String,
        }
    }

    #[derive(OpenApi)]
    #[openapi(components(schemas(custom::Todo)))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();
    let todo = doc.pointer("/components/schemas/Todo").unwrap();

    assert_ne!(
        todo,
        &Value::Null,
        "Expected components.schemas.Todo not to be null"
    );
}

#[test]
fn derive_openapi_with_responses() {
    #[allow(unused)]
    struct MyResponse;

    impl ToResponse for MyResponse {
        fn response() -> (String, Response) {
            (
                "MyResponse".to_string(),
                ResponseBuilder::new().description("Ok").build(),
            )
        }
    }

    #[derive(OpenApi)]
    #[openapi(components(responses(MyResponse)))]
    struct ApiDoc;

    let doc = serde_json::to_value(&ApiDoc::openapi()).unwrap();
    let responses = doc.pointer("/components/responses").unwrap();

    assert_json_eq!(
        responses,
        json!({
            "MyResponse": {
                "description": "Ok"
            },
        })
    )
}
