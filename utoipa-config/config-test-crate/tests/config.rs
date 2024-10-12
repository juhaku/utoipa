use std::borrow::Cow;

use utoipa::{OpenApi, ToSchema};
use utoipa_config::{Config, SchemaCollect};

#[test]
fn test_create_config_with_aliases() {
    let config: Config<'_> = Config::new().alias_for("i32", "Option<String>");
    let json = serde_json::to_string(&config).expect("config is json serializable");

    let config: Config = serde_json::from_str(&json).expect("config is json deserializable");

    assert!(!config.aliases.is_empty());
    assert!(config.aliases.contains_key("i32"));
    assert_eq!(
        config.aliases.get("i32"),
        Some(&Cow::Borrowed("Option<String>"))
    );
}

#[test]
fn test_config_with_collect_all() {
    let config: Config<'_> = Config::new().schema_collect(utoipa_config::SchemaCollect::All);
    let json = serde_json::to_string(&config).expect("config is json serializable");

    let config: Config = serde_json::from_str(&json).expect("config is json deserializable");

    assert!(matches!(config.schema_collect, SchemaCollect::All));
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

#[test]
fn test_schema_with_enum_aliases() {
    #![allow(unused)]

    #[derive(OpenApi)]
    #[openapi(components(schemas(Transactions)))]
    struct ApiDoc;

    #[derive(ToSchema)]
    pub enum Transactions {
        Transaction(EntryAlias),
        TransactionEntryString(EntryString),
    }

    pub type EntryAlias = Entry<i32>;
    pub type EntryString = Entry<String>;

    #[derive(ToSchema)]
    pub struct Entry<I> {
        pub entry_id: I,
    }

    let api = ApiDoc::openapi();
    let value = serde_json::to_value(api).expect("OpenApi must be JSON serializable");
    let schemas = value
        .pointer("/components/schemas")
        .expect("Must have schemas");

    let expected = r###"{
  "Entry_String": {
    "properties": {
      "entry_id": {
        "type": "string"
      }
    },
    "required": [
      "entry_id"
    ],
    "type": "object"
  },
  "Entry_i32": {
    "properties": {
      "entry_id": {
        "format": "int32",
        "type": "integer"
      }
    },
    "required": [
      "entry_id"
    ],
    "type": "object"
  },
  "Transactions": {
    "oneOf": [
      {
        "properties": {
          "Transaction": {
            "$ref": "#/components/schemas/Entry_i32"
          }
        },
        "required": [
          "Transaction"
        ],
        "type": "object"
      },
      {
        "properties": {
          "TransactionEntryString": {
            "$ref": "#/components/schemas/Entry_String"
          }
        },
        "required": [
          "TransactionEntryString"
        ],
        "type": "object"
      }
    ]
  }
}"###;
    assert_eq!(
        serde_json::to_string_pretty(schemas).unwrap().trim(),
        expected
    );
}
