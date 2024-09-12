use std::borrow::Cow;

use utoipa::ToSchema;
use utoipa_config::Config;

#[test]
fn test_create_config_with_aliases() {
    Config::new()
        .alias_for("i32", "Option<String>")
        .write_to_file();

    let config = Config::read_from_file();

    assert!(!config.aliases.is_empty());
    assert!(config.aliases.contains_key("i32"));
    assert_eq!(
        config.aliases.get("i32"),
        Some(&Cow::Borrowed("Option<String>"))
    );
}

#[test]
fn test_to_schema_with_aliases() {
    #[allow(unused)]
    #[derive(ToSchema)]
    struct AliasValues {
        name: String,

        #[schema(value_type = MyType)]
        my_type: String,

        #[schema(value_type = MyInt)]
        my_int: String,

        #[schema(value_type = MyValue)]
        my_value: bool,

        date: MyDateTime,
    }

    #[allow(unused)]
    struct MyDateTime {
        millis: usize,
    }

    let schema = utoipa::schema!(
        #[inline]
        AliasValues
    );

    let actual = serde_json::to_string_pretty(&schema).expect("schema must be JSON serializable");

    let expected = r#"{
  "type": "object",
  "required": [
    "name",
    "my_type",
    "my_value",
    "date"
  ],
  "properties": {
    "date": {
      "type": "string"
    },
    "my_int": {
      "type": [
        "integer",
        "null"
      ],
      "format": "int32"
    },
    "my_type": {
      "type": "boolean"
    },
    "my_value": {
      "type": "string"
    },
    "name": {
      "type": "string"
    }
  }
}"#;

    println!("{actual}");
    assert_eq!(expected.trim(), actual.trim())
}
