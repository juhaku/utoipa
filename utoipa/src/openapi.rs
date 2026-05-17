//! Rust implementation of Openapi Spec V3.2.

use serde::{
    de::{Error, Expected, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt::Formatter;

use self::path::PathsMap;
pub use self::{
    content::{Content, ContentBuilder},
    external_docs::ExternalDocs,
    header::{Header, HeaderBuilder},
    info::{Contact, ContactBuilder, Info, InfoBuilder, License, LicenseBuilder},
    path::{HttpMethod, PathItem, Paths, PathsBuilder},
    response::{Response, ResponseBuilder, Responses, ResponsesBuilder},
    schema::{
        AllOf, AllOfBuilder, Array, ArrayBuilder, Components, ComponentsBuilder, Discriminator,
        KnownFormat, Object, ObjectBuilder, OneOf, OneOfBuilder, Ref, Schema, SchemaFormat,
        ToArray, Type,
    },
    security::SecurityRequirement,
    server::{Server, ServerBuilder, ServerVariable, ServerVariableBuilder},
    tag::Tag,
};

pub mod content;
pub mod encoding;
pub mod example;
pub mod extensions;
pub mod external_docs;
pub mod header;
pub mod info;
pub mod link;
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
    #[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
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
        pub paths: Paths,

        /// Incoming requests that may be initiated by the API provider independently of an
        /// incoming API request.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub webhooks: Option<Paths>,

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

        /// The default JSON Schema dialect used by Schema Objects in this OpenAPI document.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub json_schema_dialect: Option<String>,

        /// Schema keyword can be used to override default _`$schema`_ dialect which is by default
        /// “<https://spec.openapis.org/oas/3.1/dialect/base>”.
        ///
        /// All the references and individual files could use their own schema dialect.
        #[serde(rename = "$schema", default, skip_serializing_if = "String::is_empty")]
        pub schema: String,

        /// URI identifying this OpenAPI document.
        #[serde(rename = "$self", skip_serializing_if = "Option::is_none")]
        pub self_url: Option<String>,

        /// Optional extensions "x-something".
        #[serde(skip_serializing_if = "Option::is_none", flatten)]
        pub extensions: Option<Extensions>,
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

    /// Converts this [`OpenApi`] to JSON String. This method essentially calls [`serde_json::to_string`] method.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Converts this [`OpenApi`] to pretty JSON String. This method essentially calls [`serde_json::to_string_pretty`] method.
    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Converts this [`OpenApi`] to YAML String. This method essentially calls [`serde_norway::to_string`] method.
    #[cfg(feature = "yaml")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "yaml")))]
    pub fn to_yaml(&self) -> Result<String, serde_norway::Error> {
        serde_norway::to_string(self)
    }

    /// Merge `other` [`OpenApi`] moving `self` and returning combined [`OpenApi`].
    ///
    /// In functionality wise this is exactly same as calling [`OpenApi::merge`] but but provides
    /// leaner API for chaining method calls.
    pub fn merge_from(mut self, other: OpenApi) -> OpenApi {
        self.merge(other);
        self
    }

    /// Merge `other` [`OpenApi`] consuming it and resuming it's content.
    ///
    /// Merge function will take all `self` nonexistent _`servers`, `paths`, `schemas`, `responses`,
    /// `security_schemes`, `security_requirements` and `tags`_ from _`other`_ [`OpenApi`].
    ///
    /// This function performs a shallow comparison for `paths`, `schemas`, `responses` and
    /// `security schemes` which means that only _`name`_ and _`path`_ is used for comparison. When
    /// match occurs the whole item will be ignored from merged results. Only items not
    /// found will be appended to `self`.
    ///
    /// For _`servers`_, _`tags`_ and _`security_requirements`_ the whole item will be used for
    /// comparison. Items not found from `self` will be appended to `self`.
    ///
    /// **Note!** `info`, `openapi`, `external_docs` and `schema` will not be merged.
    pub fn merge(&mut self, mut other: OpenApi) {
        if let Some(other_servers) = &mut other.servers {
            let servers = self.servers.get_or_insert(Vec::new());
            other_servers.retain(|server| !servers.contains(server));
            servers.append(other_servers);
        }

        if !other.paths.paths.is_empty() {
            self.paths.merge(other.paths);
        };

        if let Some(other_components) = &mut other.components {
            let components = self.components.get_or_insert(Components::default());

            other_components
                .schemas
                .retain(|name, _| !components.schemas.contains_key(name));
            components.schemas.append(&mut other_components.schemas);

            other_components
                .responses
                .retain(|name, _| !components.responses.contains_key(name));
            components.responses.append(&mut other_components.responses);

            other_components
                .security_schemes
                .retain(|name, _| !components.security_schemes.contains_key(name));
            components
                .security_schemes
                .append(&mut other_components.security_schemes);
        }

        if let Some(other_security) = &mut other.security {
            let security = self.security.get_or_insert(Vec::new());
            other_security.retain(|requirement| !security.contains(requirement));
            security.append(other_security);
        }

        if let Some(other_tags) = &mut other.tags {
            let tags = self.tags.get_or_insert(Vec::new());
            other_tags.retain(|tag| !tags.contains(tag));
            tags.append(other_tags);
        }
    }

    /// Nest `other` [`OpenApi`] to this [`OpenApi`].
    ///
    /// Nesting performs custom [`OpenApi::merge`] where `other` [`OpenApi`] paths are prepended with given
    /// `path` and then appended to _`paths`_ of this [`OpenApi`] instance. Rest of the  `other`
    /// [`OpenApi`] instance is merged to this [`OpenApi`] with [`OpenApi::merge_from`] method.
    ///
    /// **If multiple** APIs are being nested with same `path` only the **last** one will be retained.
    ///
    /// Method accepts two arguments, first is the path to prepend .e.g. _`/user`_. Second argument
    /// is the [`OpenApi`] to prepend paths for.
    ///
    /// # Examples
    ///
    /// _**Merge `user_api` to `api` nesting `user_api` paths under `/api/v1/user`**_
    /// ```rust
    ///  # use utoipa::openapi::{OpenApi, OpenApiBuilder};
    ///  # use utoipa::openapi::path::{PathsBuilder, PathItemBuilder, PathItem,
    ///  # HttpMethod, OperationBuilder};
    ///  let api = OpenApiBuilder::new()
    ///      .paths(
    ///          PathsBuilder::new().path(
    ///              "/api/v1/status",
    ///              PathItem::new(
    ///                  HttpMethod::Get,
    ///                  OperationBuilder::new()
    ///                      .description(Some("Get status"))
    ///                      .build(),
    ///              ),
    ///          ),
    ///      )
    ///      .build();
    ///  let user_api = OpenApiBuilder::new()
    ///     .paths(
    ///         PathsBuilder::new().path(
    ///             "/",
    ///             PathItem::new(HttpMethod::Post, OperationBuilder::new().build()),
    ///         )
    ///     )
    ///     .build();
    ///  let nested = api.nest("/api/v1/user", user_api);
    /// ```
    pub fn nest<P: Into<String>, O: Into<OpenApi>>(self, path: P, other: O) -> Self {
        self.nest_with_path_composer(path, other, |base, path| format!("{base}{path}"))
    }

    /// Nest `other` [`OpenApi`] with custom path composer.
    ///
    /// In most cases you should use [`OpenApi::nest`] instead.
    /// Only use this method if you need custom path composition for a specific use case.
    ///
    /// `composer` is a function that takes two strings, the base path and the path to nest, and returns the composed path for the API Specification.
    pub fn nest_with_path_composer<
        P: Into<String>,
        O: Into<OpenApi>,
        F: Fn(&str, &str) -> String,
    >(
        mut self,
        path: P,
        other: O,
        composer: F,
    ) -> Self {
        let path: String = path.into();
        let mut other_api: OpenApi = other.into();

        let nested_paths = other_api
            .paths
            .paths
            .into_iter()
            .map(|(item_path, item)| {
                let path = composer(&path, &item_path);
                (path, item)
            })
            .collect::<PathsMap<_, _>>();

        self.paths.paths.extend(nested_paths);

        // paths are already merged, thus we can ignore them
        other_api.paths.paths = PathsMap::new();
        self.merge_from(other_api)
    }
}

impl OpenApiBuilder {
    /// Add [`Info`] metadata of the API.
    pub fn info<I: Into<Info>>(mut self, info: I) -> Self {
        set_value!(self info info.into())
    }

    /// Add iterator of [`Server`]s to configure target servers.
    pub fn servers<I: IntoIterator<Item = Server>>(mut self, servers: Option<I>) -> Self {
        set_value!(self servers servers.map(|servers| servers.into_iter().collect()))
    }

    /// Add [`Paths`] to configure operations and endpoints of the API.
    pub fn paths<P: Into<Paths>>(mut self, paths: P) -> Self {
        set_value!(self paths paths.into())
    }

    /// Add [`Paths`] to describe incoming requests that may be initiated by the API provider.
    pub fn webhooks<P: Into<Paths>>(mut self, webhooks: Option<P>) -> Self {
        set_value!(self webhooks webhooks.map(Into::into))
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

    /// Override default `$schema` dialect for the Open API doc.
    ///
    /// # Examples
    ///
    /// _**Override default schema dialect.**_
    /// ```rust
    /// # use utoipa::openapi::OpenApiBuilder;
    /// let _ = OpenApiBuilder::new()
    ///     .schema("http://json-schema.org/draft-07/schema#")
    ///     .build();
    /// ```
    pub fn schema<S: Into<String>>(mut self, schema: S) -> Self {
        set_value!(self schema schema.into())
    }

    /// Add or change the default JSON Schema dialect for Schema Objects.
    pub fn json_schema_dialect<S: Into<String>>(mut self, json_schema_dialect: Option<S>) -> Self {
        set_value!(self json_schema_dialect json_schema_dialect.map(Into::into))
    }

    /// Add or change the URI identifying this OpenAPI document.
    pub fn self_url<S: Into<String>>(mut self, self_url: Option<S>) -> Self {
        set_value!(self self_url self_url.map(Into::into))
    }
}

/// Represents available [OpenAPI versions][version].
///
/// [version]: <https://spec.openapis.org/oas/latest.html#versions>
#[derive(Serialize, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum OpenApiVersion {
    /// Will serialize to `3.1.0`.
    #[deprecated(note = "OpenAPI 3.1 is superseded by 3.2. Use Version32.")]
    #[serde(rename = "3.1.0")]
    Version31,
    /// Will serialize to `3.2.0` the latest supported OpenAPI version.
    #[serde(rename = "3.2.0")]
    #[default]
    Version32,
}

impl<'de> Deserialize<'de> for OpenApiVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VersionVisitor;

        impl<'v> Visitor<'v> for VersionVisitor {
            type Value = OpenApiVersion;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a version string in 3.1.x or 3.2.x format")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_string(v.to_string())
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let version = v
                    .split('.')
                    .flat_map(|digit| digit.parse::<i8>())
                    .collect::<Vec<_>>();

                if version.len() == 3 && version.first() == Some(&3) && version.get(1) == Some(&1) {
                    #[allow(deprecated)]
                    Ok(OpenApiVersion::Version31)
                } else if version.len() == 3
                    && version.first() == Some(&3)
                    && version.get(1) == Some(&2)
                {
                    Ok(OpenApiVersion::Version32)
                } else {
                    let expected: &dyn Expected = &"3.1.0 or 3.2.0";
                    Err(Error::invalid_value(
                        serde::de::Unexpected::Str(&v),
                        expected,
                    ))
                }
            }
        }

        deserializer.deserialize_string(VersionVisitor)
    }
}

/// Value used to indicate whether reusable schema, parameter or operation is deprecated.
///
/// The value will serialize to boolean.
#[derive(PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[allow(missing_docs)]
pub enum Deprecated {
    True,
    #[default]
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

/// Value used to indicate whether parameter or property is required.
///
/// The value will serialize to boolean.
#[derive(PartialEq, Eq, Clone, Default)]
#[allow(missing_docs)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Required {
    True,
    #[default]
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

/// A [`Ref`] or some other type `T`.
///
/// Typically used in combination with [`Components`] and is an union type between [`Ref`] and any
/// other given type such as [`Schema`] or [`Response`].
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(untagged)]
pub enum RefOr<T> {
    /// Represents [`Ref`] reference to another OpenAPI object instance. e.g.
    /// `$ref: #/components/schemas/Hello`
    Ref(Ref),
    /// Represents any value that can be added to the [`struct@Components`] e.g. [`enum@Schema`]
    /// or [`struct@Response`].
    T(T),
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
        builder!( @type_impl $builder_name $( #[$meta] )* $vis $key $name $( $tt )* );
        builder!( @builder_impl $( #[$builder_meta] )* $builder_name $( #[$meta] )* $vis $key $name $( $tt )* );
    };

    ( @type_impl $builder_name:ident $( #[$meta:meta] )* $vis:vis $key:ident $name:ident
        { $( $( #[$field_meta:meta] )* $field_vis:vis $field:ident: $field_ty:ty, )* }
    ) => {
        $( #[$meta] )*
        $vis $key $name {
            $( $( #[$field_meta] )* $field_vis $field: $field_ty, )*
        }

        impl $name {
            #[doc = concat!("Construct a new ", stringify!($builder_name), ".")]
            #[doc = ""]
            #[doc = concat!("This is effectively same as calling [`", stringify!($builder_name), "::new`]")]
            $vis fn builder() -> $builder_name {
                $builder_name::new()
            }
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
            crate::openapi::new!($vis $builder_name);
            crate::openapi::build_fn!($vis $name $( $field ),* );
        }

        crate::openapi::from!($name $builder_name $( $field ),* );
    };
}
use crate::openapi::extensions::Extensions;
pub(crate) use builder;

#[cfg(test)]
mod tests {
    use crate::openapi::{
        encoding::EncodingBuilder,
        example::ExampleBuilder,
        info::InfoBuilder,
        link::LinkBuilder,
        path::{
            OperationBuilder, Parameter, ParameterBuilder, ParameterIn, ParameterStyle,
            PathItemBuilder, PathsBuilder,
        },
        request_body::RequestBodyBuilder,
        security::{DeviceAuthorization, Flow, OAuth2, Scopes, SecurityScheme},
        tag::TagBuilder,
        xml::XmlBuilder,
    };
    use insta::assert_json_snapshot;
    use serde_json::json;
    use std::collections::BTreeMap;

    use super::{response::Response, *};

    #[test]
    fn serialize_deserialize_openapi_version_success() -> Result<(), serde_json::Error> {
        assert_eq!(serde_json::to_value(&OpenApiVersion::Version32)?, "3.2.0");
        #[allow(deprecated)]
        {
            assert_eq!(serde_json::to_value(&OpenApiVersion::Version31)?, "3.1.0");
            assert_eq!(
                serde_json::from_str::<OpenApiVersion>("\"3.1.1\"")?,
                OpenApiVersion::Version31
            );
        }
        assert_eq!(
            serde_json::from_str::<OpenApiVersion>("\"3.2.0\"")?,
            OpenApiVersion::Version32
        );
        Ok(())
    }

    #[test]
    fn openapi_32_root_self_and_webhooks_serialize() {
        let openapi = OpenApiBuilder::new()
            .info(Info::new("Events API", "1.0.0"))
            .json_schema_dialect(Some("https://spec.openapis.org/oas/3.2/dialect/2025-09-17"))
            .self_url(Some("https://example.com/openapi.json"))
            .paths(Paths::new())
            .webhooks(Some(PathsBuilder::new().path(
                "newPet",
                PathItem::new(
                    HttpMethod::Post,
                    OperationBuilder::new().response("200", Response::new("Webhook received")),
                ),
            )))
            .build();

        assert_eq!(
            serde_json::to_value(openapi).unwrap(),
            json!({
                "openapi": "3.2.0",
                "$self": "https://example.com/openapi.json",
                "jsonSchemaDialect": "https://spec.openapis.org/oas/3.2/dialect/2025-09-17",
                "info": {
                    "title": "Events API",
                    "version": "1.0.0"
                },
                "paths": {},
                "webhooks": {
                    "newPet": {
                        "post": {
                            "responses": {
                                "200": {
                                    "description": "Webhook received"
                                }
                            }
                        }
                    }
                }
            })
        );
    }

    #[test]
    fn openapi_32_components_and_callbacks_use_spec_data_model() {
        let callback_path = PathItemBuilder::new()
            .query(Some(
                OperationBuilder::new().response("200", ResponseBuilder::new()),
            ))
            .build();
        let operation = OperationBuilder::new()
            .callback(
                "onEvent",
                BTreeMap::from([(
                    "{$request.body#/callbackUrl}".to_string(),
                    callback_path.clone().into(),
                )]),
            )
            .response("202", ResponseBuilder::new().summary(Some("Accepted")))
            .build();
        let components = ComponentsBuilder::new()
            .parameter(
                "Limit",
                ParameterBuilder::new()
                    .name("limit")
                    .parameter_in(ParameterIn::Query)
                    .schema(Some(ObjectBuilder::new().schema_type(Type::Integer)))
                    .build(),
            )
            .example(
                "Accepted",
                ExampleBuilder::new().summary("Accepted").build(),
            )
            .request_body(
                "EventRequest",
                RequestBodyBuilder::new()
                    .content(
                        "application/json",
                        ContentBuilder::new()
                            .schema(Some(Ref::from_schema_name("EventPayload")))
                            .build(),
                    )
                    .build(),
            )
            .header(
                "RateLimit",
                HeaderBuilder::new()
                    .schema(Some(ObjectBuilder::new().schema_type(Type::Integer)))
                    .build(),
            )
            .link(
                "GetEvent",
                LinkBuilder::new().operation_id("getEvent").build(),
            )
            .callback(
                "EventCallback",
                BTreeMap::from([(
                    "{$request.body#/callbackUrl}".to_string(),
                    callback_path.clone().into(),
                )]),
            )
            .path_item("EventPath", callback_path)
            .media_type(
                "SseEvent",
                ContentBuilder::new()
                    .description(Some("Server-sent event item"))
                    .item_schema(Some(Ref::from_schema_name("ServerEvent")))
                    .item_encoding(Some(
                        EncodingBuilder::new().content_type(Some("application/json")),
                    ))
                    .prefix_encoding([EncodingBuilder::new().content_type(Some("text/plain"))])
                    .build(),
            )
            .security_scheme("ApiKey", RefOr::Ref(Ref::from_schema_name("ApiKey")))
            .build();

        assert_eq!(
            serde_json::to_value(operation).unwrap(),
            json!({
                "responses": {
                    "202": {
                        "summary": "Accepted"
                    }
                },
                "callbacks": {
                    "onEvent": {
                        "{$request.body#/callbackUrl}": {
                            "query": {
                                "responses": {
                                    "200": {}
                                }
                            }
                        }
                    }
                }
            })
        );
        assert_eq!(
            serde_json::to_value(components).unwrap(),
            json!({
                "parameters": {
                    "Limit": {
                        "name": "limit",
                        "in": "query",
                        "required": false,
                        "schema": {
                            "type": "integer"
                        }
                    }
                },
                "examples": {
                    "Accepted": {
                        "summary": "Accepted"
                    }
                },
                "requestBodies": {
                    "EventRequest": {
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/EventPayload"
                                }
                            }
                        }
                    }
                },
                "headers": {
                    "RateLimit": {
                        "schema": {
                            "type": "integer"
                        }
                    }
                },
                "links": {
                    "GetEvent": {
                        "operationId": "getEvent"
                    }
                },
                "callbacks": {
                    "EventCallback": {
                        "{$request.body#/callbackUrl}": {
                            "query": {
                                "responses": {
                                    "200": {}
                                }
                            }
                        }
                    }
                },
                "pathItems": {
                    "EventPath": {
                        "query": {
                            "responses": {
                                "200": {}
                            }
                        }
                    }
                },
                "mediaTypes": {
                    "SseEvent": {
                        "description": "Server-sent event item",
                        "itemSchema": {
                            "$ref": "#/components/schemas/ServerEvent"
                        },
                        "prefixEncoding": [
                            {
                                "contentType": "text/plain"
                            }
                        ],
                        "itemEncoding": {
                            "contentType": "application/json"
                        }
                    }
                },
                "securitySchemes": {
                    "ApiKey": {
                        "$ref": "#/components/schemas/ApiKey"
                    }
                }
            })
        );
    }

    #[test]
    fn openapi_32_tags_server_xml_and_examples_serialize() {
        let tag = TagBuilder::new()
            .name("partner")
            .summary(Some("Partner"))
            .description(Some("Operations available to partners"))
            .parent(Some("external"))
            .kind(Some("audience"))
            .build();
        let server = ServerBuilder::new()
            .url("https://api.example.com")
            .name(Some("production"))
            .build();
        let xml = XmlBuilder::new()
            .name(Some("animal"))
            .node_type(Some("element"))
            .build();
        let example = ExampleBuilder::new()
            .summary("Serialized query")
            .data_value(Some(json!({"flag": true})))
            .serialized_value(Some("flag=true"))
            .build();

        assert_eq!(
            serde_json::to_value(tag).unwrap(),
            json!({
                "name": "partner",
                "summary": "Partner",
                "description": "Operations available to partners",
                "parent": "external",
                "kind": "audience"
            })
        );
        assert_eq!(
            serde_json::to_value(server).unwrap(),
            json!({
                "url": "https://api.example.com",
                "name": "production"
            })
        );
        assert_eq!(
            serde_json::to_value(xml).unwrap(),
            json!({
                "name": "animal",
                "nodeType": "element"
            })
        );
        assert_eq!(
            serde_json::to_value(example).unwrap(),
            json!({
                "summary": "Serialized query",
                "dataValue": {
                    "flag": true
                },
                "serializedValue": "flag=true"
            })
        );
    }

    #[test]
    fn openapi_32_query_and_additional_operations_serialize() {
        let path_item = PathItemBuilder::new()
            .query(Some(
                OperationBuilder::new()
                    .operation_id(Some("searchProducts"))
                    .request_body(Some(
                        request_body::RequestBodyBuilder::new()
                            .content(
                                "application/json",
                                ContentBuilder::new()
                                    .schema(Some(Ref::from_schema_name("SearchCriteria")))
                                    .build(),
                            )
                            .build(),
                    ))
                    .response("200", Response::new("Search results")),
            ))
            .additional_operation(
                "COPY",
                OperationBuilder::new()
                    .operation_id(Some("copyPet"))
                    .response("200", Response::new("Copied")),
            )
            .build();

        assert_eq!(
            serde_json::to_value(path_item).unwrap(),
            json!({
                "query": {
                    "operationId": "searchProducts",
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/SearchCriteria"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Search results"
                        }
                    }
                },
                "additionalOperations": {
                    "COPY": {
                        "operationId": "copyPet",
                        "responses": {
                            "200": {
                                "description": "Copied"
                            }
                        }
                    }
                }
            })
        );
    }

    #[test]
    fn openapi_32_querystring_parameter_and_cookie_style_serialize() {
        let querystring = ParameterBuilder::from(Parameter::new("advancedQuery"))
            .parameter_in(ParameterIn::QueryString)
            .required(Required::False)
            .content(
                "application/x-www-form-urlencoded",
                ContentBuilder::new()
                    .schema(Some(
                        ObjectBuilder::new()
                            .schema_type(Type::Object)
                            .property("foo", ObjectBuilder::new().schema_type(Type::String))
                            .property("bar", ObjectBuilder::new().schema_type(Type::Boolean)),
                    ))
                    .examples_from_iter([(
                        "spacesAndPluses",
                        ExampleBuilder::new()
                            .description("Form-encoded query string")
                            .data_value(Some(json!({
                                "foo": "a + b",
                                "bar": true
                            })))
                            .serialized_value(Some("foo=a+%2B+b&bar=true")),
                    )])
                    .build(),
            )
            .build();
        let cookie = ParameterBuilder::from(Parameter::new("greeting"))
            .parameter_in(ParameterIn::Cookie)
            .style(Some(ParameterStyle::Cookie))
            .example(Some(json!("Hello, world!")))
            .build();

        assert_eq!(
            serde_json::to_value(querystring).unwrap(),
            json!({
                "name": "advancedQuery",
                "in": "querystring",
                "required": false,
                "content": {
                    "application/x-www-form-urlencoded": {
                        "schema": {
                            "type": "object",
                            "properties": {
                                "bar": {
                                    "type": "boolean"
                                },
                                "foo": {
                                    "type": "string"
                                }
                            }
                        },
                        "examples": {
                            "spacesAndPluses": {
                                "description": "Form-encoded query string",
                                "dataValue": {
                                    "foo": "a + b",
                                    "bar": true
                                },
                                "serializedValue": "foo=a+%2B+b&bar=true"
                            }
                        }
                    }
                }
            })
        );
        assert_eq!(
            serde_json::to_value(cookie).unwrap(),
            json!({
                "name": "greeting",
                "in": "cookie",
                "required": true,
                "style": "cookie",
                "example": "Hello, world!"
            })
        );
    }

    #[test]
    fn openapi_32_components_media_types_streaming_and_response_summary_serialize() {
        let event_payload = ObjectBuilder::new()
            .schema_type(Type::Object)
            .property("pet_id", ObjectBuilder::new().schema_type(Type::Integer))
            .property("status", ObjectBuilder::new().schema_type(Type::String));
        let server_event = ObjectBuilder::new()
            .schema_type(Type::Object)
            .property("event", ObjectBuilder::new().schema_type(Type::String))
            .property(
                "data",
                ObjectBuilder::new()
                    .schema_type(Type::String)
                    .content_media_type("application/json")
                    .content_schema(Some(Ref::from_schema_name("PetEventPayload"))),
            )
            .property("id", ObjectBuilder::new().schema_type(Type::String))
            .property("retry", ObjectBuilder::new().schema_type(Type::Integer));
        let components = ComponentsBuilder::new()
            .schema("PetEventPayload", event_payload)
            .schema("ServerEvent", server_event)
            .media_type(
                "ServerSentEvents",
                ContentBuilder::new()
                    .item_schema(Some(Ref::from_schema_name("ServerEvent")))
                    .build(),
            )
            .media_type(
                "JsonLines",
                ContentBuilder::new()
                    .item_schema(Some(Ref::from_schema_name("PetEventPayload")))
                    .build(),
            )
            .media_type(
                "JsonTextSequences",
                ContentBuilder::new()
                    .item_schema(Some(Ref::from_schema_name("PetEventPayload")))
                    .build(),
            )
            .media_type(
                "MultipartMixed",
                ContentBuilder::new()
                    .item_schema(Some(Ref::from_schema_name("PetEventPayload")))
                    .build(),
            )
            .build();
        let response = ResponseBuilder::new()
            .summary(Some("Streaming response"))
            .content(
                "text/event-stream",
                ContentBuilder::new()
                    .item_schema(Some(Ref::from_schema_name("ServerEvent")))
                    .build(),
            )
            .content(
                "application/jsonl",
                ContentBuilder::new()
                    .item_schema(Some(Ref::from_schema_name("PetEventPayload")))
                    .build(),
            )
            .content(
                "application/json-seq",
                ContentBuilder::new()
                    .item_schema(Some(Ref::from_schema_name("PetEventPayload")))
                    .build(),
            )
            .content(
                "multipart/mixed",
                ContentBuilder::new()
                    .item_schema(Some(Ref::from_schema_name("PetEventPayload")))
                    .build(),
            )
            .build();

        assert_eq!(
            serde_json::to_value(components).unwrap(),
            json!({
                "schemas": {
                    "PetEventPayload": {
                        "type": "object",
                        "properties": {
                            "pet_id": {
                                "type": "integer"
                            },
                            "status": {
                                "type": "string"
                            }
                        }
                    },
                    "ServerEvent": {
                        "type": "object",
                        "properties": {
                            "data": {
                                "type": "string",
                                "contentMediaType": "application/json",
                                "contentSchema": {
                                    "$ref": "#/components/schemas/PetEventPayload"
                                }
                            },
                            "event": {
                                "type": "string"
                            },
                            "id": {
                                "type": "string"
                            },
                            "retry": {
                                "type": "integer"
                            }
                        }
                    }
                },
                "mediaTypes": {
                    "JsonLines": {
                        "itemSchema": {
                            "$ref": "#/components/schemas/PetEventPayload"
                        }
                    },
                    "JsonTextSequences": {
                        "itemSchema": {
                            "$ref": "#/components/schemas/PetEventPayload"
                        }
                    },
                    "MultipartMixed": {
                        "itemSchema": {
                            "$ref": "#/components/schemas/PetEventPayload"
                        }
                    },
                    "ServerSentEvents": {
                        "itemSchema": {
                            "$ref": "#/components/schemas/ServerEvent"
                        }
                    }
                }
            })
        );
        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({
                "summary": "Streaming response",
                "content": {
                    "application/json-seq": {
                        "itemSchema": {
                            "$ref": "#/components/schemas/PetEventPayload"
                        }
                    },
                    "application/jsonl": {
                        "itemSchema": {
                            "$ref": "#/components/schemas/PetEventPayload"
                        }
                    },
                    "multipart/mixed": {
                        "itemSchema": {
                            "$ref": "#/components/schemas/PetEventPayload"
                        }
                    },
                    "text/event-stream": {
                        "itemSchema": {
                            "$ref": "#/components/schemas/ServerEvent"
                        }
                    }
                }
            })
        );
    }

    #[test]
    fn openapi_32_oauth_device_authorization_and_deprecated_security_serialize() {
        let oauth = SecurityScheme::OAuth2(
            OAuth2::new([Flow::DeviceAuthorization(DeviceAuthorization::new(
                "https://example.com/device",
                "https://example.com/token",
                Scopes::from_iter([("read:pets", "read pets")]),
            ))])
            .with_metadata_url("https://example.com/.well-known/oauth-authorization-server")
            .deprecated(Some(Deprecated::True)),
        );

        assert_eq!(
            serde_json::to_value(oauth).unwrap(),
            json!({
                "type": "oauth2",
                "flows": {
                    "deviceAuthorization": {
                        "deviceAuthorizationUrl": "https://example.com/device",
                        "tokenUrl": "https://example.com/token",
                        "scopes": {
                            "read:pets": "read pets"
                        }
                    }
                },
                "oauth2MetadataUrl": "https://example.com/.well-known/oauth-authorization-server",
                "deprecated": true
            })
        );
    }

    #[test]
    fn serialize_openapi_json_minimal_success() {
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

        assert_json_snapshot!(openapi);
    }

    #[test]
    fn serialize_openapi_json_with_paths_success() {
        let openapi = OpenApi::new(
            Info::new("My big api", "1.1.0"),
            PathsBuilder::new()
                .path(
                    "/api/v1/users",
                    PathItem::new(
                        HttpMethod::Get,
                        OperationBuilder::new().response("200", Response::new("Get users list")),
                    ),
                )
                .path(
                    "/api/v1/users",
                    PathItem::new(
                        HttpMethod::Post,
                        OperationBuilder::new().response("200", Response::new("Post new user")),
                    ),
                )
                .path(
                    "/api/v1/users/{id}",
                    PathItem::new(
                        HttpMethod::Get,
                        OperationBuilder::new().response("200", Response::new("Get user by id")),
                    ),
                ),
        );

        assert_json_snapshot!(openapi);
    }

    #[test]
    fn merge_2_openapi_documents() {
        let mut api_1 = OpenApi::new(
            Info::new("Api", "v1"),
            PathsBuilder::new()
                .path(
                    "/api/v1/user",
                    PathItem::new(
                        HttpMethod::Get,
                        OperationBuilder::new().response("200", Response::new("Get user success")),
                    ),
                )
                .build(),
        );

        let api_2 = OpenApiBuilder::new()
            .info(Info::new("Api", "v2"))
            .paths(
                PathsBuilder::new()
                    .path(
                        "/api/v1/user",
                        PathItem::new(
                            HttpMethod::Get,
                            OperationBuilder::new()
                                .response("200", Response::new("This will not get added")),
                        ),
                    )
                    .path(
                        "/ap/v2/user",
                        PathItem::new(
                            HttpMethod::Get,
                            OperationBuilder::new()
                                .response("200", Response::new("Get user success 2")),
                        ),
                    )
                    .path(
                        "/api/v2/user",
                        PathItem::new(
                            HttpMethod::Post,
                            OperationBuilder::new()
                                .response("200", Response::new("Get user success")),
                        ),
                    )
                    .build(),
            )
            .components(Some(
                ComponentsBuilder::new()
                    .schema(
                        "User2",
                        ObjectBuilder::new().schema_type(Type::Object).property(
                            "name",
                            ObjectBuilder::new().schema_type(Type::String).build(),
                        ),
                    )
                    .build(),
            ))
            .build();

        api_1.merge(api_2);

        assert_json_snapshot!(api_1, {
            ".paths" => insta::sorted_redaction()
        });
    }

    #[test]
    fn merge_same_path_diff_methods() {
        let mut api_1 = OpenApi::new(
            Info::new("Api", "v1"),
            PathsBuilder::new()
                .path(
                    "/api/v1/user",
                    PathItem::new(
                        HttpMethod::Get,
                        OperationBuilder::new()
                            .response("200", Response::new("Get user success 1")),
                    ),
                )
                .extensions(Some(Extensions::from_iter([("x-v1-api", true)])))
                .build(),
        );

        let api_2 = OpenApiBuilder::new()
            .info(Info::new("Api", "v2"))
            .paths(
                PathsBuilder::new()
                    .path(
                        "/api/v1/user",
                        PathItem::new(
                            HttpMethod::Get,
                            OperationBuilder::new()
                                .response("200", Response::new("This will not get added")),
                        ),
                    )
                    .path(
                        "/api/v1/user",
                        PathItem::new(
                            HttpMethod::Post,
                            OperationBuilder::new()
                                .response("200", Response::new("Post user success 1")),
                        ),
                    )
                    .path(
                        "/api/v2/user",
                        PathItem::new(
                            HttpMethod::Get,
                            OperationBuilder::new()
                                .response("200", Response::new("Get user success 2")),
                        ),
                    )
                    .path(
                        "/api/v2/user",
                        PathItem::new(
                            HttpMethod::Post,
                            OperationBuilder::new()
                                .response("200", Response::new("Post user success 2")),
                        ),
                    )
                    .extensions(Some(Extensions::from_iter([("x-random", "Value")])))
                    .build(),
            )
            .components(Some(
                ComponentsBuilder::new()
                    .schema(
                        "User2",
                        ObjectBuilder::new().schema_type(Type::Object).property(
                            "name",
                            ObjectBuilder::new().schema_type(Type::String).build(),
                        ),
                    )
                    .build(),
            ))
            .build();

        api_1.merge(api_2);

        assert_json_snapshot!(api_1, {
            ".paths" => insta::sorted_redaction()
        });
    }

    #[test]
    fn test_nest_open_apis() {
        let api = OpenApiBuilder::new()
            .paths(
                PathsBuilder::new().path(
                    "/api/v1/status",
                    PathItem::new(
                        HttpMethod::Get,
                        OperationBuilder::new()
                            .description(Some("Get status"))
                            .build(),
                    ),
                ),
            )
            .build();

        let user_api = OpenApiBuilder::new()
            .paths(
                PathsBuilder::new()
                    .path(
                        "/",
                        PathItem::new(
                            HttpMethod::Get,
                            OperationBuilder::new()
                                .description(Some("Get user details"))
                                .build(),
                        ),
                    )
                    .path(
                        "/foo",
                        PathItem::new(HttpMethod::Post, OperationBuilder::new().build()),
                    ),
            )
            .build();

        let nest_merged = api.nest("/api/v1/user", user_api);
        let value = serde_json::to_value(nest_merged).expect("should serialize as json");
        let paths = value
            .pointer("/paths")
            .expect("paths should exits in openapi");

        assert_json_snapshot!(paths);
    }

    #[test]
    fn openapi_custom_extension() {
        let mut api = OpenApiBuilder::new().build();
        let extensions = api.extensions.get_or_insert(Default::default());
        extensions.insert(
            String::from("x-tagGroup"),
            String::from("anything that serializes to Json").into(),
        );

        assert_json_snapshot!(api);
    }
}
