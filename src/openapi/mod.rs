use std::collections::BTreeMap;

use serde::{Deserialize, Serialize, Serializer};

use crate::error::Error;

pub use self::{
    contact::Contact,
    external_docs::ExternalDocs,
    info::Info,
    licence::Licence,
    path::{PathItem, PathItemType, Paths},
    schema::{Array, Component, ComponentFormat, ComponentType, Object, Property, Ref, Schema},
    security::Security,
    server::Server,
    tag::Tag,
};

pub mod contact;
pub mod external_docs;
pub mod info;
pub mod licence;
pub mod path;
pub mod request_body;
pub mod response;
pub mod schema;
pub mod security;
pub mod server;
pub mod tag;

const OPENAPI_VERSION_3: &str = "3.0.3";

#[non_exhaustive]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OpenApi {
    pub openapi: OpenApiVersion,

    pub info: Info,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,

    pub paths: BTreeMap<String, PathItem>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Schema>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<Security>>,

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

    pub fn with_components(mut self, schema: Schema) -> Self {
        self.components = Some(schema);

        self
    }

    pub fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string(self).map_err(Error::Serde)
    }
}

#[derive(Debug, Deserialize)]
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

#[derive(Deserialize, Debug, PartialEq)]
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

#[derive(Deserialize, Debug, PartialEq)]
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

#[macro_export]
macro_rules! option {
    ( $val:expr ) => {
        if $val.to_string().is_empty() {
            None
        } else {
            Some($val)
        }
    };
}

#[cfg(test)]
mod tests {

    use std::rc::Rc;

    use crate::{error::Error, openapi::licence::Licence};

    use super::{path::Operation, response::Response, *};

    #[test]
    fn serialize_deserialize_openapi_version_success() -> Result<(), Error> {
        assert_eq!(
            serde_json::to_value(&OpenApiVersion::Version3).map_err(Error::Serde)?,
            "3.0.3"
        );
        Ok(())
    }

    #[test]
    fn serialize_openapi_json_minimal_success() -> Result<(), Error> {
        let raw_json = include_str!("testdata/expected_openapi_minimal.json");
        let openapi = OpenApi::new(
            Info::new("My api", "1.0.0")
                .with_description("My api description")
                .with_licence(Licence::new("MIT").with_url("http://mit.licence")),
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
    fn serialize_openapi_json_with_paths_success() -> Result<(), Error> {
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

    #[test]
    fn option() {
        assert_eq!(None, option!(""))
    }

    #[test]
    fn test_lined_list() {
        #[derive(Debug, PartialEq, Eq)]
        struct Linked {
            val: String,
            child: Option<Rc<Linked>>,
        }

        struct LinkedRef<'a, Linked> {
            inner: Option<&'a Linked>,
        }

        impl<'a> Iterator for LinkedRef<'a, Linked> {
            type Item = LinkedRef<'a, Linked>;

            fn next(&mut self) -> Option<Self::Item> {
                let current = self.inner;
                let next = current.and_then(|current| current.child.as_ref());

                if let Some(linked) = next {
                    self.inner = Some(linked.as_ref())
                } else {
                    self.inner = None
                }

                // match next {
                //     Some(linked) if linked.as_ref() != current => {
                //         Some(LinkedRef { inner: current })
                //     }
                //     _ => None,
                // }

                println!("current: {:?} == {:?}", current, next);

                current.map(|linked| LinkedRef {
                    inner: Some(linked),
                })
            }
        }

        let linked = Linked {
            val: "foo".to_string(),
            child: Some(Rc::new(Linked {
                val: "bar".to_string(),
                child: Some(Rc::new(Linked {
                    val: "finished".to_string(),
                    child: None,
                })),
            })),
        };

        let linked_refs = LinkedRef {
            inner: Some(&linked),
        };

        linked_refs.for_each(|linked| println!("Linked it: {:?}", linked.inner))
    }
}
