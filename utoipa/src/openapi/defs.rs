//! Implements [`$defs` keyword from json schema][defs]
//!
//! [defs]: https://json-schema.org/draft/2020-12/json-schema-core#name-schema-re-use-with-defs
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use super::{
    builder,
    schema::{AdditionalProperties, AnyOf},
    AllOf, Array, Components, Object, OneOf, OpenApi, RefOr, Schema,
};

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
    /// ```json
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

    /// Used to replace /#defs/ with openapi /components/schemas/<schema-name>
    /// for schema (recursively)
    ///
    /// See also: [`RefOr::refs_to_openapi_format`]
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
    ///    "$ref": /#components/schemas/<schema-name>/a/b"
    /// }
    /// ```
    pub fn refs_to_openapi_format<SN: AsRef<str>>(&mut self, schema_name: Option<SN>) -> &mut Self {
        match schema_name {
            Some(schema_name) => self.replace_defs_with_openapi_schemas(Some(schema_name.as_ref())),
            None => self.replace_defs_with_openapi_schemas(None),
        }
    }
}

impl Schema {
    /// Extract other json schemas from the root $defs
    ///
    /// Example for:
    ///
    /// ```json
    ///  "MySchema": {
    ///    "type": "object",
    ///    "$defs": {
    ///      "address": {
    ///        "type": "object",
    ///        "required": [
    ///          "street",
    ///          "city"
    ///        ],
    ///        "properties": {
    ///          "city": {
    ///            "$ref": "#/$defs/nonEmptyString"
    ///          },
    ///          "street": {
    ///            "$ref": "#/$defs/nonEmptyString"
    ///          }
    ///        }
    ///      },
    ///      "nonEmptyString": {
    ///        "type": "string",
    ///        "minLength": 1
    ///      }
    ///    }
    ///  }
    /// ````
    ///
    /// you get:
    ///
    /// ```json
    ///   "address": {
    ///     "type": "object",
    ///     "required": [
    ///       "street",
    ///       "city"
    ///     ],
    ///     "properties": {
    ///       "city": {
    ///         "$ref": "#/$defs/nonEmptyString"
    ///       },
    ///       "street": {
    ///         "$ref": "#/$defs/nonEmptyString"
    ///       }
    ///     }
    ///   },
    /// ```
    ///
    /// and
    ///
    /// ```json
    ///   "nonEmptyString": {
    ///     "type": "string",
    ///     "minLength": 1
    ///   }
    /// ```
    pub fn retrive_schemas_from_defs(&self) -> BTreeMap<&str, &Schema> {
        let mut result = BTreeMap::new();
        if let Some(defs) = self.defs() {
            result.extend(defs.iter().map(|(key, value)| (key.as_str(), value)))
        }

        result
    }
}

impl OpenApi {
    /// Used to extract defs from the direct openapi components schemas as the schemas.
    ///
    /// It keeps original defs unchanged.
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
        if let Some(components) = &mut self.components {
            let mut additional_schemas: Vec<(String, RefOr<Schema>)> = Vec::new();

            for schema in components
                .schemas
                .values()
                .filter_map(|schema| match schema {
                    RefOr::T(t) => Some(t),
                    _ => None,
                })
            {
                additional_schemas.extend(
                    schema
                        .retrive_schemas_from_defs()
                        .into_iter()
                        .map(|(key, schema)| (key.to_string(), RefOr::T(schema.clone()))),
                )
            }

            components.schemas.extend(additional_schemas);
        }

        self
    }

    /// Used to replace /#defs/ with openapi /components/schemas/<schema-name>
    /// for all components schemas (recursively)
    ///
    /// See also: [`RefOr::refs_to_openapi_format`]
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
    ///    "$ref": /#components/schemas/<schema-name>/a/b"
    /// }
    /// ```
    pub fn refs_to_openapi_format<SN: AsRef<str>>(&mut self, schema_name: Option<SN>) -> &mut Self {
        if let Some(components) = &mut self.components {
            for schema in components.schemas.values_mut() {
                match &schema_name {
                    Some(schema_name) => {
                        schema.replace_defs_with_openapi_schemas(Some(schema_name.as_ref()))
                    }
                    None => schema.replace_defs_with_openapi_schemas(None),
                };
            }
        }

        self
    }
}

const DEFS_PREFIX: &str = "/#defs/";
const COMPONENTS_PREFIX: &str = "/#components/schemas/";

impl RefOr<Schema> {
    /// Used to replace /#defs/ with openapi /components/schemas/<schema-name>
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
    ///    "$ref": /#components/schemas/<schema-name>/a/b"
    /// }
    /// ```
    pub fn refs_to_openapi_format<SN: AsRef<str>>(
        &mut self,
        schema_name: Option<SN>,
    ) -> &mut Self {
        // map caused some compiler errors due to ownership
        match schema_name {
            Some(schema_name) => self.replace_defs_with_openapi_schemas(Some(schema_name.as_ref())),
            None => self.replace_defs_with_openapi_schemas(None),
        };
        self
    }
}

trait RefRootDefsReplace {
    fn replace_defs_with_openapi_schemas(&mut self, schema_name: Option<&str>) -> &mut Self;
}

impl RefRootDefsReplace for RefOr<Schema> {
    fn replace_defs_with_openapi_schemas(&mut self, schema_name: Option<&str>) -> &mut Self {
        match self {
            // use non recursive version for the ref itself
            RefOr::Ref(ref_) => {
                if ref_.ref_location.starts_with(DEFS_PREFIX) {
                    let replacement = schema_name
                        .map(|schema| format!("{COMPONENTS_PREFIX}{schema}/"))
                        .unwrap_or(COMPONENTS_PREFIX.into());

                    ref_.ref_location = ref_.ref_location.replacen(DEFS_PREFIX, &replacement, 1);
                }
            }

            // any schema should use recursive version
            RefOr::T(t) => {
                t.replace_defs_with_openapi_schemas(schema_name);
            }
        }

        self
    }
}

impl RefRootDefsReplace for Components {
    fn replace_defs_with_openapi_schemas(&mut self, schema_name: Option<&str>) -> &mut Self {
        for schema in &mut self.schemas {
            schema.1.replace_defs_with_openapi_schemas(schema_name);
        }

        self
    }
}

impl RefRootDefsReplace for Defs {
    fn replace_defs_with_openapi_schemas(&mut self, schema_name: Option<&str>) -> &mut Self {
        for (_, schema) in self.iter_mut() {
            schema.replace_defs_with_openapi_schemas(schema_name);
        }

        self
    }
}

impl RefRootDefsReplace for Schema {
    fn replace_defs_with_openapi_schemas(&mut self, schema_name: Option<&str>) -> &mut Self {
        match self {
            Schema::Const(_) => {}
            Schema::Array(array) => {
                array.replace_defs_with_openapi_schemas(schema_name);
            }
            Schema::Object(object) => {
                object.replace_defs_with_openapi_schemas(schema_name);
            }
            Schema::OneOf(one_of) => {
                one_of.replace_defs_with_openapi_schemas(schema_name);
            }
            Schema::AllOf(all_of) => {
                all_of.replace_defs_with_openapi_schemas(schema_name);
            }
            Schema::AnyOf(any_of) => {
                any_of.replace_defs_with_openapi_schemas(schema_name);
            }
        }

        self
    }
}

impl RefRootDefsReplace for Array {
    fn replace_defs_with_openapi_schemas(&mut self, schema_name: Option<&str>) -> &mut Self {
        self.defs.replace_defs_with_openapi_schemas(schema_name);
        for item in &mut self.prefix_items {
            item.replace_defs_with_openapi_schemas(schema_name);
        }

        match &mut self.items {
            super::schema::ArrayItems::RefOrSchema(ref_or) => {
                ref_or.replace_defs_with_openapi_schemas(schema_name);
            }
            super::schema::ArrayItems::False => {}
        }

        self
    }
}

impl RefRootDefsReplace for AdditionalProperties<Schema> {
    fn replace_defs_with_openapi_schemas(&mut self, schema_name: Option<&str>) -> &mut Self {
        match self {
            AdditionalProperties::RefOr(ref_or) => {
                ref_or.replace_defs_with_openapi_schemas(schema_name);
            }
            AdditionalProperties::FreeForm(_) => {}
        }

        self
    }
}

impl RefRootDefsReplace for Object {
    fn replace_defs_with_openapi_schemas(&mut self, schema_name: Option<&str>) -> &mut Self {
        self.defs.replace_defs_with_openapi_schemas(schema_name);

        for property in self.properties.values_mut() {
            property.replace_defs_with_openapi_schemas(schema_name);
        }

        if let Some(additional_properties) = &mut self.additional_properties {
            additional_properties.replace_defs_with_openapi_schemas(schema_name);
        }

        if let Some(property_names) = &mut self.property_names {
            property_names.replace_defs_with_openapi_schemas(schema_name);
        }

        self
    }
}

impl RefRootDefsReplace for OneOf {
    fn replace_defs_with_openapi_schemas(&mut self, schema_name: Option<&str>) -> &mut Self {
        self.defs.replace_defs_with_openapi_schemas(schema_name);

        for item in &mut self.items {
            item.replace_defs_with_openapi_schemas(schema_name);
        }

        self
    }
}

impl RefRootDefsReplace for AllOf {
    fn replace_defs_with_openapi_schemas(&mut self, schema_name: Option<&str>) -> &mut Self {
        self.defs.replace_defs_with_openapi_schemas(schema_name);

        for item in &mut self.items {
            item.replace_defs_with_openapi_schemas(schema_name);
        }

        self
    }
}
impl RefRootDefsReplace for AnyOf {
    fn replace_defs_with_openapi_schemas(&mut self, schema_name: Option<&str>) -> &mut Self {
        self.defs.replace_defs_with_openapi_schemas(schema_name);

        for item in &mut self.items {
            item.replace_defs_with_openapi_schemas(schema_name);
        }

        self
    }
}

#[cfg(test)]
#[allow(missing_docs)]
pub mod test {
    use super::*;
    use crate::openapi::{
        schema::{AnyOfBuilder, ArrayItems, RefBuilder, SchemaType, Type},
        AllOfBuilder, ArrayBuilder, ComponentsBuilder, ObjectBuilder, OneOfBuilder, OpenApiBuilder,
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

        let mut openapi = OpenApiBuilder::new()
            .components(Some(
                ComponentsBuilder::new()
                    .schema("MySchema", ObjectBuilder::new().defs(defs))
                    .build(),
            ))
            .build();

        openapi.extract_components_from_schemas_def();

        // defs should be promoted to components
        assert_eq!(openapi.components.as_ref().unwrap().schemas.len(), 3);

        let my_schema = openapi
            .components
            .unwrap()
            .schemas
            .iter()
            .find(|x| x.0 == "MySchema")
            .map(|x| match x.1 {
                RefOr::T(schema) => schema,
                _ => panic!("It should be schema"),
            })
            .unwrap()
            .clone();

        // old defs should be kept intact
        assert_eq!(my_schema.defs().map(|defs| defs.len()), Some(2));
    }

    #[test]
    fn refs_to_openapi_format_simple_ref() {
        let mut simple_ref: RefOr<Schema> = RefOr::Ref(
            RefBuilder::new()
                .ref_location("/#defs/MyStruct".into())
                .into(),
        );

        simple_ref.refs_to_openapi_format(Option::<&str>::None);

        match simple_ref {
            RefOr::Ref(x) => assert_eq!(x.ref_location, "/#components/schemas/MyStruct"),
            RefOr::T(_) => unreachable!(),
        }
    }

    #[test]
    fn refs_to_openapi_format_on_schema() {
        let simple_ref: RefOr<Schema> = RefOr::Ref(
            RefBuilder::new()
                .ref_location("/#defs/MyStruct".into())
                .into(),
        );

        let mut any_of_with_ref: Schema =
            AnyOfBuilder::new().item(simple_ref.clone()).build().into();

        any_of_with_ref
            .refs_to_openapi_format(Option::<&str>::None)
            // dual+ calls should not be a problem
            .refs_to_openapi_format(Some("dummy"))
            .refs_to_openapi_format(Some("magic-dummy"));

        let items = match any_of_with_ref {
            Schema::AnyOf(any_of) => any_of.items,
            _ => unreachable!(),
        };

        assert_eq!(items.len(), 1);
        match items.first().unwrap() {
            RefOr::Ref(ref_) => assert_eq!(ref_.ref_location, "/#components/schemas/MyStruct"),
            RefOr::T(_) => unreachable!(),
        }
    }

    #[test]
    fn refs_to_openapi_format_on_openapi() {
        let simple_ref: RefOr<Schema> = RefOr::Ref(
            RefBuilder::new()
                .ref_location("/#defs/MyStruct".into())
                .into(),
        );

        let defsobj_with_ref = ObjectBuilder::new()
            .title("BaseDefsObj".into())
            .property("base", simple_ref.clone());
        let defs = DefsBuilder::new().def("DefObj", defsobj_with_ref).build();

        let any_of_with_ref = AnyOfBuilder::new()
            .title("AnyOf".into())
            .item(simple_ref.clone())
            .defs(defs.clone())
            .build();

        let one_of_with_ref = OneOfBuilder::new()
            .title("OneOf".into())
            .item(simple_ref.clone())
            .defs(defs.clone())
            .build();

        let all_of_with_ref = AllOfBuilder::new()
            .title("AllOf".into())
            .item(simple_ref.clone())
            .defs(defs.clone())
            .build();

        let object_with_ref = ObjectBuilder::new()
            .schema_type(SchemaType::Type(Type::Object))
            .title(Some("Obj"))
            .property("prop", simple_ref.clone())
            .additional_properties(Some(AdditionalProperties::RefOr(simple_ref.clone())))
            .property_names(Some(any_of_with_ref.clone()))
            .defs(defs.clone());

        let array_with_ref = ArrayBuilder::new()
            .schema_type(SchemaType::Type(Type::Array))
            .title(Some("AraryWithRef"))
            .items(ArrayItems::RefOrSchema(Box::new(simple_ref.clone())));

        let openapi = OpenApiBuilder::new()
            .components(Some(
                ComponentsBuilder::new()
                    .schema("AnyOfRef", RefOr::T(any_of_with_ref.into()))
                    .schema("OneOffRef", RefOr::T(one_of_with_ref.into()))
                    .schema("AllOffRef", RefOr::T(all_of_with_ref.into()))
                    .schema("ObjectRef", RefOr::T(object_with_ref.into()))
                    .schema("ArrayRef", RefOr::T(array_with_ref.into()))
                    .build(),
            ))
            .build();

        let stringified = serde_json::to_string_pretty(&openapi).unwrap();
        let initial_defs_ref_count = stringified.matches(DEFS_PREFIX).count();
        let non_suffixed_stringified = serde_json::to_string_pretty(
            openapi.clone().refs_to_openapi_format(Option::<&str>::None),
        )
        .unwrap();

        assert_eq!(
            non_suffixed_stringified.matches(DEFS_PREFIX).count(),
            0,
            "No original {DEFS_PREFIX} should be present"
        );

        assert_eq!(
            initial_defs_ref_count,
            non_suffixed_stringified.matches(COMPONENTS_PREFIX).count(),
            "All occurences of {DEFS_PREFIX} should be replaced with {COMPONENTS_PREFIX}\nSchema: {non_suffixed_stringified}"
        );

        let suffix = "MySuffixSchema";
        let suffixed_stringified = serde_json::to_string_pretty(
            openapi
                .clone()
                .refs_to_openapi_format(Some(suffix))
                // dual calls should not cause any issues
                .refs_to_openapi_format(Some("dummy")),
        )
        .unwrap();

        assert_eq!(
            suffixed_stringified.matches(DEFS_PREFIX).count(),
            0,
            "No original {DEFS_PREFIX} should be present"
        );

        let suffixed = format!("{COMPONENTS_PREFIX}{suffix}/");
        assert_eq!(
            initial_defs_ref_count,
            suffixed_stringified.matches(&suffixed).count(),
            "All occurences of {DEFS_PREFIX} should be replaced with {suffixed}\nSchema: {suffixed_stringified}"
        );
    }
}
