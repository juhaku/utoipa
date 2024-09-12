#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
//! This crate provides global configuration capabilities for [`utoipa`](https://docs.rs/utoipa/latest/utoipa/). Currently only
//! supports providing Rust type aliases.
//!
//! ## Install
//!
//! Add dependency declaration to `Cargo.toml`.
//!
//! ```toml
//! [build-dependencies]
//! utoipa_config = "0.1"
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
//! See full [example for utoipa-config](https://github.com/juhaku/utoipa/tree/master/examples/config-test-crate/).

use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

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
