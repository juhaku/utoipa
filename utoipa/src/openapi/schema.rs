//! Implements [OpenAPI Schema Object][schema] types which can be
//! used to define field properties, enum values, array or object types.
//!
//! [schema]: https://spec.openapis.org/oas/latest.html#schema-object
use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
#[cfg(feature = "serde_json")]
use serde_json::Value;

use super::{
    build_fn, builder, from, new, security::SecurityScheme, set_value, xml::Xml, Deprecated,
    Response,
};
use crate::ToResponse;

macro_rules! component_from_builder {
    ( $name:ident ) => {
        impl From<$name> for Schema {
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
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        pub schemas: BTreeMap<String, Schema>,

        /// Map of reusable response name, to [OpenAPI Response Object][response]s or [OpenAPI
        /// Reference][reference]s to [OpenAPI Response Object][response]s.
        ///
        /// [response]: https://spec.openapis.org/oas/latest.html#response-object
        /// [reference]: https://spec.openapis.org/oas/latest.html#reference-object
        #[serde(skip_serializing_if = "HashMap::is_empty", default)]
        pub responses: HashMap<String, RefOr<Response>>,

        /// Map of reusable [OpenAPI Security Schema Object][security_schema]s.
        ///
        /// [security_schema]: https://spec.openapis.org/oas/latest.html#security-scheme-object
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
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
    /// Add [`Schema`] to [`Components`].
    ///
    /// Accpets two arguments where first is name of the schema and second is the schema itself.
    pub fn schema<S: Into<String>, I: Into<Schema>>(mut self, name: S, schema: I) -> Self {
        self.schemas.insert(name.into(), schema.into());

        self
    }

    /// Add [`Schema`]s from iterator.
    ///
    /// # Examples
    /// ```rust
    /// # use utoipa::openapi::schema::{ComponentsBuilder, ObjectBuilder,
    /// #    SchemaType, Schema};
    /// ComponentsBuilder::new().schemas_from_iter([(
    ///     "Pet",
    ///     Schema::from(
    ///         ObjectBuilder::new()
    ///             .property(
    ///                 "name",
    ///                 ObjectBuilder::new().schema_type(SchemaType::String),
    ///             )
    ///             .required("name")
    ///     ),
    /// )]);
    /// ```
    pub fn schemas_from_iter<
        I: IntoIterator<Item = (S, C)>,
        C: Into<Schema>,
        S: Into<String>,
    >(
        mut self,
        schemas: I,
    ) -> Self {
        self.schemas.extend(
            schemas
                .into_iter()
                .map(|(name, schema)| (name.into(), schema.into())),
        );

        self
    }

    pub fn response<S: Into<String>, R: Into<RefOr<Response>>>(
        mut self,
        name: S,
        response: R,
    ) -> Self {
        self.responses.insert(name.into(), response.into());
        self
    }

    pub fn response_from_into<I: ToResponse>(self) -> Self {
        let (name, response) = I::response();
        self.response(name, response)
    }

    pub fn responses_from_iter<
        I: IntoIterator<Item = (S, R)>,
        S: Into<String>,
        R: Into<RefOr<Response>>,
    >(
        mut self,
        responses: I,
    ) -> Self {
        self.responses.extend(
            responses
                .into_iter()
                .map(|(name, response)| (name.into(), response.into())),
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

/// Is super type for [OpenAPI Schema Object][schemas]. Schema is reusable resource what can be
/// referenced from path operations and other components using [`Ref`] component.
///
/// [schemas]: https://spec.openapis.org/oas/latest.html#schema-object
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(untagged, rename_all = "camelCase")]
pub enum Schema {
    /// Defines object component. Object is either `object` hodling **properties** which are other [`Schema`]s 
    /// or can be a field within the [`Object`].
    Object(Object),
    /// Creates a reference component _`$ref=#/components/schemas/SchemaName`_. Which
    /// can be used to reference a other reusable component in [`Components`].
    Ref(Ref),
    /// Defines array component from another component. Typically used with
    /// [`Schema::Object`] component. Slice and Vec types are translated to [`Schema::Array`] types.
    Array(Array),
    /// Creates a _OneOf_ type [Discriminator Object][discriminator] component. This component
    /// is used to map multiple components together where API endpoint could return any of them.
    /// [`Schema::OneOf`] is created form complex enum where enum holds other than unit types.
    ///
    /// [discriminator]: https://spec.openapis.org/oas/latest.html#components-object
    OneOf(OneOf),
}

impl Default for Schema {
    fn default() -> Self {
        Schema::Object(Object::default())
    }
}

builder! {
    OneOfBuilder;

    /// OneOf [Discriminator Object][discriminator] component holds
    /// multiple components together where API endpoint could return any of them.
    ///
    /// See [`Schema::OneOf`] for more details.
    ///
    /// [discriminator]: https://spec.openapis.org/oas/latest.html#components-object
    #[derive(Serialize, Deserialize, Clone, Default)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct OneOf {
        /// Components of _OneOf_ component.
        #[serde(rename = "oneOf")]
        pub items: Vec<Schema>,

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
    /// Adds a given [`Schema`] to [`OneOf`] [Discriminator Object][discriminator]
    ///
    /// [discriminator]: https://spec.openapis.org/oas/latest.html#components-object
    pub fn item<I: Into<Schema>>(mut self, component: I) -> Self {
        self.items.push(component.into());

        self
    }

    /// Add or change optional description for `OneOf` component.
    pub fn description<I: Into<String>>(mut self, description: Option<I>) -> Self {
        set_value!(self description description.map(|description| description.into()))
    }

    to_array_builder!();
}

impl From<OneOf> for Schema {
    fn from(one_of: OneOf) -> Self {
        Self::OneOf(one_of)
    }
}

component_from_builder!(OneOfBuilder);

/// Implements subset of [OpenAPI Schema Object][schema] which allows
/// adding other [`Schema`]s as **properties** to this [`Schema`].
///
/// This is a generic OpenAPI schema object which can used to present `object`, `field` or an `enum`.
///
/// [schema]: https://spec.openapis.org/oas/latest.html#schema-object
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct Object {
    /// Type of [`Object`] e.g. [`SchemaType::Object`] for `object` and [`SchemaType::String`] for
    /// `string` types.
    #[serde(rename = "type")]
    schema_type: SchemaType,

    /// Changes the [`Object`] title.
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    /// Additional format for detailing the schema type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<SchemaFormat>,

    /// Description of the [`Object`]. Markdown syntax is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Default value which is provided when user has not provided the input in Swagger UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "serde_json")]
    pub default: Option<Value>,


    /// Default value which is provided when user has not provided the input in Swagger UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(not(feature = "serde_json"))]
    pub default: Option<String>,

    /// Enum variants of fields that can be represented as `unit` type `enums`
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,

    /// Vector of required field names.
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub required: Vec<String>,

    /// Map of fields with their [`Schema`] types.
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default = "BTreeMap::new")]
    pub properties: BTreeMap<String, Schema>,

    /// Additional [`Schema`] for non specified fields (Useful for typed maps).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<Box<Schema>>,

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

    /// Write only property will be only sent in _write_ requests like _POST, PUT_.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_only: Option<bool>,

    /// Read only property will be only sent in _read_ requests like _GET_.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,

    /// Additional [`Xml`] formatting of the [`Object`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xml: Option<Xml>,
}

impl Object {
    /// Initialize a new [`Object`] with default [`SchemaType`]. This effectifly same as calling
    /// [`Object::with_type(SchemaType::Object)`].
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Initialize new [`Object`] with given [`SchemaType`].
    ///
    /// Create [`string`] object type which can be used to define `string` field of an object.
    /// ```rust
    /// # use utoipa::openapi::schema::{Object, SchemaType};
    /// let object = Object::with_type(SchemaType::String);
    /// ```
    pub fn with_type(schema_type: SchemaType) -> Self {
        Self {
            schema_type,
            ..Default::default()
        }
    }
}

impl From<Object> for Schema {
    fn from(s: Object) -> Self {
        Self::Object(s)
    }
}

impl ToArray for Object {}

/// Builder for [`Object`] with chainable configuration methods to create a new [`Object`].
#[derive(Default)]
pub struct ObjectBuilder {
    schema_type: SchemaType,

    title: Option<String>,

    format: Option<SchemaFormat>,

    description: Option<String>,

    #[cfg(feature = "serde_json")]
    default: Option<Value>,

    #[cfg(not(feature = "serde_json"))]
    default: Option<String>,

    deprecated: Option<Deprecated>,

    enum_values: Option<Vec<String>>,

    required: Vec<String>,

    properties: BTreeMap<String, Schema>,

    additional_properties: Option<Box<Schema>>,

    write_only: Option<bool>,

    read_only: Option<bool>,

    #[cfg(feature = "serde_json")]
    example: Option<Value>,

    #[cfg(not(feature = "serde_json"))]
    example: Option<String>,

    xml: Option<Xml>,
}

impl ObjectBuilder {
    new!(pub ObjectBuilder);

    /// Add or change type of the object e.g [`SchemaType::String`].
    pub fn schema_type(mut self, schema_type: SchemaType) -> Self {
        set_value!(self schema_type schema_type)
    }

    /// Add or change additional format for detailing the schema type.
    pub fn format(mut self, format: Option<SchemaFormat>) -> Self {
        set_value!(self format format)
    }

    /// Add new property to the [`Object`].
    ///
    /// Method accepts property name and property component as an arguments.
    pub fn property<S: Into<String>, I: Into<Schema>>(
        mut self,
        property_name: S,
        component: I,
    ) -> Self {
        self.properties
            .insert(property_name.into(), component.into());

        self
    }

    pub fn additional_properties<I: Into<Schema>>(
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

    /// Add or change default value for the object which is provided when user has not provided the input in Swagger UI.
    #[cfg(feature = "serde_json")]
    pub fn default(mut self, default: Option<Value>) -> Self {
        set_value!(self default default)
    }

    /// Add or change default value for the object which is provided when user has not provided the input in Swagger UI.
    #[cfg(not(feature = "serde_json"))]
    pub fn default<I: Into<String>>(mut self, default: Option<I>) -> Self {
        set_value!(self default default.map(|default| default.into()))
    }

    /// Add or change deprecated status for [`Object`].
    pub fn deprecated(mut self, deprecated: Option<Deprecated>) -> Self {
        set_value!(self deprecated deprecated)
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
    #[cfg(feature = "serde_json")]
    pub fn example(mut self, example: Option<Value>) -> Self {
        set_value!(self example example)
    }

    /// Add or change example shown in UI of the value for richier documentation.
    #[cfg(not(feature = "serde_json"))]
    pub fn example<I: Into<String>>(mut self, example: Option<I>) -> Self {
        set_value!(self example example.map(|example| example.into()))
    }

    /// Add or change write only flag for [`Object`].
    pub fn write_only(mut self, write_only: Option<bool>) -> Self {
        set_value!(self write_only write_only)
    }

    /// Add or change read only flag for [`Object`].
    pub fn read_only(mut self, read_only: Option<bool>) -> Self {
        set_value!(self read_only read_only)
    }

    /// Add or change additional [`Xml`] formatting of the [`Object`].
    pub fn xml(mut self, xml: Option<Xml>) -> Self {
        set_value!(self xml xml)
    }

    to_array_builder!();

    build_fn!(pub Object schema_type, format, title, required, properties, description, 
              deprecated, default, enum_values, example, write_only, read_only, xml, additional_properties);
}

from!(Object ObjectBuilder schema_type, format, title, required, properties, description,
      deprecated, default, enum_values,  example, write_only, read_only, xml, additional_properties);

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
    /// and [`Ref::from_schema_name`] could be used instead.
    pub fn new<I: Into<String>>(ref_location: I) -> Self {
        Self {
            ref_location: ref_location.into(),
        }
    }

    /// Construct a new [`Ref`] from provided schema name. This will create a [`Ref`] that
    /// references the the reusable schemas.
    pub fn from_schema_name<I: Into<String>>(schema_name: I) -> Self {
        Self::new(&format!("#/components/schemas/{}", schema_name.into()))
    }

    /// Construct a new [`Ref`] from provided response name. This will create a [`Ref`] that
    /// references the reusable response.
    pub fn from_response_name<I: Into<String>>(response_name: I) -> Self {
        Self::new(&format!("#/components/responses/{}", response_name.into()))
    }

    to_array_builder!();
}

impl From<Ref> for Schema {
    fn from(r: Ref) -> Self {
        Self::Ref(r)
    }
}

impl ToArray for Ref {}

/// A [`Ref`] or some other type `T`
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(untagged)]
pub enum RefOr<T> {
    Ref(Ref),
    T(T),
}

impl<T> From<T> for RefOr<T> {
    fn from(t: T) -> Self {
        Self::T(t)
    }
}

builder! {
    ArrayBuilder;

    /// Array represents [`Vec`] or [`slice`] type  of items.
    ///
    /// See [`Schema::Array`] for more details.
    #[non_exhaustive]
    #[derive(Serialize, Deserialize, Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[serde(rename_all = "camelCase")]
    pub struct Array {
        /// Type will always be [`SchemaType::Array`]
        #[serde(rename = "type")]
        schema_type: SchemaType,

        /// Schema representing the array items type.
        pub items: Box<Schema>,

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
            schema_type: SchemaType::Array,
            items: Default::default(),
            max_items: Default::default(),
            min_items: Default::default(),
            xml: Default::default(),
        }
    }
}

impl Array {
    /// Construct a new [`Array`] component from given [`Schema`].
    ///
    /// # Examples
    ///
    /// Create a `String` array component.
    /// ```rust
    /// # use utoipa::openapi::schema::{Schema, Array, SchemaType, Object};
    /// let string_array = Array::new(Object::with_type(SchemaType::String));
    /// ```
    pub fn new<I: Into<Schema>>(component: I) -> Self {
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
    /// Set [`Schema`] type for the [`Array`].
    pub fn items<I: Into<Schema>>(mut self, component: I) -> Self {
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

impl From<Array> for Schema {
    fn from(array: Array) -> Self {
        Self::Array(array)
    }
}

impl ToArray for Array {}

pub trait ToArray
where
    Schema: From<Self>,
    Self: Sized,
{
    fn to_array(self) -> Array {
        Array::new(self)
    }
}

/// Represents data type of [`Schema`].
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    /// Used with [`Object`] and [`ObjectBuilder`]. Objects always have
    /// _schema_type_ [`SchemaType::Object`].
    Object,
    /// Indicates string type of content. Used with [`Object`] and [`ObjectBuilder`] on a `string`
    /// field.
    String,
    /// Indicates integer type of content. Used with [`Object`] and [`ObjectBuilder`] on a `number`
    /// field.
    Integer,
    /// Indicates floating point number type of content. Used with
    /// [`Object`] and [`ObjectBuilder`] on a `number` field.
    Number,
    /// Indicates boolean type of content. Used with [`Object`] and [`ObjectBuilder`] on
    /// a `bool` field.
    Boolean,
    /// Used with [`Array`] and [`ArrayBuilder`]. Indicates array type of content.
    Array,
}

impl Default for SchemaType {
    fn default() -> Self {
        Self::Object
    }
}

/// Additional format for [`SchemaType`] to fine tune the data type used. If the **format** is not
/// supported by the UI it may default back to [`SchemaType`] alone.
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "lowercase")]
pub enum SchemaFormat {
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
                    .schema("Person", Ref::new("#/components/PersonModel"))
                    .schema(
                        "Credential",
                        Schema::from(
                            ObjectBuilder::new()
                                .property(
                                    "id",
                                    ObjectBuilder::new()
                                        .schema_type(SchemaType::Integer)
                                        .format(Some(SchemaFormat::Int32))
                                        .description(Some("Id of credential"))
                                        .default(Some(json!(1i32))),
                                )
                                .property(
                                    "name",
                                    ObjectBuilder::new()
                                        .schema_type(SchemaType::String)
                                        .description(Some("Name of credential")),
                                )
                                .property(
                                    "status",
                                    ObjectBuilder::new()
                                        .schema_type(SchemaType::String)
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
                                    Array::new(Ref::from_schema_name("UpdateHistory")),
                                )
                                .property("tags", Object::with_type(SchemaType::String).to_array()),
                        ),
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
            .additional_properties(Some(ObjectBuilder::new().schema_type(SchemaType::String)))
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
            .additional_properties(Some(Ref::from_schema_name("ComplexModel")))
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
                ObjectBuilder::new()
                    .schema_type(SchemaType::Integer)
                    .format(Some(SchemaFormat::Int32))
                    .description(Some("Id of credential"))
                    .default(Some(json!(1i32))),
            ),
        );

        assert!(matches!(array.schema_type, SchemaType::Array));
    }

    #[test]
    fn test_array_builder() {
        let array: Array = ArrayBuilder::new()
            .items(
                ObjectBuilder::new().property(
                    "id",
                    ObjectBuilder::new()
                        .schema_type(SchemaType::Integer)
                        .format(Some(SchemaFormat::Int32))
                        .description(Some("Id of credential"))
                        .default(Some(json!(1i32))),
                ),
            )
            .build();

        assert!(matches!(array.schema_type, SchemaType::Array));
    }

    #[test]
    fn reserialize_deserialized_schema_components() {
        let components = ComponentsBuilder::new()
            .schemas_from_iter(vec![(
                "Comp",
                Schema::from(
                    ObjectBuilder::new()
                        .property(
                            "name",
                            ObjectBuilder::new().schema_type(SchemaType::String),
                        )
                        .required("name"),
                ),
            )])
            .responses_from_iter(vec![(
                "200",
                ResponseBuilder::new().description("Okay").build(),
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
                ObjectBuilder::new().schema_type(SchemaType::String),
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
        let prop = ObjectBuilder::new()
            .schema_type(SchemaType::String)
            .build();

        let serialized_components = serde_json::to_string(&prop).unwrap();
        let deserialized_components: Object =
            serde_json::from_str(serialized_components.as_str()).unwrap();

        assert_eq!(
            serialized_components,
            serde_json::to_string(&deserialized_components).unwrap()
        )
    }
}
