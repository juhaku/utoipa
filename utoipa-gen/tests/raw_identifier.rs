#![expect(
    dead_code,
    reason = "Test structs with raw identifier fields are used only for schema generation"
)]

use utoipa::{IntoParams, PartialSchema, ToSchema};

mod common;

/// Test that demonstrates raw identifiers in struct fields are properly handled
#[test]
fn struct_with_raw_identifier_fields() {
    let schema = {
        #[derive(ToSchema)]
        struct TestStruct {
            r#type: String,
            r#match: i32,
            normal_field: bool,
        }

        serde_json::to_value(<TestStruct as PartialSchema>::schema()).unwrap()
    };

    // The schema should contain clean field names without r# prefix
    assert_value! { schema=>
        "properties.type.type" = r#""string""#, "type field should be string type"
        "properties.match.type" = r#""integer""#, "match field should be integer type"
        "properties.normal_field.type" = r#""boolean""#, "normal_field should be boolean type"
    };

    // Verify that the property keys are clean (no r# prefix)
    let properties = schema
        .pointer("/properties")
        .expect("Should have properties");
    let property_names: Vec<String> = properties
        .as_object()
        .expect("Properties should be object")
        .keys()
        .cloned()
        .collect();

    println!("SUCCESS: Field names are clean: {:?}", property_names);

    assert!(
        property_names.contains(&"type".to_string()),
        "Should contain 'type' property"
    );
    assert!(
        property_names.contains(&"match".to_string()),
        "Should contain 'match' property"
    );
    assert!(
        property_names.contains(&"normal_field".to_string()),
        "Should contain 'normal_field' property"
    );

    // Assert that raw identifiers are NOT present
    assert!(
        !property_names.contains(&"r#type".to_string()),
        "Should NOT contain 'r#type' property"
    );
    assert!(
        !property_names.contains(&"r#match".to_string()),
        "Should NOT contain 'r#match' property"
    );
}

/// Test that demonstrates raw identifiers in IntoParams are properly handled
#[test]
fn into_params_with_raw_identifiers() {
    #[derive(IntoParams)]
    struct QueryParams {
        r#type: Option<String>,
        r#match: Option<i32>,
        limit: Option<u32>,
    }

    let params = QueryParams::into_params(|| None);

    // Check that parameter names are clean (without r# prefix)
    let param_names: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();

    println!("Parameter names: {:?}", param_names);

    assert!(
        param_names.contains(&"type"),
        "Should contain 'type' parameter, got: {:?}",
        param_names
    );
    assert!(
        param_names.contains(&"match"),
        "Should contain 'match' parameter, got: {:?}",
        param_names
    );
    assert!(
        param_names.contains(&"limit"),
        "Should contain 'limit' parameter, got: {:?}",
        param_names
    );

    // Assert that raw identifiers are NOT present
    assert!(
        !param_names.contains(&"r#type"),
        "Should NOT contain 'r#type' parameter"
    );
    assert!(
        !param_names.contains(&"r#match"),
        "Should NOT contain 'r#match' parameter"
    );

    println!("SUCCESS: Parameter names are clean: {:?}", param_names);
}

/// Test error messages and display formatting
#[test]
fn clean_names_in_debug_output() {
    let schema = {
        #[derive(ToSchema)]
        struct TestStruct {
            r#type: String,
            r#async: bool,
        }

        serde_json::to_value(<TestStruct as PartialSchema>::schema()).unwrap()
    };

    let debug_output = format!("{:?}", schema);

    // The debug output should contain clean field names
    assert!(
        debug_output.contains("\"type\""),
        "Debug output should contain clean 'type' field name"
    );
    assert!(
        debug_output.contains("\"async\""),
        "Debug output should contain clean 'async' field name"
    );

    println!("SUCCESS: Debug output contains clean field names");
    println!(
        "Sample debug output: {}",
        debug_output.chars().take(200).collect::<String>()
    );
}

/// Test that demonstrates raw identifiers work with primitive type aliases
#[test]
fn primitive_type_aliases_with_raw_identifiers() {
    // This test demonstrates what happens when people use raw identifiers
    // for field names with primitive types - the more common real-world scenario
    let schema = {
        #[derive(ToSchema)]
        struct PrimitiveTestStruct {
            // These field names use raw identifiers but the types are normal primitives
            r#i8: i8,
            r#u8: u8,
            r#i16: i16,
            r#u16: u16,
            r#i32: i32,
            r#u32: u32,
            r#i64: i64,
            r#f32: f32,
            r#f64: f64,
            r#bool: bool,
            r#String: String,
        }

        serde_json::to_value(<PrimitiveTestStruct as PartialSchema>::schema()).unwrap()
    };

    // Verify that all field names are clean (no r# prefix)
    let properties = schema
        .pointer("/properties")
        .expect("Should have properties");
    let property_names: Vec<String> = properties
        .as_object()
        .expect("Properties should be object")
        .keys()
        .cloned()
        .collect();

    println!("Primitive type field names: {:?}", property_names);

    // Check that all expected field names are present and clean
    let expected_fields = [
        "i8", "u8", "i16", "u16", "i32", "u32", "i64", "f32", "f64", "bool", "String",
    ];

    for field in expected_fields {
        assert!(
            property_names.contains(&field.to_string()),
            "Should contain '{}' field",
            field
        );
    }

    // Ensure raw identifier prefixes are NOT present
    for field in expected_fields {
        let raw_field = format!("r#{}", field);
        assert!(
            !property_names.contains(&raw_field),
            "Should NOT contain '{}' field",
            raw_field
        );
    }

    // Verify basic types are correctly mapped in the schema (without worrying about specific formats)
    assert_value! { schema=>
        "properties.i8.type" = r#""integer""#, "i8 field should be integer type"
        "properties.u8.type" = r#""integer""#, "u8 field should be integer type"
        "properties.String.type" = r#""string""#, "String field should be string type"
        "properties.bool.type" = r#""boolean""#, "bool field should be boolean type"
        "properties.f32.type" = r#""number""#, "f32 field should be number type"
    };

    println!("SUCCESS: All primitive type fields with raw identifier names work correctly - r#i8, r#bool, etc. are all cleaned!");
}
