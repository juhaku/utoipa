use insta::assert_json_snapshot;
use utoipa::{OpenApi, Path};

mod common;

mod derive_params_all_options {
    /// Get foo by id
    ///
    /// Get foo by id long description
    #[utoipa::path(
        get,
        path = "/foo/{id}",
        responses(
            (status = 200, description = "success"),
        ),
        params(
            ("id" = i32, Path, deprecated, description = "Search foos by ids"),
        )
    )]
    #[allow(unused)]
    async fn get_foo_by_id(id: i32) -> i32 {
        id
    }
}

#[test]
fn derive_path_parameters_with_all_options_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_params_all_options::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc.pointer("/paths/~1foo~1{id}/get/parameters").unwrap();

    common::assert_json_array_len(parameters, 1);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#""Search foos by ids""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"true"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int32""#, "Parameter schema format"
    };
}

mod derive_params_minimal {
    /// Get foo by id
    ///
    /// Get foo by id long description
    #[utoipa::path(
        get,
        path = "/foo/{id}",
        responses(
            (status = 200, description = "success"),
        ),
        params(
            ("id" = i32, description = "Search foos by ids"),
        )
    )]
    #[allow(unused)]
    async fn get_foo_by_id(id: i32) -> i32 {
        id
    }
}

#[test]
fn derive_path_parameters_minimal_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_params_minimal::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc.pointer("/paths/~1foo~1{id}/get/parameters").unwrap();

    common::assert_json_array_len(parameters, 1);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#""Search foos by ids""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"null"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int32""#, "Parameter schema format"
    };
}

mod derive_params_multiple {
    /// Get foo by id
    ///
    /// Get foo by id long description
    #[utoipa::path(
        get,
        path = "/foo/{id}/{digest}",
        responses(
            (status = 200, description = "success"),
        ),
        params(
            ("id" = i32, description = "Foo id"),
            ("digest" = String, description = "Digest of foo"),
        )
    )]
    #[allow(unused)]
    async fn get_foo_by_id(id: i32, digest: String) -> String {
        format!("{:?}{:?}", &id, &digest)
    }
}

#[test]
fn derive_path_parameter_multiple_success() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_params_multiple::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc
        .pointer("/paths/~1foo~1{id}~1{digest}/get/parameters")
        .unwrap();

    common::assert_json_array_len(parameters, 2);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#""Foo id""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"null"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int32""#, "Parameter schema format"

        "[1].in" = r#""path""#, "Parameter in"
        "[1].name" = r#""digest""#, "Parameter name"
        "[1].description" = r#""Digest of foo""#, "Parameter description"
        "[1].required" = r#"true"#, "Parameter required"
        "[1].deprecated" = r#"null"#, "Parameter deprecated"
        "[1].schema.type" = r#""string""#, "Parameter schema type"
        "[1].schema.format" = r#"null"#, "Parameter schema format"
    };
}

mod mod_derive_parameters_all_types {
    /// Get foo by id
    ///
    /// Get foo by id long description
    #[utoipa::path(
        get,
        path = "/foo/{id}",
        responses(
            (status = 200, description = "success"),
        ),
        params(
            ("id" = i32, Path, description = "Foo id"),
            ("since" = String, Query, deprecated, description = "Datetime since"),
            ("numbers" = Option<[i64]>, Query, description = "Foo numbers list"),
            ("token" = String, Header, deprecated, description = "Token of foo"),
            ("cookieval" = String, Cookie, deprecated, description = "Foo cookie"),
        )
    )]
    #[allow(unused)]
    async fn get_foo_by_id(id: i32) -> i32 {
        id
    }
}

#[test]
fn derive_parameters_with_all_types() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(mod_derive_parameters_all_types::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc.pointer("/paths/~1foo~1{id}/get/parameters").unwrap();

    common::assert_json_array_len(parameters, 5);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#""Foo id""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"null"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int32""#, "Parameter schema format"

        "[1].in" = r#""query""#, "Parameter in"
        "[1].name" = r#""since""#, "Parameter name"
        "[1].description" = r#""Datetime since""#, "Parameter description"
        "[1].required" = r#"true"#, "Parameter required"
        "[1].deprecated" = r#"true"#, "Parameter deprecated"
        "[1].schema.type" = r#""string""#, "Parameter schema type"
        "[1].schema.format" = r#"null"#, "Parameter schema format"

        "[2].in" = r#""query""#, "Parameter in"
        "[2].name" = r#""numbers""#, "Parameter name"
        "[2].description" = r#""Foo numbers list""#, "Parameter description"
        "[2].required" = r#"false"#, "Parameter required"
        "[2].deprecated" = r#"null"#, "Parameter deprecated"
        "[2].schema.type" = r#""array""#, "Parameter schema type"
        "[2].schema.format" = r#"null"#, "Parameter schema format"
        "[2].schema.items.type" = r#""integer""#, "Parameter schema items type"
        "[2].schema.items.format" = r#""int64""#, "Parameter schema items format"

        "[3].in" = r#""header""#, "Parameter in"
        "[3].name" = r#""token""#, "Parameter name"
        "[3].description" = r#""Token of foo""#, "Parameter description"
        "[3].required" = r#"true"#, "Parameter required"
        "[3].deprecated" = r#"true"#, "Parameter deprecated"
        "[3].schema.type" = r#""string""#, "Parameter schema type"
        "[3].schema.format" = r#"null"#, "Parameter schema format"

        "[4].in" = r#""cookie""#, "Parameter in"
        "[4].name" = r#""cookieval""#, "Parameter name"
        "[4].description" = r#""Foo cookie""#, "Parameter description"
        "[4].required" = r#"true"#, "Parameter required"
        "[4].deprecated" = r#"true"#, "Parameter deprecated"
        "[4].schema.type" = r#""string""#, "Parameter schema type"
        "[4].schema.format" = r#"null"#, "Parameter schema format"
    };
}

mod derive_params_without_args {
    #[utoipa::path(
        get,
        path = "/foo/{id}",
        responses(
            (status = 200, description = "success"),
        ),
        params(
            ("id" = i32, Path, description = "Foo id"),
        )
    )]
    #[allow(unused)]
    async fn get_foo_by_id() -> String {
        "".to_string()
    }
}

#[test]
fn derive_params_without_fn_args() {
    #[derive(OpenApi, Default)]
    #[openapi(paths(derive_params_without_args::get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc.pointer("/paths/~1foo~1{id}/get/parameters").unwrap();

    common::assert_json_array_len(parameters, 1);
    assert_value! {parameters=>
        "[0].in" = r#""path""#, "Parameter in"
        "[0].name" = r#""id""#, "Parameter name"
        "[0].description" = r#""Foo id""#, "Parameter description"
        "[0].required" = r#"true"#, "Parameter required"
        "[0].deprecated" = r#"null"#, "Parameter deprecated"
        "[0].schema.type" = r#""integer""#, "Parameter schema type"
        "[0].schema.format" = r#""int32""#, "Parameter schema format"
    };
}

#[test]
fn derive_params_with_params_ext() {
    #[utoipa::path(
        get,
        path = "/foo",
        responses(
            (status = 200, description = "success"),
        ),
        params(
            ("value" = Option<[String]>, Query, description = "Foo value description", style = Form, allow_reserved, deprecated, explode)
        )
    )]
    #[allow(unused)]
    async fn get_foo_by_id() -> String {
        "".to_string()
    }

    #[derive(OpenApi, Default)]
    #[openapi(paths(get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc.pointer("/paths/~1foo/get/parameters").unwrap();

    common::assert_json_array_len(parameters, 1);
    assert_value! {parameters=>
        "[0].in" = r#""query""#, "Parameter in"
        "[0].name" = r#""value""#, "Parameter name"
        "[0].description" = r#""Foo value description""#, "Parameter description"
        "[0].required" = r#"false"#, "Parameter required"
        "[0].deprecated" = r#"true"#, "Parameter deprecated"
        "[0].schema.type" = r#""array""#, "Parameter schema type"
        "[0].schema.items.type" = r#""string""#, "Parameter schema items type"
        "[0].style" = r#""form""#, "Parameter style"
        "[0].allowReserved" = r#"true"#, "Parameter allowReserved"
        "[0].explode" = r#"true"#, "Parameter explode"
    };
}

#[test]
fn derive_path_params_with_parameter_type_args() {
    #[utoipa::path(
        get,
        path = "/foo",
        responses(
            (status = 200, description = "success"),
        ),
        params(
            ("value" = Option<[String]>, Query, description = "Foo value description", style = Form, allow_reserved, deprecated, explode, max_items = 1, max_length = 20, pattern = r"\w")
        )
    )]
    #[allow(unused)]
    async fn get_foo_by_id() -> String {
        "".to_string()
    }

    #[derive(OpenApi, Default)]
    #[openapi(paths(get_foo_by_id))]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).unwrap();
    let parameters = doc.pointer("/paths/~1foo/get/parameters").unwrap();

    common::assert_json_array_len(parameters, 1);

    assert_json_snapshot!(parameters);
}

macro_rules! into_params {
    ( $( #[$me:meta] )* $key:ident $name:ident $( $tt:tt )*) => {
        {
            #[derive(utoipa::IntoParams)]
            $(#[$me])*
            $key $name $( $tt )*

            #[utoipa::path(get, path = "/handler", params($name))]
            #[allow(unused)]
            fn handler() {}

            let value = serde_json::to_value(__path_handler::operation())
                .expect("path item should serialize to json");
            value.pointer("/parameters").expect("operation should have parameters").clone()
        }
    };
}

#[test]
fn derive_into_params_required_custom_query_parameter_required() {
    #[allow(unused)]
    struct Param<T>(T);

    let value = into_params! {
        #[into_params(parameter_in = Query)]
        #[allow(unused)]
        struct TasksFilterQuery {
            /// Maximum number of results to return.
            #[param(required = false, value_type = u32, example = 12)]
            pub limit: Param<u32>,
            /// Maximum number of results to return.
            #[param(required = true, value_type = u32, example = 12)]
            pub limit_explisit_required: Param<u32>,
            /// Maximum number of results to return.
            #[param(value_type = Option<u32>, example = 12)]
            pub not_required: Param<u32>,
            /// Maximum number of results to return.
            #[param(required = true, value_type = Option<u32>, example = 12)]
            pub option_required: Param<u32>,
        }
    };

    assert_json_snapshot!(value);
}

#[test]
fn path_parameter_with_extensions() {
    #[utoipa::path(get, path = "/pets",
        params(
            (
                "a_param" = String,
                Query,
                extensions(
                    ("x-param-extension" = json!( { "type": "param" }) )
                )
            )
        ),
    )]
    #[allow(unused)]
    fn get_pets() {}
    let operation = __path_get_pets::operation();
    let value = serde_json::to_value(operation).expect("operation is JSON serializable");
    let params = value.pointer("/parameters").unwrap();
    assert_json_snapshot!(params);
}
