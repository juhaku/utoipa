use std::collections::HashMap;

use serde::{Deserialize, Serialize};
#[cfg(feature = "serde_json")]
use serde_json::Value;

use super::Deprecated;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    schemas: HashMap<String, Component>,
}

impl Schema {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn with_component<S: AsRef<str>, I: Into<Component>>(
        mut self,
        name: S,
        component: I,
    ) -> Self {
        self.schemas
            .insert(name.as_ref().to_string(), component.into());

        self
    }
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Component {
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    property: Option<Property>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    ref_component: Option<Ref>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    struct_component: Option<Object>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    array_component: Option<Array>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    one_of: Vec<Component>,
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct OneOf {
    items: Vec<Component>,
}

impl OneOf {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }

    pub fn append<I: Into<Component>>(mut self, component: I) -> Self {
        self.items.push(component.into());

        self
    }
}

impl From<OneOf> for Component {
    fn from(one_of: OneOf) -> Self {
        Self {
            one_of: one_of.items,
            ..Default::default()
        }
    }
}

#[derive(Default, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Property {
    #[serde(rename = "type")]
    component_type: ComponentType,

    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<ComponentFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "serde_json")]
    default: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(not(feature = "serde_json"))]
    default: Option<String>,

    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    enum_values: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(not(feature = "serde_json"))]
    example: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "serde_json")]
    example: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    deprecated: Option<Deprecated>,

    #[serde(skip_serializing_if = "Option::is_none")]
    write_only: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    read_only: Option<bool>,
}

impl Property {
    pub fn new(component_type: ComponentType) -> Self {
        Self {
            component_type,
            ..Default::default()
        }
    }

    pub fn with_format(mut self, format: ComponentFormat) -> Self {
        self.format = Some(format);

        self
    }

    pub fn with_description<S: AsRef<str>>(mut self, description: S) -> Self {
        self.description = Some(description.as_ref().to_string());

        self
    }

    #[cfg(feature = "serde_json")]
    pub fn with_default(mut self, default: Value) -> Self {
        self.default = Some(default);

        self
    }

    #[cfg(not(feature = "serde_json"))]
    pub fn with_default<I: Into<String>>(mut self, default: I) -> Self {
        self.default = Some(default.into());

        self
    }

    pub fn with_enum_values<S: AsRef<str>>(mut self, enum_values: &[S]) -> Self {
        self.enum_values = Some(
            enum_values
                .iter()
                .map(|str| str.as_ref().to_string())
                .collect(),
        );

        self
    }

    #[cfg(not(feature = "serde_json"))]
    pub fn with_example<I: Into<String>>(mut self, example: I) -> Self {
        self.example = Some(example.into());

        self
    }

    #[cfg(feature = "serde_json")]
    pub fn with_example(mut self, example: Value) -> Self {
        self.example = Some(example);

        self
    }

    pub fn with_deprecated(mut self, deprecated: Deprecated) -> Self {
        self.deprecated = Some(deprecated);

        self
    }

    pub fn with_write_only(mut self, write_only: bool) -> Self {
        self.write_only = Some(write_only);

        self
    }

    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = Some(read_only);

        self
    }
}

impl From<Property> for Component {
    fn from(property: Property) -> Self {
        Self {
            property: Some(property),
            ..Default::default()
        }
    }
}

impl ToArray for Property {}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Object {
    #[serde(rename = "type")]
    component_type: ComponentType,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    required: Vec<String>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    properties: HashMap<String, Component>,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    deprecated: Option<Deprecated>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "serde_json")]
    example: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(not(feature = "serde_json"))]
    example: Option<String>,
}

impl Object {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn with_property<S: AsRef<str>, I: Into<Component>>(
        mut self,
        property_name: S,
        component: I,
    ) -> Self {
        self.properties
            .insert(property_name.as_ref().to_string(), component.into());

        self
    }

    pub fn with_required<S: AsRef<str>>(mut self, required_field: S) -> Self {
        self.required.push(required_field.as_ref().to_string());

        self
    }

    pub fn with_description<S: AsRef<str>>(mut self, description: S) -> Self {
        self.description = Some(description.as_ref().to_string());

        self
    }

    pub fn with_deprecated(mut self, deprecated: Deprecated) -> Self {
        self.deprecated = Some(deprecated);

        self
    }

    #[cfg(feature = "serde_json")]
    pub fn with_example(mut self, example: Value) -> Self {
        self.example = Some(example);

        self
    }

    #[cfg(not(feature = "serde_json"))]
    pub fn with_example<I: Into<String>>(mut self, example: I) -> Self {
        self.example = Some(example.into());

        self
    }
}

impl From<Object> for Component {
    fn from(s: Object) -> Self {
        Self {
            struct_component: Some(s),
            ..Default::default()
        }
    }
}

impl ToArray for Object {}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Ref {
    #[serde(rename = "$ref")]
    ref_location: String,
}

impl Ref {
    pub fn new<S: AsRef<str>>(ref_location: S) -> Self {
        Self {
            ref_location: ref_location.as_ref().to_string(),
        }
    }

    pub fn from_component_name<S: AsRef<str>>(component_name: S) -> Self {
        Self::new(&format!("#/components/schemas/{}", component_name.as_ref()))
    }
}

impl From<Ref> for Component {
    fn from(r: Ref) -> Self {
        Self {
            ref_component: Some(r),
            ..Default::default()
        }
    }
}

impl ToArray for Ref {}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Array {
    #[serde(rename = "type")]
    component_type: ComponentType,

    items: Box<Component>,

    #[serde(skip_serializing_if = "Option::is_none")]
    max_items: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    min_items: Option<usize>,
}

impl Array {
    pub fn new<I: Into<Component>>(component: I) -> Self {
        Self {
            component_type: ComponentType::Array,
            items: Box::new(component.into()),
            ..Default::default()
        }
    }

    pub fn with_max_items(mut self, max_items: usize) -> Self {
        self.max_items = Some(max_items);

        self
    }

    pub fn with_min_items(mut self, min_items: usize) -> Self {
        self.min_items = Some(min_items);

        self
    }
}

impl From<Array> for Component {
    fn from(array: Array) -> Self {
        Self {
            array_component: Some(array),
            ..Default::default()
        }
    }
}

impl ToArray for Array {}

pub trait ToArray
where
    Component: From<Self>,
    Self: Sized,
{
    fn to_array(self) -> Array {
        Array::new(self)
    }
}

#[derive(Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ComponentType {
    Object,
    String,
    Integer,
    Number,
    Boolean,
    Array,
}

impl Default for ComponentType {
    fn default() -> Self {
        Self::Object
    }
}

impl Serialize for ComponentType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Object => serializer.serialize_str("object"),
            Self::String => serializer.serialize_str("string"),
            Self::Integer => serializer.serialize_str("integer"),
            Self::Number => serializer.serialize_str("number"),
            Self::Boolean => serializer.serialize_str("boolean"),
            Self::Array => serializer.serialize_str("array"),
        }
    }
}

#[derive(Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ComponentFormat {
    Int32,
    Int64,
    Float,
    Double,
    Byte,
    Binary,
    Date,
    DateTime,
    Password,
}

impl Serialize for ComponentFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Int32 => serializer.serialize_str("int32"),
            Self::Int64 => serializer.serialize_str("int64"),
            Self::Float => serializer.serialize_str("float"),
            Self::Double => serializer.serialize_str("double"),
            Self::Byte => serializer.serialize_str("byte"),
            Self::Binary => serializer.serialize_str("binary"),
            Self::Date => serializer.serialize_str("date"),
            Self::DateTime => serializer.serialize_str("date-time"),
            Self::Password => serializer.serialize_str("password"),
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "serde_json")]
    use serde_json::{json, Value};

    use super::*;
    use crate::openapi::*;

    #[test]
    #[cfg(feature = "serde_json")]
    fn create_schema_serializes_json() -> Result<(), serde_json::Error> {
        let openapi = OpenApi::new(Info::new("My api", "1.0.0"), Paths::new()).with_components(
            Schema::new()
                .with_component("Person", Ref::new("#/components/PersonModel"))
                .with_component(
                    "Credential",
                    Object::new()
                        .with_property(
                            "id",
                            Property::new(ComponentType::Integer)
                                .with_format(ComponentFormat::Int32)
                                .with_description("Id of credential")
                                .with_default(json!(1)),
                        )
                        .with_property(
                            "name",
                            Property::new(ComponentType::String)
                                .with_description("Name of credential"),
                        )
                        .with_property(
                            "status",
                            Property::new(ComponentType::String)
                                .with_default(json!("Active"))
                                .with_description("Credential status")
                                .with_enum_values(&["Active", "NotActive", "Locked", "Expired"]),
                        )
                        .with_property(
                            "history",
                            Array::new(Ref::from_component_name("UpdateHistory")),
                        )
                        .with_property("tags", Property::new(ComponentType::String).to_array()),
                ),
        );

        let serialized = serde_json::to_string_pretty(&openapi)?;
        println!("serialized json:\n {}", serialized);

        let value = serde_json::to_value(&openapi)?;
        let credential = get_json_path(&value, "components.schemas.Credential.properties");
        let person = get_json_path(&value, "components.schemas.Person");

        assert!(
            credential.get("id").is_some(),
            "could not find path: components.schemas.Credential.properties.id"
        );
        assert!(
            credential.get("status").is_some(),
            "could not find path: components.schemas.Credential.properties.status"
        );
        assert!(
            credential.get("name").is_some(),
            "could not find path: components.schemas.Credential.properties.name"
        );
        assert!(
            credential.get("history").is_some(),
            "could not find path: components.schemas.Credential.properties.history"
        );
        assert_eq!(
            credential
                .get("id")
                .unwrap_or(&serde_json::value::Value::Null)
                .to_string(),
            r#"{"default":1,"description":"Id of credential","format":"int32","type":"integer"}"#,
            "components.schemas.Credential.properties.id did not match"
        );
        assert_eq!(
            credential
                .get("name")
                .unwrap_or(&serde_json::value::Value::Null)
                .to_string(),
            r#"{"description":"Name of credential","type":"string"}"#,
            "components.schemas.Credential.properties.name did not match"
        );
        assert_eq!(
            credential
                .get("status")
                .unwrap_or(&serde_json::value::Value::Null)
                .to_string(),
            r#"{"default":"Active","description":"Credential status","enum":["Active","NotActive","Locked","Expired"],"type":"string"}"#,
            "components.schemas.Credential.properties.status did not match"
        );
        assert_eq!(
            credential
                .get("history")
                .unwrap_or(&serde_json::value::Value::Null)
                .to_string(),
            r###"{"items":{"$ref":"#/components/schemas/UpdateHistory"},"type":"array"}"###,
            "components.schemas.Credential.properties.history did not match"
        );
        assert_eq!(
            person.to_string(),
            r###"{"$ref":"#/components/PersonModel"}"###,
            "components.schemas.Person.ref did not match"
        );

        Ok(())
    }

    #[test]
    #[cfg(feature = "serde_json")]
    fn derive_object_with_example() {
        let expected = r#"{"type":"object","example":{"age":20,"name":"bob the cat"}}"#;
        let json_value = Object::new().with_example(json!({"age": 20, "name": "bob the cat"}));

        let value_string = serde_json::to_string(&json_value).unwrap();
        assert_eq!(
            value_string, expected,
            "value string != expected string, {} != {}",
            value_string, expected
        );
    }

    #[cfg(feature = "serde_json")]
    fn get_json_path<'a>(value: &'a Value, path: &str) -> &'a Value {
        path.split('.').into_iter().fold(value, |acc, fragment| {
            acc.get(fragment).unwrap_or(&serde_json::value::Value::Null)
        })
    }
}
