#![cfg(not(feature = "serde_json"))]

use std::{print, println};

use utoipa::ToSchema;

#[test]
fn derive_component_with_string_example_compiles_success() {
    #[derive(ToSchema)]
    #[schema(example = r#"{"foo": "bar"}"#)]
    struct Foo {
        bar: String,
    }
}

#[test]
fn derive_component_with_string_example_attributes_compiles_success() {
    #[derive(ToSchema)]
    struct Foo {
        #[schema(example = r#""bar""#, default = r#""foobar""#)]
        bar: String,
    }
}
