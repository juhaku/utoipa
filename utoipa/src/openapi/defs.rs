//! Implements [`$defs` keyword from json schema][defs]
//!
//! [defs]: https://json-schema.org/draft/2020-12/json-schema-core#name-schema-re-use-with-defs
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use super::{builder, OpenApi, RefOr, Schema};

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

impl OpenApi {
    /// Used to extract defs from the direct openapi components schemas as the schemas
    ///
    /// e.g.
    ///
    /// ```json
    /// {
    ///   "openapi": "3.1.0",
    ///   "info": {
    ///     "title": "",
    ///     "version": ""
    ///   },
    ///   "paths": {},
    ///   "components": {
    ///     "schemas": {
    ///       "MySchema": {
    ///         "type": "object",
    ///         "$defs": {
    ///           "address": {
    ///             "type": "object",
    ///             "required": [
    ///               "street",
    ///               "city"
    ///             ],
    ///             "properties": {
    ///               "city": {
    ///                 "$ref": "#/$defs/nonEmptyString"
    ///               },
    ///               "street": {
    ///                 "$ref": "#/$defs/nonEmptyString"
    ///               }
    ///             }
    ///           },
    ///           "nonEmptyString": {
    ///             "type": "string",
    ///             "minLength": 1
    ///           }
    ///         }
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// becomes
    /// ```json
    ///{
    ///   "openapi": "3.1.0",
    ///   "info": {
    ///     "title": "",
    ///     "version": ""
    ///   },
    ///   "paths": {},
    ///   "components": {
    ///     "schemas": {
    ///       "MySchema": {
    ///         "type": "object",
    ///         "$defs": {
    ///           "address": {
    ///             "type": "object",
    ///             "required": [
    ///               "street",
    ///               "city"
    ///             ],
    ///             "properties": {
    ///               "city": {
    ///                 "$ref": "#/$defs/nonEmptyString"
    ///               },
    ///               "street": {
    ///                 "$ref": "#/$defs/nonEmptyString"
    ///               }
    ///             }
    ///           },
    ///           "nonEmptyString": {
    ///             "type": "string",
    ///             "minLength": 1
    ///           }
    ///         }
    ///       },
    ///       "address": {
    ///         "type": "object",
    ///         "required": [
    ///           "street",
    ///           "city"
    ///         ],
    ///         "properties": {
    ///           "city": {
    ///             "$ref": "#/$defs/nonEmptyString"
    ///           },
    ///           "street": {
    ///             "$ref": "#/$defs/nonEmptyString"
    ///           }
    ///         }
    ///       },
    ///       "nonEmptyString": {
    ///         "type": "string",
    ///         "minLength": 1
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    pub fn extract_components_from_schemas_def(&mut self) -> &mut Self {
        let mut additional_schemas: Vec<(String, RefOr<Schema>)> = Vec::new();
        if let Some(components) = &mut self.components {
            for schema in components
                .schemas
                .values()
                .filter_map(|schema| match schema {
                    RefOr::T(t) => Some(t),
                    _ => None,
                })
            {
                if let Some(defs) = schema.defs() {
                    additional_schemas.extend(
                        defs.iter()
                            // .cloned() did not work here
                            .map(|(key, value)| (key.clone(), RefOr::T(value.clone()))),
                    )
                }
            }

            components.schemas.extend(additional_schemas);
        }

        self
    }
}

impl<T> RefOr<T> {
    /// Used to replace /#defs/ with openapi /components/schemas
    ///
    /// e.g.
    /// ```json
    /// {
    ///    "$ref": "/#defs/a/b
    /// }
    /// ```
    ///
    /// becomes
    ///
    /// ```json
    /// {
    ///    "$ref": /#components/schemas/a/b"
    /// }
    /// ```
    pub fn root_defs_to_openapi_schemas(&mut self) -> &mut Self {
        match self {
            RefOr::Ref(ref_) => {
                if ref_.ref_location.starts_with("/#defs/") {
                    ref_.ref_location =
                        ref_.ref_location
                            .replacen("/#defs/", "#/components/schemas/", 1);
                }
            }
            RefOr::T(_) => {}
        }

        self
    }
}

#[cfg(test)]
#[allow(missing_docs)]
pub mod test {
    use super::*;
    use crate::openapi::{
        schema::{SchemaType, Type},
        ComponentsBuilder, ObjectBuilder, OpenApiBuilder,
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

        let expected: Defs = serde_json::from_str(expected_str).unwrap();
        let final_expected = serde_json::to_string_pretty(&expected).unwrap();
        println!("{final_expected}");
        println!("--------------------------");

        assert_eq!(final_defs, final_expected);
    }

    #[test]
    fn additional_schemas_extraction() {
        let defs = serde_json::from_str(BASIC).unwrap();

        let mut openapi = OpenApiBuilder::new().components(Some(
            ComponentsBuilder::new()
                .schema("MySchema", ObjectBuilder::new().defs(defs))
                .build(),
        )).build();

        openapi.extract_components_from_schemas_def();
        assert_eq!(openapi.components.unwrap().schemas.len(), 3);
    }
}
