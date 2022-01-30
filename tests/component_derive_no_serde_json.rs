#![cfg(not(feature = "serde_json"))]

use std::{print, println};

use utoipa::Component;

#[test]
fn derive_component_with_string_example_compiles_success() {
    #[derive(Component)]
    #[component(example = r#"{"foo": "bar"}"#)]
    struct Foo {
        bar: String,
    }
}

#[test]
fn derive_component_with_string_example_attributes_compiles_success() {
    #[derive(Component)]
    struct Foo {
        #[component(example = r#""bar""#, default = r#""foobar""#)]
        bar: String,
    }
}
