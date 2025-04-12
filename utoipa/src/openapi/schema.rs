//! Implements [OpenAPI Schema Object][schema] types which can be
//! used to define field properties, enum values, array or object types.
//!
//! [schema]: https://spec.openapis.org/oas/latest.html#schema-object
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::extensions::Extensions;
use super::RefOr;
use super::{builder, security::SecurityScheme, set_value, xml::Xml, Deprecated, Response};
use crate::{ToResponse, ToSchema};

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

/// Create an _`empty`_ [`Schema`] that serializes to _`null`_.
///
/// Can be used in places where an item can be serialized as `null`. This is used with unit type
/// enum variants and tuple unit types.
pub fn empty() -> Schema {
    Schema::Object(
        ObjectBuilder::new()
            .schema_type(SchemaType::AnyValue)
            .default(Some(serde_json::Value::Null))
            .into(),
    )
}

builder! {
    ComponentsBuilder;

    /// Implements [OpenAPI Components Object][components] which holds supported
    /// reusable objects.
    ///
    /// Components can hold either reusable types themselves or references to other reusable
    /// types.
    ///
    /// [components]: https://spec.openapis.org/oas/latest.html#components-object
    #[non_exhaustive]
    #[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[serde(rename_all = "camelCase")]
    pub struct Components {
        /// Map of reusable [OpenAPI Schema Object][schema]s.
        ///
        /// [schema]: https://spec.openapis.org/oas/latest.html#schema-object
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        pub schemas: BTreeMap<String, RefOr<Schema>>,

        /// Map of reusable response name, to [OpenAPI Response Object][response]s or [OpenAPI
        /// Reference][reference]s to [OpenAPI Response Object][response]s.
        ///
        /// [response]: https://spec.openapis.org/oas/latest.html#response-object
        /// [reference]: https://spec.openapis.org/oas/latest.html#reference-object
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        pub responses: BTreeMap<String, RefOr<Response>>,

        /// Map of reusable [OpenAPI Security Scheme Object][security_scheme]s.
        ///
        /// [security_scheme]: https://spec.openapis.org/oas/latest.html#security-scheme-object
        #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
        pub security_schemes: BTreeMap<String, SecurityScheme>,

        /// Optional extensions "x-something".
        #[serde(skip_serializing_if = "Option::is_none", flatten)]
        pub extensions: Option<Extensions>,
    }
}

impl Components {
    /// Construct a new [`Components`].
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    /// Add [`SecurityScheme`] to [`Components`].
    ///
    /// Accepts two arguments where first is the name of the [`SecurityScheme`]. This is later when
    /// referenced by [`SecurityRequirement`][requirement]s. Second parameter is the [`SecurityScheme`].
    ///
    /// [requirement]: ../security/struct.SecurityRequirement.html
    pub fn add_security_scheme<N: Into<String>, S: Into<SecurityScheme>>(
        &mut self,
        name: N,
        security_scheme: S,
    ) {
        self.security_schemes
            .insert(name.into(), security_scheme.into());
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
    /// Accepts two arguments where first is name of the schema and second is the schema itself.
    pub fn schema<S: Into<String>, I: Into<RefOr<Schema>>>(mut self, name: S, schema: I) -> Self {
        self.schemas.insert(name.into(), schema.into());

        self
    }

    /// Add [`Schema`] to [`Components`].
    ///
    /// This is effectively same as calling [`ComponentsBuilder::schema`] but expects to be called
    /// with one generic argument that implements [`ToSchema`][trait@ToSchema] trait.
    ///
    /// # Examples
    ///
    /// _**Add schema from `Value` type that derives `ToSchema`.**_
    ///
    /// ```rust
    /// # use utoipa::{ToSchema, openapi::schema::ComponentsBuilder};
    ///  #[derive(ToSchema)]
    ///  struct Value(String);
    ///
    ///  let _ = ComponentsBuilder::new().schema_from::<Value>().build();
    /// ```
    pub fn schema_from<I: ToSchema>(mut self) -> Self {
        let name = I::name();
        let schema = I::schema();
        self.schemas.insert(name.to_string(), schema);

        self
    }

    /// Add [`Schema`]s from iterator.
    ///
    /// # Examples
    /// ```rust
    /// # use utoipa::openapi::schema::{ComponentsBuilder, ObjectBuilder,
    /// #    Type, Schema};
    /// ComponentsBuilder::new().schemas_from_iter([(
    ///     "Pet",
    ///     Schema::from(
    ///         ObjectBuilder::new()
    ///             .property(
    ///                 "name",
    ///                 ObjectBuilder::new().schema_type(Type::String),
    ///             )
    ///             .required("name")
    ///     ),
    /// )]);
    /// ```
    pub fn schemas_from_iter<
        I: IntoIterator<Item = (S, C)>,
        C: Into<RefOr<Schema>>,
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

    /// Add [`struct@Response`] to [`Components`].
    ///
    /// Method accepts tow arguments; `name` of the reusable response and `response` which is the
    /// reusable response itself.
    pub fn response<S: Into<String>, R: Into<RefOr<Response>>>(
        mut self,
        name: S,
        response: R,
    ) -> Self {
        self.responses.insert(name.into(), response.into());
        self
    }

    /// Add [`struct@Response`] to [`Components`].
    ///
    /// This behaves the same way as [`ComponentsBuilder::schema_from`] but for responses. It
    /// allows adding response from type implementing [`trait@ToResponse`] trait. Method is
    /// expected to be called with one generic argument that implements the trait.
    pub fn response_from<'r, I: ToResponse<'r>>(self) -> Self {
        let (name, response) = I::response();
        self.response(name, response)
    }

    /// Add multiple [`struct@Response`]s to [`Components`] from iterator.
    ///
    /// Like the [`ComponentsBuilder::schemas_from_iter`] this allows adding multiple responses by
    /// any iterator what returns tuples of (name, response) values.
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
        security_scheme: S,
    ) -> Self {
        self.security_schemes
            .insert(name.into(), security_scheme.into());

        self
    }

    /// Add openapi extensions (x-something) of the API.
    pub fn extensions(mut self, extensions: Option<Extensions>) -> Self {
        set_value!(self extensions extensions)
    }
}

/// Is super type for [OpenAPI Schema Object][schemas]. Schema is reusable resource what can be
/// referenced from path operations and other components using [`Ref`].
///
/// [schemas]: https://spec.openapis.org/oas/latest.html#schema-object
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(untagged, rename_all = "camelCase")]
pub enum Schema {
    /// Defines array schema from another schema. Typically used with
    /// [`Schema::Object`]. Slice and Vec types are translated to [`Schema::Array`] types.
    Array(Array),
    /// Defines object schema. Object is either `object` holding **properties** which are other [`Schema`]s
    /// or can be a field within the [`Object`].
    Object(Object),
    /// Creates a _OneOf_ type [composite Object][composite] schema. This schema
    /// is used to map multiple schemas together where API endpoint could return any of them.
    /// [`Schema::OneOf`] is created form mixed enum where enum contains various variants.
    ///
    /// [composite]: https://spec.openapis.org/oas/latest.html#components-object
    OneOf(OneOf),

    /// Creates a _AllOf_ type [composite Object][composite] schema.
    ///
    /// [composite]: https://spec.openapis.org/oas/latest.html#components-object
    AllOf(AllOf),

    /// Creates a _AnyOf_ type [composite Object][composite] schema.
    ///
    /// [composite]: https://spec.openapis.org/oas/latest.html#components-object
    AnyOf(AnyOf),
}

impl Default for Schema {
    fn default() -> Self {
        Schema::Object(Object::default())
    }
}

/// OpenAPI [Discriminator][discriminator] object which can be optionally used together with
/// [`OneOf`] composite object.
///
/// [discriminator]: https://spec.openapis.org/oas/latest.html#discriminator-object
#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Discriminator {
    /// Defines a discriminator property name which must be found within all composite
    /// objects.
    pub property_name: String,

    /// An object to hold mappings between payload values and schema names or references.
    /// This field can only be populated manually. There is no macro support and no
    /// validation.
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub mapping: BTreeMap<String, String>,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

impl Discriminator {
    /// Construct a new [`Discriminator`] object with property name.
    ///
    /// # Examples
    ///
    /// Create a new [`Discriminator`] object for `pet_type` property.
    /// ```rust
    /// # use utoipa::openapi::schema::Discriminator;
    /// let discriminator = Discriminator::new("pet_type");
    /// ```
    pub fn new<I: Into<String>>(property_name: I) -> Self {
        Self {
            property_name: property_name.into(),
            mapping: BTreeMap::new(),
            ..Default::default()
        }
    }

    /// Construct a new [`Discriminator`] object with property name and mappings.
    ///
    ///
    /// Method accepts two arguments. First _`property_name`_ to use as `discriminator` and
    /// _`mapping`_ for custom property name mappings.
    ///
    /// # Examples
    ///
    ///_**Construct an ew [`Discriminator`] with custom mapping.**_
    ///
    /// ```rust
    /// # use utoipa::openapi::schema::Discriminator;
    /// let discriminator = Discriminator::with_mapping("pet_type", [
    ///     ("cat","#/components/schemas/Cat")
    /// ]);
    /// ```
    pub fn with_mapping<
        P: Into<String>,
        M: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    >(
        property_name: P,
        mapping: M,
    ) -> Self {
        Self {
            property_name: property_name.into(),
            mapping: BTreeMap::from_iter(
                mapping
                    .into_iter()
                    .map(|(key, val)| (key.into(), val.into())),
            ),
            ..Default::default()
        }
    }
}

builder! {
    OneOfBuilder;

    /// OneOf [Composite Object][oneof] component holds
    /// multiple components together where API endpoint could return any of them.
    ///
    /// See [`Schema::OneOf`] for more details.
    ///
    /// [oneof]: https://spec.openapis.org/oas/latest.html#components-object
    #[derive(Serialize, Deserialize, Clone, PartialEq)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct OneOf {
        /// Components of _OneOf_ component.
        #[serde(rename = "oneOf")]
        pub items: Vec<RefOr<Schema>>,

        /// Type of [`OneOf`] e.g. `SchemaType::new(Type::Object)` for `object`.
        ///
        /// By default this is [`SchemaType::AnyValue`] as the type is defined by items
        /// themselves.
        #[serde(rename = "type", default = "SchemaType::any", skip_serializing_if = "SchemaType::is_any_value")]
        pub schema_type: SchemaType,

        /// Changes the [`OneOf`] title.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub title: Option<String>,

        /// Description of the [`OneOf`]. Markdown syntax is supported.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,

        /// Default value which is provided when user has not provided the input in Swagger UI.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub default: Option<Value>,

        /// Example shown in UI of the value for richer documentation.
        ///
        /// **Deprecated since 3.0.x. Prefer [`OneOf::examples`] instead**
        #[serde(skip_serializing_if = "Option::is_none")]
        pub example: Option<Value>,

        /// Examples shown in UI of the value for richer documentation.
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        pub examples: Vec<Value>,

        /// Optional discriminator field can be used to aid deserialization, serialization and validation of a
        /// specific schema.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub discriminator: Option<Discriminator>,

        /// Optional extensions `x-something`.
        #[serde(skip_serializing_if = "Option::is_none", flatten)]
        pub extensions: Option<Extensions>,
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
            ..Default::default()
        }
    }
}

impl Default for OneOf {
    fn default() -> Self {
        Self {
            items: Default::default(),
            schema_type: SchemaType::AnyValue,
            title: Default::default(),
            description: Default::default(),
            default: Default::default(),
            example: Default::default(),
            examples: Default::default(),
            discriminator: Default::default(),
            extensions: Default::default(),
        }
    }
}

impl OneOfBuilder {
    /// Adds a given [`Schema`] to [`OneOf`] [Composite Object][composite].
    ///
    /// [composite]: https://spec.openapis.org/oas/latest.html#components-object
    pub fn item<I: Into<RefOr<Schema>>>(mut self, component: I) -> Self {
        self.items.push(component.into());

        self
    }

    /// Add or change type of the object e.g. to change type to _`string`_
    /// use value `SchemaType::Type(Type::String)`.
    pub fn schema_type<T: Into<SchemaType>>(mut self, schema_type: T) -> Self {
        set_value!(self schema_type schema_type.into())
    }

    /// Add or change the title of the [`OneOf`].
    pub fn title<I: Into<String>>(mut self, title: Option<I>) -> Self {
        set_value!(self title title.map(|title| title.into()))
    }

    /// Add or change optional description for `OneOf` component.
    pub fn description<I: Into<String>>(mut self, description: Option<I>) -> Self {
        set_value!(self description description.map(|description| description.into()))
    }

    /// Add or change default value for the object which is provided when user has not provided the input in Swagger UI.
    pub fn default(mut self, default: Option<Value>) -> Self {
        set_value!(self default default)
    }

    /// Add or change example shown in UI of the value for richer documentation.
    ///
    /// **Deprecated since 3.0.x. Prefer [`OneOfBuilder::examples`] instead**
    #[deprecated = "Since OpenAPI 3.1 prefer using `examples`"]
    pub fn example(mut self, example: Option<Value>) -> Self {
        set_value!(self example example)
    }

    /// Add or change examples shown in UI of the value for richer documentation.
    pub fn examples<I: IntoIterator<Item = V>, V: Into<Value>>(mut self, examples: I) -> Self {
        set_value!(self examples examples.into_iter().map(Into::into).collect())
    }

    /// Add or change discriminator field of the composite [`OneOf`] type.
    pub fn discriminator(mut self, discriminator: Option<Discriminator>) -> Self {
        set_value!(self discriminator discriminator)
    }

    /// Add openapi extensions (`x-something`) for [`OneOf`].
    pub fn extensions(mut self, extensions: Option<Extensions>) -> Self {
        set_value!(self extensions extensions)
    }

    to_array_builder!();
}

impl From<OneOf> for Schema {
    fn from(one_of: OneOf) -> Self {
        Self::OneOf(one_of)
    }
}

impl From<OneOfBuilder> for RefOr<Schema> {
    fn from(one_of: OneOfBuilder) -> Self {
        Self::T(Schema::OneOf(one_of.build()))
    }
}

impl From<OneOfBuilder> for ArrayItems {
    fn from(value: OneOfBuilder) -> Self {
        Self::RefOrSchema(Box::new(value.into()))
    }
}

component_from_builder!(OneOfBuilder);

builder! {
    AllOfBuilder;

    /// AllOf [Composite Object][allof] component holds
    /// multiple components together where API endpoint will return a combination of all of them.
    ///
    /// See [`Schema::AllOf`] for more details.
    ///
    /// [allof]: https://spec.openapis.org/oas/latest.html#components-object
    #[derive(Serialize, Deserialize, Clone, PartialEq)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct AllOf {
        /// Components of _AllOf_ component.
        #[serde(rename = "allOf")]
        pub items: Vec<RefOr<Schema>>,

        /// Type of [`AllOf`] e.g. `SchemaType::new(Type::Object)` for `object`.
        ///
        /// By default this is [`SchemaType::AnyValue`] as the type is defined by items
        /// themselves.
        #[serde(rename = "type", default = "SchemaType::any", skip_serializing_if = "SchemaType::is_any_value")]
        pub schema_type: SchemaType,

        /// Changes the [`AllOf`] title.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub title: Option<String>,

        /// Description of the [`AllOf`]. Markdown syntax is supported.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,

        /// Default value which is provided when user has not provided the input in Swagger UI.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub default: Option<Value>,

        /// Example shown in UI of the value for richer documentation.
        ///
        /// **Deprecated since 3.0.x. Prefer [`AllOf::examples`] instead**
        #[serde(skip_serializing_if = "Option::is_none")]
        pub example: Option<Value>,

        /// Examples shown in UI of the value for richer documentation.
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        pub examples: Vec<Value>,

        /// Optional discriminator field can be used to aid deserialization, serialization and validation of a
        /// specific schema.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub discriminator: Option<Discriminator>,

        /// Optional extensions `x-something`.
        #[serde(skip_serializing_if = "Option::is_none", flatten)]
        pub extensions: Option<Extensions>,
    }
}

impl AllOf {
    /// Construct a new [`AllOf`] component.
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Construct a new [`AllOf`] component with given capacity.
    ///
    /// AllOf component is then able to contain number of components without
    /// reallocating.
    ///
    /// # Examples
    ///
    /// Create [`AllOf`] component with initial capacity of 5.
    /// ```rust
    /// # use utoipa::openapi::schema::AllOf;
    /// let one_of = AllOf::with_capacity(5);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            ..Default::default()
        }
    }
}

impl Default for AllOf {
    fn default() -> Self {
        Self {
            items: Default::default(),
            schema_type: SchemaType::AnyValue,
            title: Default::default(),
            description: Default::default(),
            default: Default::default(),
            example: Default::default(),
            examples: Default::default(),
            discriminator: Default::default(),
            extensions: Default::default(),
        }
    }
}

impl AllOfBuilder {
    /// Adds a given [`Schema`] to [`AllOf`] [Composite Object][composite].
    ///
    /// [composite]: https://spec.openapis.org/oas/latest.html#components-object
    pub fn item<I: Into<RefOr<Schema>>>(mut self, component: I) -> Self {
        self.items.push(component.into());

        self
    }

    /// Add or change type of the object e.g. to change type to _`string`_
    /// use value `SchemaType::Type(Type::String)`.
    pub fn schema_type<T: Into<SchemaType>>(mut self, schema_type: T) -> Self {
        set_value!(self schema_type schema_type.into())
    }

    /// Add or change the title of the [`AllOf`].
    pub fn title<I: Into<String>>(mut self, title: Option<I>) -> Self {
        set_value!(self title title.map(|title| title.into()))
    }

    /// Add or change optional description for `AllOf` component.
    pub fn description<I: Into<String>>(mut self, description: Option<I>) -> Self {
        set_value!(self description description.map(|description| description.into()))
    }

    /// Add or change default value for the object which is provided when user has not provided the input in Swagger UI.
    pub fn default(mut self, default: Option<Value>) -> Self {
        set_value!(self default default)
    }

    /// Add or change example shown in UI of the value for richer documentation.
    ///
    /// **Deprecated since 3.0.x. Prefer [`AllOfBuilder::examples`] instead**
    #[deprecated = "Since OpenAPI 3.1 prefer using `examples`"]
    pub fn example(mut self, example: Option<Value>) -> Self {
        set_value!(self example example)
    }

    /// Add or change examples shown in UI of the value for richer documentation.
    pub fn examples<I: IntoIterator<Item = V>, V: Into<Value>>(mut self, examples: I) -> Self {
        set_value!(self examples examples.into_iter().map(Into::into).collect())
    }

    /// Add or change discriminator field of the composite [`AllOf`] type.
    pub fn discriminator(mut self, discriminator: Option<Discriminator>) -> Self {
        set_value!(self discriminator discriminator)
    }

    /// Add openapi extensions (`x-something`) for [`AllOf`].
    pub fn extensions(mut self, extensions: Option<Extensions>) -> Self {
        set_value!(self extensions extensions)
    }

    to_array_builder!();
}

impl From<AllOf> for Schema {
    fn from(one_of: AllOf) -> Self {
        Self::AllOf(one_of)
    }
}

impl From<AllOfBuilder> for RefOr<Schema> {
    fn from(one_of: AllOfBuilder) -> Self {
        Self::T(Schema::AllOf(one_of.build()))
    }
}

impl From<AllOfBuilder> for ArrayItems {
    fn from(value: AllOfBuilder) -> Self {
        Self::RefOrSchema(Box::new(value.into()))
    }
}

component_from_builder!(AllOfBuilder);

builder! {
    AnyOfBuilder;

    /// AnyOf [Composite Object][anyof] component holds
    /// multiple components together where API endpoint will return a combination of one or more of them.
    ///
    /// See [`Schema::AnyOf`] for more details.
    ///
    /// [anyof]: https://spec.openapis.org/oas/latest.html#components-object
    #[derive(Serialize, Deserialize, Clone, PartialEq)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct AnyOf {
        /// Components of _AnyOf component.
        #[serde(rename = "anyOf")]
        pub items: Vec<RefOr<Schema>>,

        /// Type of [`AnyOf`] e.g. `SchemaType::new(Type::Object)` for `object`.
        ///
        /// By default this is [`SchemaType::AnyValue`] as the type is defined by items
        /// themselves.
        #[serde(rename = "type", default = "SchemaType::any", skip_serializing_if = "SchemaType::is_any_value")]
        pub schema_type: SchemaType,

        /// Description of the [`AnyOf`]. Markdown syntax is supported.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,

        /// Default value which is provided when user has not provided the input in Swagger UI.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub default: Option<Value>,

        /// Example shown in UI of the value for richer documentation.
        ///
        /// **Deprecated since 3.0.x. Prefer [`AnyOf::examples`] instead**
        #[serde(skip_serializing_if = "Option::is_none")]
        pub example: Option<Value>,

        /// Examples shown in UI of the value for richer documentation.
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        pub examples: Vec<Value>,

        /// Optional discriminator field can be used to aid deserialization, serialization and validation of a
        /// specific schema.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub discriminator: Option<Discriminator>,

        /// Optional extensions `x-something`.
        #[serde(skip_serializing_if = "Option::is_none", flatten)]
        pub extensions: Option<Extensions>,
    }
}

impl AnyOf {
    /// Construct a new [`AnyOf`] component.
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Construct a new [`AnyOf`] component with given capacity.
    ///
    /// AnyOf component is then able to contain number of components without
    /// reallocating.
    ///
    /// # Examples
    ///
    /// Create [`AnyOf`] component with initial capacity of 5.
    /// ```rust
    /// # use utoipa::openapi::schema::AnyOf;
    /// let one_of = AnyOf::with_capacity(5);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            ..Default::default()
        }
    }
}

impl Default for AnyOf {
    fn default() -> Self {
        Self {
            items: Default::default(),
            schema_type: SchemaType::AnyValue,
            description: Default::default(),
            default: Default::default(),
            example: Default::default(),
            examples: Default::default(),
            discriminator: Default::default(),
            extensions: Default::default(),
        }
    }
}

impl AnyOfBuilder {
    /// Adds a given [`Schema`] to [`AnyOf`] [Composite Object][composite].
    ///
    /// [composite]: https://spec.openapis.org/oas/latest.html#components-object
    pub fn item<I: Into<RefOr<Schema>>>(mut self, component: I) -> Self {
        self.items.push(component.into());

        self
    }

    /// Add or change type of the object e.g. to change type to _`string`_
    /// use value `SchemaType::Type(Type::String)`.
    pub fn schema_type<T: Into<SchemaType>>(mut self, schema_type: T) -> Self {
        set_value!(self schema_type schema_type.into())
    }

    /// Add or change optional description for `AnyOf` component.
    pub fn description<I: Into<String>>(mut self, description: Option<I>) -> Self {
        set_value!(self description description.map(|description| description.into()))
    }

    /// Add or change default value for the object which is provided when user has not provided the input in Swagger UI.
    pub fn default(mut self, default: Option<Value>) -> Self {
        set_value!(self default default)
    }

    /// Add or change example shown in UI of the value for richer documentation.
    ///
    /// **Deprecated since 3.0.x. Prefer [`AllOfBuilder::examples`] instead**
    #[deprecated = "Since OpenAPI 3.1 prefer using `examples`"]
    pub fn example(mut self, example: Option<Value>) -> Self {
        set_value!(self example example)
    }

    /// Add or change examples shown in UI of the value for richer documentation.
    pub fn examples<I: IntoIterator<Item = V>, V: Into<Value>>(mut self, examples: I) -> Self {
        set_value!(self examples examples.into_iter().map(Into::into).collect())
    }

    /// Add or change discriminator field of the composite [`AnyOf`] type.
    pub fn discriminator(mut self, discriminator: Option<Discriminator>) -> Self {
        set_value!(self discriminator discriminator)
    }

    /// Add openapi extensions (`x-something`) for [`AnyOf`].
    pub fn extensions(mut self, extensions: Option<Extensions>) -> Self {
        set_value!(self extensions extensions)
    }

    to_array_builder!();
}

impl From<AnyOf> for Schema {
    fn from(any_of: AnyOf) -> Self {
        Self::AnyOf(any_of)
    }
}

impl From<AnyOfBuilder> for RefOr<Schema> {
    fn from(any_of: AnyOfBuilder) -> Self {
        Self::T(Schema::AnyOf(any_of.build()))
    }
}

impl From<AnyOfBuilder> for ArrayItems {
    fn from(value: AnyOfBuilder) -> Self {
        Self::RefOrSchema(Box::new(value.into()))
    }
}

component_from_builder!(AnyOfBuilder);

#[cfg(not(feature = "preserve_order"))]
type ObjectPropertiesMap<K, V> = BTreeMap<K, V>;
#[cfg(feature = "preserve_order")]
type ObjectPropertiesMap<K, V> = indexmap::IndexMap<K, V>;

builder! {
    ObjectBuilder;

    /// Implements subset of [OpenAPI Schema Object][schema] which allows
    /// adding other [`Schema`]s as **properties** to this [`Schema`].
    ///
    /// This is a generic OpenAPI schema object which can used to present `object`, `field` or an `enum`.
    ///
    /// [schema]: https://spec.openapis.org/oas/latest.html#schema-object
    #[non_exhaustive]
    #[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[serde(rename_all = "camelCase")]
    pub struct Object {
        /// Type of [`Object`] e.g. [`Type::Object`] for `object` and [`Type::String`] for
        /// `string` types.
        #[serde(rename = "type", skip_serializing_if="SchemaType::is_any_value")]
        pub schema_type: SchemaType,

        /// Changes the [`Object`] title.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub title: Option<String>,

        /// Additional format for detailing the schema type.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub format: Option<SchemaFormat>,

        /// Description of the [`Object`]. Markdown syntax is supported.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,

        /// Default value which is provided when user has not provided the input in Swagger UI.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub default: Option<Value>,

        /// Enum variants of fields that can be represented as `unit` type `enums`.
        #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
        pub enum_values: Option<Vec<Value>>,

        /// Vector of required field names.
        #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
        pub required: Vec<String>,

        /// Map of fields with their [`Schema`] types.
        ///
        /// With **preserve_order** feature flag [`indexmap::IndexMap`] will be used as
        /// properties map backing implementation to retain property order of [`ToSchema`][to_schema].
        /// By default [`BTreeMap`] will be used.
        ///
        /// [to_schema]: crate::ToSchema
        #[serde(skip_serializing_if = "ObjectPropertiesMap::is_empty", default = "ObjectPropertiesMap::new")]
        pub properties: ObjectPropertiesMap<String, RefOr<Schema>>,

        /// Additional [`Schema`] for non specified fields (Useful for typed maps).
        #[serde(skip_serializing_if = "Option::is_none")]
        pub additional_properties: Option<Box<AdditionalProperties<Schema>>>,

        /// Additional [`Schema`] to describe property names of an object such as a map. See more
        /// details <https://json-schema.org/draft/2020-12/draft-bhutton-json-schema-01#name-propertynames>
        #[serde(skip_serializing_if = "Option::is_none")]
        pub property_names: Option<Box<Schema>>,

        /// Changes the [`Object`] deprecated status.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub deprecated: Option<Deprecated>,

        /// Example shown in UI of the value for richer documentation.
        ///
        /// **Deprecated since 3.0.x. Prefer [`Object::examples`] instead**
        #[serde(skip_serializing_if = "Option::is_none")]
        pub example: Option<Value>,

        /// Examples shown in UI of the value for richer documentation.
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        pub examples: Vec<Value>,

        /// Write only property will be only sent in _write_ requests like _POST, PUT_.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub write_only: Option<bool>,

        /// Read only property will be only sent in _read_ requests like _GET_.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub read_only: Option<bool>,

        /// Additional [`Xml`] formatting of the [`Object`].
        #[serde(skip_serializing_if = "Option::is_none")]
        pub xml: Option<Xml>,

        /// Must be a number strictly greater than `0`. Numeric value is considered valid if value
        /// divided by the _`multiple_of`_ value results an integer.
        #[serde(skip_serializing_if = "Option::is_none", serialize_with = "omit_decimal_zero")]
        pub multiple_of: Option<crate::utoipa::Number>,

        /// Specify inclusive upper limit for the [`Object`]'s value. Number is considered valid if
        /// it is equal or less than the _`maximum`_.
        #[serde(skip_serializing_if = "Option::is_none", serialize_with = "omit_decimal_zero")]
        pub maximum: Option<crate::utoipa::Number>,

        /// Specify inclusive lower limit for the [`Object`]'s value. Number value is considered
        /// valid if it is equal or greater than the _`minimum`_.
        #[serde(skip_serializing_if = "Option::is_none", serialize_with = "omit_decimal_zero")]
        pub minimum: Option<crate::utoipa::Number>,

        /// Specify exclusive upper limit for the [`Object`]'s value. Number value is considered
        /// valid if it is strictly less than _`exclusive_maximum`_.
        #[serde(skip_serializing_if = "Option::is_none", serialize_with = "omit_decimal_zero")]
        pub exclusive_maximum: Option<crate::utoipa::Number>,

        /// Specify exclusive lower limit for the [`Object`]'s value. Number value is considered
        /// valid if it is strictly above the _`exclusive_minimum`_.
        #[serde(skip_serializing_if = "Option::is_none", serialize_with = "omit_decimal_zero")]
        pub exclusive_minimum: Option<crate::utoipa::Number>,

        /// Specify maximum length for `string` values. _`max_length`_ cannot be a negative integer
        /// value. Value is considered valid if content length is equal or less than the _`max_length`_.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub max_length: Option<usize>,

        /// Specify minimum length for `string` values. _`min_length`_ cannot be a negative integer
        /// value. Setting this to _`0`_ has the same effect as omitting this field. Value is
        /// considered valid if content length is equal or more than the _`min_length`_.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub min_length: Option<usize>,

        /// Define a valid `ECMA-262` dialect regular expression. The `string` content is
        /// considered valid if the _`pattern`_ matches the value successfully.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub pattern: Option<String>,

        /// Specify inclusive maximum amount of properties an [`Object`] can hold.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub max_properties: Option<usize>,

        /// Specify inclusive minimum amount of properties an [`Object`] can hold. Setting this to
        /// `0` will have same effect as omitting the attribute.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub min_properties: Option<usize>,

        /// Optional extensions `x-something`.
        #[serde(skip_serializing_if = "Option::is_none", flatten)]
        pub extensions: Option<Extensions>,

        /// The `content_encoding` keyword specifies the encoding used to store the contents, as specified in
        /// [RFC 2054, part 6.1](https://tools.ietf.org/html/rfc2045) and [RFC 4648](RFC 2054, part 6.1).
        ///
        /// Typically this is either unset for _`string`_ content types which then uses the content
        /// encoding of the underlying JSON document. If the content is in _`binary`_ format such as an image or an audio
        /// set it to `base64` to encode it as _`Base64`_.
        ///
        /// See more details at <https://json-schema.org/understanding-json-schema/reference/non_json_data#contentencoding>
        #[serde(skip_serializing_if = "String::is_empty", default)]
        pub content_encoding: String,

        /// The _`content_media_type`_ keyword specifies the MIME type of the contents of a string,
        /// as described in [RFC 2046](https://tools.ietf.org/html/rfc2046).
        ///
        /// See more details at <https://json-schema.org/understanding-json-schema/reference/non_json_data#contentmediatype>
        #[serde(skip_serializing_if = "String::is_empty", default)]
        pub content_media_type: String,
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl Object {
    /// Initialize a new [`Object`] with default [`SchemaType`]. This effectively same as calling
    /// `Object::with_type(SchemaType::Object)`.
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Initialize new [`Object`] with given [`SchemaType`].
    ///
    /// Create [`std::string`] object type which can be used to define `string` field of an object.
    /// ```rust
    /// # use utoipa::openapi::schema::{Object, Type};
    /// let object = Object::with_type(Type::String);
    /// ```
    pub fn with_type<T: Into<SchemaType>>(schema_type: T) -> Self {
        Self {
            schema_type: schema_type.into(),
            ..Default::default()
        }
    }
}

impl From<Object> for Schema {
    fn from(s: Object) -> Self {
        Self::Object(s)
    }
}

impl From<Object> for ArrayItems {
    fn from(value: Object) -> Self {
        Self::RefOrSchema(Box::new(value.into()))
    }
}

impl ToArray for Object {}

impl ObjectBuilder {
    /// Add or change type of the object e.g. to change type to _`string`_
    /// use value `SchemaType::Type(Type::String)`.
    pub fn schema_type<T: Into<SchemaType>>(mut self, schema_type: T) -> Self {
        set_value!(self schema_type schema_type.into())
    }

    /// Add or change additional format for detailing the schema type.
    pub fn format(mut self, format: Option<SchemaFormat>) -> Self {
        set_value!(self format format)
    }

    /// Add new property to the [`Object`].
    ///
    /// Method accepts property name and property component as an arguments.
    pub fn property<S: Into<String>, I: Into<RefOr<Schema>>>(
        mut self,
        property_name: S,
        component: I,
    ) -> Self {
        self.properties
            .insert(property_name.into(), component.into());

        self
    }

    /// Add additional [`Schema`] for non specified fields (Useful for typed maps).
    pub fn additional_properties<I: Into<AdditionalProperties<Schema>>>(
        mut self,
        additional_properties: Option<I>,
    ) -> Self {
        set_value!(self additional_properties additional_properties.map(|additional_properties| Box::new(additional_properties.into())))
    }

    /// Add additional [`Schema`] to describe property names of an object such as a map. See more
    /// details <https://json-schema.org/draft/2020-12/draft-bhutton-json-schema-01#name-propertynames>
    pub fn property_names<S: Into<Schema>>(mut self, property_name: Option<S>) -> Self {
        set_value!(self property_names property_name.map(|property_name| Box::new(property_name.into())))
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
    pub fn default(mut self, default: Option<Value>) -> Self {
        set_value!(self default default)
    }

    /// Add or change deprecated status for [`Object`].
    pub fn deprecated(mut self, deprecated: Option<Deprecated>) -> Self {
        set_value!(self deprecated deprecated)
    }

    /// Add or change enum property variants.
    pub fn enum_values<I: IntoIterator<Item = E>, E: Into<Value>>(
        mut self,
        enum_values: Option<I>,
    ) -> Self {
        set_value!(self enum_values
            enum_values.map(|values| values.into_iter().map(|enum_value| enum_value.into()).collect()))
    }

    /// Add or change example shown in UI of the value for richer documentation.
    ///
    /// **Deprecated since 3.0.x. Prefer [`Object::examples`] instead**
    #[deprecated = "Since OpenAPI 3.1 prefer using `examples`"]
    pub fn example(mut self, example: Option<Value>) -> Self {
        set_value!(self example example)
    }

    /// Add or change examples shown in UI of the value for richer documentation.
    pub fn examples<I: IntoIterator<Item = V>, V: Into<Value>>(mut self, examples: I) -> Self {
        set_value!(self examples examples.into_iter().map(Into::into).collect())
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

    /// Set or change _`multiple_of`_ validation flag for `number` and `integer` type values.
    pub fn multiple_of<N: Into<crate::utoipa::Number>>(mut self, multiple_of: Option<N>) -> Self {
        set_value!(self multiple_of multiple_of.map(|multiple_of| multiple_of.into()))
    }

    /// Set or change inclusive maximum value for `number` and `integer` values.
    pub fn maximum<N: Into<crate::utoipa::Number>>(mut self, maximum: Option<N>) -> Self {
        set_value!(self maximum maximum.map(|max| max.into()))
    }

    /// Set or change inclusive minimum value for `number` and `integer` values.
    pub fn minimum<N: Into<crate::utoipa::Number>>(mut self, minimum: Option<N>) -> Self {
        set_value!(self minimum minimum.map(|min| min.into()))
    }

    /// Set or change exclusive maximum value for `number` and `integer` values.
    pub fn exclusive_maximum<N: Into<crate::utoipa::Number>>(
        mut self,
        exclusive_maximum: Option<N>,
    ) -> Self {
        set_value!(self exclusive_maximum exclusive_maximum.map(|exclusive_maximum| exclusive_maximum.into()))
    }

    /// Set or change exclusive minimum value for `number` and `integer` values.
    pub fn exclusive_minimum<N: Into<crate::utoipa::Number>>(
        mut self,
        exclusive_minimum: Option<N>,
    ) -> Self {
        set_value!(self exclusive_minimum exclusive_minimum.map(|exclusive_minimum| exclusive_minimum.into()))
    }

    /// Set or change maximum length for `string` values.
    pub fn max_length(mut self, max_length: Option<usize>) -> Self {
        set_value!(self max_length max_length)
    }

    /// Set or change minimum length for `string` values.
    pub fn min_length(mut self, min_length: Option<usize>) -> Self {
        set_value!(self min_length min_length)
    }

    /// Set or change a valid regular expression for `string` value to match.
    pub fn pattern<I: Into<String>>(mut self, pattern: Option<I>) -> Self {
        set_value!(self pattern pattern.map(|pattern| pattern.into()))
    }

    /// Set or change maximum number of properties the [`Object`] can hold.
    pub fn max_properties(mut self, max_properties: Option<usize>) -> Self {
        set_value!(self max_properties max_properties)
    }

    /// Set or change minimum number of properties the [`Object`] can hold.
    pub fn min_properties(mut self, min_properties: Option<usize>) -> Self {
        set_value!(self min_properties min_properties)
    }

    /// Add openapi extensions (`x-something`) for [`Object`].
    pub fn extensions(mut self, extensions: Option<Extensions>) -> Self {
        set_value!(self extensions extensions)
    }

    /// Set of change [`Object::content_encoding`]. Typically left empty but could be `base64` for
    /// example.
    pub fn content_encoding<S: Into<String>>(mut self, content_encoding: S) -> Self {
        set_value!(self content_encoding content_encoding.into())
    }

    /// Set of change [`Object::content_media_type`]. Value must be valid MIME type e.g.
    /// `application/json`.
    pub fn content_media_type<S: Into<String>>(mut self, content_media_type: S) -> Self {
        set_value!(self content_media_type content_media_type.into())
    }

    to_array_builder!();
}

component_from_builder!(ObjectBuilder);

impl From<ObjectBuilder> for RefOr<Schema> {
    fn from(builder: ObjectBuilder) -> Self {
        Self::T(Schema::Object(builder.build()))
    }
}

impl From<RefOr<Schema>> for Schema {
    fn from(value: RefOr<Schema>) -> Self {
        match value {
            RefOr::Ref(_) => {
                panic!("Invalid type `RefOr::Ref` provided, cannot convert to RefOr::T<Schema>")
            }
            RefOr::T(value) => value,
        }
    }
}

impl From<ObjectBuilder> for ArrayItems {
    fn from(value: ObjectBuilder) -> Self {
        Self::RefOrSchema(Box::new(value.into()))
    }
}

/// AdditionalProperties is used to define values of map fields of the [`Schema`].
///
/// The value can either be [`RefOr`] or _`bool`_.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(untagged)]
pub enum AdditionalProperties<T> {
    /// Use when value type of the map is a known [`Schema`] or [`Ref`] to the [`Schema`].
    RefOr(RefOr<T>),
    /// Use _`AdditionalProperties::FreeForm(true)`_ when any value is allowed in the map.
    FreeForm(bool),
}

impl<T> From<RefOr<T>> for AdditionalProperties<T> {
    fn from(value: RefOr<T>) -> Self {
        Self::RefOr(value)
    }
}

impl From<ObjectBuilder> for AdditionalProperties<Schema> {
    fn from(value: ObjectBuilder) -> Self {
        Self::RefOr(RefOr::T(Schema::Object(value.build())))
    }
}

impl From<ArrayBuilder> for AdditionalProperties<Schema> {
    fn from(value: ArrayBuilder) -> Self {
        Self::RefOr(RefOr::T(Schema::Array(value.build())))
    }
}

impl From<Ref> for AdditionalProperties<Schema> {
    fn from(value: Ref) -> Self {
        Self::RefOr(RefOr::Ref(value))
    }
}

impl From<RefBuilder> for AdditionalProperties<Schema> {
    fn from(value: RefBuilder) -> Self {
        Self::RefOr(RefOr::Ref(value.build()))
    }
}

impl From<Schema> for AdditionalProperties<Schema> {
    fn from(value: Schema) -> Self {
        Self::RefOr(RefOr::T(value))
    }
}

impl From<AllOfBuilder> for AdditionalProperties<Schema> {
    fn from(value: AllOfBuilder) -> Self {
        Self::RefOr(RefOr::T(Schema::AllOf(value.build())))
    }
}

builder! {
    RefBuilder;

    /// Implements [OpenAPI Reference Object][reference] that can be used to reference
    /// reusable components such as [`Schema`]s or [`Response`]s.
    ///
    /// [reference]: https://spec.openapis.org/oas/latest.html#reference-object
    #[non_exhaustive]
    #[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Ref {
        /// Reference location of the actual component.
        #[serde(rename = "$ref")]
        pub ref_location: String,

        /// A description which by default should override that of the referenced component.
        /// Description supports markdown syntax. If referenced object type does not support
        /// description this field does not have effect.
        #[serde(skip_serializing_if = "String::is_empty", default)]
        pub description: String,

        /// A short summary which by default should override that of the referenced component. If
        /// referenced component does not support summary field this does not have effect.
        #[serde(skip_serializing_if = "String::is_empty", default)]
        pub summary: String,
    }
}

impl Ref {
    /// Construct a new [`Ref`] with custom ref location. In most cases this is not necessary
    /// and [`Ref::from_schema_name`] could be used instead.
    pub fn new<I: Into<String>>(ref_location: I) -> Self {
        Self {
            ref_location: ref_location.into(),
            ..Default::default()
        }
    }

    /// Construct a new [`Ref`] from provided schema name. This will create a [`Ref`] that
    /// references the the reusable schemas.
    pub fn from_schema_name<I: Into<String>>(schema_name: I) -> Self {
        Self::new(format!("#/components/schemas/{}", schema_name.into()))
    }

    /// Construct a new [`Ref`] from provided response name. This will create a [`Ref`] that
    /// references the reusable response.
    pub fn from_response_name<I: Into<String>>(response_name: I) -> Self {
        Self::new(format!("#/components/responses/{}", response_name.into()))
    }

    to_array_builder!();
}

impl RefBuilder {
    /// Add or change reference location of the actual component.
    pub fn ref_location(mut self, ref_location: String) -> Self {
        set_value!(self ref_location ref_location)
    }

    /// Add or change reference location of the actual component automatically formatting the $ref
    /// to `#/components/schemas/...` format.
    pub fn ref_location_from_schema_name<S: Into<String>>(mut self, schema_name: S) -> Self {
        set_value!(self ref_location format!("#/components/schemas/{}", schema_name.into()))
    }

    // TODO: REMOVE THE unnecessary description Option wrapping.

    /// Add or change description which by default should override that of the referenced component.
    /// Description supports markdown syntax. If referenced object type does not support
    /// description this field does not have effect.
    pub fn description<S: Into<String>>(mut self, description: Option<S>) -> Self {
        set_value!(self description description.map(Into::into).unwrap_or_default())
    }

    /// Add or change short summary which by default should override that of the referenced component. If
    /// referenced component does not support summary field this does not have effect.
    pub fn summary<S: Into<String>>(mut self, summary: S) -> Self {
        set_value!(self summary summary.into())
    }
}

impl From<RefBuilder> for RefOr<Schema> {
    fn from(builder: RefBuilder) -> Self {
        Self::Ref(builder.build())
    }
}

impl From<RefBuilder> for ArrayItems {
    fn from(value: RefBuilder) -> Self {
        Self::RefOrSchema(Box::new(value.into()))
    }
}

impl From<Ref> for RefOr<Schema> {
    fn from(r: Ref) -> Self {
        Self::Ref(r)
    }
}

impl From<Ref> for ArrayItems {
    fn from(value: Ref) -> Self {
        Self::RefOrSchema(Box::new(value.into()))
    }
}

impl<T> From<T> for RefOr<T> {
    fn from(t: T) -> Self {
        Self::T(t)
    }
}

impl Default for RefOr<Schema> {
    fn default() -> Self {
        Self::T(Schema::Object(Object::new()))
    }
}

impl ToArray for RefOr<Schema> {}

impl From<Object> for RefOr<Schema> {
    fn from(object: Object) -> Self {
        Self::T(Schema::Object(object))
    }
}

impl From<Array> for RefOr<Schema> {
    fn from(array: Array) -> Self {
        Self::T(Schema::Array(array))
    }
}

fn omit_decimal_zero<S>(
    maybe_value: &Option<crate::utoipa::Number>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match maybe_value {
        Some(crate::utoipa::Number::Float(float)) => {
            if float.fract() == 0.0 && *float >= i64::MIN as f64 && *float <= i64::MAX as f64 {
                serializer.serialize_i64(float.trunc() as i64)
            } else {
                serializer.serialize_f64(*float)
            }
        }
        Some(crate::utoipa::Number::Int(int)) => serializer.serialize_i64(*int as i64),
        Some(crate::utoipa::Number::UInt(uint)) => serializer.serialize_u64(*uint as u64),
        None => serializer.serialize_none(),
    }
}

/// Represents [`Array`] items in [JSON Schema Array][json_schema_array].
///
/// [json_schema_array]: <https://json-schema.org/understanding-json-schema/reference/array#items>
#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(untagged)]
pub enum ArrayItems {
    /// Defines [`Array::items`] as [`RefOr::T(Schema)`]. This is the default for [`Array`].
    RefOrSchema(Box<RefOr<Schema>>),
    /// Defines [`Array::items`] as `false` indicating that no extra items are allowed to the
    /// [`Array`]. This can be used together with [`Array::prefix_items`] to disallow [additional
    /// items][additional_items] in [`Array`].
    ///
    /// [additional_items]: <https://json-schema.org/understanding-json-schema/reference/array#additionalitems>
    #[serde(with = "array_items_false")]
    False,
}

mod array_items_false {
    use serde::de::Visitor;

    pub fn serialize<S: serde::Serializer>(serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bool(false)
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<(), D::Error> {
        struct ItemsFalseVisitor;

        impl<'de> Visitor<'de> for ItemsFalseVisitor {
            type Value = ();
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if !v {
                    Ok(())
                } else {
                    Err(serde::de::Error::custom(format!(
                        "invalid boolean value: {v}, expected false"
                    )))
                }
            }

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("expected boolean false")
            }
        }

        deserializer.deserialize_bool(ItemsFalseVisitor)
    }
}

impl Default for ArrayItems {
    fn default() -> Self {
        Self::RefOrSchema(Box::new(Object::with_type(SchemaType::AnyValue).into()))
    }
}

impl From<RefOr<Schema>> for ArrayItems {
    fn from(value: RefOr<Schema>) -> Self {
        Self::RefOrSchema(Box::new(value))
    }
}

builder! {
    ArrayBuilder;

    /// Array represents [`Vec`] or [`slice`] type  of items.
    ///
    /// See [`Schema::Array`] for more details.
    #[non_exhaustive]
    #[derive(Serialize, Deserialize, Clone, PartialEq)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[serde(rename_all = "camelCase")]
    pub struct Array {
        /// Type will always be [`SchemaType::Array`].
        #[serde(rename = "type")]
        pub schema_type: SchemaType,

        /// Changes the [`Array`] title.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub title: Option<String>,

        /// Items of the [`Array`].
        pub items: ArrayItems,

        /// Prefix items of [`Array`] is used to define item validation of tuples according [JSON schema
        /// item validation][item_validation].
        ///
        /// [item_validation]: <https://json-schema.org/understanding-json-schema/reference/array#tupleValidation>
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        pub prefix_items: Vec<Schema>,

        /// Description of the [`Array`]. Markdown syntax is supported.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,

        /// Marks the [`Array`] deprecated.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub deprecated: Option<Deprecated>,

        /// Example shown in UI of the value for richer documentation.
        ///
        /// **Deprecated since 3.0.x. Prefer [`Array::examples`] instead**
        #[serde(skip_serializing_if = "Option::is_none")]
        pub example: Option<Value>,

        /// Examples shown in UI of the value for richer documentation.
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        pub examples: Vec<Value>,

        /// Default value which is provided when user has not provided the input in Swagger UI.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub default: Option<Value>,

        /// Max length of the array.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub max_items: Option<usize>,

        /// Min length of the array.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub min_items: Option<usize>,

        /// Setting this to `true` will validate successfully if all elements of this [`Array`] are
        /// unique.
        #[serde(default, skip_serializing_if = "is_false")]
        pub unique_items: bool,

        /// Xml format of the array.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub xml: Option<Xml>,

        /// The `content_encoding` keyword specifies the encoding used to store the contents, as specified in
        /// [RFC 2054, part 6.1](https://tools.ietf.org/html/rfc2045) and [RFC 4648](RFC 2054, part 6.1).
        ///
        /// Typically this is either unset for _`string`_ content types which then uses the content
        /// encoding of the underlying JSON document. If the content is in _`binary`_ format such as an image or an audio
        /// set it to `base64` to encode it as _`Base64`_.
        ///
        /// See more details at <https://json-schema.org/understanding-json-schema/reference/non_json_data#contentencoding>
        #[serde(skip_serializing_if = "String::is_empty", default)]
        pub content_encoding: String,

        /// The _`content_media_type`_ keyword specifies the MIME type of the contents of a string,
        /// as described in [RFC 2046](https://tools.ietf.org/html/rfc2046).
        ///
        /// See more details at <https://json-schema.org/understanding-json-schema/reference/non_json_data#contentmediatype>
        #[serde(skip_serializing_if = "String::is_empty", default)]
        pub content_media_type: String,

        /// Optional extensions `x-something`.
        #[serde(skip_serializing_if = "Option::is_none", flatten)]
        pub extensions: Option<Extensions>,
    }
}

impl Default for Array {
    fn default() -> Self {
        Self {
            title: Default::default(),
            schema_type: Type::Array.into(),
            unique_items: bool::default(),
            items: Default::default(),
            prefix_items: Vec::default(),
            description: Default::default(),
            deprecated: Default::default(),
            example: Default::default(),
            examples: Default::default(),
            default: Default::default(),
            max_items: Default::default(),
            min_items: Default::default(),
            xml: Default::default(),
            extensions: Default::default(),
            content_encoding: Default::default(),
            content_media_type: Default::default(),
        }
    }
}

impl Array {
    /// Construct a new [`Array`] component from given [`Schema`].
    ///
    /// # Examples
    ///
    /// _**Create a `String` array component**_.
    /// ```rust
    /// # use utoipa::openapi::schema::{Schema, Array, Type, Object};
    /// let string_array = Array::new(Object::with_type(Type::String));
    /// ```
    pub fn new<I: Into<RefOr<Schema>>>(component: I) -> Self {
        Self {
            items: ArrayItems::RefOrSchema(Box::new(component.into())),
            ..Default::default()
        }
    }

    /// Construct a new nullable [`Array`] component from given [`Schema`].
    ///
    /// # Examples
    ///
    /// _**Create a nullable `String` array component**_.
    /// ```rust
    /// # use utoipa::openapi::schema::{Schema, Array, Type, Object};
    /// let string_array = Array::new_nullable(Object::with_type(Type::String));
    /// ```
    pub fn new_nullable<I: Into<RefOr<Schema>>>(component: I) -> Self {
        Self {
            items: ArrayItems::RefOrSchema(Box::new(component.into())),
            schema_type: SchemaType::from_iter([Type::Array, Type::Null]),
            ..Default::default()
        }
    }
}

impl ArrayBuilder {
    /// Set [`Schema`] type for the [`Array`].
    pub fn items<I: Into<ArrayItems>>(mut self, items: I) -> Self {
        set_value!(self items items.into())
    }

    /// Add prefix items of [`Array`] to define item validation of tuples according [JSON schema
    /// item validation][item_validation].
    ///
    /// [item_validation]: <https://json-schema.org/understanding-json-schema/reference/array#tupleValidation>
    pub fn prefix_items<I: IntoIterator<Item = S>, S: Into<Schema>>(mut self, items: I) -> Self {
        self.prefix_items = items
            .into_iter()
            .map(|item| item.into())
            .collect::<Vec<_>>();

        self
    }

    /// Change type of the array e.g. to change type to _`string`_
    /// use value `SchemaType::Type(Type::String)`.
    ///
    /// # Examples
    ///
    /// _**Make nullable string array.**_
    /// ```rust
    /// # use utoipa::openapi::schema::{ArrayBuilder, SchemaType, Type, Object};
    /// let _ = ArrayBuilder::new()
    ///     .schema_type(SchemaType::from_iter([Type::Array, Type::Null]))
    ///     .items(Object::with_type(Type::String))
    ///     .build();
    /// ```
    pub fn schema_type<T: Into<SchemaType>>(mut self, schema_type: T) -> Self {
        set_value!(self schema_type schema_type.into())
    }

    /// Add or change the title of the [`Array`].
    pub fn title<I: Into<String>>(mut self, title: Option<I>) -> Self {
        set_value!(self title title.map(|title| title.into()))
    }

    /// Add or change description of the property. Markdown syntax is supported.
    pub fn description<I: Into<String>>(mut self, description: Option<I>) -> Self {
        set_value!(self description description.map(|description| description.into()))
    }

    /// Add or change deprecated status for [`Array`].
    pub fn deprecated(mut self, deprecated: Option<Deprecated>) -> Self {
        set_value!(self deprecated deprecated)
    }

    /// Add or change example shown in UI of the value for richer documentation.
    ///
    /// **Deprecated since 3.0.x. Prefer [`Array::examples`] instead**
    #[deprecated = "Since OpenAPI 3.1 prefer using `examples`"]
    pub fn example(mut self, example: Option<Value>) -> Self {
        set_value!(self example example)
    }

    /// Add or change examples shown in UI of the value for richer documentation.
    pub fn examples<I: IntoIterator<Item = V>, V: Into<Value>>(mut self, examples: I) -> Self {
        set_value!(self examples examples.into_iter().map(Into::into).collect())
    }

    /// Add or change default value for the object which is provided when user has not provided the input in Swagger UI.
    pub fn default(mut self, default: Option<Value>) -> Self {
        set_value!(self default default)
    }

    /// Set maximum allowed length for [`Array`].
    pub fn max_items(mut self, max_items: Option<usize>) -> Self {
        set_value!(self max_items max_items)
    }

    /// Set minimum allowed length for [`Array`].
    pub fn min_items(mut self, min_items: Option<usize>) -> Self {
        set_value!(self min_items min_items)
    }

    /// Set or change whether [`Array`] should enforce all items to be unique.
    pub fn unique_items(mut self, unique_items: bool) -> Self {
        set_value!(self unique_items unique_items)
    }

    /// Set [`Xml`] formatting for [`Array`].
    pub fn xml(mut self, xml: Option<Xml>) -> Self {
        set_value!(self xml xml)
    }

    /// Set of change [`Object::content_encoding`]. Typically left empty but could be `base64` for
    /// example.
    pub fn content_encoding<S: Into<String>>(mut self, content_encoding: S) -> Self {
        set_value!(self content_encoding content_encoding.into())
    }

    /// Set of change [`Object::content_media_type`]. Value must be valid MIME type e.g.
    /// `application/json`.
    pub fn content_media_type<S: Into<String>>(mut self, content_media_type: S) -> Self {
        set_value!(self content_media_type content_media_type.into())
    }

    /// Add openapi extensions (`x-something`) for [`Array`].
    pub fn extensions(mut self, extensions: Option<Extensions>) -> Self {
        set_value!(self extensions extensions)
    }

    to_array_builder!();
}

component_from_builder!(ArrayBuilder);

impl From<Array> for Schema {
    fn from(array: Array) -> Self {
        Self::Array(array)
    }
}

impl From<ArrayBuilder> for ArrayItems {
    fn from(value: ArrayBuilder) -> Self {
        Self::RefOrSchema(Box::new(value.into()))
    }
}

impl From<ArrayBuilder> for RefOr<Schema> {
    fn from(array: ArrayBuilder) -> Self {
        Self::T(Schema::Array(array.build()))
    }
}

impl ToArray for Array {}

/// This convenience trait allows quick way to wrap any `RefOr<Schema>` with [`Array`] schema.
pub trait ToArray
where
    RefOr<Schema>: From<Self>,
    Self: Sized,
{
    /// Wrap this `RefOr<Schema>` with [`Array`].
    fn to_array(self) -> Array {
        Array::new(self)
    }
}

/// Represents type of [`Schema`].
///
/// This is a collection type for [`Type`] that can be represented as a single value
/// or as [`slice`] of [`Type`]s.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(untagged)]
pub enum SchemaType {
    /// Single type known from OpenAPI spec 3.0
    Type(Type),
    /// Multiple types rendered as [`slice`]
    Array(Vec<Type>),
    /// Type that is considered typeless. _`AnyValue`_ will omit the type definition from the schema
    /// making it to accept any type possible.
    AnyValue,
}

impl Default for SchemaType {
    fn default() -> Self {
        Self::Type(Type::default())
    }
}

impl From<Type> for SchemaType {
    fn from(value: Type) -> Self {
        SchemaType::new(value)
    }
}

impl FromIterator<Type> for SchemaType {
    fn from_iter<T: IntoIterator<Item = Type>>(iter: T) -> Self {
        Self::Array(iter.into_iter().collect())
    }
}

impl SchemaType {
    /// Instantiate new [`SchemaType`] of given [`Type`]
    ///
    /// Method accepts one argument `type` to create [`SchemaType`] for.
    ///
    /// # Examples
    ///
    /// _**Create string [`SchemaType`]**_
    /// ```rust
    /// # use utoipa::openapi::schema::{SchemaType, Type};
    /// let ty = SchemaType::new(Type::String);
    /// ```
    pub fn new(r#type: Type) -> Self {
        Self::Type(r#type)
    }

    //// Instantiate new [`SchemaType::AnyValue`].
    ///
    /// This is same as calling [`SchemaType::AnyValue`] but in a function form `() -> SchemaType`
    /// allowing it to be used as argument for _serde's_ _`default = "..."`_.
    pub fn any() -> Self {
        SchemaType::AnyValue
    }

    /// Check whether this [`SchemaType`] is any value _(typeless)_ returning true on any value
    /// schema type.
    pub fn is_any_value(&self) -> bool {
        matches!(self, Self::AnyValue)
    }
}

/// Represents data type fragment of [`Schema`].
///
/// [`Type`] is used to create a [`SchemaType`] that defines the type of the [`Schema`].
/// [`SchemaType`] can be created from a single [`Type`] or multiple [`Type`]s according to the
/// OpenAPI 3.1 spec. Since the OpenAPI 3.1 is fully compatible with JSON schema the definition of
/// the _**type**_ property comes from [JSON Schema type](https://json-schema.org/understanding-json-schema/reference/type).
///
/// # Examples
/// _**Create nullable string [`SchemaType`]**_
/// ```rust
/// # use std::iter::FromIterator;
/// # use utoipa::openapi::schema::{Type, SchemaType};
/// let _: SchemaType = [Type::String, Type::Null].into_iter().collect();
/// ```
/// _**Create string [`SchemaType`]**_
/// ```rust
/// # use utoipa::openapi::schema::{Type, SchemaType};
/// let _ = SchemaType::new(Type::String);
/// ```
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "lowercase")]
pub enum Type {
    /// Used with [`Object`] and [`ObjectBuilder`] to describe schema that has _properties_
    /// describing fields.
    #[default]
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
    /// Null type. Used together with other type to indicate nullable values.
    Null,
}

/// Additional format for [`SchemaType`] to fine tune the data type used. If the **format** is not
/// supported by the UI it may default back to [`SchemaType`] alone.
/// Format is an open value, so you can use any formats, even not those defined by the
/// OpenAPI Specification.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "lowercase", untagged)]
pub enum SchemaFormat {
    /// Use to define additional detail about the value.
    KnownFormat(KnownFormat),
    /// Can be used to provide additional detail about the value when [`SchemaFormat::KnownFormat`]
    /// is not suitable.
    Custom(String),
}

/// Known schema format modifier property to provide fine detail of the primitive type.
///
/// Known format is defined in <https://spec.openapis.org/oas/latest.html#data-types> and
/// <https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#section-7.3> as
/// well as by few known data types that are enabled by specific feature flag e.g. _`uuid`_.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "kebab-case")]
pub enum KnownFormat {
    /// 8 bit integer.
    #[cfg(feature = "non_strict_integers")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "non_strict_integers")))]
    Int8,
    /// 16 bit integer.
    #[cfg(feature = "non_strict_integers")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "non_strict_integers")))]
    Int16,
    /// 32 bit integer.
    Int32,
    /// 64 bit integer.
    Int64,
    /// 8 bit unsigned integer.
    #[cfg(feature = "non_strict_integers")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "non_strict_integers")))]
    UInt8,
    /// 16 bit unsigned integer.
    #[cfg(feature = "non_strict_integers")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "non_strict_integers")))]
    UInt16,
    /// 32 bit unsigned integer.
    #[cfg(feature = "non_strict_integers")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "non_strict_integers")))]
    UInt32,
    /// 64 bit unsigned integer.
    #[cfg(feature = "non_strict_integers")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "non_strict_integers")))]
    UInt64,
    /// floating point number.
    Float,
    /// double (floating point) number.
    Double,
    /// base64 encoded chars.
    Byte,
    /// binary data (octet).
    Binary,
    /// ISO-8601 full time format [RFC3339](https://xml2rfc.ietf.org/public/rfc/html/rfc3339.html#anchor14).
    Time,
    /// ISO-8601 full date [RFC3339](https://xml2rfc.ietf.org/public/rfc/html/rfc3339.html#anchor14).
    Date,
    /// ISO-8601 full date time [RFC3339](https://xml2rfc.ietf.org/public/rfc/html/rfc3339.html#anchor14).
    DateTime,
    /// duration format from [RFC3339 Appendix-A](https://datatracker.ietf.org/doc/html/rfc3339#appendix-A).
    Duration,
    /// Hint to UI to obscure input.
    Password,
    /// Used with [`String`] values to indicate value is in UUID format.
    ///
    /// **uuid** feature need to be enabled.
    #[cfg(feature = "uuid")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "uuid")))]
    Uuid,
    /// Used with [`String`] values to indicate value is in ULID format.
    #[cfg(feature = "ulid")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "ulid")))]
    Ulid,
    /// Used with [`String`] values to indicate value is in Url format according to
    /// [RFC3986](https://datatracker.ietf.org/doc/html/rfc3986).
    #[cfg(feature = "url")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "url")))]
    Uri,
    /// A string instance is valid against this attribute if it is a valid URI Reference
    /// (either a URI or a relative-reference) according to
    /// [RFC3986](https://datatracker.ietf.org/doc/html/rfc3986).
    #[cfg(feature = "url")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "url")))]
    UriReference,
    /// A string instance is valid against this attribute if it is a
    /// valid IRI, according to [RFC3987](https://datatracker.ietf.org/doc/html/rfc3987).
    #[cfg(feature = "url")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "url")))]
    Iri,
    /// A string instance is valid against this attribute if it is a valid IRI Reference
    /// (either an IRI or a relative-reference)
    /// according to [RFC3987](https://datatracker.ietf.org/doc/html/rfc3987).
    #[cfg(feature = "url")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "url")))]
    IriReference,
    /// As defined in "Mailbox" rule [RFC5321](https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.2).
    Email,
    /// As defined by extended "Mailbox" rule [RFC6531](https://datatracker.ietf.org/doc/html/rfc6531#section-3.3).
    IdnEmail,
    /// As defined by [RFC1123](https://datatracker.ietf.org/doc/html/rfc1123#section-2.1), including host names
    /// produced using the Punycode algorithm
    /// specified in [RFC5891](https://datatracker.ietf.org/doc/html/rfc5891#section-4.4).
    Hostname,
    /// As defined by either [RFC1123](https://datatracker.ietf.org/doc/html/rfc1123#section-2.1) as for hostname,
    /// or an internationalized hostname as defined by [RFC5890](https://datatracker.ietf.org/doc/html/rfc5890#section-2.3.2.3).
    IdnHostname,
    /// An IPv4 address according to [RFC2673](https://datatracker.ietf.org/doc/html/rfc2673#section-3.2).
    Ipv4,
    /// An IPv6 address according to [RFC4291](https://datatracker.ietf.org/doc/html/rfc4291#section-2.2).
    Ipv6,
    /// A string instance is a valid URI Template if it is according to
    /// [RFC6570](https://datatracker.ietf.org/doc/html/rfc6570).
    ///
    /// _**Note!**_ There are no separate IRL template.
    UriTemplate,
    /// A valid JSON string representation of a JSON Pointer according to [RFC6901](https://datatracker.ietf.org/doc/html/rfc6901#section-5).
    JsonPointer,
    /// A valid relative JSON Pointer according to [draft-handrews-relative-json-pointer-01](https://datatracker.ietf.org/doc/html/draft-handrews-relative-json-pointer-01).
    RelativeJsonPointer,
    /// Regular expression, which SHOULD be valid according to the
    /// [ECMA-262](https://datatracker.ietf.org/doc/html/draft-bhutton-json-schema-validation-00#ref-ecma262).
    Regex,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
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
                                        .schema_type(Type::Integer)
                                        .format(Some(SchemaFormat::KnownFormat(KnownFormat::Int32)))
                                        .description(Some("Id of credential"))
                                        .default(Some(json!(1i32))),
                                )
                                .property(
                                    "name",
                                    ObjectBuilder::new()
                                        .schema_type(Type::String)
                                        .description(Some("Name of credential")),
                                )
                                .property(
                                    "status",
                                    ObjectBuilder::new()
                                        .schema_type(Type::String)
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
                                .property("tags", Object::with_type(Type::String).to_array()),
                        ),
                    )
                    .build(),
            ))
            .build();

        let serialized = serde_json::to_string_pretty(&openapi)?;
        println!("serialized json:\n {serialized}");

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
    fn test_property_order() {
        let json_value = ObjectBuilder::new()
            .property(
                "id",
                ObjectBuilder::new()
                    .schema_type(Type::Integer)
                    .format(Some(SchemaFormat::KnownFormat(KnownFormat::Int32)))
                    .description(Some("Id of credential"))
                    .default(Some(json!(1i32))),
            )
            .property(
                "name",
                ObjectBuilder::new()
                    .schema_type(Type::String)
                    .description(Some("Name of credential")),
            )
            .property(
                "status",
                ObjectBuilder::new()
                    .schema_type(Type::String)
                    .default(Some(json!("Active")))
                    .description(Some("Credential status"))
                    .enum_values(Some(["Active", "NotActive", "Locked", "Expired"])),
            )
            .property(
                "history",
                Array::new(Ref::from_schema_name("UpdateHistory")),
            )
            .property("tags", Object::with_type(Type::String).to_array())
            .build();

        #[cfg(not(feature = "preserve_order"))]
        assert_eq!(
            json_value.properties.keys().collect::<Vec<_>>(),
            vec!["history", "id", "name", "status", "tags"]
        );

        #[cfg(feature = "preserve_order")]
        assert_eq!(
            json_value.properties.keys().collect::<Vec<_>>(),
            vec!["id", "name", "status", "history", "tags"]
        );
    }

    // Examples taken from https://spec.openapis.org/oas/latest.html#model-with-map-dictionary-properties
    #[test]
    fn test_additional_properties() {
        let json_value = ObjectBuilder::new()
            .additional_properties(Some(ObjectBuilder::new().schema_type(Type::String)))
            .build();
        assert_json_snapshot!(json_value, @r#"
        {
          "type": "object",
          "additionalProperties": {
            "type": "string"
          }
        }
        "#);

        let json_value = ObjectBuilder::new()
            .additional_properties(Some(ArrayBuilder::new().items(ArrayItems::RefOrSchema(
                Box::new(ObjectBuilder::new().schema_type(Type::Number).into()),
            ))))
            .build();
        assert_json_snapshot!(json_value, @r#"
        {
          "type": "object",
          "additionalProperties": {
            "type": "array",
            "items": {
              "type": "number"
            }
          }
        }
        "#);

        let json_value = ObjectBuilder::new()
            .additional_properties(Some(Ref::from_schema_name("ComplexModel")))
            .build();
        assert_json_snapshot!(json_value, @r##"
        {
          "type": "object",
          "additionalProperties": {
            "$ref": "#/components/schemas/ComplexModel"
          }
        }
        "##);
    }

    #[test]
    fn test_object_with_title() {
        let json_value = ObjectBuilder::new().title(Some("SomeName")).build();
        assert_json_snapshot!(json_value, @r#"
        {
          "type": "object",
          "title": "SomeName"
        }
        "#);
    }

    #[test]
    fn derive_object_with_examples() {
        let json_value = ObjectBuilder::new()
            .examples([Some(json!({"age": 20, "name": "bob the cat"}))])
            .build();
        assert_json_snapshot!(json_value, @r#"
        {
          "type": "object",
          "examples": [
            {
              "age": 20,
              "name": "bob the cat"
            }
          ]
        }
        "#);
    }

    fn get_json_path<'a>(value: &'a Value, path: &str) -> &'a Value {
        path.split('.').fold(value, |acc, fragment| {
            acc.get(fragment).unwrap_or(&serde_json::value::Value::Null)
        })
    }

    #[test]
    fn test_array_new() {
        let array = Array::new(
            ObjectBuilder::new().property(
                "id",
                ObjectBuilder::new()
                    .schema_type(Type::Integer)
                    .format(Some(SchemaFormat::KnownFormat(KnownFormat::Int32)))
                    .description(Some("Id of credential"))
                    .default(Some(json!(1i32))),
            ),
        );

        assert!(matches!(array.schema_type, SchemaType::Type(Type::Array)));
    }

    #[test]
    fn test_array_builder() {
        let array: Array = ArrayBuilder::new()
            .items(
                ObjectBuilder::new().property(
                    "id",
                    ObjectBuilder::new()
                        .schema_type(Type::Integer)
                        .format(Some(SchemaFormat::KnownFormat(KnownFormat::Int32)))
                        .description(Some("Id of credential"))
                        .default(Some(json!(1i32))),
                ),
            )
            .build();

        assert!(matches!(array.schema_type, SchemaType::Type(Type::Array)));
    }

    #[test]
    fn reserialize_deserialized_schema_components() {
        let components = ComponentsBuilder::new()
            .schemas_from_iter(vec![(
                "Comp",
                Schema::from(
                    ObjectBuilder::new()
                        .property("name", ObjectBuilder::new().schema_type(Type::String))
                        .required("name"),
                ),
            )])
            .responses_from_iter(vec![(
                "200",
                ResponseBuilder::new().description("Okay").build(),
            )])
            .security_scheme(
                "TLS",
                SecurityScheme::MutualTls {
                    description: None,
                    extensions: None,
                },
            )
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
            .property("name", ObjectBuilder::new().schema_type(Type::String))
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
        let prop = ObjectBuilder::new().schema_type(Type::String).build();

        let serialized_components = serde_json::to_string(&prop).unwrap();
        let deserialized_components: Object =
            serde_json::from_str(serialized_components.as_str()).unwrap();

        assert_eq!(
            serialized_components,
            serde_json::to_string(&deserialized_components).unwrap()
        )
    }

    #[test]
    fn serialize_deserialize_array_within_ref_or_t_object_builder() {
        let ref_or_schema = RefOr::T(Schema::Object(
            ObjectBuilder::new()
                .property(
                    "test",
                    RefOr::T(Schema::Array(
                        ArrayBuilder::new()
                            .items(RefOr::T(Schema::Object(
                                ObjectBuilder::new()
                                    .property("element", RefOr::Ref(Ref::new("#/test")))
                                    .build(),
                            )))
                            .build(),
                    )),
                )
                .build(),
        ));

        let json_str = serde_json::to_string(&ref_or_schema).expect("");
        println!("----------------------------");
        println!("{json_str}");

        let deserialized: RefOr<Schema> = serde_json::from_str(&json_str).expect("");

        let json_de_str = serde_json::to_string(&deserialized).expect("");
        println!("----------------------------");
        println!("{json_de_str}");

        assert_eq!(json_str, json_de_str);
    }

    #[test]
    fn serialize_deserialize_one_of_within_ref_or_t_object_builder() {
        let ref_or_schema = RefOr::T(Schema::Object(
            ObjectBuilder::new()
                .property(
                    "test",
                    RefOr::T(Schema::OneOf(
                        OneOfBuilder::new()
                            .item(Schema::Array(
                                ArrayBuilder::new()
                                    .items(RefOr::T(Schema::Object(
                                        ObjectBuilder::new()
                                            .property("element", RefOr::Ref(Ref::new("#/test")))
                                            .build(),
                                    )))
                                    .build(),
                            ))
                            .item(Schema::Array(
                                ArrayBuilder::new()
                                    .items(RefOr::T(Schema::Object(
                                        ObjectBuilder::new()
                                            .property("foobar", RefOr::Ref(Ref::new("#/foobar")))
                                            .build(),
                                    )))
                                    .build(),
                            ))
                            .build(),
                    )),
                )
                .build(),
        ));

        let json_str = serde_json::to_string(&ref_or_schema).expect("");
        println!("----------------------------");
        println!("{json_str}");

        let deserialized: RefOr<Schema> = serde_json::from_str(&json_str).expect("");

        let json_de_str = serde_json::to_string(&deserialized).expect("");
        println!("----------------------------");
        println!("{json_de_str}");

        assert_eq!(json_str, json_de_str);
    }

    #[test]
    fn serialize_deserialize_all_of_of_within_ref_or_t_object_builder() {
        let ref_or_schema = RefOr::T(Schema::Object(
            ObjectBuilder::new()
                .property(
                    "test",
                    RefOr::T(Schema::AllOf(
                        AllOfBuilder::new()
                            .item(Schema::Array(
                                ArrayBuilder::new()
                                    .items(RefOr::T(Schema::Object(
                                        ObjectBuilder::new()
                                            .property("element", RefOr::Ref(Ref::new("#/test")))
                                            .build(),
                                    )))
                                    .build(),
                            ))
                            .item(RefOr::T(Schema::Object(
                                ObjectBuilder::new()
                                    .property("foobar", RefOr::Ref(Ref::new("#/foobar")))
                                    .build(),
                            )))
                            .build(),
                    )),
                )
                .build(),
        ));

        let json_str = serde_json::to_string(&ref_or_schema).expect("");
        println!("----------------------------");
        println!("{json_str}");

        let deserialized: RefOr<Schema> = serde_json::from_str(&json_str).expect("");

        let json_de_str = serde_json::to_string(&deserialized).expect("");
        println!("----------------------------");
        println!("{json_de_str}");

        assert_eq!(json_str, json_de_str);
    }

    #[test]
    fn deserialize_reserialize_one_of_default_type() {
        let a = OneOfBuilder::new()
            .item(Schema::Array(
                ArrayBuilder::new()
                    .items(RefOr::T(Schema::Object(
                        ObjectBuilder::new()
                            .property("element", RefOr::Ref(Ref::new("#/test")))
                            .build(),
                    )))
                    .build(),
            ))
            .item(Schema::Array(
                ArrayBuilder::new()
                    .items(RefOr::T(Schema::Object(
                        ObjectBuilder::new()
                            .property("foobar", RefOr::Ref(Ref::new("#/foobar")))
                            .build(),
                    )))
                    .build(),
            ))
            .build();

        let serialized_json = serde_json::to_string(&a).expect("should serialize to json");
        let b: OneOf = serde_json::from_str(&serialized_json).expect("should deserialize OneOf");
        let reserialized_json = serde_json::to_string(&b).expect("reserialized json");

        println!("{serialized_json}");
        println!("{reserialized_json}",);
        assert_eq!(serialized_json, reserialized_json);
    }

    #[test]
    fn serialize_deserialize_any_of_of_within_ref_or_t_object_builder() {
        let ref_or_schema = RefOr::T(Schema::Object(
            ObjectBuilder::new()
                .property(
                    "test",
                    RefOr::T(Schema::AnyOf(
                        AnyOfBuilder::new()
                            .item(Schema::Array(
                                ArrayBuilder::new()
                                    .items(RefOr::T(Schema::Object(
                                        ObjectBuilder::new()
                                            .property("element", RefOr::Ref(Ref::new("#/test")))
                                            .build(),
                                    )))
                                    .build(),
                            ))
                            .item(RefOr::T(Schema::Object(
                                ObjectBuilder::new()
                                    .property("foobar", RefOr::Ref(Ref::new("#/foobar")))
                                    .build(),
                            )))
                            .build(),
                    )),
                )
                .build(),
        ));

        let json_str = serde_json::to_string(&ref_or_schema).expect("");
        println!("----------------------------");
        println!("{json_str}");

        let deserialized: RefOr<Schema> = serde_json::from_str(&json_str).expect("");

        let json_de_str = serde_json::to_string(&deserialized).expect("");
        println!("----------------------------");
        println!("{json_de_str}");
        assert!(json_str.contains("\"anyOf\""));
        assert_eq!(json_str, json_de_str);
    }

    #[test]
    fn serialize_deserialize_schema_array_ref_or_t() {
        let ref_or_schema = RefOr::T(Schema::Array(
            ArrayBuilder::new()
                .items(RefOr::T(Schema::Object(
                    ObjectBuilder::new()
                        .property("element", RefOr::Ref(Ref::new("#/test")))
                        .build(),
                )))
                .build(),
        ));

        let json_str = serde_json::to_string(&ref_or_schema).expect("");
        println!("----------------------------");
        println!("{json_str}");

        let deserialized: RefOr<Schema> = serde_json::from_str(&json_str).expect("");

        let json_de_str = serde_json::to_string(&deserialized).expect("");
        println!("----------------------------");
        println!("{json_de_str}");

        assert_eq!(json_str, json_de_str);
    }

    #[test]
    fn serialize_deserialize_schema_array_builder() {
        let ref_or_schema = ArrayBuilder::new()
            .items(RefOr::T(Schema::Object(
                ObjectBuilder::new()
                    .property("element", RefOr::Ref(Ref::new("#/test")))
                    .build(),
            )))
            .build();

        let json_str = serde_json::to_string(&ref_or_schema).expect("");
        println!("----------------------------");
        println!("{json_str}");

        let deserialized: RefOr<Schema> = serde_json::from_str(&json_str).expect("");

        let json_de_str = serde_json::to_string(&deserialized).expect("");
        println!("----------------------------");
        println!("{json_de_str}");

        assert_eq!(json_str, json_de_str);
    }

    #[test]
    fn serialize_deserialize_schema_with_additional_properties() {
        let schema = Schema::Object(
            ObjectBuilder::new()
                .property(
                    "map",
                    ObjectBuilder::new()
                        .additional_properties(Some(AdditionalProperties::FreeForm(true))),
                )
                .build(),
        );

        let json_str = serde_json::to_string(&schema).unwrap();
        println!("----------------------------");
        println!("{json_str}");

        let deserialized: RefOr<Schema> = serde_json::from_str(&json_str).unwrap();

        let json_de_str = serde_json::to_string(&deserialized).unwrap();
        println!("----------------------------");
        println!("{json_de_str}");

        assert_eq!(json_str, json_de_str);
    }

    #[test]
    fn serialize_deserialize_schema_with_additional_properties_object() {
        let schema = Schema::Object(
            ObjectBuilder::new()
                .property(
                    "map",
                    ObjectBuilder::new().additional_properties(Some(
                        ObjectBuilder::new().property("name", Object::with_type(Type::String)),
                    )),
                )
                .build(),
        );

        let json_str = serde_json::to_string(&schema).unwrap();
        println!("----------------------------");
        println!("{json_str}");

        let deserialized: RefOr<Schema> = serde_json::from_str(&json_str).unwrap();

        let json_de_str = serde_json::to_string(&deserialized).unwrap();
        println!("----------------------------");
        println!("{json_de_str}");

        assert_eq!(json_str, json_de_str);
    }

    #[test]
    fn serialize_discriminator_with_mapping() {
        let mut discriminator = Discriminator::new("type");
        discriminator.mapping = [("int".to_string(), "#/components/schemas/MyInt".to_string())]
            .into_iter()
            .collect::<BTreeMap<_, _>>();
        let one_of = OneOfBuilder::new()
            .item(Ref::from_schema_name("MyInt"))
            .discriminator(Some(discriminator))
            .build();
        assert_json_snapshot!(one_of, @r##"
        {
          "oneOf": [
            {
              "$ref": "#/components/schemas/MyInt"
            }
          ],
          "discriminator": {
            "propertyName": "type",
            "mapping": {
              "int": "#/components/schemas/MyInt"
            }
          }
        }
        "##);
    }

    #[test]
    fn serialize_deserialize_object_with_multiple_schema_types() {
        let object = ObjectBuilder::new()
            .schema_type(SchemaType::from_iter([Type::Object, Type::Null]))
            .build();

        let json_str = serde_json::to_string(&object).unwrap();
        println!("----------------------------");
        println!("{json_str}");

        let deserialized: Object = serde_json::from_str(&json_str).unwrap();

        let json_de_str = serde_json::to_string(&deserialized).unwrap();
        println!("----------------------------");
        println!("{json_de_str}");

        assert_eq!(json_str, json_de_str);
    }

    #[test]
    fn object_with_extensions() {
        let expected = json!("value");
        let extensions = extensions::ExtensionsBuilder::new()
            .add("x-some-extension", expected.clone())
            .build();
        let json_value = ObjectBuilder::new().extensions(Some(extensions)).build();

        let value = serde_json::to_value(&json_value).unwrap();
        assert_eq!(value.get("x-some-extension"), Some(&expected));
    }

    #[test]
    fn array_with_extensions() {
        let expected = json!("value");
        let extensions = extensions::ExtensionsBuilder::new()
            .add("x-some-extension", expected.clone())
            .build();
        let json_value = ArrayBuilder::new().extensions(Some(extensions)).build();

        let value = serde_json::to_value(&json_value).unwrap();
        assert_eq!(value.get("x-some-extension"), Some(&expected));
    }

    #[test]
    fn oneof_with_extensions() {
        let expected = json!("value");
        let extensions = extensions::ExtensionsBuilder::new()
            .add("x-some-extension", expected.clone())
            .build();
        let json_value = OneOfBuilder::new().extensions(Some(extensions)).build();

        let value = serde_json::to_value(&json_value).unwrap();
        assert_eq!(value.get("x-some-extension"), Some(&expected));
    }

    #[test]
    fn allof_with_extensions() {
        let expected = json!("value");
        let extensions = extensions::ExtensionsBuilder::new()
            .add("x-some-extension", expected.clone())
            .build();
        let json_value = AllOfBuilder::new().extensions(Some(extensions)).build();

        let value = serde_json::to_value(&json_value).unwrap();
        assert_eq!(value.get("x-some-extension"), Some(&expected));
    }

    #[test]
    fn anyof_with_extensions() {
        let expected = json!("value");
        let extensions = extensions::ExtensionsBuilder::new()
            .add("x-some-extension", expected.clone())
            .build();
        let json_value = AnyOfBuilder::new().extensions(Some(extensions)).build();

        let value = serde_json::to_value(&json_value).unwrap();
        assert_eq!(value.get("x-some-extension"), Some(&expected));
    }
}
