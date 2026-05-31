#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
//! This crate provides global configuration capabilities for [`utoipa`](https://docs.rs/utoipa/latest/utoipa/).
//!
//! ## Config options
//!
//! * Define rust type aliases for `utoipa` with `.alias_for(...)` method.
//! * Define schema collect mode for `utoipa` with `.schema_collect(...)` method.
//!   * [`SchemaCollect::All`] will collect all schemas from usages including inlined with `inline(T)`
//!   * [`SchemaCollect::NonInlined`] will only collect non inlined schemas from usages.
//!
//! <div class="warning">
//!
//! <b>Warning!</b><br>
//! The build config will be stored to projects `OUTPUT` directory. It is then read from there via `OUTPUT` environment
//! variable which will return **any instance** rust compiler might find at that time (Whatever the `OUTPUT` environment variable points to).
//! **Be aware** that sometimes you might face a situation where the config is not aligned with your Rust aliases.
//! This might need you to change something on your code before changed config might apply.
//!
//! </div>
//!
//! ## Install
//!
//! Add dependency declaration to `Cargo.toml`.
//!
//! ```toml
//! [build-dependencies]
//! utoipa-config = "0.1"
//! ```
//!
//! ## Examples
//!
//! _**Create `build.rs` file with following content, then in your code you can just use `MyType` as
//! alternative for `i32`.**_
//!
//! ```rust
//! # #![allow(clippy::needless_doctest_main)]
//! use utoipa_config::Config;
//!
//! fn main() {
//!     Config::new()
//!         .alias_for("MyType", "i32")
//!         .write_to_file();
//! }
//! ```
//!
//! See full [example for utoipa-config](https://github.com/juhaku/utoipa/tree/master/examples/utoipa-config-test/).

use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::de::Visitor;
use serde::{Deserialize, Serialize};

/// Global configuration initialized in `build.rs` of user project.
///
/// This works similar fashion to what `hyperium/tonic` grpc library does with the project configuration. See
/// the quick usage from [module documentation][module]
///
/// [module]: ./index.html
#[derive(Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Config<'c> {
    /// A map of global aliases `utoipa` will recognize as types.
    #[doc(hidden)]
    pub aliases: HashMap<Cow<'c, str>, Cow<'c, str>>,
    /// Schema collect mode for `utoipa`. By default only non inlined schemas are collected.
    pub schema_collect: SchemaCollect,
    /// Automatically include parameters from extractor types that implement `IntoParams`.
    /// This acts as a global default; individual paths can still override via `auto_params`.
    pub auto_into_params: bool,
}

/// Configures schema collect mode. By default only non explicitly inlined schemas are collected.
/// but this behavior can be changed to collect also inlined schemas by setting
/// [`SchemaCollect::All`].
#[derive(Default)]
pub enum SchemaCollect {
    /// Makes sure that all schemas from usages are collected including inlined.
    All,
    /// Collect only non explicitly inlined schemas to the OpenAPI. This will result smaller schema
    /// foot print in the OpenAPI if schemas are typically inlined with `inline(T)` on usage.
    #[default]
    NonInlined,
}

impl Serialize for SchemaCollect {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::All => serializer.serialize_str("all"),
            Self::NonInlined => serializer.serialize_str("non_inlined"),
        }
    }
}

impl<'de> Deserialize<'de> for SchemaCollect {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SchemaCollectVisitor;
        impl<'d> Visitor<'d> for SchemaCollectVisitor {
            type Value = SchemaCollect;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("expected str `all` or `non_inlined`")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v == "all" {
                    Ok(SchemaCollect::All)
                } else {
                    Ok(SchemaCollect::NonInlined)
                }
            }
        }

        deserializer.deserialize_str(SchemaCollectVisitor)
    }
}

impl<'c> Config<'c> {
    const NAME: &'static str = "utoipa-config.json";

    /// Construct a new [`Config`].
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Add new global alias.
    ///
    /// This method accepts two arguments. First being identifier of the user's type alias.
    /// Second is the type path definition to be used as alias value. The _`value`_ can be anything
    /// that `utoipa` can parse as `TypeTree` and can be used as type for a value.
    ///
    /// Because of `TypeTree` the aliased value can also be a fairly complex type and not limited
    /// to primitive types. This also allows users create custom types which can be treated as
    /// primitive types. E.g. One could create custom date time type that is treated as chrono's
    /// DateTime or a String.
    ///
    /// # Examples
    ///
    /// _**Create `MyType` alias for `i32`.**_
    /// ```rust
    /// use utoipa_config::Config;
    ///
    /// let _ = Config::new()
    ///     .alias_for("MyType", "i32");
    /// ```
    ///
    /// _**Create `Json` alias for `serde_json::Value`.**_
    /// ```rust
    /// use utoipa_config::Config;
    ///
    /// let _ = Config::new()
    ///     .alias_for("Json", "Value");
    /// ```
    /// _**Create `NullableString` alias for `Option<String>`.**_
    /// ```rust
    /// use utoipa_config::Config;
    ///
    /// let _ = Config::new()
    ///     .alias_for("NullableString", "Option<String>");
    /// ```
    pub fn alias_for(mut self, alias: &'c str, value: &'c str) -> Config<'c> {
        self.aliases
            .insert(Cow::Borrowed(alias), Cow::Borrowed(value));

        self
    }

    /// Define schema collect mode for `utoipa`.
    ///
    /// Method accepts one argument [`SchemaCollect`] which defines the collect mode to be used by
    /// `utiopa`. If none is defined [`SchemaCollect::NonInlined`] schemas will be collected by
    /// default.
    ///
    /// This can be changed to [`SchemaCollect::All`] if schemas called with `inline(T)` is wished
    /// to be collected to the resulting OpenAPI.
    pub fn schema_collect(mut self, schema_collect: SchemaCollect) -> Self {
        self.schema_collect = schema_collect;

        self
    }

    /// Define default behavior for automatically including `IntoParams` implementations.
    ///
    /// When enabled, `utoipa::path` will include parameters from extractors that implement
    /// `IntoParams` unless the path explicitly disables it with `auto_params = false`.
    pub fn auto_into_params(mut self, enabled: bool) -> Self {
        self.auto_into_params = enabled;

        self
    }

    fn get_out_dir() -> Option<String> {
        match std::env::var("OUT_DIR") {
            Ok(out_dir) => Some(out_dir),
            Err(_) => None,
        }
    }

    /// Write the current [`Config`] to a file. This persists the [`Config`] for `utoipa` to read
    /// and use later.
    pub fn write_to_file(&self) {
        let json = serde_json::to_string(self).expect("Config must be JSON serializable");

        let Some(out_dir) = Config::get_out_dir() else {
            return;
        };

        match fs::write([&*out_dir, Config::NAME].iter().collect::<PathBuf>(), json) {
            Ok(_) => (),
            Err(error) => panic!("Failed to write config {}, error: {error}", Config::NAME),
        };
    }

    /// Read a [`Config`] from a file. Used internally by `utiopa`.
    #[doc(hidden)]
    pub fn read_from_file() -> Config<'c> {
        let Some(out_dir) = Config::get_out_dir() else {
            return Config::default();
        };

        let str = match fs::read_to_string([&*out_dir, Config::NAME].iter().collect::<PathBuf>()) {
            Ok(str) => str,
            Err(error) => panic!("Failed to read config: {}, error: {error}", Config::NAME),
        };

        serde_json::from_str(&str).expect("Config muts be JSON deserializable")
    }
}
