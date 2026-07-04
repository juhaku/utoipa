//! Implements [`$defs` keyword from json schema][defs]
//!
//! [defs]: https://json-schema.org/draft/2020-12/json-schema-core#name-schema-re-use-with-defs
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use super::{builder, Schema};

builder! {
    DefsBuilder;

    /// The "$defs" keyword reserves a location for schema authors to inline re-usable JSON Schemas
    /// into a more general schema. The keyword does not directly affect the validation result.
    ///
    /// This keyword's value MUST be an object. Each member value of this object MUST be a valid
    /// JSON Schema.
    ///
    /// As an example, here is a schema describing an array of positive integers, where the
    /// positive integer constraint is a subschema in "$defs":
    /// ```
    /// {
    ///     "type": "array",
    ///     "items": { "$ref": "#/$defs/positiveInteger" },
    ///     "$defs": {
    ///         "positiveInteger": {
    ///             "type": "integer",
    ///             "exclusiveMinimum": 0
    ///         }
    ///     }
    /// }
    /// ```

    #[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Defs {
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default, rename = "$defs")]
        defs: BTreeMap<String, Schema>,
    }
}

impl DefsBuilder {
    /// Extend `$defs`
    ///
    /// Read more:
    /// <https://json-schema.org/draft/2020-12/json-schema-core#name-schema-re-use-with-defs>
    pub fn defs<K: Into<String>, V: Into<Schema>, T: IntoIterator<Item = (K, V)>>(
        mut self,
        defs: T,
    ) -> Self {
        self.defs
            .extend(defs.into_iter().map(|(k, v)| (k.into(), v.into())));
        self
    }

    /// Add field describing `const` value
    ///
    /// Read more:
    /// <https://json-schema.org/draft/2020-12/json-schema-core#name-schema-re-use-with-defs>
    pub fn def<K: Into<String>, V: Into<Schema>>(mut self, key: K, value: V) -> Self {
        self.defs.insert(key.into(), value.into());
        self
    }
}

impl Defs {
    /// Get internal iterator
    pub fn iter(&self) -> std::collections::btree_map::Iter<String, Schema> {
        self.defs.iter()
    }

    /// Get internal iterator (mutable)
    pub fn iter_mut(&mut self) -> std::collections::btree_map::IterMut<String, Schema> {
        self.defs.iter_mut()
    }
}

impl<K, V> FromIterator<(K, V)> for Defs
where
    K: Into<String>,
    V: Into<Schema>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let iter = iter.into_iter().map(|(k, v)| (k.into(), v.into()));
        let defs = BTreeMap::from_iter(iter);
        Self { defs }
    }
}

impl From<Defs> for BTreeMap<String, Schema> {
    fn from(value: Defs) -> Self {
        value.defs
    }
}

impl Deref for Defs {
    type Target = BTreeMap<String, Schema>;

    fn deref(&self) -> &Self::Target {
        &self.defs
    }
}

impl DerefMut for Defs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.defs
    }
}

impl IntoIterator for Defs {
    type Item = (String, Schema);

    type IntoIter = std::collections::btree_map::IntoIter<String, Schema>;

    fn into_iter(self) -> Self::IntoIter {
        self.defs.into_iter()
    }
}

impl Schema {
    /// Retrive [`$defs`] keyword for the [`Schema`]
    pub fn defs(&self) -> Option<&Defs> {
        match self {
            Schema::Const(_) => None,
            Schema::Array(array) => Some(&array.defs),
            Schema::Object(object) => Some(&object.defs),
            Schema::OneOf(one_of) => Some(&one_of.defs),
            Schema::AllOf(all_of) => Some(&all_of.defs),
            Schema::AnyOf(any_of) => Some(&any_of.defs),
        }
    }
}

#[cfg(test)]
#[allow(missing_docs)]
pub mod test {
    use super::*;
    use crate::openapi::{
        schema::{SchemaType, Type},
        ObjectBuilder,
    };

    const BASIC: &str = r##"
    {
        "$defs": {
            "nonEmptyString": {
              "type": "string",
              "minLength": 1
            },
            "address": {
              "type": "object",
              "properties": {
                "street": {
                  "$ref": "#/$defs/nonEmptyString"
                },
                "city": {
                  "$ref": "#/$defs/nonEmptyString"
                }
              },
              "required": ["street", "city"]
            }
        }
    }
    "##;

    #[test]
    fn basic_defs() {
        assert!(serde_json::from_str::<Defs>(BASIC).is_ok());
    }

    #[test]
    fn test_builder() {
        let defs: Defs = DefsBuilder::new()
            .def(
                "MySchema",
                ObjectBuilder::new().schema_type(SchemaType::Type(Type::String)),
            )
            .defs([
                (
                    "OtherSchema",
                    ObjectBuilder::new().schema_type(SchemaType::Type(Type::Integer)),
                ),
                (
                    "ThirdSchema",
                    ObjectBuilder::new().schema_type(SchemaType::Type(Type::Number)),
                ),
            ])
            .def(
                "FourthSchema",
                ObjectBuilder::new().schema_type(SchemaType::Type(Type::Array)),
            )
            .into();

        let expected_str = r#"
        {
            "$defs": {
                "MySchema": {
                    "type": "string"
                },
                "OtherSchema": {
                    "type": "integer"
                },
                "ThirdSchema": {
                    "type": "number"
                },
                "FourthSchema": {
                    "type": "array"
                }
            }
        }
        "#;

        let final_defs = serde_json::to_string_pretty(&defs).unwrap();
        println!("--------------------------");
        println!("{final_defs}");

        let expected : Defs = serde_json::from_str(expected_str).unwrap();
        let final_expected = serde_json::to_string_pretty(&expected).unwrap();
        println!("{final_expected}");
        println!("--------------------------");

        assert_eq!(final_defs, final_expected);

    }
}
