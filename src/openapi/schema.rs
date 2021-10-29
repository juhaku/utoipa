use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default)]
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

    pub fn with_component<S: AsRef<str>>(mut self, name: S, component: Component) -> Self {
        self.schemas.insert(name.as_ref().to_string(), component);

        self
    }
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    base_component: Option<BaseComponent>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    ref_component: Option<RefComponent>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    object_component: Option<ObjectComponent>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    array_component: Option<ArrayComponent>,
}

impl Component {
    pub fn new<S: AsRef<str>>(
        component_type: ComponentType,
        component_format: Option<ComponentFormat>,
        default_value: Option<S>,
        description: Option<S>,
        enum_values: Option<Vec<S>>,
    ) -> Self {
        Self {
            base_component: Some(BaseComponent {
                component_type,
                format: component_format,
                default: default_value.map(|value| value.as_ref().to_string()),
                description: description.map(|value| value.as_ref().to_string()),
                enum_values: enum_values.map(|values| {
                    values
                        .into_iter()
                        .map(|value| value.as_ref().to_string())
                        .collect()
                }),
            }),
            ..Default::default()
        }
    }
}

impl From<RefComponent> for Component {
    fn from(ref_component: RefComponent) -> Self {
        Self {
            ref_component: Some(ref_component),
            ..Default::default()
        }
    }
}

impl From<ObjectComponent> for Component {
    fn from(object_component: ObjectComponent) -> Self {
        Self {
            base_component: Some(BaseComponent::default()),
            object_component: Some(object_component),
            ..Default::default()
        }
    }
}

impl From<ArrayComponent> for Component {
    fn from(array_component: ArrayComponent) -> Self {
        Self {
            base_component: Some(BaseComponent {
                component_type: ComponentType::Array,
                ..Default::default()
            }),
            array_component: Some(array_component),
            ..Default::default()
        }
    }
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
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

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BaseComponent {
    #[serde(rename = "type")]
    component_type: ComponentType,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<ComponentFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<String>,

    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    enum_values: Option<Vec<String>>,
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ObjectComponent {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    required: Vec<String>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    properties: HashMap<String, Component>,
}

impl ObjectComponent {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn with_property<S: AsRef<str>>(mut self, property_name: S, component: Component) -> Self {
        self.properties
            .insert(property_name.as_ref().to_string(), component);

        self
    }
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default)]
pub struct RefComponent {
    #[serde(rename = "$ref")]
    ref_location: String,
}

impl RefComponent {
    pub fn new<S: AsRef<str>>(ref_location: S) -> Self {
        Self {
            ref_location: ref_location.as_ref().to_string(),
        }
    }

    pub fn from_component_name<S: AsRef<str>>(component_name: S) -> Self {
        Self::new(&format!("#/components/schemas/{}", component_name.as_ref()))
    }
}

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default)]
pub struct ArrayComponent {
    items: Box<Component>,
}

impl ArrayComponent {
    pub fn new(component: Component) -> Self {
        Self {
            items: Box::new(component),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;
    use crate::openapi::*;

    #[test]
    fn create_schema_serializes_json() -> Result<(), Error> {
        let openapi = OpenApi::new(Info::new("My api", "1.0.0"), Paths::new()).with_components(
            Schema::new()
                .with_component(
                    "Person",
                    RefComponent::new("#/components/PersonModel").into(),
                )
                .with_component(
                    "Credential",
                    ObjectComponent::new()
                        .with_property(
                            "id",
                            Component::new(
                                ComponentType::Integer,
                                Some(ComponentFormat::Int32),
                                Some("1"),
                                Some("Id of credential"),
                                None,
                            ),
                        )
                        .with_property(
                            "name",
                            Component::new(
                                ComponentType::String,
                                None,
                                None,
                                Some("Name of credential"),
                                None,
                            ),
                        )
                        .with_property(
                            "status",
                            Component::new(
                                ComponentType::String,
                                None,
                                Some("Active"),
                                Some("Credential status"),
                                Some(vec!["Active", "NotActive", "Locked", "Expired"]),
                            ),
                        )
                        .with_property(
                            "history",
                            ArrayComponent::new(
                                RefComponent::from_component_name("UpdateHistory").into(),
                            )
                            .into(),
                        )
                        .into(),
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
            r#"{"default":"1","description":"Id of credential","format":"int32","type":"integer"}"#,
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

    fn get_json_path<'a>(value: &'a Value, path: &str) -> &'a Value {
        path.split('.').into_iter().fold(value, |acc, fragment| {
            acc.get(fragment).unwrap_or(&serde_json::value::Value::Null)
        })
    }
}
