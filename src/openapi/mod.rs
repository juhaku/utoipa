use std::collections::BTreeMap;

use serde::{Deserialize, Serialize, Serializer};

pub use self::{
    contact::Contact,
    content::Content,
    external_docs::ExternalDocs,
    header::Header,
    info::Info,
    licence::License,
    path::{PathItem, PathItemType, Paths},
    response::{Response, Responses},
    schema::{
        Array, Component, ComponentFormat, ComponentType, Components, Object, OneOf, Property, Ref,
        ToArray,
    },
    security::SecurityRequirement,
    server::Server,
    tag::Tag,
};

pub mod contact;
pub mod content;
pub mod external_docs;
pub mod header;
pub mod info;
pub mod licence;
pub mod path;
pub mod request_body;
pub mod response;
pub mod schema;
pub mod security;
pub mod server;
pub mod tag;
pub mod xml;

const OPENAPI_VERSION_3: &str = "3.0.3";

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenApi {
    pub openapi: OpenApiVersion,

    pub info: Info,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,

    pub paths: BTreeMap<String, PathItem>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<SecurityRequirement>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocs>,
}

impl OpenApi {
    pub fn new(info: Info, paths: Paths) -> Self {
        Self {
            info,
            paths: paths.to_map(),
            ..Default::default()
        }
    }

    pub fn with_servers<I: IntoIterator<Item = Server>>(mut self, servers: I) -> Self {
        self.servers = Some(servers.into_iter().collect());

        self
    }

    pub fn with_components(mut self, components: Components) -> Self {
        self.components = Some(components);

        self
    }

    /// Add list of [`SecurityRequirement`]s that are globally available for all operations.
    pub fn with_securities<I: IntoIterator<Item = SecurityRequirement>>(
        mut self,
        securities: I,
    ) -> Self {
        self.security = Some(securities.into_iter().collect());

        self
    }

    /// Add [`SecurityRequirement`] that is globally available for all operations.
    pub fn with_security(mut self, security: SecurityRequirement) -> Self {
        self.security.as_mut().unwrap().push(security);

        self
    }

    #[cfg(feature = "serde_json")]
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    #[cfg(feature = "serde_json")]
    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum OpenApiVersion {
    Version3,
}

impl Serialize for OpenApiVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Version3 => serializer.serialize_str(OPENAPI_VERSION_3),
        }
    }
}

impl Default for OpenApiVersion {
    fn default() -> Self {
        Self::Version3
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub enum Deprecated {
    True,
    False,
}

impl Deprecated {
    pub fn to_bool(&self) -> bool {
        matches!(self, Self::True)
    }
}

impl Serialize for Deprecated {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(self.to_bool())
    }
}

impl Default for Deprecated {
    fn default() -> Self {
        Deprecated::False
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub enum Required {
    True,
    False,
}

impl Required {
    pub fn to_bool(&self) -> bool {
        matches!(self, Self::True)
    }
}

impl Serialize for Required {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(self.to_bool())
    }
}

impl Default for Required {
    fn default() -> Self {
        Required::False
    }
}

#[cfg(test)]
#[cfg(feature = "serde_json")]
mod tests {
    use crate::openapi::licence::License;

    use super::{path::Operation, response::Response, *};

    #[test]
    fn serialize_deserialize_openapi_version_success() -> Result<(), serde_json::Error> {
        assert_eq!(serde_json::to_value(&OpenApiVersion::Version3)?, "3.0.3");
        Ok(())
    }

    #[test]
    fn serialize_openapi_json_minimal_success() -> Result<(), serde_json::Error> {
        let raw_json = include_str!("testdata/expected_openapi_minimal.json");
        let openapi = OpenApi::new(
            Info::new("My api", "1.0.0")
                .with_description("My api description")
                .with_license(License::new("MIT").with_url("http://mit.licence")),
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
            Paths::new()
                .append(
                    "/api/v1/users",
                    PathItem::new(
                        PathItemType::Get,
                        Operation::new().with_response("200", Response::new("Get users list")),
                    ),
                )
                .append(
                    "/api/v1/users",
                    PathItem::new(
                        PathItemType::Post,
                        Operation::new().with_response("200", Response::new("Post new user")),
                    ),
                )
                .append(
                    "/api/v1/users/{id}",
                    PathItem::new(
                        PathItemType::Get,
                        Operation::new().with_response("200", Response::new("Get user by id")),
                    ),
                ),
        );

        let serialized = serde_json::to_string_pretty(&openapi)?;
        let expected = include_str!("./testdata/expected_openapi_with_paths.json");

        assert_eq!(
            serialized, expected,
            "expected serialized json to match raw: \nserialized: \n{} \nraw: \n{}",
            serialized, expected
        );
        Ok(())
    }
}
