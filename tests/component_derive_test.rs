use std::collections::HashMap;

use serde_json::Value;
use utoipa::{Component, OpenApi};

use crate::common::{get_json_path, value_as_string};

mod common;

macro_rules! api_doc {
    ( $( #[$attr:meta] )* $key:ident $name:ident $body:tt ) => {{
        #[allow(dead_code)]
        #[derive(Component)]
        $(#[$attr])*
        $key $name $body

        #[derive(OpenApi)]
        #[openapi(handler_files = [], components = [$name])]
        struct ApiDoc;

        serde_json::to_value(ApiDoc::openapi()).unwrap()
    }};
}

macro_rules! assert_value {
    ($value:expr=> $( $path:literal = $expected:literal, $error:literal)* ) => {{
        $(
            let actual = value_as_string(Some(get_json_path(&$value, $path)));
            assert_eq!(actual, $expected, "{}: {} expected to be: {} but was: {}", $error, $path, $expected, actual);
         )*
    }};

    ($value:expr=> $( $path:literal = $expected:expr, $error:literal)*) => {
        {
            $(
                let actual = get_json_path(&$value, $path);
                assert!(actual == &$expected, "{}: {} expected to be: {:?} but was: {:?}", $error, $path, $expected, actual);
             )*
        }
    }
}

#[test]
fn derive_enum_with_additional_properties_success() {
    let api_doc_value = api_doc! {
        #[component(default = "Mode1", example = "Mode2")]
        enum Mode {
            Mode1, Mode2
        }
    };
    let mode = get_json_path(&api_doc_value, "components.schemas.Mode");

    assert_value! {mode=>
        "default" = r#""Mode1""#, "Mode default"
        "example" = r#""Mode2""#, "Mode example"
        "enum" = r#"["Mode1","Mode2"]"#, "Mode enum variants"
        "type" = r#""string""#, "Mode type"
    };
}

#[test]
fn derive_enum_with_defaults_success() {
    let api_doc_value = api_doc! {
        enum Mode {
            Mode1,
            Mode2
        }
    };
    let mode = get_json_path(&api_doc_value, "components.schemas.Mode");

    assert_value! {mode=>
        "enum" = r#"["Mode1","Mode2"]"#, "Mode enum variants"
        "type" = r#""string""#, "Mode type"
    };
    assert_value! {mode=>
        "default" = Value::Null, "Mode default"
        "example" = Value::Null, "Mode example"
    }
}

#[test]
fn derive_enum_with_with_custom_default_fn_success() {
    let api_doc_value = api_doc! {
        #[component(default = "crate::mode_custom_default_fn")]
        enum Mode {
            Mode1,
            Mode2
        }
    };
    let mode = get_json_path(&api_doc_value, "components.schemas.Mode");

    assert_value! {mode=>
        "default" = r#""Mode2""#, "Mode default"
        "enum" = r#"["Mode1","Mode2"]"#, "Mode enum variants"
        "type" = r#""string""#, "Mode type"
    };
    assert_value! {mode=>
        "example" = Value::Null, "Mode example"
    }
}

fn mode_custom_default_fn() -> String {
    "Mode2".to_string()
}

#[test]
fn derive_struct_with_defaults_success() {
    let api_doc_value = api_doc! {
        struct Book {
            name: String,
            hash: String,
        }
    };
    let book = get_json_path(&api_doc_value, "components.schemas.Book");

    assert_value! {book=>
        "type" = r#""object""#, "Book type"
        "properties.name.type" = r#""string""#, "Book name type"
        "properties.hash.type" = r#""string""#, "Book hash type"
        "required" = r#"["name","hash"]"#, "Book required fields"
    };
}

#[test]
fn derive_struct_with_custom_properties_success() {
    let api_doc_value = api_doc! {
        struct Book {
            name: String,
            #[component(
                default = "testhash"
                example = "base64 text",
                format = "ComponentFormat::Byte"
            )]
            hash: String,
        }
    };
    let book = get_json_path(&api_doc_value, "components.schemas.Book");

    assert_value! {book=>
        "type" = r#""object""#, "Book type"
        "properties.name.type" = r#""string""#, "Book name type"
        "properties.hash.type" = r#""string""#, "Book hash type"
        "properties.hash.format" = r#""byte""#, "Book hash format"
        "properties.hash.example" = r#""base64 text""#, "Book hash example"
        "properties.hash.default" = r#""testhash""#, "Book hash default"
        "required" = r#"["name","hash"]"#, "Book required fields"
    };
}

#[test]
fn derive_struct_with_optional_properties_success() {
    struct Book;
    let api_doc_value = api_doc! {
        struct Owner {
            #[component(default = 1)]
            id: u64,
            enabled: Option<bool>,
            books: Option<Vec<Book>>,
            metadata: Option<HashMap<String, String>>
        }
    };
    let owner = get_json_path(&api_doc_value, "components.schemas.Owner");

    assert_value! {owner=>
        "type" = r#""object""#, "Owner type"
        "properties.id.type" = r#""integer""#, "Owner id type"
        "properties.id.format" = r#""int64""#, "Owner id format"
        "properties.id.default" = r#""1""#, "Owner id default"
        "properties.enabled.type" = r#""boolean""#, "Owner enabled"
        "properties.books.type" = r#""array""#, "Owner books"
        "properties.books.items.$ref" = r###""#/components/schemas/Book""###, "Owner books items ref"
        "properties.metadata.type" = r#""object""#, "Owner metadata"
    };
    assert_value! {owner=>
        "required" = Value::Array(vec![Value::String("id".to_string())]), "Owner required"
    }
}

#[test]
fn derive_struct_with_comments_success() {
    let api_doc_value = api_doc! {
        /// This is user account dto object
        ///
        /// Detailed documentation here
        /// Only first line is added to the description so far
        struct Account {
            /// Database autogenerated id
            id: i64,
            /// Users username
            username: String,
            role_ids: Vec<i32>
        }
    };
    let account = get_json_path(&api_doc_value, "components.schemas.Account");

    assert_value! {account=>
        "description" = r#""This is user account dto object""#, "Account description"
        "properties.id.description" = r#""Database autogenerated id""#, "Account id description"
        "properties.username.description" = r#""Users username""#, "Account username description"
        "properties.role_ids.type" = r#""array""#, "Account role_ids type"
        "properties.role_ids.items.type" = r#""integer""#, "Account role_ids item type"
        "properties.role_ids.items.format" = r#""int32""#, "Account role_ids item format"
        "required" = r#"["id","username","role_ids"]"#, "Account required"
    }
}

#[test]
fn derive_enum_with_comments_success() {
    let api_doc_value = api_doc! {
        /// This is user account status enum
        ///
        /// Detailed documentation here
        /// Only first line is added to the description so far
        enum AccountStatus {
            /// When user is valid to login, these enum variant level docs are omitted!!!!!
            /// Since the OpenAPI spec does not have a place to put such infomation.
            Enabled,
            /// Login failed too many times
            Locked,
            Disabled
        }
    };
    let account = get_json_path(&api_doc_value, "components.schemas.AccountStatus");

    assert_value! {account=>
        "description" = r#""This is user account status enum""#, "AccountStatus description"
    }
}

// Not supported at least at the moment
// #[test]
// fn derive_struct_tuple_type_success() {
//     #[allow(dead_code)]
//     #[derive(Component)]
//     struct Point(i32);

//     #[derive(OpenApi)]
//     #[openapi(handler_files = [], components = [$name])]
//     struct ApiDoc;

//     let api_doc_value = serde_json::to_value(ApiDoc::openapi()).unwrap();
//     let account = get_json_path(&api_doc_value, "components.schemas.AccountStatus");

//     assert_value! {account=>
//         "description" = r#""This is user account status enum""#, "AccountStatus description"
//     }
// }
