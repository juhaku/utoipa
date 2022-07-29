//! Rust implementation of Openapi Spec V3.

use serde::{de::Visitor, Deserialize, Serialize, Serializer};

pub use self::{
    content::{Content, ContentBuilder},
    external_docs::ExternalDocs,
    header::{Header, HeaderBuilder},
    info::{Contact, ContactBuilder, Info, InfoBuilder, License, LicenseBuilder},
    path::{PathItem, PathItemType, Paths, PathsBuilder},
    response::{Response, ResponseBuilder, Responses, ResponsesBuilder},
    schema::{
        Array, ArrayBuilder, Component, ComponentFormat, ComponentType, Components,
        ComponentsBuilder, Object, ObjectBuilder, OneOf, OneOfBuilder, Property, PropertyBuilder,
        Ref, ToArray,
    },
    security::SecurityRequirement,
    server::{Server, ServerBuilder, ServerVariable, ServerVariableBuilder},
    tag::Tag,
};

pub mod content;
pub mod encoding;
pub mod external_docs;
pub mod header;
pub mod info;
pub mod path;
pub mod request_body;
pub mod response;
pub mod schema;
pub mod security;
pub mod server;
pub mod tag;
pub mod xml;

builder! {
    /// # Examples
    ///
    /// Create [`OpenApi`] using [`OpenApiBuilder`].
    /// ```rust
    /// # use utoipa::openapi::{Info, Paths, Components, OpenApiBuilder};
    /// let openapi = OpenApiBuilder::new()
    ///      .info(Info::new("My api", "1.0.0"))
    ///      .paths(Paths::new())
    ///      .components(Some(
    ///          Components::new()
    ///      ))
    ///      .build();
    /// ```
    OpenApiBuilder;

    /// Root object of the OpenAPI document.
    ///
    /// You can use [`OpenApi::new`] function to construct a new [`OpenApi`] instance and then
    /// use the fields with mutable access to modify them. This is quite tedious if you are not simply
    /// just changing one thing thus you can also use the [`OpenApiBuilder::new`] to use builder to
    /// construct a new [`OpenApi`] object.
    ///
    /// See more details at <https://spec.openapis.org/oas/latest.html#openapi-object>.
    #[non_exhaustive]
    #[derive(Serialize, Deserialize, Default, Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[serde(rename_all = "camelCase")]
    pub struct OpenApi {
        /// OpenAPI document version.
        pub openapi: OpenApiVersion,

        /// Provides metadata about the API.
        ///
        /// See more details at <https://spec.openapis.org/oas/latest.html#info-object>.
        pub info: Info,

        /// Optional list of servers that provides the connectivity information to target servers.
        ///
        /// This is implicitly one server with `url` set to `/`.
        ///
        /// See more details at <https://spec.openapis.org/oas/latest.html#server-object>.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub servers: Option<Vec<Server>>,

        /// Available paths and operations for the API.
        ///
        /// See more details at <https://spec.openapis.org/oas/latest.html#paths-object>.
        #[serde(flatten)]
        pub paths: Paths,

        /// Holds various reusable schemas for the OpenAPI document.
        ///
        /// Few of these elements are security schemas and object schemas.
        ///
        /// See more details at <https://spec.openapis.org/oas/latest.html#components-object>.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub components: Option<Components>,

        /// Declaration of global security mechanisms that can be used across the API. The individual operations
        /// can override the declarations. You can use `SecurityRequirement::default()` if you wish to make security
        /// optional by adding it to the list of securities.
        ///
        /// See more details at <https://spec.openapis.org/oas/latest.html#security-requirement-object>.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub security: Option<Vec<SecurityRequirement>>,

        /// Optional list of tags can be used to add additional documentation to matching tags of operations.
        ///
        /// See more details at <https://spec.openapis.org/oas/latest.html#tag-object>.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub tags: Option<Vec<Tag>>,

        /// Optional global additional documentation reference.
        ///
        /// See more details at <https://spec.openapis.org/oas/latest.html#external-documentation-object>.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub external_docs: Option<ExternalDocs>,
    }
}

impl OpenApi {
    /// Construct a new [`OpenApi`] object.
    ///
    /// Function accepts two arguments one which is [`Info`] metadata of the API; two which is [`Paths`]
    /// containing operations for the API.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use utoipa::openapi::{Info, Paths, OpenApi};
    /// #
    /// let openapi = OpenApi::new(Info::new("pet api", "0.1.0"), Paths::new());
    /// ```
    pub fn new<P: Into<Paths>>(info: Info, paths: P) -> Self {
        Self {
            info,
            paths: paths.into(),
            ..Default::default()
        }
    }

    /// Converts this [`OpenApi`] to JSON String. This method essentially calls [`serde_json::to_string`] method. [^json]
    ///
    /// [^json]: **json** feature is needed.
    #[cfg(feature = "json")]
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Converts this [`OpenApi`] to pretty JSON String. This method essentially calls [`serde_json::to_string_pretty`] method. [^json]
    ///
    /// [^json]: **json** feature is needed.
    #[cfg(feature = "json")]
    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Converts this [`OpenApi`] to YAML String. This method essentially calls [`serde_yaml::to_string`] method. [^yaml]
    ///
    /// [^yaml]: **yaml** feature is needed.
    #[cfg(feature = "yaml")]
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }
}

impl OpenApiBuilder {
    /// Add [`Info`] metadata of the API.
    pub fn info(mut self, info: Info) -> Self {
        set_value!(self info info)
    }

    /// Add iterator of [`Server`]s to configure target servers.
    pub fn servers<I: IntoIterator<Item = Server>>(mut self, servers: Option<I>) -> Self {
        set_value!(self servers servers.map(|servers| servers.into_iter().collect()))
    }

    /// Add [`Paths`] to configure operations and endpoints of the API.
    pub fn paths<P: Into<Paths>>(mut self, paths: P) -> Self {
        set_value!(self paths paths.into())
    }

    /// Add [`Components`] to configure reusable schemas.
    pub fn components(mut self, components: Option<Components>) -> Self {
        set_value!(self components components)
    }

    /// Add iterator of [`SecurityRequirement`]s that are globally available for all operations.
    pub fn security<I: IntoIterator<Item = SecurityRequirement>>(
        mut self,
        security: Option<I>,
    ) -> Self {
        set_value!(self security security.map(|security| security.into_iter().collect()))
    }

    /// Add iterator of [`Tag`]s to add additional documentation for **operations** tags.
    pub fn tags<I: IntoIterator<Item = Tag>>(mut self, tags: Option<I>) -> Self {
        set_value!(self tags tags.map(|tags| tags.into_iter().collect()))
    }

    /// Add [`ExternalDocs`] for referring additional documentation.
    pub fn external_docs(mut self, external_docs: Option<ExternalDocs>) -> Self {
        set_value!(self external_docs external_docs)
    }
}

/// Represents available [OpenAPI versions][version].
///
/// [version]: <https://spec.openapis.org/oas/latest.html#versions>
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum OpenApiVersion {
    /// Will serialize to `3.0.3` the latest from 3.0 serde.
    #[serde(rename = "3.0.3")]
    Version3,
}

impl Default for OpenApiVersion {
    fn default() -> Self {
        Self::Version3
    }
}

/// Value used to indicate whether reusable schema, parameter or operation is deprecated.
///
/// The value will serialize to boolean.
#[derive(PartialEq, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Deprecated {
    True,
    False,
}

impl Serialize for Deprecated {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(matches!(self, Self::True))
    }
}

impl<'de> Deserialize<'de> for Deprecated {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BoolVisitor;
        impl<'de> Visitor<'de> for BoolVisitor {
            type Value = Deprecated;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a bool true or false")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    true => Ok(Deprecated::True),
                    false => Ok(Deprecated::False),
                }
            }
        }
        deserializer.deserialize_bool(BoolVisitor)
    }
}

impl Default for Deprecated {
    fn default() -> Self {
        Deprecated::False
    }
}

/// Value used to indicate whether parameter or property is required.
///
/// The value will serialize to boolean.
#[derive(PartialEq, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Required {
    True,
    False,
}

impl Serialize for Required {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(matches!(self, Self::True))
    }
}

impl<'de> Deserialize<'de> for Required {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BoolVisitor;
        impl<'de> Visitor<'de> for BoolVisitor {
            type Value = Required;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a bool true or false")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    true => Ok(Required::True),
                    false => Ok(Required::False),
                }
            }
        }
        deserializer.deserialize_bool(BoolVisitor)
    }
}

impl Default for Required {
    fn default() -> Self {
        Required::False
    }
}

macro_rules! build_fn {
    ( $vis:vis $name:ident $( $field:ident ),+ ) => {
        #[doc = concat!("Constructs a new [`", stringify!($name),"`] taking all fields values from this object.")]
        $vis fn build(self) -> $name {
            $name {
                $(
                    $field: self.$field,
                )*
            }
        }
    };
}
pub(crate) use build_fn;

macro_rules! set_value {
    ( $self:ident $field:ident $value:expr ) => {{
        $self.$field = $value;

        $self
    }};
}
pub(crate) use set_value;

macro_rules! new {
    ( $vis:vis $name:ident ) => {
        #[doc = concat!("Constructs a new [`", stringify!($name),"`].")]
        $vis fn new() -> $name {
            $name {
                ..Default::default()
            }
        }
    };
}
pub(crate) use new;

macro_rules! from {
    ( $name:ident $to:ident $( $field:ident ),+ ) => {
        impl From<$name> for $to {
            fn from(value: $name) -> Self {
                Self {
                    $( $field: value.$field, )*
                }
            }
        }

        impl From<$to> for $name {
            fn from(value: $to) -> Self {
                value.build()
            }
        }
    };
}
pub(crate) use from;

macro_rules! builder {
    ( $( #[$builder_meta:meta] )* $builder_name:ident; $(#[$meta:meta])* $vis:vis $key:ident $name:ident $( $tt:tt )* ) => {
        builder!( @type_impl $( #[$meta] )* $vis $key $name $( $tt )* );
        builder!( @builder_impl $( #[$builder_meta] )* $builder_name $( #[$meta] )* $vis $key $name $( $tt )* );
    };

    ( @type_impl $( #[$meta:meta] )* $vis:vis $key:ident $name:ident
        { $( $( #[$field_meta:meta] )* $field_vis:vis $field:ident: $field_ty:ty, )* }
    ) => {

        $( #[$meta] )*
        $vis $key $name {
            $( $( #[$field_meta] )* $field_vis $field: $field_ty, )*
        }
    };

    ( @builder_impl $( #[$builder_meta:meta] )* $builder_name:ident $( #[$meta:meta] )* $vis:vis $key:ident $name:ident
        { $( $( #[$field_meta:meta] )* $field_vis:vis $field:ident: $field_ty:ty, )* }
    ) => {
        #[doc = concat!("Builder for [`", stringify!($name),
            "`] with chainable configuration methods to create a new [`", stringify!($name) , "`].")]
        $( #[$builder_meta] )*
        #[cfg_attr(feature = "debug", derive(Debug))]
        $vis $key $builder_name {
            $( $field: $field_ty, )*
        }

        impl Default for $builder_name {
            fn default() -> Self {
                let meta_default: $name = $name::default();
                Self {
                    $( $field: meta_default.$field, )*
                }
            }
        }

        impl $builder_name {
            new!($vis $builder_name);
            build_fn!($vis $name $( $field ),* );
        }

        from!($name $builder_name $( $field ),* );
    };
}
pub(crate) use builder;

#[cfg(test)]
#[cfg(feature = "serde_json")]
mod tests {
    use crate::openapi::{
        info::InfoBuilder,
        path::{OperationBuilder, PathsBuilder},
    };

    use super::{response::Response, *};

    #[test]
    fn serialize_deserialize_openapi_version_success() -> Result<(), serde_json::Error> {
        assert_eq!(serde_json::to_value(&OpenApiVersion::Version3)?, "3.0.3");
        Ok(())
    }

    #[test]
    fn serialize_openapi_json_minimal_success() -> Result<(), serde_json::Error> {
        let raw_json = include_str!("openapi/testdata/expected_openapi_minimal.json");
        let openapi = OpenApi::new(
            InfoBuilder::new()
                .title("My api")
                .version("1.0.0")
                .description(Some("My api description"))
                .license(Some(
                    LicenseBuilder::new()
                        .name("MIT")
                        .url(Some("http://mit.licence"))
                        .build(),
                ))
                .build(),
            Paths::new(),
        );
        let serialized = serde_json::to_string_pretty(&openapi)?;

        assert_eq!(
            serialized, raw_json,
            "expected serialized json to match raw: \nserialized: \n{} \nraw: \n{}",
            serialized, raw_json
        );
        Ok(())
    }

    #[test]
    fn serialize_openapi_json_with_paths_success() -> Result<(), serde_json::Error> {
        let openapi = OpenApi::new(
            Info::new("My big api", "1.1.0"),
            PathsBuilder::new()
                .path(
                    "/api/v1/users",
                    PathItem::new(
                        PathItemType::Get,
                        OperationBuilder::new().response("200", Response::new("Get users list")),
                    ),
                )
                .path(
                    "/api/v1/users",
                    PathItem::new(
                        PathItemType::Post,
                        OperationBuilder::new().response("200", Response::new("Post new user")),
                    ),
                )
                .path(
                    "/api/v1/users/{id}",
                    PathItem::new(
                        PathItemType::Get,
                        OperationBuilder::new().response("200", Response::new("Get user by id")),
                    ),
                ),
        );

        let serialized = serde_json::to_string_pretty(&openapi)?;
        let expected = include_str!("./openapi/testdata/expected_openapi_with_paths.json");

        assert_eq!(
            serialized, expected,
            "expected serialized json to match raw: \nserialized: \n{} \nraw: \n{}",
            serialized, expected
        );
        Ok(())
    }
}
