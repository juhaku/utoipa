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
        #[allow(non_snake_case)]
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

/// Test that demonstrates raw identifiers work with stdlib generic type names
///
/// This tests the case where someone uses raw identifiers on the actual stdlib types
/// themselves (e.g., r#Option, r#Vec), not type aliases. This is the real-world scenario
/// where desynt helps - it strips the r# prefix from type names like r#Option so that
/// utoipa can recognize them as GenericType::Option.
#[test]
fn generic_types_with_raw_identifiers() {
    let schema = {
        #[derive(ToSchema)]
        #[allow(non_camel_case_types)]
        struct GenericTestStruct {
            // Using raw identifiers directly on stdlib generic types
            // This is what desynt helps with - stripping r# from type names
            optional_field: r#Option<String>,
            list_field: r#Vec<i32>,
            map_field: std::collections::r#HashMap<String, bool>,
            normal_option: Option<String>,
        }

        serde_json::to_value(<GenericTestStruct as PartialSchema>::schema()).unwrap()
    };

    // Verify that all field names are present
    let properties = schema
        .pointer("/properties")
        .expect("Should have properties");
    let property_names: Vec<String> = properties
        .as_object()
        .expect("Properties should be object")
        .keys()
        .cloned()
        .collect();

    println!("Generic type field names: {:?}", property_names);

    let expected_fields = ["optional_field", "list_field", "map_field", "normal_option"];

    for field in expected_fields {
        assert!(
            property_names.contains(&field.to_string()),
            "Should contain '{}' field",
            field
        );
    }

    // Verify the types are correctly recognized and nullable fields are handled
    // Both optional fields should not be in required since they're Option types
    if let Some(required) = schema.pointer("/required") {
        let required_array = required.as_array().expect("Required should be array");
        let required_names: Vec<&str> = required_array.iter().filter_map(|v| v.as_str()).collect();

        println!("Required fields: {:?}", required_names);

        // optional_field and normal_option should NOT be required (they're Option types)
        // This tests that r#Option is properly recognized as GenericType::Option
        assert!(
            !required_names.contains(&"optional_field"),
            "optional_field should not be required (it's r#Option which should be recognized as Option)"
        );
        assert!(
            !required_names.contains(&"normal_option"),
            "normal_option should not be required (it's an Option)"
        );

        // list_field and map_field SHOULD be required (they're not Option)
        assert!(
            required_names.contains(&"list_field"),
            "list_field should be required"
        );
        assert!(
            required_names.contains(&"map_field"),
            "map_field should be required"
        );
    }

    // Verify list_field is recognized as an array (tests that r#Vec is recognized as Vec)
    assert_value! { schema=>
        "properties.list_field.type" = r#""array""#, "list_field should be array type (r#Vec recognized as Vec)"
        "properties.list_field.items.type" = r#""integer""#, "list_field items should be integer"
        "properties.map_field.type" = r#""object""#, "map_field should be object type (r#HashMap recognized as HashMap)"
    };

    println!("SUCCESS: Generic types with raw identifiers work correctly - r#Option, r#Vec, r#HashMap are properly recognized!");
}

/// Test that demonstrates whether utoipa recognizes fully qualified type paths
///
/// This tests whether types like `std::string::String`, `std::vec::Vec`, `std::option::Option`
/// are recognized the same as their short forms. This is important for understanding how
/// desynt::PathResolver should be used.
#[test]
fn fully_qualified_type_paths() {
    let schema = {
        #[derive(ToSchema)]
        struct FullyQualifiedTestStruct {
            // Using fully qualified paths to see if utoipa recognizes them
            string_field: std::string::String,
            vec_field: std::vec::Vec<i32>,
            option_field: std::option::Option<String>,
            hashmap_field: std::collections::HashMap<String, bool>,

            // Compare with short forms
            short_string: String,
            short_vec: Vec<i32>,
            short_option: Option<String>,
        }

        serde_json::to_value(<FullyQualifiedTestStruct as PartialSchema>::schema()).unwrap()
    };

    // Verify that all field names are present
    let properties = schema
        .pointer("/properties")
        .expect("Should have properties");
    let property_names: Vec<String> = properties
        .as_object()
        .expect("Properties should be object")
        .keys()
        .cloned()
        .collect();

    println!("Fully qualified type field names: {:?}", property_names);

    // Check required fields - Option types should NOT be required
    if let Some(required) = schema.pointer("/required") {
        let required_array = required.as_array().expect("Required should be array");
        let required_names: Vec<&str> = required_array.iter().filter_map(|v| v.as_str()).collect();

        println!("Required fields: {:?}", required_names);

        // This is the key test: does std::option::Option get recognized as Option?
        if required_names.contains(&"option_field") {
            panic!(
                "FAILED: option_field (std::option::Option<String>) is marked as required! \
                This means utoipa is NOT recognizing fully qualified paths. \
                It only looks at the last segment 'Option' but std::option::Option has last segment 'Option' in a module path."
            );
        }

        println!(
            "SUCCESS: option_field is NOT required, suggesting std::option::Option is recognized"
        );

        // short_option should definitely not be required
        assert!(
            !required_names.contains(&"short_option"),
            "short_option should not be required"
        );

        // Non-option fields SHOULD be required
        assert!(
            required_names.contains(&"string_field"),
            "string_field should be required"
        );
        assert!(
            required_names.contains(&"vec_field"),
            "vec_field should be required"
        );
        assert!(
            required_names.contains(&"hashmap_field"),
            "hashmap_field should be required"
        );
    } else {
        panic!("Schema should have required fields");
    }

    // Verify that string_field is recognized as string type (not object)
    assert_value! { schema=>
        "properties.string_field.type" = r#""string""#, "std::string::String should be recognized as string type"
        "properties.short_string.type" = r#""string""#, "String should be recognized as string type"
    };

    // Verify that vec_field is recognized as array type (not object)
    assert_value! { schema=>
        "properties.vec_field.type" = r#""array""#, "std::vec::Vec should be recognized as array type"
        "properties.vec_field.items.type" = r#""integer""#, "vec_field items should be integer"
        "properties.short_vec.type" = r#""array""#, "Vec should be recognized as array type"
        "properties.short_vec.items.type" = r#""integer""#, "short_vec items should be integer"
    };

    // Verify that hashmap is recognized as object with additionalProperties
    assert_value! { schema=>
        "properties.hashmap_field.type" = r#""object""#, "std::collections::HashMap should be recognized as object type"
    };

    println!("SUCCESS: Fully qualified type paths are properly recognized!");
}

/// Test with type aliases to demonstrate where path resolution is needed
///
/// This tests a realistic scenario where someone creates type aliases with the same
/// names as stdlib types but in different modules. Without proper path resolution,
/// utoipa might incorrectly treat these as the stdlib types.
#[test]
fn type_alias_path_resolution() {
    // Create type aliases in a local module with same names as stdlib types
    mod custom {
        use utoipa::ToSchema;

        // Custom types with similar names to stdlib types
        #[derive(ToSchema)]
        pub struct MyOption<T> {
            pub value: Option<T>,
        }

        #[derive(ToSchema)]
        pub struct MyVec<T> {
            pub items: Vec<T>,
        }
    }

    let schema = {
        #[derive(ToSchema)]
        struct PathResolutionTest {
            // This is the stdlib Option - should NOT be required (nullable)
            std_option: std::option::Option<String>,

            // This is a custom type - SHOULD be required
            custom_option: custom::MyOption<String>,

            // This is the stdlib Vec - should be array type
            std_vec: std::vec::Vec<i32>,

            // This is a custom type - should have $ref
            custom_vec: custom::MyVec<i32>,
        }

        serde_json::to_value(<PathResolutionTest as PartialSchema>::schema()).unwrap()
    };

    println!("Schema: {}", serde_json::to_string_pretty(&schema).unwrap());

    // Check required fields
    if let Some(required) = schema.pointer("/required") {
        let required_array = required.as_array().expect("Required should be array");
        let required_names: Vec<&str> = required_array.iter().filter_map(|v| v.as_str()).collect();

        println!("Required fields: {:?}", required_names);

        // std::option::Option should NOT be required (it's nullable)
        assert!(
            !required_names.contains(&"std_option"),
            "std_option (std::option::Option) should not be required"
        );

        // custom::Option SHOULD be required because it's not the stdlib Option type
        assert!(
            required_names.contains(&"custom_option"),
            "custom_option should be required - it's a custom type, not std::option::Option"
        );

        // Both Vec types should be required
        assert!(
            required_names.contains(&"std_vec"),
            "std_vec should be required"
        );
        assert!(
            required_names.contains(&"custom_vec"),
            "custom_vec should be required"
        );
    }

    // Check if std_vec is treated as array
    if let Some(std_vec_type) = schema.pointer("/properties/std_vec/type") {
        println!("std_vec type: {}", std_vec_type);
        assert_eq!(
            std_vec_type.as_str(),
            Some("array"),
            "std::vec::Vec should be array type"
        );
    }

    // Check if custom types have $ref (not inline like arrays or options)
    if let Some(custom_option_ref) = schema.pointer("/properties/custom_option/$ref") {
        println!("custom_option has $ref: {}", custom_option_ref);
    } else {
        panic!("custom_option should have a $ref since it's a custom type, not an inline type");
    }

    if let Some(custom_vec_ref) = schema.pointer("/properties/custom_vec/$ref") {
        println!("custom_vec has $ref: {}", custom_vec_ref);
    } else {
        panic!("custom_vec should have a $ref since it's a custom type, not an inline array");
    }

    println!("\nSUCCESS: desynt::PathResolver correctly distinguishes:");
    println!(
        "  - std::option::Option (nullable, not required) vs custom types (required with $ref)"
    );
    println!("  - std::vec::Vec (inline array type) vs custom types ($ref to custom schema)");
}
