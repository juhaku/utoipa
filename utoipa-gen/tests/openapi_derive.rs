use std::{borrow::Cow, marker::PhantomData};

use insta::assert_json_snapshot;
use serde::Serialize;
use serde_json::Value;
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
            ("token_jwt" = []),
            ("api_key1" = [], "api_key2" = []),
        ))]
    struct ApiDoc;

    let doc_value = serde_json::to_value(ApiDoc::openapi()).unwrap();

    assert_value! {doc_value=>
        "security.[0]" = "{}", "Optional security requirement"
        "security.[1].my_auth.[0]" = r###""read:items""###, "api_oauth first scope"
        "security.[1].my_auth.[1]" = r###""edit:items""###, "api_oauth second scope"
        "security.[2].token_jwt" = "[]", "jwt_token auth scopes"
        "security.[3].api_key1" = "[]", "api_key1 auth scopes"
        "security.[3].api_key2" = "[]", "api_key2 auth scopes"
    }
}

#[test]
fn derive_logical_or_security_requirement() {
    #[derive(Default, OpenApi)]
    #[openapi(security(
        ("oauth" = ["a"]),
        ("oauth" = ["b"]),
    ))]
    struct ApiDoc;

    let doc_value = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let security = doc_value
        .pointer("/security")
        .expect("should have security requirements");

    assert_json_snapshot!(security);
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

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

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
fn derive_openapi_tags_include_str() {
    #[derive(OpenApi)]
    #[openapi(tags(
        (name = "random::api", description = include_str!("testdata/openapi-derive-info-description")),
    ))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "tags.[0].name" = r###""random::api""###, "Tags random_api name"
        "tags.[0].description" = r###""this is include description\n""###, "Tags random_api description"
    }
}

#[test]
fn derive_openapi_tags_with_const_name() {
    const TAG: &str = "random::api";
    #[derive(OpenApi)]
    #[openapi(tags(
        (name = TAG),
    ))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

    assert_value! {doc=>
        "tags.[0].name" = r###""random::api""###, "Tags random_api name"
        "tags.[0].description" = r###"null"###, "Tags random_api description"
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

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

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

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();

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

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
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

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let responses = doc.pointer("/components/responses").unwrap();

    assert_json_snapshot!(responses);
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

    let value = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let servers = value.pointer("/servers");

    assert_json_snapshot!(servers);
}

#[test]
fn derive_openapi_with_licence() {
    #[derive(OpenApi)]
    #[openapi(info(license(name = "licence_name", identifier = "MIT"), version = "1.0.0",))]
    struct ApiDoc;

    let value = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let info = value.pointer("/info/license");

    assert_json_snapshot!(info);
}

#[test]
fn derive_openapi_with_custom_info() {
    #[derive(OpenApi)]
    #[openapi(info(
        terms_of_service = "http://localhost/terms",
        title = "title override",
        description = "description override",
        version = "1.0.0",
        contact(name = "Test")
    ))]
    struct ApiDoc;

    let value = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let info = value.pointer("/info");

    assert_json_snapshot!(info);
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

    let mut doc = ApiDoc::openapi();
    doc.info.version = "static".to_string();

    let value = serde_json::to_value(doc).unwrap();
    let info = value.pointer("/info");

    assert_json_snapshot!(info);
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

    assert_json_snapshot!(response);
}

#[test]
fn derive_openapi_with_generic_schema() {
    #[derive(ToSchema)]
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
    let schema = doc.pointer("/components/schemas/Pet_Value");

    assert_json_snapshot!(schema);
}

#[test]
fn derive_openapi_with_generic_schema_with_as() {
    #[derive(ToSchema)]
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
    let schema = doc.pointer("/components/schemas/api.models.Pet_Value");

    assert_json_snapshot!(schema);
}

#[test]
fn derive_nest_openapi_with_tags() {
    #[utoipa::path(get, path = "/api/v1/status")]
    #[allow(dead_code)]
    fn test_path_status() {}

    mod random {
        #[utoipa::path(get, path = "/random")]
        #[allow(dead_code)]
        fn random() {}
    }

    mod user_api {
        #[utoipa::path(get, path = "/test")]
        #[allow(dead_code)]
        fn user_test_path() {}

        #[derive(super::OpenApi)]
        #[openapi(paths(user_test_path))]
        pub(super) struct UserApi;
    }

    #[utoipa::path(get, path = "/", tag = "mytag", tags = ["yeah", "wowow"])]
    #[allow(dead_code)]
    fn foobar() {}

    #[utoipa::path(get, path = "/another", tag = "mytaganother")]
    #[allow(dead_code)]
    fn foobaranother() {}

    #[utoipa::path(get, path = "/", tags = ["yeah", "wowow"])]
    #[allow(dead_code)]
    fn foobar2() {}

    #[derive(OpenApi)]
    #[openapi(paths(foobar, foobaranother), nest(
        (path = "/nest2", api = FooBarNestedApi)
    ))]
    struct FooBarApi;

    #[derive(OpenApi)]
    #[openapi(paths(foobar2))]
    struct FooBarNestedApi;

    const TAG: &str = "tag1";

    #[derive(OpenApi)]
    #[openapi(
        paths(
            test_path_status,
            random::random
        ),
        nest(
            (path = "/api/v1/user", api = user_api::UserApi, tags = ["user", TAG]),
            (path = "/api/v1/foobar", api = FooBarApi, tags = ["foobarapi"])
        )
    )]
    struct ApiDoc;

    let api = serde_json::to_value(ApiDoc::openapi()).expect("should serialize to value");
    let paths = api.pointer("/paths");

    assert_json_snapshot!(paths);
}

#[test]
fn derive_merge_openapi_with_tags() {
    mod one {
        use utoipa::OpenApi;

        #[derive(OpenApi)]
        #[openapi(paths(api_one_handler))]
        pub struct OneApi;

        #[utoipa::path(get, path = "/api/v1/one")]
        #[allow(dead_code)]
        fn api_one_handler() {}
    }

    mod two {
        use utoipa::OpenApi;

        #[derive(OpenApi)]
        #[openapi(paths(api_two_handler))]
        pub struct TwoApi;

        #[utoipa::path(get, path = "/api/v1/two")]
        #[allow(dead_code)]
        fn api_two_handler() {}
    }

    mod three {
        use utoipa::OpenApi;

        #[derive(OpenApi)]
        #[openapi(paths(api_three_handler))]
        pub struct ThreeApi;

        #[utoipa::path(get, path = "/api/v1/three")]
        #[allow(dead_code)]
        fn api_three_handler() {}
    }

    #[derive(OpenApi)]
    #[openapi(
        merge(
            (api = one::OneApi, tags = ["one"]),
            (api = two::TwoApi, tags = ["two"]),
            (api = three::ThreeApi)
        )
    )]
    struct ApiDoc;

    let api = serde_json::to_value(ApiDoc::openapi()).expect("should serialize to value");
    let paths = api.pointer("/paths");

    assert_json_snapshot!(paths);
}

#[test]
fn openapi_schemas_resolve_generic_enum_schema() {
    #![allow(dead_code)]
    use utoipa::ToSchema;

    #[derive(ToSchema)]
    enum Element<T> {
        One(T),
        Many(Vec<T>),
    }

    #[derive(OpenApi)]
    #[openapi(components(schemas(Element<String>)))]
    struct ApiDoc;

    let doc = ApiDoc::openapi();

    let value = serde_json::to_value(&doc).expect("OpenAPI is JSON serializable");
    let schemas = value.pointer("/components/schemas").unwrap();
    let json = serde_json::to_string_pretty(&schemas).expect("OpenAPI is json serializable");
    println!("{json}");

    assert_json_snapshot!(schemas);
}

#[test]
fn openapi_schemas_resolve_schema_references() {
    #![allow(dead_code)]
    use utoipa::ToSchema;

    #[derive(ToSchema)]
    enum Element<T> {
        One(T),
        Many(Vec<T>),
    }

    #[derive(ToSchema)]
    struct Foobar;

    #[derive(ToSchema)]
    struct Account {
        id: i32,
    }

    #[derive(ToSchema)]
    struct Person {
        name: String,
        foo_bar: Foobar,
        accounts: Vec<Option<Account>>,
    }

    #[derive(ToSchema)]
    struct Yeah {
        name: String,
        foo_bar: Foobar,
        accounts: Vec<Option<Account>>,
    }

    #[derive(ToSchema)]
    struct Boo {
        boo: bool,
    }

    #[derive(ToSchema)]
    struct OneOfOne(Person);

    #[derive(ToSchema)]
    struct OneOfYeah(Yeah);

    #[derive(ToSchema)]
    struct ThisIsNone;

    #[derive(ToSchema)]
    enum EnumMixedContent {
        ContentZero,
        One(Foobar),
        NamedSchema {
            value: Account,
            value2: Boo,
            foo: ThisIsNone,
            int: i32,
            f: bool,
        },
        Many(Vec<Person>),
    }

    #[derive(ToSchema)]
    struct Foob {
        item: Element<String>,
        item2: Element<Yeah>,
    }

    #[derive(OpenApi)]
    #[openapi(components(schemas(Person, Foob, OneOfYeah, OneOfOne, EnumMixedContent, Element<String>)))]
    struct ApiDoc;

    let doc = ApiDoc::openapi();

    let value = serde_json::to_value(&doc).expect("OpenAPI is JSON serializable");
    let schemas = value.pointer("/components").unwrap();

    assert_json_snapshot!(schemas);
}

#[test]
fn openapi_resolvle_recursive_references() {
    #![allow(dead_code)]
    use utoipa::ToSchema;

    #[derive(ToSchema)]
    struct Foobar;

    #[derive(ToSchema)]
    struct Account {
        id: i32,
        foobar: Foobar,
    }

    #[derive(ToSchema)]
    struct Person {
        name: String,
        accounts: Vec<Option<Account>>,
    }

    #[derive(OpenApi)]
    #[openapi(components(schemas(Person)))]
    struct ApiDoc;

    let doc = ApiDoc::openapi();

    let value = serde_json::to_value(doc).expect("OpenAPI is serde serializable");
    let schemas = value
        .pointer("/components/schemas")
        .expect("OpenAPI must have schemas");

    assert_json_snapshot!(schemas);
}

#[test]
fn derive_generic_openapi_component_schemas() {
    #[derive(Serialize, ToSchema)]
    #[schema(as = dto::page::Response)]
    #[serde(rename_all = "camelCase")]
    pub struct Response<T: Serialize> {
        pub list: Vec<T>,
        pub num: u64,
        pub size: u64,
        pub total: u64,
    }

    pub mod unit {
        use serde::Serialize;
        use utoipa::ToSchema;

        #[derive(Serialize, ToSchema)]
        #[schema(as = dto::get::unit::Response)]
        #[serde(rename_all = "camelCase")]
        pub struct Response {
            pub id: i64,
            pub latitude: f64,
            pub longitude: f64,
            pub title: String,
            pub description: Option<String>,
            pub country: String,
            pub region: Option<String>,
            pub city: String,
            pub address: String,
        }
    }

    #[derive(OpenApi)]
    #[openapi(
        components(
            schemas(
                Response<unit::Response>,
            )
        )
    )]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).expect("OpenApi is JSON serializable");
    let schemas = doc.pointer("/components");

    assert_json_snapshot!(schemas)
}
