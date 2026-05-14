use utoipa::ToSchema;
use serde::{Serialize, Deserialize};

#[test]
#[cfg(feature = "tagged_discriminator")]
fn derive_enum_tagged_discriminator() {

    #[derive(ToSchema, Serialize, Deserialize)]
    struct OperationStatePending {
        id: String,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    struct OperationStateCompleted {
        result: String,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    #[serde(tag = "type")]
    enum OperationState {
        #[serde(rename = "pending")]
        Pending(OperationStatePending),
        #[serde(rename = "completed")]
        Completed(OperationStateCompleted),
    }

    let schema = <OperationState as utoipa::PartialSchema>::schema();
    let value = serde_json::to_value(schema).unwrap();

    let expected = serde_json::json!({
      "discriminator": {
        "mapping": {
          "completed": "#/components/schemas/OperationStateCompleted",
          "pending": "#/components/schemas/OperationStatePending"
        },
        "propertyName": "type"
      },
      "oneOf": [
        {
          "$ref": "#/components/schemas/OperationStatePending"
        },
        {
          "$ref": "#/components/schemas/OperationStateCompleted"
        }
      ]
    });

    assert_eq!(value, expected);
}

#[test]
#[cfg(feature = "tagged_discriminator")]
fn derive_enum_tagged_discriminator_complex() {
    #[derive(ToSchema, Serialize, Deserialize)]
    struct Item {
        name: String,
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    enum ComplexEnum {
        #[serde(rename = "renamed_variant")]
        VariantRef(Item),
        
        InlineVariant {
            value: i32
        }
    }
    
    let schema = <ComplexEnum as utoipa::PartialSchema>::schema();
    let value = serde_json::to_value(schema).unwrap();

    // Inline variants are NOT added to discriminator mapping, but have the tag injected.
    // Ref variants are added to mapping and are bare refs in oneOf.
    
    let expected = serde_json::json!({
      "discriminator": {
        "mapping": {
          "renamed_variant": "#/components/schemas/Item"
        },
        "propertyName": "kind"
      },
      "oneOf": [
        {
          "$ref": "#/components/schemas/Item"
        },
        {
          "type": "object",
          "required": [
            "value",
            "kind"
          ],
          "properties": {
            "kind": {
              "type": "string",
              "enum": [
                "inlineVariant"
              ]
            },
            "value": {
              "type": "integer",
              "format": "int32"
            }
          }
        }
      ]
    });

    assert_eq!(value, expected);
}
