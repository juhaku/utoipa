//! Implements [`$defs` keyword from json schema][defs]
//!
//! [defs]: https://json-schema.org/draft/2020-12/json-schema-core#name-schema-re-use-with-defs
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use super::Schema;

/// The "$defs" keyword reserves a location for schema authors to inline re-usable JSON Schemas
/// into a more general schema. The keyword does not directly affect the validation result.
///
/// This keyword's value MUST be an object. Each member value of this object MUST be a valid
/// JSON Schema.
///
/// As an example, here is a schema describing an array of positive integers, where the
/// positive integer constraint is a subschema in "$defs":
/// ```
/// {
///     "type": "array",
///     "items": { "$ref": "#/$defs/positiveInteger" },
///     "$defs": {
///         "positiveInteger": {
///             "type": "integer",
///             "exclusiveMinimum": 0
///         }
///     }
/// }
/// ```
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Defs(
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default, rename = "#defs")]
    BTreeMap<String, Schema>,
);

/// Builder for the [`Defs`]
pub struct DefsBuilder(BTreeMap<String, Schema>);

impl DefsBuilder {
    /// Extend `$defs`
    ///
    /// Read more:
    /// <https://json-schema.org/draft/2020-12/json-schema-core#name-schema-re-use-with-defs>
    pub fn defs<T: IntoIterator<Item = (String, Schema)>>(mut self, defs: T) -> Self {
        self.0.extend(defs);
        self
    }

    /// Add field describing `const` value
    ///
    /// Read more:
    /// <https://json-schema.org/draft/2020-12/json-schema-core#name-schema-re-use-with-defs>
    pub fn def<K: Into<String>, V: Into<Schema>>(mut self, key: K, value: V) -> Self {
        self.0.insert(key.into(), value.into());
        self
    }
}

impl Defs {
    /// Get internal iterator
    pub fn iter(&self) -> std::collections::btree_map::Iter<String, Schema> {
        self.0.iter()
    }

    /// Get internal iterator (mutable)
    pub fn iter_mut(&mut self) -> std::collections::btree_map::IterMut<String, Schema> {
        self.0.iter_mut()
    }
}

impl<K, V> FromIterator<(K, V)> for Defs
where
    K: Into<String>,
    V: Into<Schema>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let iter = iter.into_iter().map(|(k, v)| (k.into(), v.into()));
        let defs = BTreeMap::from_iter(iter);
        Self(defs)
    }
}

impl From<Defs> for BTreeMap<String, Schema> {
    fn from(value: Defs) -> Self {
        value.0
    }
}

impl Deref for Defs {
    type Target = BTreeMap<String, Schema>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Defs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Defs {
    type Item = (String, Schema);

    type IntoIter = std::collections::btree_map::IntoIter<String, Schema>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Schema {

    /// Retrive [`$defs`] keyword for the [`Schema`]
    pub fn defs(&self) -> Option<&Defs> {
        match self {
            Schema::Const(_) => None,
            Schema::Array(array) => Some(&array.defs),
            Schema::Object(object) => Some(&object.defs),
            Schema::OneOf(one_of) => Some(&one_of.defs),
            Schema::AllOf(all_of) => Some(&all_of.defs),
            Schema::AnyOf(any_of) => Some(&any_of.defs),
        }
    }
}
