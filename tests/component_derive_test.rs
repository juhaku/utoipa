#![cfg(feature = "serde_json")]
use std::{borrow::Cow, cell::RefCell, collections::HashMap, vec};

use serde_json::Value;
use utoipa::{Component, OpenApi};

use crate::common::get_json_path;

mod common;

macro_rules! api_doc {
    ( $( #[$attr:meta] )* $key:ident $name:ident $body:tt ) => {{
        #[allow(dead_code)]
        #[derive(Component)]
        $(#[$attr])*
        $key $name $body

        api_doc!(@doc $name)
    }};

    ( $( #[$attr:meta] )* $key:ident $name:ident $body:tt; ) => {{
        #[allow(dead_code)]
        #[derive(Component)]
        $(#[$attr])*
        $key $name $body;

        api_doc!(@doc $name)
    }};

    ( $( #[$attr:meta] )* $key:ident $name:ident< $($life:lifetime)? $($generic:ident)? > $body:tt ) => {{
        #[allow(dead_code)]
        #[derive(Component)]
        $(#[$attr])*
        $key $name<$($life)? $($generic)?> $body

        api_doc!(@doc $name < $($life)? $($generic)?> )
    }};

    ( @doc $name:ident $( $generic:tt )* ) => {{
        #[derive(OpenApi)]
        #[openapi(components($name$($generic)*))]
        struct ApiDoc;

        let json = serde_json::to_value(ApiDoc::openapi()).unwrap();
        let component = get_json_path(&json, &format!("components.schemas.{}", stringify!($name)));

        component.clone()
    }};
}

#[test]
fn derive_enum_with_additional_properties_success() {
    let mode = api_doc! {
        #[component(default = "Mode1", example = "Mode2")]
        enum Mode {
            Mode1, Mode2
        }
    };

    assert_value! {mode=>
        "default" = r#""Mode1""#, "Mode default"
        "example" = r#""Mode2""#, "Mode example"
        "enum" = r#"["Mode1","Mode2"]"#, "Mode enum variants"
        "type" = r#""string""#, "Mode type"
    };
}

#[test]
fn derive_enum_with_defaults_success() {
    let mode = api_doc! {
        enum Mode {
            Mode1,
            Mode2
        }
    };

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
    let mode = api_doc! {
        #[component(default = mode_custom_default_fn)]
        enum Mode {
            Mode1,
            Mode2
        }
    };

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
    let book = api_doc! {
        struct Book {
            name: String,
            hash: String,
        }
    };

    assert_value! {book=>
        "type" = r#""object""#, "Book type"
        "properties.name.type" = r#""string""#, "Book name type"
        "properties.hash.type" = r#""string""#, "Book hash type"
        "required" = r#"["name","hash"]"#, "Book required fields"
    };
}

#[test]
fn derive_struct_with_custom_properties_success() {
    let book = api_doc! {
        struct Book {
            #[component(default = String::default)]
            name: String,
            #[component(
                default = "testhash",
                example = "base64 text",
                format = ComponentFormat::Byte,
            )]
            hash: String,
        }
    };

    assert_value! {book=>
        "type" = r#""object""#, "Book type"
        "properties.name.type" = r#""string""#, "Book name type"
        "properties.name.default" = r#""""#, "Book name default"
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
    let owner = api_doc! {
        struct Owner {
            #[component(default = 1)]
            id: u64,
            enabled: Option<bool>,
            books: Option<Vec<Book>>,
            metadata: Option<HashMap<String, String>>
        }
    };

    assert_value! {owner=>
        "type" = r#""object""#, "Owner type"
        "properties.id.type" = r#""integer""#, "Owner id type"
        "properties.id.format" = r#""int64""#, "Owner id format"
        "properties.id.default" = r#"1"#, "Owner id default"
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
    let account = api_doc! {
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
    let account = api_doc! {
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

    assert_value! {account=>
        "description" = r#""This is user account status enum""#, "AccountStatus description"
    }
}

#[test]
fn derive_struct_unnamed_field_single_value_type_success() {
    let point = api_doc! {
        struct Point(f64);
    };

    assert_value! {point=>
        "type" = r#""number""#, "Point type"
        "format" = r#""float""#, "Point format"
    }
}

#[test]
fn derive_struct_unnamed_fields_tuple_with_same_type_success() {
    let point = api_doc! {
        /// Contains x and y coordinates
        ///
        /// Coordinates are used to pinpoint location on a map
        struct Point(f64, f64);
    };

    assert_value! {point=>
        "type" = r#""array""#, "Point type"
        "items.type" = r#""number""#, "Point items type"
        "items.format" = r#""float""#, "Point items format"
        "items.description" = r#""Contains x and y coordinates""#, "Point items description"
        "maxItems" = r#"2"#, "Wrapper max items"
        "minItems" = r#"2"#, "Wrapper min items"
    }
}

#[test]
fn derive_struct_unnamed_fields_tuple_with_different_types_success() {
    let point = api_doc! {
        struct Point(f64, String);
    };

    assert_value! {point=>
        "type" = r#""array""#, "Point type"
        "items.type" = r#""object""#, "Point items type"
        "items.format" = r#"null"#, "Point items format"
    }
}

#[test]
fn derive_struct_unnamed_field_with_generic_types_success() {
    let point = api_doc! {
        struct Wrapper(Option<String>);
    };

    assert_value! {point=>
        "type" = r#""string""#, "Wrapper type"
    }
}

#[test]
fn derive_struct_unnamed_field_with_nested_generic_type_success() {
    let point = api_doc! {
        struct Wrapper(Option<Vec<i32>>);
    };

    assert_value! {point=>
        "type" = r#""array""#, "Wrapper type"
        "items.type" = r#""integer""#, "Wrapper items type"
        "items.format" = r#""int32""#, "Wrapper items format"
    }
}

#[test]
fn derive_struct_unnamed_field_with_multiple_nested_generic_type_success() {
    let point = api_doc! {
        struct Wrapper(Option<Vec<i32>>, String);
    };

    assert_value! {point=>
        "type" = r#""array""#, "Wrapper type"
        "items.type" = r#""object""#, "Wrapper items type"
        "items.format" = r#"null"#, "Wrapper items format"
    }
}

#[test]
fn derive_struct_unnamed_field_vec_type_success() {
    let point = api_doc! {
        struct Wrapper(Vec<i32>);
    };

    assert_value! {point=>
        "type" = r#""array""#, "Wrapper type"
        "items.type" = r#""integer""#, "Wrapper items type"
        "items.format" = r#""int32""#, "Wrapper items format"
        "maxItems" = r#"null"#, "Wrapper max items"
        "minItems" = r#"null"#, "Wrapper min items"
    }
}

#[test]
fn derive_struct_nested_vec_success() {
    let vecs = api_doc! {
        struct VecTest {
            vecs: Vec<Vec<String>>
        }
    };

    assert_value! {vecs=>
        "properties.vecs.type" = r#""array""#, "Vecs property type"
        "properties.vecs.items.type" = r#""array""#, "Vecs property items type"
        "properties.vecs.items.items.type" = r#""string""#, "Vecs property items item type"
        "type" = r#""object""#, "Property type"
        "required.[0]" = r#""vecs""#, "Required properties"
    }
    common::assert_json_array_len(vecs.get("required").unwrap(), 1);
}

#[test]
fn derive_struct_with_example() {
    let pet = api_doc! {
        #[component(example = json!({"name": "bob the cat", "age": 8}))]
        struct Pet {
            name: String,
            age: i32
        }
    };

    assert_value! {pet=>
        "example.name" = r#""bob the cat""#, "Pet example name"
        "example.age" = r#"8"#, "Pet example age"
    }
}

#[test]
fn derive_struct_with_deprecated() {
    #[allow(deprecated)]
    let pet = api_doc! {
        #[deprecated]
        struct Pet {
            name: String,
            #[deprecated]
            age: i32
        }
    };

    assert_value! {pet=>
        "deprecated" = r#"true"#, "Pet deprecated"
        "properties.name.type" = r#""string""#, "Pet properties name type"
        "properties.name.deprecated" = r#"null"#, "Pet properties name deprecated"
        "properties.age.type" = r#""integer""#, "Pet properties age type"
        "properties.age.deprecated" = r#"true"#, "Pet properties age deprecated"
        "example" = r#"null"#, "Pet example"
    }
}

#[test]
fn derive_unnamed_struct_deprecated_success() {
    #[allow(deprecated)]
    let pet_age = api_doc! {
        #[deprecated]
        #[component(example = 8)]
        struct PetAge(u64);
    };

    assert_value! {pet_age=>
        "deprecated" = r#"true"#, "PetAge deprecated"
        "example" = r#"8"#, "PetAge example"
    }
}

#[test]
fn derive_unnamed_struct_example_json_array_success() {
    let pet_age = api_doc! {
        #[component(example = "0", default = u64::default)]
        struct PetAge(u64, u64);
    };

    assert_value! {pet_age=>
        "type" = r#""array""#, "PetAge type"
        "items.example" = r#""0""#, "PetAge example"
        "items.default" = r#"0"#, "PetAge default"
        "items.type" = r#""integer""#, "PetAge default"
        "items.format" = r#""int64""#, "PetAge default"
        "maxItems" = r#"2"#, "PetAge max items"
        "minItems" = r#"2"#, "PetAge min items"
    }
}

#[test]
fn derive_enum_with_deprecated() {
    #[allow(deprecated)]
    let mode = api_doc! {
        #[deprecated]
        enum Mode {
            Mode1, Mode2
        }
    };

    assert_value! {mode=>
        "enum" = r#"["Mode1","Mode2"]"#, "Mode enum variants"
        "type" = r#""string""#, "Mode type"
        "deprecated" = r#"true"#, "Mode deprecated"
    };
}

#[test]
fn derive_struct_with_generics() {
    #[allow(unused)]
    enum Type {
        Foo,
        Bar,
    }
    let status = api_doc! {
        struct Status<Type> {
            t: Type
        }
    };

    assert_value! {status=>
        "properties.t.$ref" = r###""#/components/schemas/Type""###, "Status t field"
    };
}

#[test]
fn derive_struct_with_lifetime_generics() {
    #[allow(unused)]
    let greeting = api_doc! {
        struct Greeting<'a> {
            greeting: &'a str
        }
    };

    assert_value! {greeting=>
        "properties.greeting.type" = r###""string""###, "Greeting greeting field type"
    };
}

#[test]
fn derive_struct_with_cow() {
    #[allow(unused)]
    let greeting = api_doc! {
        struct Greeting<'a> {
            greeting: Cow<'a, str>
        }
    };

    common::assert_json_array_len(greeting.get("required").unwrap(), 1);
    assert_value! {greeting=>
        "properties.greeting.type" = r###""string""###, "Greeting greeting field type"
        "required.[0]" = r###""greeting""###, "Greeting required"
    };
}

#[test]
fn derive_with_box_and_refcell() {
    #[allow(unused)]
    struct Foo {
        name: &'static str,
    }

    let greeting = api_doc! {
        struct Greeting {
            foo: Box<Foo>,
            ref_cell_foo: RefCell<Foo>
        }
    };

    common::assert_json_array_len(greeting.get("required").unwrap(), 2);
    assert_value! {greeting=>
        "properties.foo.$ref" = r###""#/components/schemas/Foo""###, "Greeting foo field"
        "properties.ref_cell_foo.$ref" = r###""#/components/schemas/Foo""###, "Greeting ref_cell_foo field"
        "required.[0]" = r###""foo""###, "Greeting required 0"
        "required.[1]" = r###""ref_cell_foo""###, "Greeting required 1"
    };
}

#[test]
fn derive_complex_enum_with_named_and_unnamed_fields() {
    struct Foo;
    let complex_enum = api_doc! {
        enum Bar {
            UnitValue,
            NamedFields {
                id: &'static str,
                names: Option<Vec<String>>
            },
            UnnamedFields(Foo),
        }
    };

    common::assert_json_array_len(complex_enum.get("oneOf").unwrap(), 3);
    assert_value! {complex_enum=>
        "oneOf.[0].type" = r###""string""###, "Complex enum unit value type"
        "oneOf.[0].enum" = r###"["UnitValue"]"###, "Complex enum unit value enum"
        "oneOf.[1].type" = r###""object""###, "Complex enum named fields type"
        "oneOf.[1].properties.NamedFields.type" = r###""object""###, "Complex enum named fields object type"
        "oneOf.[1].properties.NamedFields.properties.id.type" = r###""string""###, "Complex enum named fields id type"
        "oneOf.[1].properties.NamedFields.properties.names.type" = r###""array""###, "Complex enum named fields names type"
        "oneOf.[2].type" = r###""object""###, "Complex enum unnamed fields type"
        "oneOf.[2].properties.UnnamedFields.$ref" = r###""#/components/schemas/Foo""###, "Complex enum unnamed fields type"
    }
}

#[test]
fn derive_struct_with_read_only_and_write_only() {
    let user = api_doc! {
        struct User {
            #[component(read_only)]
            username: String,
            #[component(write_only)]
            password: String
        }
    };

    assert_value! {user=>
        "properties.password.type" = r###""string""###, "User password type"
        "properties.password.writeOnly" = r###"true"###, "User password write only"
        "properties.password.readOnly" = r###"null"###, "User password read only"
        "properties.username.type" = r###""string""###, "User username type"
        "properties.username.readOnly" = r###"true"###, "User username read only"
        "properties.username.writeOnly" = r###"null"###, "User username write only"
    }
}

#[test]
fn derive_struct_xml() {
    let user = api_doc! {
        #[component(xml(name = "user", prefix = "u", namespace = "https://mynamespace.test"))]
        struct User {
            #[component(xml(attribute, prefix = "u"))]
            id: i64,
            #[component(xml(name = "user_name", prefix = "u"))]
            username: String,
            #[component(xml(wrapped(name = "linkList"), name = "link"))]
            links: Vec<String>,
            #[component(xml(wrapped, name = "photo_url"))]
            photos_urls: Vec<String>
        }
    };

    assert_value! {user=>
        "xml.attribute" = r###"null"###, "User xml attribute"
        "xml.name" = r###""user""###, "User xml name"
        "xml.prefix" = r###""u""###, "User xml prefix"
        "xml.namespace" = r###""https://mynamespace.test""###, "User xml namespace"
        "properties.id.xml.attribute" = r###"true"###, "User id xml attribute"
        "properties.id.xml.name" = r###"null"###, "User id xml name"
        "properties.id.xml.prefix" = r###""u""###, "User id xml prefix"
        "properties.id.xml.namespace" = r###"null"###, "User id xml namespace"
        "properties.username.xml.attribute" = r###"null"###, "User username xml attribute"
        "properties.username.xml.name" = r###""user_name""###, "User username xml name"
        "properties.username.xml.prefix" = r###""u""###, "User username xml prefix"
        "properties.username.xml.namespace" = r###"null"###, "User username xml namespace"
        "properties.links.xml.attribute" = r###"null"###, "User links xml attribute"
        "properties.links.xml.name" = r###""linkList""###, "User links xml name"
        "properties.links.xml.prefix" = r###"null"###, "User links xml prefix"
        "properties.links.xml.namespace" = r###"null"###, "User links xml namespace"
        "properties.links.xml.wrapped" = r###"true"###, "User links xml wrapped"
        "properties.links.items.xml.attribute" = r###"null"###, "User links xml items attribute"
        "properties.links.items.xml.name" = r###""link""###, "User links xml items name"
        "properties.links.items.xml.prefix" = r###"null"###, "User links xml items prefix"
        "properties.links.items.xml.namespace" = r###"null"###, "User links xml items namespace"
        "properties.links.items.xml.wrapped" = r###"null"###, "User links xml items wrapped"
        "properties.photos_urls.xml.attribute" = r###"null"###, "User photos_urls xml attribute"
        "properties.photos_urls.xml.name" = r###"null"###, "User photos_urls xml name"
        "properties.photos_urls.xml.prefix" = r###"null"###, "User photos_urls xml prefix"
        "properties.photos_urls.xml.namespace" = r###"null"###, "User photos_urls xml namespace"
        "properties.photos_urls.xml.wrapped" = r###"true"###, "User photos_urls xml wrapped"
        "properties.photos_urls.items.xml.attribute" = r###"null"###, "User photos_urls xml items attribute"
        "properties.photos_urls.items.xml.name" = r###""photo_url""###, "User photos_urls xml items name"
        "properties.photos_urls.items.xml.prefix" = r###"null"###, "User photos_urls xml items prefix"
        "properties.photos_urls.items.xml.namespace" = r###"null"###, "User photos_urls xml items namespace"
        "properties.photos_urls.items.xml.wrapped" = r###"null"###, "User photos_urls links xml items wrapped"
    }
}
