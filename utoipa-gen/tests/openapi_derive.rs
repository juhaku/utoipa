use std::{borrow::Cow, marker::PhantomData};

use assert_json_diff::{assert_json_eq, assert_json_include};
use serde::Serialize;
use serde_json::{json, Value};
use utoipa::{
    openapi::{RefOr, Response, ResponseBuilder},
    OpenApi, ToResponse,
};
use utoipa_gen::ToSchema;

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

    impl<'r> ToResponse<'r> for MyResponse {
        fn response() -> (&'r str, RefOr<Response>) {
            (
                "MyResponse",
                ResponseBuilder::new().description("Ok").build().into(),
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

#[test]
fn derive_openapi_with_servers() {
    #[derive(OpenApi)]
    #[openapi(
        servers(
            (url = "http://localhost:8989", description = "this is description"),
            (url = "http://api.{username}:{port}", description = "remote api", 
                variables(
                    ("username" = (default = "demo", description = "Default username for API")),
                    ("port" = (default = "8080", enum_values("8080", "5000", "3030"), description = "Supported ports for the API"))
                )
            )
        )
    )]
    struct ApiDoc;

    let value = serde_json::to_value(&ApiDoc::openapi()).unwrap();
    let servers = value.pointer("/servers");

    assert_json_eq!(
        servers,
        json!([
            {
                "description": "this is description",
                "url": "http://localhost:8989"
            },
            {
                "description": "remote api",
                "url": "http://api.{username}:{port}",
                "variables": {
                    "port": {
                        "default": "8080",
                        "enum": [
                            "8080",
                            "5000",
                            "3030"
                        ],
                        "description": "Supported ports for the API"
                    },
                    "username": {
                        "default": "demo",
                        "description": "Default username for API"
                    }
                }
            }
        ])
    )
}

#[test]
fn derive_openapi_with_custom_info() {
    #[derive(OpenApi)]
    #[openapi(info(
        title = "title override",
        description = "description override",
        version = "1.0.0",
        contact(name = "Test")
    ))]
    struct ApiDoc;

    let value = serde_json::to_value(&ApiDoc::openapi()).unwrap();
    let info = value.pointer("/info");

    assert_json_include!(
        actual: info,
        expected:
            json!(
                {
                    "title": "title override",
                    "description": "description override",
                    "license": {
                        "name": "MIT OR Apache-2.0",
                    },
                    "contact": {
                        "name": "Test"
                    },
                    "version": "1.0.0",
                }
            )
    )
}

#[test]
fn derive_openapi_with_include_str_description() {
    #[derive(OpenApi)]
    #[openapi(info(
        title = "title override",
        description = include_str!("./testdata/openapi-derive-info-description"),
        contact(name = "Test")
    ))]
    struct ApiDoc;

    let value = serde_json::to_value(&ApiDoc::openapi()).unwrap();
    let info = value.pointer("/info");

    assert_json_include!(
        actual: info,
        expected:
            json!(
            {
                "title": "title override",
                "description": "this is include description\n",
                "license": {
                    "name": "MIT OR Apache-2.0",
                },
                "contact": {
                    "name": "Test"
                }
            }
            )
    )
}

#[test]
fn derive_openapi_with_generic_response() {
    struct Resp;

    #[derive(Serialize, ToResponse)]
    struct Response<'a, Resp> {
        #[serde(skip)]
        _p: PhantomData<Resp>,
        value: Cow<'a, str>,
    }

    #[derive(OpenApi)]
    #[openapi(components(responses(Response<Resp>)))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let response = doc.pointer("/components/responses/Response");

    assert_json_eq!(
        response,
        json!({
            "content": {
                "application/json": {
                    "schema": {
                        "properties": {
                            "value": {
                                "type": "string"
                            }
                        },
                        "required": ["value"],
                        "type": "object"
                    }
                }
            },
            "description": ""
        })
    )
}

#[test]
fn derive_openapi_with_generic_schema() {
    struct Value;

    #[derive(Serialize, ToSchema)]
    struct Pet<'a, Resp> {
        #[serde(skip)]
        _p: PhantomData<Resp>,
        value: Cow<'a, str>,
    }

    #[derive(OpenApi)]
    #[openapi(components(schemas(Pet<Value>)))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let schema = doc.pointer("/components/schemas/Pet");

    assert_json_eq!(
        schema,
        json!({
            "properties": {
                "value": {
                    "type": "string"
                }
            },
            "required": ["value"],
            "type": "object"
        })
    )
}

#[test]
fn derive_openapi_with_generic_schema_with_as() {
    struct Value;

    #[derive(Serialize, ToSchema)]
    #[schema(as = api::models::Pet)]
    struct Pet<'a, Resp> {
        #[serde(skip)]
        _p: PhantomData<Resp>,
        value: Cow<'a, str>,
    }

    #[derive(OpenApi)]
    #[openapi(components(schemas(Pet<Value>)))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let schema = doc.pointer("/components/schemas/api.models.Pet");

    assert_json_eq!(
        schema,
        json!({
            "properties": {
                "value": {
                    "type": "string"
                }
            },
            "required": ["value"],
            "type": "object"
        })
    )
}
