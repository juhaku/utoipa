//! Implements [OpenAPI Schema Object][schema] types which can be
//! used to define field properties, enum values, array or object types.
//!
//! [schema]: https://spec.openapis.org/oas/latest.html#schema-object
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
#[cfg(feature = "serde_json")]
use serde_json::Value;

use super::{
    build_fn, builder, from, new, security::SecurityScheme, set_value, xml::Xml, Deprecated,
};

macro_rules! component_from_builder {
    ( $name:ident ) => {
        impl From<$name> for Component {
            fn from(builder: $name) -> Self {
                builder.build().into()
            }
        }
    };
}

macro_rules! to_array_builder {
    () => {
        /// Construct a new [`ArrayBuilder`] with this component set to [`ArrayBuilder::items`].
        pub fn to_array_builder(self) -> ArrayBuilder {
            ArrayBuilder::from(Array::new(self))
        }
    };
}

builder! {
    ComponentsBuilder;

    /// Implements [OpenAPI Components Object][components] which holds supported
    /// reusable objects.
    ///
    /// [components]: https://spec.openapis.org/oas/latest.html#components-object
    #[non_exhaustive]
    #[derive(Serialize, Deserialize, Default, Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[serde(rename_all = "camelCase")]
    pub struct Components {
        /// Map of reusable [OpenAPI Schema Object][schema]s.
        ///
        /// [schema]: https://spec.openapis.org/oas/latest.html#schema-object
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        pub schemas: BTreeMap<String, Component>,

        /// Map of reusable [OpenAPI Security Schema Object][security_schema]s.
        ///
        /// [security_schema]: https://spec.openapis.org/oas/latest.html#security-scheme-object
        #[serde(skip_serializing_if = "BTreeMap::is_empty")]
        pub security_schemes: BTreeMap<String, SecurityScheme>,
    }
}

impl Components {
    /// Construct a new [`Components`].
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    /// Add [`SecurityScheme`] to [`Components`]
    ///
    /// Accepts two arguments where first is the name of the [`SecurityScheme`]. This is later when
    /// referenced by [`SecurityRequirement`][requirement]s. Second parameter is the [`SecurityScheme`].
    ///
    /// [requirement]: ../security/struct.SecurityRequirement.html
    pub fn add_security_scheme<N: Into<String>, S: Into<SecurityScheme>>(
        &mut self,
        name: N,
        security_schema: S,
    ) {
        self.security_schemes
            .insert(name.into(), security_schema.into());
    }

    /// Add iterator of [`SecurityScheme`]s to [`Components`].
    ///
    /// Accepts two arguments where first is the name of the [`SecurityScheme`]. This is later when
    /// referenced by [`SecurityRequirement`][requirement]s. Second parameter is the [`SecurityScheme`].
    ///
    /// [requirement]: ../security/struct.SecurityRequirement.html
    pub fn add_security_schemes_from_iter<
        I: IntoIterator<Item = (N, S)>,
        N: Into<String>,
        S: Into<SecurityScheme>,
    >(
        &mut self,
        schemas: I,
    ) {
        self.security_schemes.extend(
            schemas
                .into_iter()
                .map(|(name, item)| (name.into(), item.into())),
        );
    }
}

impl ComponentsBuilder {
    /// Add [`Component`] to [`Components`].
    ///
    /// Accpets two arguments where first is name of the component and second is the component itself.
    pub fn component<S: Into<String>, I: Into<Component>>(mut self, name: S, component: I) -> Self {
        self.schemas.insert(name.into(), component.into());

        self
    }

    /// Add [`Component`]s from iterator.
    ///
    /// # Examples
    /// ```rust
    /// # use utoipa::openapi::schema::{ComponentsBuilder, ObjectBuilder,
    /// #    PropertyBuilder, ComponentType};
    /// ComponentsBuilder::new().components_from_iter([(
    ///     "Pet",
    ///     ObjectBuilder::new()
    ///         .property(
    ///             "name",
    ///             PropertyBuilder::new().component_type(ComponentType::String),
    ///         )
    ///         .required("name"),
    /// )]);
    /// ```
    pub fn components_from_iter<
        I: IntoIterator<Item = (S, C)>,
        C: Into<Component>,
        S: Into<String>,
    >(
        mut self,
        components: I,
    ) -> Self {
        self.schemas.extend(
            components
                .into_iter()
                .map(|(name, component)| (name.into(), component.into())),
        );

        self
    }

    /// Add [`SecurityScheme`] to [`Components`].
    ///
    /// Accepts two arguments where first is the name of the [`SecurityScheme`]. This is later when
    /// referenced by [`SecurityRequirement`][requirement]s. Second parameter is the [`SecurityScheme`].
    ///
    /// [requirement]: ../security/struct.SecurityRequirement.html
    pub fn security_scheme<N: Into<String>, S: Into<SecurityScheme>>(
        mut self,
        name: N,
        security_schema: S,
    ) -> Self {
        self.security_schemes
            .insert(name.into(), security_schema.into());

        self
    }
}

/// Is super type for [OpenAPI Schema Object][components] components. Component
/// is reusable resource what can be referenced from path operations and other
/// components using [`Ref`] component.
///
/// [components]: https://spec.openapis.org/oas/latest.html#components-object
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(untagged, rename_all = "camelCase")]
pub enum Component {
    /// Defines object component. This is formed from structs holding [`Property`] components
    /// created from it's fields.
    Object(Object),
    /// Defines property component typically used together with
    /// [`Component::Object`] or [`Component::Array`]. It is used to map
    /// field types to OpenAPI documentation.
    Property(Property),
    /// Creates a reference component _`$ref=#/components/schemas/ComponentName`_. Which
    /// can be used to reference a other reusable component in [`Components`].
    Ref(Ref),
    /// Defines array component from another component. Typically used with
    /// [`Component::Property`] or [`Component::Object`] component. Slice and Vec
    /// types are translated to [`Component::Array`] types.
    Array(Array),
    /// Creates a _OneOf_ type [Discriminator Object][discriminator] component. This component
    /// is used to map multiple components together where API endpoint could return any of them.
    /// [`Component::OneOf`] is created form complex enum where enum holds other than unit types.
    ///
    /// [discriminator]: https://spec.openapis.org/oas/latest.html#components-object
    OneOf(OneOf),
}

impl Default for Component {
    fn default() -> Self {
        Component::Object(Object::default())
    }
}

builder! {
    OneOfBuilder;

    /// OneOf [Discriminator Object][discriminator] component holds
    /// multiple components together where API endpoint could return any of them.
    ///
    /// See [`Component::OneOf`] for more details.
    ///
    /// [discriminator]: https://spec.openapis.org/oas/latest.html#components-object
    #[derive(Serialize, Deserialize, Clone, Default)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct OneOf {
        /// Components of _OneOf_ component.
        #[serde(rename = "oneOf")]
        pub items: Vec<Component>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
    }
}

impl OneOf {
    /// Construct a new [`OneOf`] component.
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Construct a new [`OneOf`] component with given capacity.
    ///
    /// OneOf component is then able to contain number of components without
    /// reallocating.
    ///
    /// # Examples
    ///
    /// Create [`OneOf`] component with initial capacity of 5.
    /// ```rust
    /// # use utoipa::openapi::schema::OneOf;
    /// let one_of = OneOf::with_capacity(5);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            description: None,
        }
    }
}

impl OneOfBuilder {
    /// Adds a given [`Component`] to [`OneOf`] [Discriminator Object][discriminator]
    ///
    /// [discriminator]: https://spec.openapis.org/oas/latest.html#components-object
    pub fn item<I: Into<Component>>(mut self, component: I) -> Self {
        self.items.push(component.into());

        self
    }

    /// Add or change optional description for `OneOf` component.
    pub fn description<I: Into<String>>(mut self, description: Option<I>) -> Self {
        set_value!(self description description.map(|description| description.into()))
    }

    to_array_builder!();
}

impl From<OneOf> for Component {
    fn from(one_of: OneOf) -> Self {
        Self::OneOf(one_of)
    }
}

component_from_builder!(OneOfBuilder);

/// Implements special subset of [OpenAPI Schema Object][schema] which can be
/// used to define field property or enum values or type for array items.
///
/// [schema]: https://spec.openapis.org/oas/latest.html#schema-object
#[derive(Default, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Property {
    /// Type of the property e.g [`ComponentType::String`].
    #[serde(rename = "type")]
    pub component_type: ComponentType,

    /// Changes the [`Property`] title.
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    /// Additional format for detailing the component type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ComponentFormat>,

    /// Description of the property. Markdown syntax is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Default value for the property which is provided when user has not provided the input.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "serde_json")]
    pub default: Option<Value>,

    /// Default value for the property which is provided when user has not provided the input.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(not(feature = "serde_json"))]
    pub default: Option<String>,

    /// Enum type property possible variants.
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,

    /// Example shown in UI of the value for richier documentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(not(feature = "serde_json"))]
    pub example: Option<String>,

    /// Example shown in UI of the value for richier documentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "serde_json")]
    pub example: Option<Value>,

    /// Changes the [`Property`] deprecated status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<Deprecated>,

    /// Write only property will be only sent in _write_ requests like _POST, PUT_.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_only: Option<bool>,

    /// Read only property will be only sent in _read_ requests like _GET_.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,

    /// Additional [`Xml`] formatting of the [`Property`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xml: Option<Xml>,
}

impl Property {
    pub fn new(component_type: ComponentType) -> Self {
        Self {
            component_type,
            ..Default::default()
        }
    }
}

impl From<Property> for Component {
    fn from(property: Property) -> Self {
        Self::Property(property)
    }
}

impl ToArray for Property {}

/// Builder for [`Property`] with chainable configuration methods to create a new [`Property`].
#[derive(Default)]
pub struct PropertyBuilder {
    component_type: ComponentType,

    title: Option<String>,

    format: Option<ComponentFormat>,

    description: Option<String>,

    #[cfg(feature = "serde_json")]
    default: Option<Value>,

    #[cfg(not(feature = "serde_json"))]
    default: Option<String>,

    enum_values: Option<Vec<String>>,

    #[cfg(not(feature = "serde_json"))]
    example: Option<String>,

    #[cfg(feature = "serde_json")]
    example: Option<Value>,

    deprecated: Option<Deprecated>,

    write_only: Option<bool>,

    read_only: Option<bool>,

    xml: Option<Xml>,
}

from!(Property PropertyBuilder
    component_type, title, format, description, default, enum_values, example, deprecated, write_only, read_only, xml);

impl PropertyBuilder {
    new!(pub PropertyBuilder);

    /// Add or change type of the property e.g [`ComponentType::String`].
    pub fn component_type(mut self, component_type: ComponentType) -> Self {
        set_value!(self component_type component_type)
    }

    /// Add or change the title of the [`Property`].
    pub fn title<I: Into<String>>(mut self, title: Option<I>) -> Self {
        set_value!(self title title.map(|title| title.into()))
    }

    /// Add or change additional format for detailing the component type.
    pub fn format(mut self, format: Option<ComponentFormat>) -> Self {
        set_value!(self format format)
    }

    /// Add or change description of the property. Markdown syntax is supported.
    pub fn description<I: Into<String>>(mut self, description: Option<I>) -> Self {
        set_value!(self description description.map(|description| description.into()))
    }

    /// Add or change default value for the property which is provided when user has not provided the input.
    #[cfg(feature = "serde_json")]
    pub fn default(mut self, default: Option<Value>) -> Self {
        set_value!(self default default)
    }

    /// Add or change default value for the property which is provided when user has not provided the input.
    #[cfg(not(feature = "serde_json"))]
    pub fn default<I: Into<String>>(mut self, default: Option<I>) -> Self {
        set_value!(self default default.map(|default| default.into()))
    }

    /// Add or change enum property variants.
    pub fn enum_values<I: IntoIterator<Item = E>, E: Into<String>>(
        mut self,
        enum_values: Option<I>,
    ) -> Self {
        set_value!(self enum_values
            enum_values.map(|values| values.into_iter().map(|enum_value| enum_value.into()).collect()))
    }

    /// Add or change example shown in UI of the value for richier documentation.
    #[cfg(not(feature = "serde_json"))]
    pub fn example<I: Into<String>>(mut self, example: Option<I>) -> Self {
        set_value!(self example example.map(|example| example.into()))
    }

    /// Add or change example shown in UI of the value for richier documentation.
    #[cfg(feature = "serde_json")]
    pub fn example(mut self, example: Option<Value>) -> Self {
        set_value!(self example example)
    }

    /// Add or change deprecated status for [`Property`].
    pub fn deprecated(mut self, deprecated: Option<Deprecated>) -> Self {
        set_value!(self deprecated deprecated)
    }

    /// Add or change write only flag for [`Property`].
    pub fn write_only(mut self, write_only: Option<bool>) -> Self {
        set_value!(self write_only write_only)
    }

    /// Add or change read only flag for [`Property`].
    pub fn read_only(mut self, read_only: Option<bool>) -> Self {
        set_value!(self read_only read_only)
    }

    /// Add or change additional [`Xml`] formatting of the [`Property`].
    pub fn xml(mut self, xml: Option<Xml>) -> Self {
        set_value!(self xml xml)
    }

    to_array_builder!();

    build_fn!(pub Property
        component_type, title, format, description, default, enum_values, example, deprecated, write_only, read_only, xml);
}

component_from_builder!(PropertyBuilder);

/// Implements subset of [OpenAPI Schema Object][schema] which allows
/// adding other [`Component`]s as **properties** to this [`Component`].
///
/// [schema]: https://spec.openapis.org/oas/latest.html#schema-object
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Object {
    /// Data type of [`Object`]. Will always be [`ComponentType::Object`]
    #[serde(rename = "type")]
    component_type: ComponentType,

    /// Changes the [`Object`] title.
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    /// Vector of required field names.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,

    /// Map of fields with their [`Component`] types.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, Component>,

    /// Additional [`Component`] for non specified fields (Useful for typed maps).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<Box<Component>>,

    /// Description of the [`Object`]. Markdown syntax is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Changes the [`Object`] deprecated status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<Deprecated>,

    /// Example shown in UI of the value for richier documentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "serde_json")]
    pub example: Option<Value>,

    /// Example shown in UI of the value for richier documentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(not(feature = "serde_json"))]
    pub example: Option<String>,

    /// Additional [`Xml`] formatting of the [`Object`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xml: Option<Xml>,
}

impl Object {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl From<Object> for Component {
    fn from(s: Object) -> Self {
        Self::Object(s)
    }
}

impl ToArray for Object {}

/// Builder for [`Object`] with chainable configuration methods to create a new [`Object`].
#[derive(Default)]
pub struct ObjectBuilder {
    component_type: ComponentType,

    title: Option<String>,

    required: Vec<String>,

    properties: BTreeMap<String, Component>,

    additional_properties: Option<Box<Component>>,

    description: Option<String>,

    deprecated: Option<Deprecated>,

    #[cfg(feature = "serde_json")]
    example: Option<Value>,

    #[cfg(not(feature = "serde_json"))]
    example: Option<String>,

    xml: Option<Xml>,
}

impl ObjectBuilder {
    new!(pub ObjectBuilder);

    /// Add new property to the [`Object`].
    ///
    /// Method accepts property name and property component as an arguments.
    pub fn property<S: Into<String>, I: Into<Component>>(
        mut self,
        property_name: S,
        component: I,
    ) -> Self {
        self.properties
            .insert(property_name.into(), component.into());

        self
    }

    pub fn additional_properties<I: Into<Component>>(
        mut self,
        additional_properties: Option<I>,
    ) -> Self {
        set_value!(self additional_properties additional_properties.map(|additional_properties| Box::new(additional_properties.into())))
    }

    /// Add field to the required fields of [`Object`].
    pub fn required<I: Into<String>>(mut self, required_field: I) -> Self {
        self.required.push(required_field.into());

        self
    }

    /// Add or change the title of the [`Object`].
    pub fn title<I: Into<String>>(mut self, title: Option<I>) -> Self {
        set_value!(self title title.map(|title| title.into()))
    }

    /// Add or change description of the property. Markdown syntax is supported.
    pub fn description<I: Into<String>>(mut self, description: Option<I>) -> Self {
        set_value!(self description description.map(|description| description.into()))
    }

    /// Add or change deprecated status for [`Object`].
    pub fn deprecated(mut self, deprecated: Option<Deprecated>) -> Self {
        set_value!(self deprecated deprecated)
    }

    /// Add or change example shown in UI of the value for richier documentation.
    #[cfg(feature = "serde_json")]
    pub fn example(mut self, example: Option<Value>) -> Self {
        set_value!(self example example)
    }

    /// Add or change example shown in UI of the value for richier documentation.
    #[cfg(not(feature = "serde_json"))]
    pub fn example<I: Into<String>>(mut self, example: Option<I>) -> Self {
        set_value!(self example example.map(|example| example.into()))
    }

    /// Add or change additional [`Xml`] formatting of the [`Object`].
    pub fn xml(mut self, xml: Option<Xml>) -> Self {
        set_value!(self xml xml)
    }

    to_array_builder!();

    build_fn!(pub Object component_type, title, required, properties, description, deprecated, example, xml, additional_properties);
}

from!(Object ObjectBuilder component_type, title, required, properties, description, deprecated, example, xml, additional_properties);
component_from_builder!(ObjectBuilder);

/// Implements [OpenAPI Reference Object][reference] that can be used to reference
/// reusable components.
///
/// [reference]: https://spec.openapis.org/oas/latest.html#reference-object
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Ref {
    /// Reference location of the actual component.
    #[serde(rename = "$ref")]
    pub ref_location: String,
}

impl Ref {
    /// Construct a new [`Ref`] with custom ref location. In most cases this is not necessary
    /// and [`Ref::from_component_name`] could be used instead.
    pub fn new<I: Into<String>>(ref_location: I) -> Self {
        Self {
            ref_location: ref_location.into(),
        }
    }

    /// Construct a new [`Ref`] from provided component name. This will create a [`Ref`] that
    /// references the the reusable schemas.
    pub fn from_component_name<I: Into<String>>(component_name: I) -> Self {
        Self::new(&format!("#/components/schemas/{}", component_name.into()))
    }

    /// Construct a new [`Ref`] from provided response name. This will create a [`Ref`] that
    /// references the reusable response.
    pub fn from_response_name<I: Into<String>>(response_name: I) -> Self {
        Self::new(&format!("#/components/responses/{}", response_name.into()))
    }

    to_array_builder!();
}

impl From<Ref> for Component {
    fn from(r: Ref) -> Self {
        Self::Ref(r)
    }
}

impl ToArray for Ref {}

builder! {
    ArrayBuilder;

    /// Component represents [`Vec`] or [`slice`] type  of items.
    ///
    /// See [`Component::Array`] for more details.
    #[non_exhaustive]
    #[derive(Serialize, Deserialize, Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[serde(rename_all = "camelCase")]
    pub struct Array {
        /// Type will always be [`ComponentType::Array`]
        #[serde(rename = "type")]
        component_type: ComponentType,

        /// Component representing the array items type.
        pub items: Box<Component>,

        /// Max length of the array.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub max_items: Option<usize>,

        /// Min lenght of the array.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub min_items: Option<usize>,

        /// Xml format of the array.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub xml: Option<Xml>,
    }
}

impl Default for Array {
    fn default() -> Self {
        Self {
            component_type: ComponentType::Array,
            items: Default::default(),
            max_items: Default::default(),
            min_items: Default::default(),
            xml: Default::default(),
        }
    }
}

impl Array {
    /// Construct a new [`Array`] component from given [`Component`].
    ///
    /// # Examples
    ///
    /// Create a `String` array component.
    /// ```rust
    /// # use utoipa::openapi::schema::{Component, Array, ComponentType, Property};
    /// let string_array = Array::new(Property::new(ComponentType::String));
    /// ```
    pub fn new<I: Into<Component>>(component: I) -> Self {
        Self {
            items: Box::new(component.into()),
            ..Default::default()
        }
    }

    /// Convert this [`Array`] to [`ArrayBuilder`].
    pub fn to_builder(self) -> ArrayBuilder {
        self.into()
    }
}

impl ArrayBuilder {
    /// Set [`Component`] type for the [`Array`].
    pub fn items<I: Into<Component>>(mut self, component: I) -> Self {
        set_value!(self items Box::new(component.into()))
    }

    /// Set maximun allowed lenght for [`Array`].
    pub fn max_items(mut self, max_items: Option<usize>) -> Self {
        set_value!(self max_items max_items)
    }

    /// Set minimum allowed lenght for [`Array`].
    pub fn min_items(mut self, min_items: Option<usize>) -> Self {
        set_value!(self min_items min_items)
    }

    /// Set [`Xml`] formatting for [`Array`].
    pub fn xml(mut self, xml: Option<Xml>) -> Self {
        set_value!(self xml xml)
    }

    to_array_builder!();
}

component_from_builder!(ArrayBuilder);

impl From<Array> for Component {
    fn from(array: Array) -> Self {
        Self::Array(array)
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

/// Represents data type of [`Component`].
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "lowercase")]
pub enum ComponentType {
    /// Used with [`Object`] and [`ObjectBuilder`]. Objects always have
    /// _component_type_ [`ComponentType::Object`].
    Object,
    /// Indicates string type of content. Typically used with [`Property`] and [`PropertyBuilder`].
    String,
    /// Indicates integer type of content. Typically used with [`Property`] and [`PropertyBuilder`].
    Integer,
    /// Indicates floating point number type of content. Typically used with
    /// [`Property`] and [`PropertyBuilder`].
    Number,
    /// Indicates boolean type of content. Typically used with [`Property`] and [`PropertyBuilder`].
    Boolean,
    /// Used with [`Array`] and [`ArrayBuilder`]. Indicates array type of content.
    Array,
}

impl Default for ComponentType {
    fn default() -> Self {
        Self::Object
    }
}

/// Additional format for [`ComponentType`] to fine tune the data type used. If the **format** is not
/// supported by the UI it may default back to [`ComponentType`] alone.
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "lowercase")]
pub enum ComponentFormat {
    /// 32 bit integer.
    Int32,
    /// 64 bit integer.
    Int64,
    /// floating point number.
    Float,
    /// double (floating point) number.
    Double,
    /// base64 encoded chars.
    Byte,
    /// binary data (octec).
    Binary,
    /// ISO-8601 full date [FRC3339](https://xml2rfc.ietf.org/public/rfc/html/rfc3339.html#anchor14).
    Date,
    /// ISO-8601 full date time [FRC3339](https://xml2rfc.ietf.org/public/rfc/html/rfc3339.html#anchor14).
    #[serde(rename = "date-time")]
    DateTime,
    /// Hint to UI to obsucre input.
    Password,
    /// Used with [`String`] values to indicate value is in UUID format.
    ///
    /// **uuid** feature need to be enabled.
    #[cfg(feature = "uuid")]
    Uuid,
}

#[cfg(test)]
#[cfg(feature = "serde_json")]
mod tests {
    use assert_json_diff::assert_json_eq;
    use serde_json::{json, Value};

    use super::*;
    use crate::openapi::*;

    #[test]
    fn create_schema_serializes_json() -> Result<(), serde_json::Error> {
        let openapi = OpenApiBuilder::new()
            .info(Info::new("My api", "1.0.0"))
            .paths(Paths::new())
            .components(Some(
                ComponentsBuilder::new()
                    .component("Person", Ref::new("#/components/PersonModel"))
                    .component(
                        "Credential",
                        ObjectBuilder::new()
                            .property(
                                "id",
                                PropertyBuilder::new()
                                    .component_type(ComponentType::Integer)
                                    .format(Some(ComponentFormat::Int32))
                                    .description(Some("Id of credential"))
                                    .default(Some(json!(1i32))),
                            )
                            .property(
                                "name",
                                PropertyBuilder::new()
                                    .component_type(ComponentType::String)
                                    .description(Some("Name of credential")),
                            )
                            .property(
                                "status",
                                PropertyBuilder::new()
                                    .component_type(ComponentType::String)
                                    .default(Some(json!("Active")))
                                    .description(Some("Credential status"))
                                    .enum_values(Some([
                                        "Active",
                                        "NotActive",
                                        "Locked",
                                        "Expired",
                                    ])),
                            )
                            .property(
                                "history",
                                Array::new(Ref::from_component_name("UpdateHistory")),
                            )
                            .property("tags", Property::new(ComponentType::String).to_array()),
                    )
                    .build(),
            ))
            .build();

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

    // Examples taken from https://spec.openapis.org/oas/latest.html#model-with-map-dictionary-properties
    #[test]
    fn test_additional_properties() {
        let json_value = ObjectBuilder::new()
            .additional_properties(Some(
                PropertyBuilder::new().component_type(ComponentType::String),
            ))
            .build();
        assert_json_eq!(
            json_value,
            json!({
                "type": "object",
                "additionalProperties": {
                    "type": "string"
                }
            })
        );

        let json_value = ObjectBuilder::new()
            .additional_properties(Some(Ref::from_component_name("ComplexModel")))
            .build();
        assert_json_eq!(
            json_value,
            json!({
                "type": "object",
                "additionalProperties": {
                    "$ref": "#/components/schemas/ComplexModel"
                }
            })
        )
    }

    #[test]
    fn test_object_with_title() {
        let json_value = ObjectBuilder::new().title(Some("SomeName")).build();
        assert_json_eq!(
            json_value,
            json!({
                "type": "object",
                "title": "SomeName"
            })
        );
    }

    #[test]
    fn derive_object_with_example() {
        let expected = r#"{"type":"object","example":{"age":20,"name":"bob the cat"}}"#;
        let json_value = ObjectBuilder::new()
            .example(Some(json!({"age": 20, "name": "bob the cat"})))
            .build();

        let value_string = serde_json::to_string(&json_value).unwrap();
        assert_eq!(
            value_string, expected,
            "value string != expected string, {} != {}",
            value_string, expected
        );
    }

    fn get_json_path<'a>(value: &'a Value, path: &str) -> &'a Value {
        path.split('.').into_iter().fold(value, |acc, fragment| {
            acc.get(fragment).unwrap_or(&serde_json::value::Value::Null)
        })
    }

    #[test]
    fn test_array_new() {
        let array = Array::new(
            ObjectBuilder::new().property(
                "id",
                PropertyBuilder::new()
                    .component_type(ComponentType::Integer)
                    .format(Some(ComponentFormat::Int32))
                    .description(Some("Id of credential"))
                    .default(Some(json!(1i32))),
            ),
        );

        assert!(matches!(array.component_type, ComponentType::Array));
    }

    #[test]
    fn test_array_builder() {
        let array: Array = ArrayBuilder::new()
            .items(
                ObjectBuilder::new().property(
                    "id",
                    PropertyBuilder::new()
                        .component_type(ComponentType::Integer)
                        .format(Some(ComponentFormat::Int32))
                        .description(Some("Id of credential"))
                        .default(Some(json!(1i32))),
                ),
            )
            .build();

        assert!(matches!(array.component_type, ComponentType::Array));
    }

    #[test]
    fn reserialize_deserialized_schema_components() {
        let components = ComponentsBuilder::new()
            .components_from_iter(vec![(
                "Comp",
                ObjectBuilder::new()
                    .property(
                        "name",
                        PropertyBuilder::new().component_type(ComponentType::String),
                    )
                    .required("name"),
            )])
            .security_scheme("TLS", SecurityScheme::MutualTls { description: None })
            .build();

        let serialized_components = serde_json::to_string(&components).unwrap();
        let deserialized_components: Components =
            serde_json::from_str(serialized_components.as_str()).unwrap();

        assert_eq!(
            serialized_components,
            serde_json::to_string(&deserialized_components).unwrap()
        )
    }

    #[test]
    fn reserialize_deserialized_object_component() {
        let prop = ObjectBuilder::new()
            .property(
                "name",
                PropertyBuilder::new().component_type(ComponentType::String),
            )
            .required("name")
            .build();

        let serialized_components = serde_json::to_string(&prop).unwrap();
        let deserialized_components: Object =
            serde_json::from_str(serialized_components.as_str()).unwrap();

        assert_eq!(
            serialized_components,
            serde_json::to_string(&deserialized_components).unwrap()
        )
    }

    #[test]
    fn reserialize_deserialized_property() {
        let prop = PropertyBuilder::new()
            .component_type(ComponentType::String)
            .build();

        let serialized_components = serde_json::to_string(&prop).unwrap();
        let deserialized_components: Property =
            serde_json::from_str(serialized_components.as_str()).unwrap();

        assert_eq!(
            serialized_components,
            serde_json::to_string(&deserialized_components).unwrap()
        )
    }
}
