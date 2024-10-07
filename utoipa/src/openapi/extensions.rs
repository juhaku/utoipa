//! Implements [OpenAPI Extensions][extensions].
//!
//! [extensions]: https://spec.openapis.org/oas/latest.html#specification-extensions
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use serde::Serialize;

use super::builder;

const EXTENSION_PREFIX: &str = "x-";

builder! {
    ExtensionsBuilder;

    /// Additional [data for extending][extensions] the OpenAPI specification.
    ///
    /// [extensions]: https://spec.openapis.org/oas/latest.html#specification-extensions
    #[derive(Default, Serialize, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Extensions{
        #[serde(flatten)]
        extensions: HashMap<String, serde_json::Value>,
    }
}

impl Extensions {
    /// Merge other [`Extensions`] into _`self`_.
    pub fn merge(&mut self, other: Extensions) {
        self.extensions.extend(other.extensions);
    }
}

impl Deref for Extensions {
    type Target = HashMap<String, serde_json::Value>;

    fn deref(&self) -> &Self::Target {
        &self.extensions
    }
}

impl DerefMut for Extensions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.extensions
    }
}

impl<K, V> FromIterator<(K, V)> for Extensions
where
    K: Into<String>,
    V: Into<serde_json::Value>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let iter = iter.into_iter().map(|(k, v)| (k.into(), v.into()));
        let extensions = HashMap::from_iter(iter);
        Self { extensions }
    }
}

impl From<Extensions> for HashMap<String, serde_json::Value> {
    fn from(value: Extensions) -> Self {
        value.extensions
    }
}

impl<'de> serde::de::Deserialize<'de> for Extensions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let extensions: HashMap<String, _> = HashMap::deserialize(deserializer)?;
        let extensions = extensions
            .into_iter()
            .filter(|(k, _)| k.starts_with(EXTENSION_PREFIX))
            .collect();
        Ok(Self { extensions })
    }
}

impl ExtensionsBuilder {
    /// Adds a key-value pair to the extensions. Extensions keys are prefixed with `"x-"` if
    /// not done already.
    pub fn add<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        let mut key: String = key.into();
        if !key.starts_with(EXTENSION_PREFIX) {
            key = format!("{EXTENSION_PREFIX}{key}");
        }
        self.extensions.insert(key, value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extensions_builder() {
        let expected = json!("value");
        let extensions = ExtensionsBuilder::new()
            .add("x-some-extension", expected.clone())
            .add("another-extension", expected.clone())
            .build();

        let value = serde_json::to_value(&extensions).unwrap();
        assert_eq!(value.get("x-some-extension"), Some(&expected));
        assert_eq!(value.get("x-another-extension"), Some(&expected));
    }

    #[test]
    fn extensions_from_iter() {
        let expected = json!("value");
        let extensions: Extensions = [
            ("x-some-extension", expected.clone()),
            ("another-extension", expected.clone()),
        ]
        .into_iter()
        .collect();

        assert_eq!(extensions.get("x-some-extension"), Some(&expected));
        assert_eq!(extensions.get("another-extension"), Some(&expected));
    }
}
