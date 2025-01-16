//! This is **private** utoipa codegen library and is not used alone.
//!
//! The library contains macro implementations for utoipa library. Content
//! of the library documentation is available through **utoipa** library itself.
//! Consider browsing via the **utoipa** crate so all links will work correctly.

#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

#[cfg(all(feature = "decimal", feature = "decimal_float"))]
compile_error!("`decimal` and `decimal_float` are mutually exclusive feature flags");

#[cfg(all(
    feature = "actix_extras",
    feature = "axum_extras",
    feature = "rocket_extras"
))]
compile_error!(
    "`actix_extras`, `axum_extras` and `rocket_extras` are mutually exclusive feature flags"
);

use std::{
    borrow::{Borrow, Cow},
    error::Error,
    fmt::Display,
    mem,
    ops::Deref,
};

use component::schema::Schema;
use doc_comment::CommentAttributes;

use component::into_params::IntoParams;
use ext::{PathOperationResolver, PathOperations, PathResolver};
use openapi::OpenApi;
use proc_macro::TokenStream;
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};

use proc_macro2::{Group, Ident, Punct, Span, TokenStream as TokenStream2};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Bracket,
    DeriveInput, ExprPath, GenericParam, ItemFn, Lit, LitStr, Member, Token,
};

mod component;
mod doc_comment;
mod ext;
mod openapi;
mod path;
mod schema_type;
mod security_requirement;

use crate::path::{Path, PathAttr};

use self::{
    component::{
        features::{self, Feature},
        ComponentSchema, ComponentSchemaProps, TypeTree,
    },
    openapi::parse_openapi_attrs,
    path::response::derive::{IntoResponses, ToResponse},
};

#[cfg(feature = "config")]
static CONFIG: once_cell::sync::Lazy<utoipa_config::Config> =
    once_cell::sync::Lazy::new(utoipa_config::Config::read_from_file);

#[proc_macro_derive(ToSchema, attributes(schema))]
/// Generate reusable OpenAPI schema to be used
/// together with [`OpenApi`][openapi_derive].
///
/// This is `#[derive]` implementation for [`ToSchema`][to_schema] trait. The macro accepts one
/// `schema`
/// attribute optionally which can be used to enhance generated documentation. The attribute can be placed
/// at item level or field and variant levels in structs and enum.
///
/// You can use the Rust's own `#[deprecated]` attribute on any struct, enum or field to mark it as deprecated and it will
/// reflect to the generated OpenAPI spec.
///
/// `#[deprecated]` attribute supports adding additional details such as a reason and or since version but this is is not supported in
/// OpenAPI. OpenAPI has only a boolean flag to determine deprecation. While it is totally okay to declare deprecated with reason
/// `#[deprecated  = "There is better way to do this"]` the reason would not render in OpenAPI spec.
///
/// Doc comments on fields will resolve to field descriptions in generated OpenAPI doc. On struct
/// level doc comments will resolve to object descriptions.
///
/// Schemas derived with `ToSchema` will be automatically collected from usage. In case of looping
/// schema tree _`no_recursion`_ attribute must be used to break from recurring into infinite loop.
/// See [more details from example][derive@ToSchema#examples]. All arguments of generic schemas
/// must implement `ToSchema` trait.
///
/// ```rust
/// /// This is a pet
/// #[derive(utoipa::ToSchema)]
/// struct Pet {
///     /// Name for your pet
///     name: String,
/// }
/// ```
///
/// # Named Field Struct Optional Configuration Options for `#[schema(...)]`
///
/// * `description = ...` Can be literal string or Rust expression e.g. _`const`_ reference or
///   `include_str!(...)` statement. This can be used to override **default** description what is
///   resolved from doc comments of the type.
/// * `example = ...` Can be any value e.g. literal, method reference or _`json!(...)`_.
///   **Deprecated since OpenAPI 3.0, using `examples` is preferred instead.**
/// * `examples(..., ...)` Comma separated list defining multiple _`examples`_ for the schema. Each
///   _`example`_ Can be any value e.g. literal, method reference or _`json!(...)`_.
/// * `xml(...)` Can be used to define [`Xml`][xml] object properties applicable to Structs.
/// * `title = ...` Literal string value. Can be used to define title for struct in OpenAPI
///   document. Some OpenAPI code generation libraries also use this field as a name for the
///   struct.
/// * `rename_all = ...` Supports same syntax as _serde_ _`rename_all`_ attribute. Will rename all fields
///   of the structs accordingly. If both _serde_ `rename_all` and _schema_ _`rename_all`_ are defined
///   __serde__ will take precedence.
/// * `as = ...` Can be used to define alternative path and name for the schema what will be used in
///   the OpenAPI. E.g _`as = path::to::Pet`_. This would make the schema appear in the generated
///   OpenAPI spec as _`path.to.Pet`_. This same name will be used throughout the OpenAPI generated
///   with `utoipa` when the type is being referenced in [`OpenApi`][openapi_derive] derive macro
///   or in [`utoipa::path(...)`][path_macro] macro.
/// * `bound = ...` Can be used to override default trait bounds on generated `impl`s.
///   See [Generic schemas section](#generic-schemas) below for more details.
/// * `default` Can be used to populate default values on all fields using the struct's
///   [`Default`] implementation.
/// * `deprecated` Can be used to mark all fields as deprecated in the generated OpenAPI spec but
///   not in the code. If you'd like to mark the fields as deprecated in the code as well use
///   Rust's own `#[deprecated]` attribute instead.
/// * `max_properties = ...` Can be used to define maximum number of properties this struct can
///   contain. Value must be a number.
/// * `min_properties = ...` Can be used to define minimum number of properties this struct can
///   contain. Value must be a number.
///* `no_recursion` Is used to break from recursion in case of looping schema tree e.g. `Pet` ->
///  `Owner` -> `Pet`. _`no_recursion`_ attribute must be used within `Ower` type not to allow
///  recurring into `Pet`. Failing to do so will cause infinite loop and runtime **panic**. On
///  struct level the _`no_recursion`_ rule will be applied to all of its fields.
///
/// ## Named Fields Optional Configuration Options for `#[schema(...)]`
///
/// * `example = ...` Can be any value e.g. literal, method reference or _`json!(...)`_.
///   **Deprecated since OpenAPI 3.0, using `examples` is preferred instead.**
/// * `examples(..., ...)` Comma separated list defining multiple _`examples`_ for the schema. Each
///   _`example`_ Can be any value e.g. literal, method reference or _`json!(...)`_.
/// * `default = ...` Can be any value e.g. literal, method reference or _`json!(...)`_.
/// * `format = ...` May either be variant of the [`KnownFormat`][known_format] enum, or otherwise
///   an open value as a string. By default the format is derived from the type of the property
///   according OpenApi spec.
/// * `write_only` Defines property is only used in **write** operations *POST,PUT,PATCH* but not in *GET*
/// * `read_only` Defines property is only used in **read** operations *GET* but not in *POST,PUT,PATCH*
/// * `xml(...)` Can be used to define [`Xml`][xml] object properties applicable to named fields.
///    See configuration options at xml attributes of [`ToSchema`][to_schema_xml]
/// * `value_type = ...` Can be used to override default type derived from type of the field used in OpenAPI spec.
///   This is useful in cases where the default type does not correspond to the actual type e.g. when
///   any third-party types are used which are not [`ToSchema`][to_schema]s nor [`primitive` types][primitive].
///   The value can be any Rust type what normally could be used to serialize to JSON, or either virtual type _`Object`_
///   or _`Value`_.
///   _`Object`_ will be rendered as generic OpenAPI object _(`type: object`)_.
///   _`Value`_ will be rendered as any OpenAPI value (i.e. no `type` restriction).
/// * `inline` If the type of this field implements [`ToSchema`][to_schema], then the schema definition
///   will be inlined. **warning:** Don't use this for recursive data types!
///   
///   **Note!**<br>Using `inline` with generic arguments might lead to incorrect spec generation.
///   This is due to the fact that during compilation we cannot know how to treat the generic
///   argument and there is difference whether it is a primitive type or another generic type.
/// * `required = ...` Can be used to enforce required status for the field. [See
///   rules][derive@ToSchema#field-nullability-and-required-rules]
/// * `nullable` Defines property is nullable (note this is different to non-required).
/// * `rename = ...` Supports same syntax as _serde_ _`rename`_ attribute. Will rename field
///   accordingly. If both _serde_ `rename` and _schema_ _`rename`_ are defined __serde__ will take
///   precedence.
/// * `multiple_of = ...` Can be used to define multiplier for a value. Value is considered valid
///   division will result an `integer`. Value must be strictly above _`0`_.
/// * `maximum = ...` Can be used to define inclusive upper bound to a `number` value.
/// * `minimum = ...` Can be used to define inclusive lower bound to a `number` value.
/// * `exclusive_maximum = ...` Can be used to define exclusive upper bound to a `number` value.
/// * `exclusive_minimum = ...` Can be used to define exclusive lower bound to a `number` value.
/// * `max_length = ...` Can be used to define maximum length for `string` types.
/// * `min_length = ...` Can be used to define minimum length for `string` types.
/// * `pattern = ...` Can be used to define valid regular expression in _ECMA-262_ dialect the field value must match.
/// * `max_items = ...` Can be used to define maximum items allowed for `array` fields. Value must
///   be non-negative integer.
/// * `min_items = ...` Can be used to define minimum items allowed for `array` fields. Value must
///   be non-negative integer.
/// * `schema_with = ...` Use _`schema`_ created by provided function reference instead of the
///   default derived _`schema`_. The function must match to `fn() -> Into<RefOr<Schema>>`. It does
///   not accept arguments and must return anything that can be converted into `RefOr<Schema>`.
/// * `additional_properties = ...` Can be used to define free form types for maps such as
///   [`HashMap`](std::collections::HashMap) and [`BTreeMap`](std::collections::BTreeMap).
///   Free form type enables use of arbitrary types within map values.
///   Supports formats _`additional_properties`_ and _`additional_properties = true`_.
/// * `deprecated` Can be used to mark the field as deprecated in the generated OpenAPI spec but
///   not in the code. If you'd like to mark the field as deprecated in the code as well use
///   Rust's own `#[deprecated]` attribute instead.
/// * `content_encoding = ...` Can be used to define content encoding used for underlying schema object.
///   See [`Object::content_encoding`][schema_object_encoding]
/// * `content_media_type = ...` Can be used to define MIME type of a string for underlying schema object.
///   See [`Object::content_media_type`][schema_object_media_type]
///* `ignore` or `ignore = ...` Can be used to skip the field from being serialized to OpenAPI schema. It accepts either a literal `bool` value
///   or a path to a function that returns `bool` (`Fn() -> bool`).
///* `no_recursion` Is used to break from recursion in case of looping schema tree e.g. `Pet` ->
///  `Owner` -> `Pet`. _`no_recursion`_ attribute must be used within `Ower` type not to allow
///  recurring into `Pet`. Failing to do so will cause infinite loop and runtime **panic**.
///
/// #### Field nullability and required rules
///
/// Field is considered _`required`_ if
/// * it is not `Option` field
/// * and it does not have _`skip_serializing_if`_ property
/// * and it does not have _`serde_with`_ _[`double_option`](https://docs.rs/serde_with/latest/serde_with/rust/double_option/index.html)_
/// * and it does not have default value provided with serde _`default`_
///   attribute
///
/// Field is considered _`nullable`_ when field type is _`Option`_.
///
/// ## Xml attribute Configuration Options
///
/// * `xml(name = "...")` Will set name for property or type.
/// * `xml(namespace = "...")` Will set namespace for xml element which needs to be valid uri.
/// * `xml(prefix = "...")` Will set prefix for name.
/// * `xml(attribute)` Will translate property to xml attribute instead of xml element.
/// * `xml(wrapped)` Will make wrapped xml element.
/// * `xml(wrapped(name = "wrap_name"))` Will override the wrapper elements name.
///
/// See [`Xml`][xml] for more details.
///
/// # Unnamed Field Struct Optional Configuration Options for `#[schema(...)]`
///
/// * `description = ...` Can be literal string or Rust expression e.g. [_`const`_][const] reference or
///   `include_str!(...)` statement. This can be used to override **default** description what is
///   resolved from doc comments of the type.
/// * `example = ...` Can be any value e.g. literal, method reference or _`json!(...)`_.
///   **Deprecated since OpenAPI 3.0, using `examples` is preferred instead.**
/// * `examples(..., ...)` Comma separated list defining multiple _`examples`_ for the schema. Each
///   _`example`_ Can be any value e.g. literal, method reference or _`json!(...)`_.
/// * `default = ...` Can be any value e.g. literal, method reference or _`json!(...)`_. If no value
///   is specified, and the struct has only one field, the field's default value in the schema will be
///   set from the struct's [`Default`] implementation.
/// * `format = ...` May either be variant of the [`KnownFormat`][known_format] enum, or otherwise
///   an open value as a string. By default the format is derived from the type of the property
///   according OpenApi spec.
/// * `value_type = ...` Can be used to override default type derived from type of the field used in OpenAPI spec.
///   This is useful in cases where the default type does not correspond to the actual type e.g. when
///   any third-party types are used which are not [`ToSchema`][to_schema]s nor [`primitive` types][primitive].
///   The value can be any Rust type what normally could be used to serialize to JSON or either virtual type _`Object`_
///   or _`Value`_.
///   _`Object`_ will be rendered as generic OpenAPI object _(`type: object`)_.
///   _`Value`_ will be rendered as any OpenAPI value (i.e. no `type` restriction).
/// * `title = ...` Literal string value. Can be used to define title for struct in OpenAPI
///   document. Some OpenAPI code generation libraries also use this field as a name for the
///   struct.
/// * `as = ...` Can be used to define alternative path and name for the schema what will be used in
///   the OpenAPI. E.g _`as = path::to::Pet`_. This would make the schema appear in the generated
///   OpenAPI spec as _`path.to.Pet`_. This same name will be used throughout the OpenAPI generated
///   with `utoipa` when the type is being referenced in [`OpenApi`][openapi_derive] derive macro
///   or in [`utoipa::path(...)`][path_macro] macro.
/// * `bound = ...` Can be used to override default trait bounds on generated `impl`s.
///   See [Generic schemas section](#generic-schemas) below for more details.
/// * `deprecated` Can be used to mark the field as deprecated in the generated OpenAPI spec but
///   not in the code. If you'd like to mark the field as deprecated in the code as well use
///   Rust's own `#[deprecated]` attribute instead.
/// * `content_encoding = ...` Can be used to define content encoding used for underlying schema object.
///   See [`Object::content_encoding`][schema_object_encoding]
/// * `content_media_type = ...` Can be used to define MIME type of a string for underlying schema object.
///   See [`Object::content_media_type`][schema_object_media_type]
///* `no_recursion` Is used to break from recursion in case of looping schema tree e.g. `Pet` ->
///  `Owner` -> `Pet`. _`no_recursion`_ attribute must be used within `Ower` type not to allow
///  recurring into `Pet`. Failing to do so will cause infinite loop and runtime **panic**.
///
/// # Enum Optional Configuration Options for `#[schema(...)]`
///
/// ## Plain Enum having only `Unit` variants Optional Configuration Options for `#[schema(...)]`
///
/// * `description = ...` Can be literal string or Rust expression e.g. [_`const`_][const] reference or
///   `include_str!(...)` statement. This can be used to override **default** description what is
///   resolved from doc comments of the type.
/// * `example = ...` Can be any value e.g. literal, method reference or _`json!(...)`_.
///   **Deprecated since OpenAPI 3.0, using `examples` is preferred instead.**
/// * `examples(..., ...)` Comma separated list defining multiple _`examples`_ for the schema. Each
///   _`example`_ Can be any value e.g. literal, method reference or _`json!(...)`_.
/// * `default = ...` Can be any value e.g. literal, method reference or _`json!(...)`_.
/// * `title = ...` Literal string value. Can be used to define title for enum in OpenAPI
///   document. Some OpenAPI code generation libraries also use this field as a name for the
///   enum.
/// * `rename_all = ...` Supports same syntax as _serde_ _`rename_all`_ attribute. Will rename all
///   variants of the enum accordingly. If both _serde_ `rename_all` and _schema_ _`rename_all`_
///   are defined __serde__ will take precedence.
/// * `as = ...` Can be used to define alternative path and name for the schema what will be used in
///   the OpenAPI. E.g _`as = path::to::Pet`_. This would make the schema appear in the generated
///   OpenAPI spec as _`path.to.Pet`_. This same name will be used throughout the OpenAPI generated
///   with `utoipa` when the type is being referenced in [`OpenApi`][openapi_derive] derive macro
///   or in [`utoipa::path(...)`][path_macro] macro.
/// * `bound = ...` Can be used to override default trait bounds on generated `impl`s.
///   See [Generic schemas section](#generic-schemas) below for more details.
/// * `deprecated` Can be used to mark the enum as deprecated in the generated OpenAPI spec but
///   not in the code. If you'd like to mark the enum as deprecated in the code as well use
///   Rust's own `#[deprecated]` attribute instead.
///
/// ### Plain Enum Variant Optional Configuration Options for `#[schema(...)]`
///
/// * `rename = ...` Supports same syntax as _serde_ _`rename`_ attribute. Will rename variant
///   accordingly. If both _serde_ `rename` and _schema_ _`rename`_ are defined __serde__ will take
///   precedence. **Note!** [`Repr enum`][macro@ToSchema#repr-attribute-support] variant does not
///   support _`rename`_.
///
/// ## Mixed Enum Optional Configuration Options for `#[schema(...)]`
///
/// * `description = ...` Can be literal string or Rust expression e.g. [_`const`_][const] reference or
///   `include_str!(...)` statement. This can be used to override **default** description what is
///   resolved from doc comments of the type.
/// * `example = ...` Can be any value e.g. literal, method reference or _`json!(...)`_.
///   **Deprecated since OpenAPI 3.0, using `examples` is preferred instead.**
/// * `examples(..., ...)` Comma separated list defining multiple _`examples`_ for the schema. Each
/// * `default = ...` Can be any value e.g. literal, method reference or _`json!(...)`_.
/// * `title = ...` Literal string value. Can be used to define title for enum in OpenAPI
///   document. Some OpenAPI code generation libraries also use this field as a name for the
///   enum.
/// * `rename_all = ...` Supports same syntax as _serde_ _`rename_all`_ attribute. Will rename all
///   variants of the enum accordingly. If both _serde_ `rename_all` and _schema_ _`rename_all`_
///   are defined __serde__ will take precedence.
/// * `as = ...` Can be used to define alternative path and name for the schema what will be used in
///   the OpenAPI. E.g _`as = path::to::Pet`_. This would make the schema appear in the generated
///   OpenAPI spec as _`path.to.Pet`_. This same name will be used throughout the OpenAPI generated
///   with `utoipa` when the type is being referenced in [`OpenApi`][openapi_derive] derive macro
///   or in [`utoipa::path(...)`][path_macro] macro.
/// * `bound = ...` Can be used to override default trait bounds on generated `impl`s.
///   See [Generic schemas section](#generic-schemas) below for more details.
/// * `deprecated` Can be used to mark the enum as deprecated in the generated OpenAPI spec but
///   not in the code. If you'd like to mark the enum as deprecated in the code as well use
///   Rust's own `#[deprecated]` attribute instead.
/// * `discriminator = ...` or `discriminator(...)` Can be used to define OpenAPI discriminator
///   field for enums with single unnamed _`ToSchema`_ reference field. See the [discriminator
///   syntax][derive@ToSchema#schemadiscriminator-syntax].
///* `no_recursion` Is used to break from recursion in case of looping schema tree e.g. `Pet` ->
///  `Owner` -> `Pet`. _`no_recursion`_ attribute must be used within `Ower` type not to allow
///  recurring into `Pet`. Failing to do so will cause infinite loop and runtime **panic**. On
///  enum level the _`no_recursion`_ rule will be applied to all of its variants.
///
///  ### `#[schema(discriminator)]` syntax
///
///  Discriminator can **only** be used with enums having **`#[serde(untagged)]`** attribute and
///  each variant must have only one unnamed field schema reference to type implementing
///  _`ToSchema`_.
///
///  **Simple form `discriminator = ...`**
///
///  Can be literal string or expression e.g. [_`const`_][const] reference. It can be defined as
///  _`discriminator = "value"`_ where the assigned value is the
///  discriminator field that must exists in each variant referencing schema.
///
/// **Complex form `discriminator(...)`**
///
/// * `property_name = ...` Can be literal string or expression e.g. [_`const`_][const] reference.
/// * mapping `key` Can be literal string or expression e.g. [_`const`_][const] reference.
/// * mapping `value` Can be literal string or expression e.g. [_`const`_][const] reference.
///
/// Additionally discriminator can be defined with custom mappings as show below. The _`mapping`_
/// values defines _**key = value**_ pairs where _**key**_ is the expected value for _**property_name**_ field
/// and _**value**_ is schema to map.
/// ```text
/// discriminator(property_name = "my_field", mapping(
///      ("value" = "#/components/schemas/Schema1"),
///      ("value2" = "#/components/schemas/Schema2")
/// ))
/// ```
///
/// ### Mixed Enum Named Field Variant Optional Configuration Options for `#[serde(schema)]`
///
/// * `example = ...` Can be any value e.g. literal, method reference or _`json!(...)`_.
///   **Deprecated since OpenAPI 3.0, using `examples` is preferred instead.**
/// * `examples(..., ...)` Comma separated list defining multiple _`examples`_ for the schema. Each
/// * `default = ...` Can be any value e.g. literal, method reference or _`json!(...)`_.
/// * `title = ...` Literal string value. Can be used to define title for enum variant in OpenAPI
///   document. Some OpenAPI code generation libraries also use this field as a name for the
///   enum.
/// * `xml(...)` Can be used to define [`Xml`][xml] object properties applicable to Structs.
/// * `rename = ...` Supports same syntax as _serde_ _`rename`_ attribute. Will rename variant
///   accordingly. If both _serde_ `rename` and _schema_ _`rename`_ are defined __serde__ will take
///   precedence.
/// * `rename_all = ...` Supports same syntax as _serde_ _`rename_all`_ attribute. Will rename all
///   variant fields accordingly. If both _serde_ `rename_all` and _schema_ _`rename_all`_
///   are defined __serde__ will take precedence.
/// * `deprecated` Can be used to mark the enum as deprecated in the generated OpenAPI spec but
///   not in the code. If you'd like to mark the enum as deprecated in the code as well use
///   Rust's own `#[deprecated]` attribute instead.
/// * `max_properties = ...` Can be used to define maximum number of properties this struct can
///   contain. Value must be a number.
/// * `min_properties = ...` Can be used to define minimum number of properties this struct can
///   contain. Value must be a number.
///* `no_recursion` Is used to break from recursion in case of looping schema tree e.g. `Pet` ->
///  `Owner` -> `Pet`. _`no_recursion`_ attribute must be used within `Ower` type not to allow
///  recurring into `Pet`. Failing to do so will cause infinite loop and runtime **panic**. On
///  named field variant level the _`no_recursion`_ rule will be applied to all of its fields.
///
/// ## Mixed Enum Unnamed Field Variant Optional Configuration Options for `#[serde(schema)]`
///
/// * `example = ...` Can be any value e.g. literal, method reference or _`json!(...)`_.
///   **Deprecated since OpenAPI 3.0, using `examples` is preferred instead.**
/// * `examples(..., ...)` Comma separated list defining multiple _`examples`_ for the schema. Each
///   _`example`_ Can be any value e.g. literal, method reference or _`json!(...)`_.
/// * `default = ...` Can be any value e.g. literal, method reference or _`json!(...)`_.
/// * `title = ...` Literal string value. Can be used to define title for enum variant in OpenAPI
///   document. Some OpenAPI code generation libraries also use this field as a name for the
///   struct.
/// * `rename = ...` Supports same syntax as _serde_ _`rename`_ attribute. Will rename variant
///   accordingly. If both _serde_ `rename` and _schema_ _`rename`_ are defined __serde__ will take
///   precedence.
/// * `format = ...` May either be variant of the [`KnownFormat`][known_format] enum, or otherwise
///   an open value as a string. By default the format is derived from the type of the property
///   according OpenApi spec.
/// * `value_type = ...` Can be used to override default type derived from type of the field used in OpenAPI spec.
///   This is useful in cases where the default type does not correspond to the actual type e.g. when
///   any third-party types are used which are not [`ToSchema`][to_schema]s nor [`primitive` types][primitive].
///   The value can be any Rust type what normally could be used to serialize to JSON or either virtual type _`Object`_
///   or _`Value`_.
///   _`Object`_ will be rendered as generic OpenAPI object _(`type: object`)_.
///   _`Value`_ will be rendered as any OpenAPI value (i.e. no `type` restriction).
/// * `deprecated` Can be used to mark the field as deprecated in the generated OpenAPI spec but
///   not in the code. If you'd like to mark the field as deprecated in the code as well use
///   Rust's own `#[deprecated]` attribute instead.
///* `no_recursion` Is used to break from recursion in case of looping schema tree e.g. `Pet` ->
///  `Owner` -> `Pet`. _`no_recursion`_ attribute must be used within `Ower` type not to allow
///  recurring into `Pet`. Failing to do so will cause infinite loop and runtime **panic**.
///
/// #### Mixed Enum Unnamed Field Variant's Field Configuration Options
///
/// * `inline` If the type of this field implements [`ToSchema`][to_schema], then the schema definition
///   will be inlined. **warning:** Don't use this for recursive data types!
///
///   **Note!**<br>Using `inline` with generic arguments might lead to incorrect spec generation.
///   This is due to the fact that during compilation we cannot know how to treat the generic
///   argument and there is difference whether it is a primitive type or another generic type.
///
///   _**Inline unnamed field variant schemas.**_
///   ```rust
///   # use utoipa::ToSchema;
///   # #[derive(ToSchema)]
///   # enum Number {
///   #     One,
///   # }
///   #
///   # #[derive(ToSchema)]
///   # enum Color {
///   #     Spade,
///   # }
///    #[derive(ToSchema)]
///    enum Card {
///        Number(#[schema(inline)] Number),
///        Color(#[schema(inline)] Color),
///    }
///   ```
///
/// ## Mixed Enum Unit Field Variant Optional Configuration Options for `#[serde(schema)]`
///
/// * `example = ...` Can be any value e.g. literal, method reference or _`json!(...)`_.
///   **Deprecated since OpenAPI 3.0, using `examples` is preferred instead.**
/// * `examples(..., ...)` Comma separated list defining multiple _`examples`_ for the schema. Each
///   _`example`_ Can be any value e.g. literal, method reference or _`json!(...)`_.
/// * `title = ...` Literal string value. Can be used to define title for enum variant in OpenAPI
///   document. Some OpenAPI code generation libraries also use this field as a name for the
///   struct.
/// * `rename = ...` Supports same syntax as _serde_ _`rename`_ attribute. Will rename variant
///   accordingly. If both _serde_ `rename` and _schema_ _`rename`_ are defined __serde__ will take
///   precedence.
/// * `deprecated` Can be used to mark the field as deprecated in the generated OpenAPI spec but
///   not in the code. If you'd like to mark the field as deprecated in the code as well use
///   Rust's own `#[deprecated]` attribute instead.
///
/// # Partial `#[serde(...)]` attributes support
///
/// ToSchema derive has partial support for [serde attributes]. These supported attributes will reflect to the
/// generated OpenAPI doc. For example if _`#[serde(skip)]`_ is defined the attribute will not show up in the OpenAPI spec at all since it will not never
/// be serialized anyway. Similarly the _`rename`_ and _`rename_all`_ will reflect to the generated OpenAPI doc.
///
/// * `rename_all = "..."` Supported at the container level.
/// * `rename = "..."` Supported **only** at the field or variant level.
/// * `skip = "..."` Supported  **only** at the field or variant level.
/// * `skip_serializing = "..."` Supported  **only** at the field or variant level.
/// * `skip_deserializing = "..."` Supported  **only** at the field or variant level.
/// * `skip_serializing_if = "..."` Supported  **only** at the field level.
/// * `with = ...` Supported **only at field level.**
/// * `tag = "..."` Supported at the container level.
/// * `content = "..."` Supported at the container level, allows [adjacently-tagged enums](https://serde.rs/enum-representations.html#adjacently-tagged).
///   This attribute requires that a `tag` is present, otherwise serde will trigger a compile-time
///   failure.
/// * `untagged` Supported at the container level. Allows [untagged
///    enum representation](https://serde.rs/enum-representations.html#untagged).
/// * `default` Supported at the container level and field level according to [serde attributes].
/// * `deny_unknown_fields` Supported at the container level.
/// * `flatten` Supported at the field level.
///
/// Other _`serde`_ attributes works as is but does not have any effect on the generated OpenAPI doc.
///
/// **Note!** `tag` attribute has some limitations like it cannot be used with **tuple types**. See more at
/// [enum representation docs](https://serde.rs/enum-representations.html).
///
/// **Note!** `with` attribute is used in tandem with [serde_with](https://github.com/jonasbb/serde_with) to recognize
/// _[`double_option`](https://docs.rs/serde_with/latest/serde_with/rust/double_option/index.html)_ from **field value**.
/// _`double_option`_ is **only** supported attribute from _`serde_with`_ crate.
///
/// ```rust
/// # use serde::Serialize;
/// # use utoipa::ToSchema;
/// #[derive(Serialize, ToSchema)]
/// struct Foo(String);
///
/// #[derive(Serialize, ToSchema)]
/// #[serde(rename_all = "camelCase")]
/// enum Bar {
///     UnitValue,
///     #[serde(rename_all = "camelCase")]
///     NamedFields {
///         #[serde(rename = "id")]
///         named_id: &'static str,
///         name_list: Option<Vec<String>>
///     },
///     UnnamedFields(Foo),
///     #[serde(skip)]
///     SkipMe,
/// }
/// ```
///
/// _**Add custom `tag` to change JSON representation to be internally tagged.**_
/// ```rust
/// # use serde::Serialize;
/// # use utoipa::ToSchema;
/// #[derive(Serialize, ToSchema)]
/// struct Foo(String);
///
/// #[derive(Serialize, ToSchema)]
/// #[serde(tag = "tag")]
/// enum Bar {
///     UnitValue,
///     NamedFields {
///         id: &'static str,
///         names: Option<Vec<String>>
///     },
/// }
/// ```
///
/// _**Add serde `default` attribute for MyValue struct. Similarly `default` could be added to
/// individual fields as well. If `default` is given the field's affected will be treated
/// as optional.**_
/// ```rust
///  #[derive(utoipa::ToSchema, serde::Deserialize, Default)]
///  #[serde(default)]
///  struct MyValue {
///      field: String
///  }
/// ```
///
/// # `#[repr(...)]` attribute support
///
/// [Serde repr](https://github.com/dtolnay/serde-repr) allows field-less enums be represented by
/// their numeric value.
///
/// * `repr(u*)` for unsigned integer.
/// * `repr(i*)` for signed integer.
///
/// **Supported schema attributes**
///
/// * `example = ...` Can be any value e.g. literal, method reference or _`json!(...)`_.
///   **Deprecated since OpenAPI 3.0, using `examples` is preferred instead.**
/// * `examples(..., ...)` Comma separated list defining multiple _`examples`_ for the schema. Each
///   _`example`_ Can be any value e.g. literal, method reference or _`json!(...)`_.
/// * `title = ...` Literal string value. Can be used to define title for enum in OpenAPI
///   document. Some OpenAPI code generation libraries also use this field as a name for the
///   struct.
/// * `as = ...` Can be used to define alternative path and name for the schema what will be used in
///   the OpenAPI. E.g _`as = path::to::Pet`_. This would make the schema appear in the generated
///   OpenAPI spec as _`path.to.Pet`_. This same name will be used throughout the OpenAPI generated
///   with `utoipa` when the type is being referenced in [`OpenApi`][openapi_derive] derive macro
///   or in [`utoipa::path(...)`][path_macro] macro.
///
/// _**Create enum with numeric values.**_
/// ```rust
/// # use utoipa::ToSchema;
/// #[derive(ToSchema)]
/// #[repr(u8)]
/// #[schema(default = default_value, example = 2)]
/// enum Mode {
///     One = 1,
///     Two,
///  }
///
/// fn default_value() -> u8 {
///     1
/// }
/// ```
///
/// _**You can use `skip` and `tag` attributes from serde.**_
/// ```rust
/// # use utoipa::ToSchema;
/// #[derive(ToSchema, serde::Serialize)]
/// #[repr(i8)]
/// #[serde(tag = "code")]
/// enum ExitCode {
///     Error = -1,
///     #[serde(skip)]
///     Unknown = 0,
///     Ok = 1,
///  }
/// ```
///
/// # Generic schemas
///
/// Utoipa supports full set of deeply nested generics as shown below. The type will implement
/// [`ToSchema`][to_schema] if and only if all the generic types implement `ToSchema` by default.
/// That is in Rust `impl<T> ToSchema for MyType<T> where T: Schema { ... }`.
/// You can also specify `bound = ...` on the item to override the default auto bounds.
///
/// The _`as = ...`_ attribute is used to define the prefixed or alternative name for the component
/// in question. This same name will be used throughout the OpenAPI generated with `utoipa` when
/// the type is being referenced in [`OpenApi`][openapi_derive] derive macro or in [`utoipa::path(...)`][path_macro] macro.
///
/// ```rust
/// # use utoipa::ToSchema;
/// # use std::borrow::Cow;
///  #[derive(ToSchema)]
///  #[schema(as = path::MyType<T>)]
///  struct Type<T> {
///      t: T,
///  }
///
///  #[derive(ToSchema)]
///  struct Person<'p, T: Sized, P> {
///      id: usize,
///      name: Option<Cow<'p, str>>,
///      field: T,
///      t: P,
///  }
///
///  #[derive(ToSchema)]
///  #[schema(as = path::to::PageList)]
///  struct Page<T> {
///      total: usize,
///      page: usize,
///      pages: usize,
///      items: Vec<T>,
///  }
///
///  #[derive(ToSchema)]
///  #[schema(as = path::to::Element<T>)]
///  enum E<T> {
///      One(T),
///      Many(Vec<T>),
///  }
/// ```
/// When generic types are registered to the `OpenApi` the full type declaration must be provided.
/// See the full example in test [schema_generics.rs](https://github.com/juhaku/utoipa/blob/master/utoipa-gen/tests/schema_generics.rs)
///
/// # Examples
///
/// _**Simple example of a Pet with descriptions and object level example.**_
/// ```rust
/// # use utoipa::ToSchema;
/// /// This is a pet.
/// #[derive(ToSchema)]
/// #[schema(example = json!({"name": "bob the cat", "id": 0}))]
/// struct Pet {
///     /// Unique id of a pet.
///     id: u64,
///     /// Name of a pet.
///     name: String,
///     /// Age of a pet if known.
///     age: Option<i32>,
/// }
/// ```
///
/// _**The `schema` attribute can also be placed at field level as follows.**_
/// ```rust
/// # use utoipa::ToSchema;
/// #[derive(ToSchema)]
/// struct Pet {
///     #[schema(example = 1, default = 0)]
///     id: u64,
///     name: String,
///     age: Option<i32>,
/// }
/// ```
///
/// _**You can also use method reference for attribute values.**_
/// ```rust
/// # use utoipa::ToSchema;
/// #[derive(ToSchema)]
/// struct Pet {
///     #[schema(example = u64::default, default = u64::default)]
///     id: u64,
///     #[schema(default = default_name)]
///     name: String,
///     age: Option<i32>,
/// }
///
/// fn default_name() -> String {
///     "bob".to_string()
/// }
/// ```
///
/// _**For enums and unnamed field structs you can define `schema` at type level.**_
/// ```rust
/// # use utoipa::ToSchema;
/// #[derive(ToSchema)]
/// #[schema(example = "Bus")]
/// enum VehicleType {
///     Rocket, Car, Bus, Submarine
/// }
/// ```
///
/// _**Also you write mixed enum combining all above types.**_
/// ```rust
/// # use utoipa::ToSchema;
/// #[derive(ToSchema)]
/// enum ErrorResponse {
///     InvalidCredentials,
///     #[schema(default = String::default, example = "Pet not found")]
///     NotFound(String),
///     System {
///         #[schema(example = "Unknown system failure")]
///         details: String,
///     }
/// }
/// ```
///
/// _**It is possible to specify the title of each variant to help generators create named structures.**_
/// ```rust
/// # use utoipa::ToSchema;
/// #[derive(ToSchema)]
/// enum ErrorResponse {
///     #[schema(title = "InvalidCredentials")]
///     InvalidCredentials,
///     #[schema(title = "NotFound")]
///     NotFound(String),
/// }
/// ```
///
/// _**Use `xml` attribute to manipulate xml output.**_
/// ```rust
/// # use utoipa::ToSchema;
/// #[derive(ToSchema)]
/// #[schema(xml(name = "user", prefix = "u", namespace = "https://user.xml.schema.test"))]
/// struct User {
///     #[schema(xml(attribute, prefix = "u"))]
///     id: i64,
///     #[schema(xml(name = "user_name", prefix = "u"))]
///     username: String,
///     #[schema(xml(wrapped(name = "linkList"), name = "link"))]
///     links: Vec<String>,
///     #[schema(xml(wrapped, name = "photo_url"))]
///     photos_urls: Vec<String>
/// }
/// ```
///
/// _**Use of Rust's own `#[deprecated]` attribute will reflect to generated OpenAPI spec.**_
/// ```rust
/// # use utoipa::ToSchema;
/// #[derive(ToSchema)]
/// #[deprecated]
/// struct User {
///     id: i64,
///     username: String,
///     links: Vec<String>,
///     #[deprecated]
///     photos_urls: Vec<String>
/// }
/// ```
///
/// _**Enforce type being used in OpenAPI spec to [`String`] with `value_type` and set format to octet stream
/// with [`SchemaFormat::KnownFormat(KnownFormat::Binary)`][binary].**_
/// ```rust
/// # use utoipa::ToSchema;
/// #[derive(ToSchema)]
/// struct Post {
///     id: i32,
///     #[schema(value_type = String, format = Binary)]
///     value: Vec<u8>,
/// }
/// ```
///
/// _**Enforce type being used in OpenAPI spec to [`String`] with `value_type` option.**_
/// ```rust
/// # use utoipa::ToSchema;
/// #[derive(ToSchema)]
/// #[schema(value_type = String)]
/// struct Value(i64);
/// ```
///
/// _**Override the `Bar` reference with a `custom::NewBar` reference.**_
/// ```rust
/// # use utoipa::ToSchema;
/// #  mod custom {
/// #      #[derive(utoipa::ToSchema)]
/// #      pub struct NewBar;
/// #  }
/// #
/// # struct Bar;
/// #[derive(ToSchema)]
/// struct Value {
///     #[schema(value_type = custom::NewBar)]
///     field: Bar,
/// };
/// ```
///
/// _**Use a virtual `Object` type to render generic `object` _(`type: object`)_ in OpenAPI spec.**_
/// ```rust
/// # use utoipa::ToSchema;
/// # mod custom {
/// #    struct NewBar;
/// # }
/// #
/// # struct Bar;
/// #[derive(ToSchema)]
/// struct Value {
///     #[schema(value_type = Object)]
///     field: Bar,
/// };
/// ```
/// More examples for _`value_type`_ in [`IntoParams` derive docs][into_params].
///
/// _**Serde `rename` / `rename_all` will take precedence over schema `rename` / `rename_all`.**_
/// ```rust
/// #[derive(utoipa::ToSchema, serde::Deserialize)]
/// #[serde(rename_all = "lowercase")]
/// #[schema(rename_all = "UPPERCASE")]
/// enum Random {
///     #[serde(rename = "string_value")]
///     #[schema(rename = "custom_value")]
///     String(String),
///
///     Number {
///         id: i32,
///     }
/// }
/// ```
///
/// _**Add `title` to the enum.**_
/// ```rust
/// #[derive(utoipa::ToSchema)]
/// #[schema(title = "UserType")]
/// enum UserType {
///     Admin,
///     Moderator,
///     User,
/// }
/// ```
///
/// _**Example with validation attributes.**_
/// ```rust
/// #[derive(utoipa::ToSchema)]
/// struct Item {
///     #[schema(maximum = 10, minimum = 5, multiple_of = 2.5)]
///     id: i32,
///     #[schema(max_length = 10, min_length = 5, pattern = "[a-z]*")]
///     value: String,
///     #[schema(max_items = 5, min_items = 1)]
///     items: Vec<String>,
/// }
/// ````
///
/// _**Use `schema_with` to manually implement schema for a field.**_
/// ```rust
/// # use utoipa::openapi::schema::{Object, ObjectBuilder};
/// fn custom_type() -> Object {
///     ObjectBuilder::new()
///         .schema_type(utoipa::openapi::schema::Type::String)
///         .format(Some(utoipa::openapi::SchemaFormat::Custom(
///             "email".to_string(),
///         )))
///         .description(Some("this is the description"))
///         .build()
/// }
///
/// #[derive(utoipa::ToSchema)]
/// struct Value {
///     #[schema(schema_with = custom_type)]
///     id: String,
/// }
/// ```
///
/// _**Use `as` attribute to change the name and the path of the schema in the generated OpenAPI
/// spec.**_
/// ```rust
///  #[derive(utoipa::ToSchema)]
///  #[schema(as = api::models::person::Person)]
///  struct Person {
///      name: String,
///  }
/// ```
///
/// _**Use `bound` attribute to override the default impl bounds.**_
///
/// `bound = ...` accepts a string containing zero or more where-predicates separated by comma, as
/// the similar syntax to [`serde(bound = ...)`](https://serde.rs/container-attrs.html#bound).
/// If `bound = ...` exists, the default auto bounds (requiring all generic types to implement
/// `ToSchema`) will not be applied anymore, and only the specified predicates are added to the
/// `where` clause of generated `impl` blocks.
///
/// ```rust
/// // Override the default bounds to only require `T: ToSchema`, ignoring unused `U`.
/// #[derive(utoipa::ToSchema, serde::Serialize)]
/// #[schema(bound = "T: utoipa::ToSchema")]
/// struct Partial<T, U> {
///     used_in_api: T,
///     #[serde(skip)]
///     not_in_api: std::marker::PhantomData<U>,
/// }
///
/// // Just remove the auto-bounds. So we got `Unused<T>: ToSchema` for any `T`.
/// #[derive(utoipa::ToSchema, serde::Serialize)]
/// #[schema(bound = "")]
/// struct Unused<T> {
///     #[serde(skip)]
///     _marker: std::marker::PhantomData<T>,
/// }
/// ```
///
/// _**Use `no_recursion` attribute to break from looping schema tree e.g. `Pet` -> `Owner` ->
/// `Pet`.**_
///
/// `no_recursion` attribute can be provided on named field of a struct, on unnamed struct or unnamed
/// enum variant. It must be provided in case of looping schema tree in order to stop recursion.
/// Failing to do so will cause runtime **panic**.
/// ```rust
/// # use utoipa::ToSchema;
/// #
/// #[derive(ToSchema)]
/// pub struct Pet {
///     name: String,
///     owner: Owner,
/// }
///
/// #[derive(ToSchema)]
/// pub struct Owner {
///     name: String,
///     #[schema(no_recursion)]
///     pets: Vec<Pet>,
/// }
/// ```
///
/// [to_schema]: trait.ToSchema.html
/// [known_format]: openapi/schema/enum.KnownFormat.html
/// [binary]: openapi/schema/enum.KnownFormat.html#variant.Binary
/// [xml]: openapi/xml/struct.Xml.html
/// [into_params]: derive.IntoParams.html
/// [primitive]: https://doc.rust-lang.org/std/primitive/index.html
/// [serde attributes]: https://serde.rs/attributes.html
/// [discriminator]: openapi/schema/struct.Discriminator.html
/// [enum_schema]: derive.ToSchema.html#enum-optional-configuration-options-for-schema
/// [openapi_derive]: derive.OpenApi.html
/// [to_schema_xml]: macro@ToSchema#xml-attribute-configuration-options
/// [schema_object_encoding]: openapi/schema/struct.Object.html#structfield.content_encoding
/// [schema_object_media_type]: openapi/schema/struct.Object.html#structfield.content_media_type
/// [path_macro]: macro@path
/// [const]: https://doc.rust-lang.org/std/keyword.const.html
pub fn derive_to_schema(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs,
        ident,
        data,
        generics,
        ..
    } = syn::parse_macro_input!(input);

    Schema::new(&data, &attrs, &ident, &generics)
        .as_ref()
        .map_or_else(Diagnostics::to_token_stream, Schema::to_token_stream)
        .into()
}

#[proc_macro_attribute]
/// Path attribute macro implements OpenAPI path for the decorated function.
///
/// This is a `#[derive]` implementation for [`Path`][path] trait. Macro accepts set of attributes that can
/// be used to configure and override default values what are resolved automatically.
///
/// You can use the Rust's own `#[deprecated]` attribute on functions to mark it as deprecated and it will
/// reflect to the generated OpenAPI spec. Only **parameters** has a special **deprecated** attribute to define them as deprecated.
///
/// `#[deprecated]` attribute supports adding additional details such as a reason and or since version but this is is not supported in
/// OpenAPI. OpenAPI has only a boolean flag to determine deprecation. While it is totally okay to declare deprecated with reason
/// `#[deprecated  = "There is better way to do this"]` the reason would not render in OpenAPI spec.
///
/// Doc comment at decorated function will be used for _`description`_ and _`summary`_ of the path.
/// First line of the doc comment will be used as the _`summary`_ while the remaining lines will be
/// used as _`description`_.
/// ```rust
/// /// This is a summary of the operation
/// ///
/// /// The rest of the doc comment will be included to operation description.
/// #[utoipa::path(get, path = "/operation")]
/// fn operation() {}
/// ```
///
/// # Path Attributes
///
/// * `operation` _**Must be first parameter!**_ Accepted values are known HTTP operations such as
///   _`get, post, put, delete, head, options, patch, trace`_.
///
/// * `method(get, head, ...)` Http methods for the operation. This allows defining multiple
///   HTTP methods at once for single operation. Either _`operation`_ or _`method(...)`_ _**must be
///   provided.**_
///
/// * `path = "..."` Must be OpenAPI format compatible str with arguments within curly braces. E.g _`{id}`_
///
/// * `impl_for = ...` Optional type to implement the [`Path`][path] trait. By default a new type
///   is used for the implementation.
///
/// * `operation_id = ...` Unique operation id for the endpoint. By default this is mapped to function name.
///   The operation_id can be any valid expression (e.g. string literals, macro invocations, variables) so long
///   as its result can be converted to a `String` using `String::from`.
///
/// * `context_path = "..."` Can add optional scope for **path**. The **context_path** will be prepended to beginning of **path**.
///   This is particularly useful when **path** does not contain the full path to the endpoint. For example if web framework
///   allows operation to be defined under some context path or scope which does not reflect to the resolved path then this
///   **context_path** can become handy to alter the path.
///
/// * `tag = "..."` Can be used to group operations. Operations with same tag are grouped together. By default
///   this is derived from the module path of the handler that is given to [`OpenApi`][openapi].
///
/// * `tags = ["tag1", ...]` Can be used to group operations. Operations with same tag are grouped
///   together. Tags attribute can be used to add additional _tags_ for the operation. If both
///   _`tag`_ and _`tags`_ are provided then they will be combined to a single _`tags`_ array.
///
/// * `request_body = ... | request_body(...)` Defining request body indicates that the request is expecting request body within
///   the performed request.
///
/// * `responses(...)` Slice of responses the endpoint is going to possibly return to the caller.
///
/// * `params(...)` Slice of params that the endpoint accepts.
///
/// * `security(...)` List of [`SecurityRequirement`][security]s local to the path operation.
///
/// * `summary = ...` Allows overriding summary of the path. Value can be literal string or valid
///   rust expression e.g. `include_str!(...)` or `const` reference.
///
/// * `description = ...` Allows overriding description of the path. Value can be literal string or valid
///   rust expression e.g. `include_str!(...)` or `const` reference.
///
/// * `extensions(...)` List of extensions local to the path operation.
///
/// # Request Body Attributes
///
/// ## Simple format definition by `request_body = ...`
/// * _`request_body = Type`_, _`request_body = inline(Type)`_ or _`request_body = ref("...")`_.
///   The given _`Type`_ can be any Rust type that is JSON parseable. It can be Option, Vec or Map etc.
///   With _`inline(...)`_ the schema will be inlined instead of a referenced which is the default for
///   [`ToSchema`][to_schema] types. _`ref("./external.json")`_ can be used to reference external
///   json file for body schema. **Note!** Utoipa does **not** guarantee that free form _`ref`_ is accessible via
///   OpenAPI doc or Swagger UI, users are responsible for making these guarantees.
///
/// ## Advanced format definition by `request_body(...)`
///
/// With advanced format the request body supports defining either one or multiple request bodies by `content` attribute.
///
/// ### Common request body attributes
///
/// * `description = "..."` Define the description for the request body object as str.
///
/// * `example = ...` Can be _`json!(...)`_. _`json!(...)`_ should be something that
///   _`serde_json::json!`_ can parse as a _`serde_json::Value`_.
///
/// * `examples(...)` Define multiple examples for single request body. This attribute is mutually
///   exclusive to the _`example`_ attribute and if both are defined this will override the _`example`_.
///   This has same syntax as _`examples(...)`_ in [Response Attributes](#response-attributes)
///   _examples(...)_
///
/// ### Single request body content
///
/// * `content = ...` Can be _`content = Type`_, _`content = inline(Type)`_ or _`content = ref("...")`_. The
///   given _`Type`_ can be any Rust type that is JSON parseable. It can be Option, Vec
///   or Map etc. With _`inline(...)`_ the schema will be inlined instead of a referenced
///   which is the default for [`ToSchema`][to_schema] types. _`ref("./external.json")`_
///   can be used to reference external json file for body schema. **Note!** Utoipa does **not** guarantee
///   that free form _`ref`_ is accessible via OpenAPI doc or Swagger UI, users are responsible for making
///   these guarantees.
///
/// * `content_type = "..."` Can be used to override the default behavior
///   of auto resolving the content type from the `content` attribute. If defined the value should be valid
///   content type such as _`application/json`_ . By default the content type is _`text/plain`_
///   for [primitive Rust types][primitive], `application/octet-stream` for _`[u8]`_ and _`application/json`_
///   for struct and mixed enum types.
///
/// _**Example of single request body definitions.**_
/// ```text
///  request_body(content = String, description = "Xml as string request", content_type = "text/xml"),
///  request_body(content_type = "application/json"),
///  request_body = Pet,
///  request_body = Option<[Pet]>,
/// ```
///
/// ### Multiple request body content
///
/// * `content(...)` Can be tuple of content tuples according to format below.
///   ```text
///   ( schema )
///   ( schema = "content/type", example = ..., examples(..., ...)  )
///   ( "content/type", ),
///   ( "content/type", example = ..., examples(..., ...) )
///   ```
///
///   First argument of content tuple is _`schema`_, which is optional as long as either _`schema`_
///   or _`content/type`_ is defined. The _`schema`_ and _`content/type`_ is separated with equals
///   (=) sign. Optionally content tuple supports defining _`example`_  and _`examples`_ arguments. See
///   [common request body attributes][macro@path#common-request-body-attributes]
///
/// _**Example of multiple request body definitions.**_
///
/// ```text
///  // guess the content type for Pet and Pet2
///  request_body(description = "Common description",
///     content(
///         (Pet),
///         (Pet2)
///     )
///  ),
///  // define explicit content types
///  request_body(description = "Common description",
///     content(
///         (Pet = "application/json", examples(..., ...), example = ...),
///         (Pet2 = "text/xml", examples(..., ...), example = ...)
///     )
///  ),
///  // omit schema and accept arbitrary content types
///  request_body(description = "Common description",
///     content(
///         ("application/json"),
///         ("text/xml", examples(..., ...), example = ...)
///     )
///  ),
/// ```
///
/// # Response Attributes
///
/// * `status = ...` Is either a valid http status code integer. E.g. _`200`_ or a string value representing
///   a range such as _`"4XX"`_ or `"default"` or a valid _`http::status::StatusCode`_.
///   _`StatusCode`_ can either be use path to the status code or _status code_ constant directly.
///
/// * `description = "..."` Define description for the response as str.
///
/// * `body = ...` Optional response body object type. When left empty response does not expect to send any
///   response body. Can be _`body = Type`_, _`body = inline(Type)`_, or _`body = ref("...")`_.
///   The given _`Type`_ can be any Rust type that is JSON parseable. It can be Option, Vec or Map etc.
///   With _`inline(...)`_ the schema will be inlined instead of a referenced which is the default for
///   [`ToSchema`][to_schema] types. _`ref("./external.json")`_
///   can be used to reference external json file for body schema. **Note!** Utoipa does **not** guarantee
///   that free form _`ref`_ is accessible via OpenAPI doc or Swagger UI, users are responsible for making
///   these guarantees.
///
/// * `content_type = "..."` Can be used to override the default behavior
///   of auto resolving the content type from the `body` attribute. If defined the value should be valid
///   content type such as _`application/json`_ . By default the content type is _`text/plain`_
///   for [primitive Rust types][primitive], `application/octet-stream` for _`[u8]`_ and _`application/json`_
///   for struct and mixed enum types.
///
/// * `headers(...)` Slice of response headers that are returned back to a caller.
///
/// * `example = ...` Can be _`json!(...)`_. _`json!(...)`_ should be something that
///   _`serde_json::json!`_ can parse as a _`serde_json::Value`_.
///
/// * `response = ...` Type what implements [`ToResponse`][to_response_trait] trait. This can alternatively be used to
///    define response attributes. _`response`_ attribute cannot co-exist with other than _`status`_ attribute.
///
/// * `content((...), (...))` Can be used to define multiple return types for single response status. Supports same syntax as
///   [multiple request body content][`macro@path#multiple-request-body-content`].
///
/// * `examples(...)` Define multiple examples for single response. This attribute is mutually
///   exclusive to the _`example`_ attribute and if both are defined this will override the _`example`_.
///
/// * `links(...)` Define a map of operations links that can be followed from the response.
///
/// ## Response `examples(...)` syntax
///
/// * `name = ...` This is first attribute and value must be literal string.
/// * `summary = ...` Short description of example. Value must be literal string.
/// * `description = ...` Long description of example. Attribute supports markdown for rich text
///   representation. Value must be literal string.
/// * `value = ...` Example value. It must be _`json!(...)`_. _`json!(...)`_ should be something that
///   _`serde_json::json!`_ can parse as a _`serde_json::Value`_.
/// * `external_value = ...` Define URI to literal example value. This is mutually exclusive to
///   the _`value`_ attribute. Value must be literal string.
///
///  _**Example of example definition.**_
/// ```text
///  ("John" = (summary = "This is John", value = json!({"name": "John"})))
/// ```
///
/// ## Response `links(...)` syntax
///
/// * `operation_ref = ...` Define a relative or absolute URI reference to an OAS operation. This field is
///   mutually exclusive of the _`operation_id`_ field, and **must** point to an [Operation Object][operation].
///   Value can be be [`str`] or an expression such as [`include_str!`][include_str] or static
///   [`const`][const] reference.
///
/// * `operation_id = ...` Define the name of an existing, resolvable OAS operation, as defined with a unique
///   _`operation_id`_. This field is mutually exclusive of the _`operation_ref`_ field.
///   Value can be be [`str`] or an expression such as [`include_str!`][include_str] or static
///   [`const`][const] reference.
///
/// * `parameters(...)` A map representing parameters to pass to an operation as specified with _`operation_id`_
///   or identified by _`operation_ref`_. The key is parameter name to be used and value can
///   be any value supported by JSON or an [expression][expression] e.g. `$path.id`
///     * `name = ...` Define name for the parameter.
///       Value can be be [`str`] or an expression such as [`include_str!`][include_str] or static
///       [`const`][const] reference.
///     * `value` = Any value that can be supported by JSON or an [expression][expression].
///
///     _**Example of parameters syntax:**_
///     ```text
///     parameters(
///          ("name" = value),
///          ("name" = value)
///     ),
///     ```
///
/// * `request_body = ...` Define a literal value or an [expression][expression] to be used as request body when
///   operation is called
///
/// * `description = ...` Define description of the link. Value supports Markdown syntax.Value can be be [`str`] or
///   an expression such as [`include_str!`][include_str] or static [`const`][const] reference.
///
/// * `server(...)` Define [Server][server] object to be used by the target operation. See
///   [server syntax][server_derive_syntax]
///
/// **Links syntax example:** See the full example below in [examples](#examples).
/// ```text
/// responses(
///     (status = 200, description = "success response",
///         links(
///             ("link_name" = (
///                 operation_id = "test_links",
///                 parameters(("key" = "value"), ("json_value" = json!(1))),
///                 request_body = "this is body",
///                 server(url = "http://localhost")
///             ))
///         )
///     )
/// )
/// ```
///
/// **Minimal response format:**
/// ```text
/// responses(
///     (status = 200, description = "success response"),
///     (status = 404, description = "resource missing"),
///     (status = "5XX", description = "server error"),
///     (status = StatusCode::INTERNAL_SERVER_ERROR, description = "internal server error"),
///     (status = IM_A_TEAPOT, description = "happy easter")
/// )
/// ```
///
/// **More complete Response:**
/// ```text
/// responses(
///     (status = 200, description = "Success response", body = Pet, content_type = "application/json",
///         headers(...),
///         example = json!({"id": 1, "name": "bob the cat"})
///     )
/// )
/// ```
///
/// **Multiple response return types with _`content(...)`_ attribute:**
///
/// _**Define multiple response return types for single response status with their own example.**_
/// ```text
/// responses(
///    (status = 200, content(
///            (User = "application/vnd.user.v1+json", example = json!(User {id: "id".to_string()})),
///            (User2 = "application/vnd.user.v2+json", example = json!(User2 {id: 2}))
///        )
///    )
/// )
/// ```
///
/// ### Using `ToResponse` for reusable responses
///
/// _**`ReusableResponse` must be a type that implements [`ToResponse`][to_response_trait].**_
/// ```text
/// responses(
///     (status = 200, response = ReusableResponse)
/// )
/// ```
///
/// _**[`ToResponse`][to_response_trait] can also be inlined to the responses map.**_
/// ```text
/// responses(
///     (status = 200, response = inline(ReusableResponse))
/// )
/// ```
///
/// ## Responses from `IntoResponses`
///
/// _**Responses for a path can be specified with one or more types that implement
/// [`IntoResponses`][into_responses_trait].**_
/// ```text
/// responses(MyResponse)
/// ```
///
/// # Response Header Attributes
///
/// * `name` Name of the header. E.g. _`x-csrf-token`_
///
/// * `type` Additional type of the header value. Can be `Type` or `inline(Type)`.
///   The given _`Type`_ can be any Rust type that is JSON parseable. It can be Option, Vec or Map etc.
///   With _`inline(...)`_ the schema will be inlined instead of a referenced which is the default for
///   [`ToSchema`][to_schema] types. **Reminder!** It's up to the user to use valid type for the
///   response header.
///
/// * `description = "..."` Can be used to define optional description for the response header as str.
///
/// **Header supported formats:**
///
/// ```text
/// ("x-csrf-token"),
/// ("x-csrf-token" = String, description = "New csrf token"),
/// ```
///
/// # Params Attributes
///
/// The list of attributes inside the `params(...)` attribute can take two forms: [Tuples](#tuples) or [IntoParams
/// Type](#intoparams-type).
///
/// ## Tuples
///
/// In the tuples format, parameters are specified using the following attributes inside a list of
/// tuples separated by commas:
///
/// * `name` _**Must be the first argument**_. Define the name for parameter.
///
/// * `parameter_type` Define possible type for the parameter. Can be `Type` or `inline(Type)`.
///   The given _`Type`_ can be any Rust type that is JSON parseable. It can be Option, Vec or Map etc.
///   With _`inline(...)`_ the schema will be inlined instead of a referenced which is the default for
///   [`ToSchema`][to_schema] types. Parameter type is placed after `name` with
///   equals sign E.g. _`"id" = string`_
///
/// * `in` _**Must be placed after name or parameter_type**_. Define the place of the parameter.
///   This must be one of the variants of [`openapi::path::ParameterIn`][in_enum].
///   E.g. _`Path, Query, Header, Cookie`_
///
/// * `deprecated` Define whether the parameter is deprecated or not. Can optionally be defined
///    with explicit `bool` value as _`deprecated = bool`_.
///
/// * `description = "..."` Define possible description for the parameter as str.
///
/// * `style = ...` Defines how parameters are serialized by [`ParameterStyle`][style]. Default values are based on _`in`_ attribute.
///
/// * `explode` Defines whether new _`parameter=value`_ is created for each parameter within _`object`_ or _`array`_.
///
/// * `allow_reserved` Defines whether reserved characters _`:/?#[]@!$&'()*+,;=`_ is allowed within value.
///
/// * `example = ...` Can method reference or _`json!(...)`_. Given example
///   will override any example in underlying parameter type.
///
/// * `extensions(...)` List of extensions local to the parameter
///
/// ##### Parameter type attributes
///
/// These attributes supported when _`parameter_type`_ is present. Either by manually providing one
/// or otherwise resolved e.g from path macro argument when _`actix_extras`_ crate feature is
/// enabled.
///
/// * `format = ...` May either be variant of the [`KnownFormat`][known_format] enum, or otherwise
///   an open value as a string. By default the format is derived from the type of the property
///   according OpenApi spec.
///
/// * `write_only` Defines property is only used in **write** operations *POST,PUT,PATCH* but not in *GET*
///
/// * `read_only` Defines property is only used in **read** operations *GET* but not in *POST,PUT,PATCH*
///
/// * `xml(...)` Can be used to define [`Xml`][xml] object properties for the parameter type.
///    See configuration options at xml attributes of [`ToSchema`][to_schema_xml]
///
/// * `nullable` Defines property is nullable (note this is different to non-required).
///
/// * `multiple_of = ...` Can be used to define multiplier for a value. Value is considered valid
///   division will result an `integer`. Value must be strictly above _`0`_.
///
/// * `maximum = ...` Can be used to define inclusive upper bound to a `number` value.
///
/// * `minimum = ...` Can be used to define inclusive lower bound to a `number` value.
///
/// * `exclusive_maximum = ...` Can be used to define exclusive upper bound to a `number` value.
///
/// * `exclusive_minimum = ...` Can be used to define exclusive lower bound to a `number` value.
///
/// * `max_length = ...` Can be used to define maximum length for `string` types.
///
/// * `min_length = ...` Can be used to define minimum length for `string` types.
///
/// * `pattern = ...` Can be used to define valid regular expression in _ECMA-262_ dialect the field value must match.
///
/// * `max_items = ...` Can be used to define maximum items allowed for `array` fields. Value must
///   be non-negative integer.
///
/// * `min_items = ...` Can be used to define minimum items allowed for `array` fields. Value must
///   be non-negative integer.
///
/// ##### Parameter Formats
/// ```test
/// ("name" = ParameterType, ParameterIn, ...)
/// ("name", ParameterIn, ...)
/// ```
///
/// **For example:**
///
/// ```text
/// params(
///     ("limit" = i32, Query),
///     ("x-custom-header" = String, Header, description = "Custom header"),
///     ("id" = String, Path, deprecated, description = "Pet database id"),
///     ("name", Path, deprecated, description = "Pet name"),
///     (
///         "value" = inline(Option<[String]>),
///         Query,
///         description = "Value description",
///         style = Form,
///         allow_reserved,
///         deprecated,
///         explode,
///         example = json!(["Value"])),
///         max_length = 10,
///         min_items = 1
///     )
/// )
/// ```
///
/// ## IntoParams Type
///
/// In the IntoParams parameters format, the parameters are specified using an identifier for a type
/// that implements [`IntoParams`][into_params]. See [`IntoParams`][into_params] for an
/// example.
///
/// ```text
/// params(MyParameters)
/// ```
///
/// **Note!** that `MyParameters` can also be used in combination with the [tuples
/// representation](#tuples) or other structs.
/// ```text
/// params(
///     MyParameters1,
///     MyParameters2,
///     ("id" = String, Path, deprecated, description = "Pet database id"),
/// )
/// ```
///
/// # Security Requirement Attributes
///
/// * `name` Define the name for security requirement. This must match to name of existing
///   [`SecurityScheme`][security_scheme].
/// * `scopes = [...]` Define the list of scopes needed. These must be scopes defined already in
///   existing [`SecurityScheme`][security_scheme].
///
/// **Security Requirement supported formats:**
///
/// ```text
/// (),
/// ("name" = []),
/// ("name" = ["scope1", "scope2"]),
/// ("name" = ["scope1", "scope2"], "name2" = []),
/// ```
///
/// Leaving empty _`()`_ creates an empty [`SecurityRequirement`][security] this is useful when
/// security requirement is optional for operation.
///
/// You can define multiple security requirements within same parenthesis separated by comma. This
/// allows you to define keys that must be simultaneously provided for the endpoint / API.
///
/// _**Following could be explained as: Security is optional and if provided it must either contain
/// `api_key` or `key AND key2`.**_
/// ```text
/// (),
/// ("api_key" = []),
/// ("key" = [], "key2" = []),
/// ```
///
/// # Extensions Requirements Attributes
///
/// * `x-property` defines the name of the extension.
/// * `json!(...)` defines the value associated with the named extension as a `serde_json::Value`.
///
/// **Extensions Requitement supported formats:**
///
/// ```text
/// ("x-property" = json!({ "type": "mock" }) ),
/// ("x-an-extension" = json!({ "type": "mock" }) ),
/// ("x-another-extension" = json!( "body" ) ),
/// ```
///
/// # actix_extras feature support for actix-web
///
/// **actix_extras** feature gives **utoipa** ability to parse path operation information from **actix-web** types and macros.
///
/// 1. Ability to parse `path` from **actix-web** path attribute macros e.g. _`#[get(...)]`_ or
///    `#[route(...)]`.
/// 2. Ability to parse [`std::primitive`]  or [`String`] or [`tuple`] typed `path` parameters from **actix-web** _`web::Path<...>`_.
/// 3. Ability to parse `path` and `query` parameters form **actix-web** _`web::Path<...>`_, _`web::Query<...>`_ types
///    with [`IntoParams`][into_params] trait.
///
/// See the **actix_extras** in action in examples [todo-actix](https://github.com/juhaku/utoipa/tree/master/examples/todo-actix).
///
/// With **actix_extras** feature enabled the you can leave out definitions for **path**, **operation**
/// and **parameter types**.
/// ```rust
/// use actix_web::{get, web, HttpResponse, Responder};
/// use serde_json::json;
///
/// /// Get Pet by id
/// #[utoipa::path(
///     responses(
///         (status = 200, description = "Pet found from database")
///     ),
///     params(
///         ("id", description = "Pet id"),
///     )
/// )]
/// #[get("/pet/{id}")]
/// async fn get_pet_by_id(id: web::Path<i32>) -> impl Responder {
///     HttpResponse::Ok().json(json!({ "pet": format!("{:?}", &id.into_inner()) }))
/// }
/// ```
///
/// With **actix_extras** you may also not to list any _**params**_ if you do not want to specify any description for them. Params are
/// resolved from path and the argument types of handler
/// ```rust
/// use actix_web::{get, web, HttpResponse, Responder};
/// use serde_json::json;
///
/// /// Get Pet by id
/// #[utoipa::path(
///     responses(
///         (status = 200, description = "Pet found from database")
///     )
/// )]
/// #[get("/pet/{id}")]
/// async fn get_pet_by_id(id: web::Path<i32>) -> impl Responder {
///     HttpResponse::Ok().json(json!({ "pet": format!("{:?}", &id.into_inner()) }))
/// }
/// ```
///
/// # rocket_extras feature support for rocket
///
/// **rocket_extras** feature enhances path operation parameter support. It gives **utoipa** ability to parse `path`, `path parameters`
/// and `query parameters` based on arguments given to **rocket**  proc macros such as _**`#[get(...)]`**_.
///
/// 1. It is able to parse parameter types for [primitive types][primitive], [`String`], [`Vec`], [`Option`] or [`std::path::PathBuf`]
///    type.
/// 2. It is able to determine `parameter_in` for [`IntoParams`][into_params] trait used for `FromForm` type of query parameters.
///
/// See the **rocket_extras** in action in examples [rocket-todo](https://github.com/juhaku/utoipa/tree/master/examples/rocket-todo).
///
///
/// # axum_extras feature support for axum
///
/// **axum_extras** feature enhances parameter support for path operation in following ways.
///
/// 1. It allows users to use tuple style path parameters e.g. _`Path((id, name)): Path<(i32, String)>`_ and resolves
///    parameter names and types from it.
/// 2. It enhances [`IntoParams` derive][into_params_derive] functionality by automatically resolving _`parameter_in`_ from
///     _`Path<...>`_ or _`Query<...>`_ handler function arguments.
///
/// _**Resole path argument types from tuple style handler arguments.**_
/// ```rust
/// # use axum::extract::Path;
/// /// Get todo by id and name.
/// #[utoipa::path(
///     get,
///     path = "/todo/{id}",
///     params(
///         ("id", description = "Todo id"),
///         ("name", description = "Todo name")
///     ),
///     responses(
///         (status = 200, description = "Get todo success", body = String)
///     )
/// )]
/// async fn get_todo(
///     Path((id, name)): Path<(i32, String)>
/// ) -> String {
///     String::new()
/// }
/// ```
///
/// _**Use `IntoParams` to resolve query parameters.**_
/// ```rust
/// # use serde::Deserialize;
/// # use utoipa::IntoParams;
/// # use axum::{extract::Query, Json};
/// #[derive(Deserialize, IntoParams)]
/// struct TodoSearchQuery {
///     /// Search by value. Search is incase sensitive.
///     value: String,
///     /// Search by `done` status.
///     done: bool,
/// }
///
/// /// Search Todos by query params.
/// #[utoipa::path(
///     get,
///     path = "/todo/search",
///     params(
///         TodoSearchQuery
///     ),
///     responses(
///         (status = 200, description = "List matching todos by query", body = [String])
///     )
/// )]
/// async fn search_todos(
///     query: Query<TodoSearchQuery>,
/// ) -> Json<Vec<String>> {
///     Json(vec![])
/// }
/// ```
///
/// # Defining file uploads
///
/// File uploads can be defined in accordance to Open API specification [file uploads][file_uploads].
///
///
/// _**Example sending `jpg` and `png` images as `application/octet-stream`.**_
/// ```rust
/// #[utoipa::path(
///     post,
///     request_body(
///         content(
///             ("image/png"),
///             ("image/jpg"),
///         ),
///     ),
///     path = "/test_images"
/// )]
/// async fn test_images(_body: Vec<u8>) {}
/// ```
///
/// _**Example of sending `multipart` form.**_
/// ```rust
/// #[derive(utoipa::ToSchema)]
/// struct MyForm {
///     order_id: i32,
///     #[schema(content_media_type = "application/octet-stream")]
///     file_bytes: Vec<u8>,
/// }
///
/// #[utoipa::path(
///     post,
///     request_body(content = inline(MyForm), content_type = "multipart/form-data"),
///     path = "/test_multipart"
/// )]
/// async fn test_multipart(_body: MyForm) {}
/// ```
///
/// _**Example of sending arbitrary binary content as `application/octet-stream`.**_
/// ```rust
/// #[utoipa::path(
///     post,
///     request_body = Vec<u8>,
///     path = "/test-octet-stream",
///     responses(
///         (status = 200, description = "success response")
///     ),
/// )]
/// async fn test_octet_stream(_body: Vec<u8>) {}
/// ```
///
/// _**Example of sending `png` image as `base64` encoded.**_
/// ```rust
/// #[derive(utoipa::ToSchema)]
/// #[schema(content_encoding = "base64")]
/// struct MyPng(String);
///
/// #[utoipa::path(
///     post,
///     request_body(content = inline(MyPng), content_type = "image/png"),
///     path = "/test_png",
///     responses(
///         (status = 200, description = "success response")
///     ),
/// )]
/// async fn test_png(_body: MyPng) {}
/// ```
///
/// # Examples
///
/// _**More complete example.**_
/// ```rust
/// # #[derive(utoipa::ToSchema)]
/// # struct Pet {
/// #    id: u64,
/// #    name: String,
/// # }
/// #
/// #[utoipa::path(
///    post,
///    operation_id = "custom_post_pet",
///    path = "/pet",
///    tag = "pet_handlers",
///    request_body(content = Pet, description = "Pet to store the database", content_type = "application/json"),
///    responses(
///         (status = 200, description = "Pet stored successfully", body = Pet, content_type = "application/json",
///             headers(
///                 ("x-cache-len" = String, description = "Cache length")
///             ),
///             example = json!({"id": 1, "name": "bob the cat"})
///         ),
///    ),
///    params(
///      ("x-csrf-token" = String, Header, deprecated, description = "Current csrf token of user"),
///    ),
///    security(
///        (),
///        ("my_auth" = ["read:items", "edit:items"]),
///        ("token_jwt" = [])
///    )
/// )]
/// fn post_pet(pet: Pet) -> Pet {
///     Pet {
///         id: 4,
///         name: "bob the cat".to_string(),
///     }
/// }
/// ```
///
/// _**More minimal example with the defaults.**_
/// ```rust
/// # #[derive(utoipa::ToSchema)]
/// # struct Pet {
/// #    id: u64,
/// #    name: String,
/// # }
/// #
/// #[utoipa::path(
///    post,
///    path = "/pet",
///    request_body = Pet,
///    responses(
///         (status = 200, description = "Pet stored successfully", body = Pet,
///             headers(
///                 ("x-cache-len", description = "Cache length")
///             )
///         ),
///    ),
///    params(
///      ("x-csrf-token", Header, description = "Current csrf token of user"),
///    )
/// )]
/// fn post_pet(pet: Pet) -> Pet {
///     Pet {
///         id: 4,
///         name: "bob the cat".to_string(),
///     }
/// }
/// ```
///
/// _**Use of Rust's own `#[deprecated]` attribute will reflect to the generated OpenAPI spec and mark this operation as deprecated.**_
/// ```rust
/// # use actix_web::{get, web, HttpResponse, Responder};
/// # use serde_json::json;
/// #[utoipa::path(
///     responses(
///         (status = 200, description = "Pet found from database")
///     ),
///     params(
///         ("id", description = "Pet id"),
///     )
/// )]
/// #[get("/pet/{id}")]
/// #[deprecated]
/// async fn get_pet_by_id(id: web::Path<i32>) -> impl Responder {
///     HttpResponse::Ok().json(json!({ "pet": format!("{:?}", &id.into_inner()) }))
/// }
/// ```
///
/// _**Define context path for endpoint. The resolved **path** shown in OpenAPI doc will be `/api/pet/{id}`.**_
/// ```rust
/// # use actix_web::{get, web, HttpResponse, Responder};
/// # use serde_json::json;
/// #[utoipa::path(
///     context_path = "/api",
///     responses(
///         (status = 200, description = "Pet found from database")
///     )
/// )]
/// #[get("/pet/{id}")]
/// async fn get_pet_by_id(id: web::Path<i32>) -> impl Responder {
///     HttpResponse::Ok().json(json!({ "pet": format!("{:?}", &id.into_inner()) }))
/// }
/// ```
///
/// _**Example with multiple return types**_
/// ```rust
/// # trait User {}
/// # #[derive(utoipa::ToSchema)]
/// # struct User1 {
/// #   id: String
/// # }
/// # impl User for User1 {}
/// # #[derive(utoipa::ToSchema)]
/// # struct User2 {
/// #   id: String
/// # }
/// # impl User for User2 {}
/// #[utoipa::path(
///     get,
///     path = "/user",
///     responses(
///         (status = 200, content(
///                 (User1 = "application/vnd.user.v1+json", example = json!({"id": "id".to_string()})),
///                 (User2 = "application/vnd.user.v2+json", example = json!({"id": 2}))
///             )
///         )
///     )
/// )]
/// fn get_user() -> Box<dyn User> {
///   Box::new(User1 {id: "id".to_string()})
/// }
/// ````
///
/// _**Example with multiple examples on single response.**_
/// ```rust
/// # #[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
/// # struct User {
/// #   name: String
/// # }
/// #[utoipa::path(
///     get,
///     path = "/user",
///     responses(
///         (status = 200, body = User,
///             examples(
///                 ("Demo" = (summary = "This is summary", description = "Long description",
///                             value = json!(User{name: "Demo".to_string()}))),
///                 ("John" = (summary = "Another user", value = json!({"name": "John"})))
///              )
///         )
///     )
/// )]
/// fn get_user() -> User {
///   User {name: "John".to_string()}
/// }
/// ```
///
/// _**Example of using links in response.**_
/// ```rust
/// # use serde_json::json;
///  #[utoipa::path(
///     get,
///     path = "/test-links",
///     responses(
///         (status = 200, description = "success response",
///             links(
///                 ("getFoo" = (
///                     operation_id = "test_links",
///                     parameters(("key" = "value"), ("json_value" = json!(1))),
///                     request_body = "this is body",
///                     server(url = "http://localhost")
///                 )),
///                 ("getBar" = (
///                     operation_ref = "this is ref"
///                 ))
///             )
///         )
///     ),
/// )]
/// async fn test_links() -> &'static str {
///     ""
/// }
/// ```
///
/// [in_enum]: openapi/path/enum.ParameterIn.html
/// [path]: trait.Path.html
/// [to_schema]: trait.ToSchema.html
/// [openapi]: derive.OpenApi.html
/// [security]: openapi/security/struct.SecurityRequirement.html
/// [security_scheme]: openapi/security/enum.SecurityScheme.html
/// [primitive]: https://doc.rust-lang.org/std/primitive/index.html
/// [into_params]: trait.IntoParams.html
/// [style]: openapi/path/enum.ParameterStyle.html
/// [into_responses_trait]: trait.IntoResponses.html
/// [into_params_derive]: derive.IntoParams.html
/// [to_response_trait]: trait.ToResponse.html
/// [known_format]: openapi/schema/enum.KnownFormat.html
/// [xml]: openapi/xml/struct.Xml.html
/// [to_schema_xml]: macro@ToSchema#xml-attribute-configuration-options
/// [relative_references]: https://spec.openapis.org/oas/latest.html#relative-references-in-uris
/// [operation]: openapi/path/struct.Operation.html
/// [expression]: https://spec.openapis.org/oas/latest.html#runtime-expressions
/// [const]: https://doc.rust-lang.org/std/keyword.const.html
/// [include_str]: https://doc.rust-lang.org/std/macro.include_str.html
/// [server_derive_syntax]: derive.OpenApi.html#servers-attribute-syntax
/// [server]: openapi/server/struct.Server.html
/// [file_uploads]: <https://spec.openapis.org/oas/v3.1.0.html#considerations-for-file-uploads>
pub fn path(attr: TokenStream, item: TokenStream) -> TokenStream {
    let path_attribute = syn::parse_macro_input!(attr as PathAttr);

    #[cfg(any(
        feature = "actix_extras",
        feature = "rocket_extras",
        feature = "axum_extras",
        feature = "auto_into_responses"
    ))]
    let mut path_attribute = path_attribute;

    let ast_fn = match syn::parse::<ItemFn>(item) {
        Ok(ast_fn) => ast_fn,
        Err(error) => return error.into_compile_error().into_token_stream().into(),
    };

    #[cfg(feature = "auto_into_responses")]
    {
        if let Some(responses) = ext::auto_types::parse_fn_operation_responses(&ast_fn) {
            path_attribute.responses_from_into_responses(responses);
        };
    }

    let mut resolved_methods = match PathOperations::resolve_operation(&ast_fn) {
        Ok(operation) => operation,
        Err(diagnostics) => return diagnostics.into_token_stream().into(),
    };
    let resolved_path = PathOperations::resolve_path(
        &resolved_methods
            .as_mut()
            .map(|operation| mem::take(&mut operation.path).to_string())
            .or_else(|| path_attribute.path.as_ref().map(|path| path.to_string())), // cannot use mem take because we need this later
    );

    #[cfg(any(
        feature = "actix_extras",
        feature = "rocket_extras",
        feature = "axum_extras"
    ))]
    let mut resolved_path = resolved_path;

    #[cfg(any(
        feature = "actix_extras",
        feature = "rocket_extras",
        feature = "axum_extras"
    ))]
    {
        use ext::ArgumentResolver;
        use path::parameter::Parameter;
        let path_args = resolved_path.as_mut().map(|path| mem::take(&mut path.args));
        let body = resolved_methods
            .as_mut()
            .map(|path| mem::take(&mut path.body))
            .unwrap_or_default();

        let (arguments, into_params_types, body) =
            match PathOperations::resolve_arguments(&ast_fn.sig.inputs, path_args, body) {
                Ok(args) => args,
                Err(diagnostics) => return diagnostics.into_token_stream().into(),
            };

        let parameters = arguments
            .into_iter()
            .flatten()
            .map(Parameter::from)
            .chain(into_params_types.into_iter().flatten().map(Parameter::from));
        path_attribute.update_parameters_ext(parameters);

        path_attribute.update_request_body(body);
    }

    let path = Path::new(path_attribute, &ast_fn.sig.ident)
        .ext_methods(resolved_methods.map(|operation| operation.methods))
        .path(resolved_path.map(|path| path.path))
        .doc_comments(CommentAttributes::from_attributes(&ast_fn.attrs).0)
        .deprecated(ast_fn.attrs.has_deprecated());

    let handler = path::handler::Handler {
        path,
        handler_fn: &ast_fn,
    };
    handler.to_token_stream().into()
}

#[proc_macro_derive(OpenApi, attributes(openapi))]
/// Generate OpenApi base object with defaults from
/// project settings.
///
/// This is `#[derive]` implementation for [`OpenApi`][openapi] trait. The macro accepts one `openapi` argument.
///
/// # OpenApi `#[openapi(...)]` attributes
///
/// * `paths(...)`  List of method references having attribute [`#[utoipa::path]`][path] macro.
/// * `components(schemas(...), responses(...))` Takes available _`component`_ configurations. Currently only
///    _`schema`_ and _`response`_ components are supported.
///    * `schemas(...)` List of [`ToSchema`][to_schema]s in OpenAPI schema.
///    * `responses(...)` List of types that implement [`ToResponse`][to_response_trait].
/// * `modifiers(...)` List of items implementing [`Modify`][modify] trait for runtime OpenApi modification.
///   See the [trait documentation][modify] for more details.
/// * `security(...)` List of [`SecurityRequirement`][security]s global to all operations.
///   See more details in [`#[utoipa::path(...)]`][path] [attribute macro security options][path_security].
/// * `tags(...)` List of [`Tag`][tags]s which must match the tag _**path operation**_.  Tags can be used to
///   define extra information for the API to produce richer documentation. See [tags attribute syntax][tags_syntax].
/// * `external_docs(...)` Can be used to reference external resource to the OpenAPI doc for extended documentation.
///   External docs can be in [`OpenApi`][openapi_struct] or in [`Tag`][tags] level.
/// * `servers(...)` Define [`servers`][servers] as derive argument to the _`OpenApi`_. Servers
///   are completely optional and thus can be omitted from the declaration. See [servers attribute
///   syntax][servers_syntax]
/// * `info(...)` Declare [`Info`][info] attribute values used to override the default values
///   generated from Cargo environment variables. **Note!** Defined attributes will override the
///   whole attribute from generated values of Cargo environment variables. E.g. defining
///   `contact(name = ...)` will ultimately override whole contact of info and not just partially
///   the name. See [info attribute syntax][info_syntax]
/// * `nest(...)` Allows nesting [`OpenApi`][openapi_struct]s to this _`OpenApi`_ instance. Nest
///   takes comma separated list of tuples of nested `OpenApi`s. _`OpenApi`_ instance must
///   implement [`OpenApi`][openapi] trait. Nesting allows defining one `OpenApi` per defined path.
///   If more instances is defined only latest one will be rentained.
///   See the _[nest(...) attribute syntax below]( #nest-attribute-syntax )_
///
///
/// OpenApi derive macro will also derive [`Info`][info] for OpenApi specification using Cargo
/// environment variables.
///
/// * env `CARGO_PKG_NAME` map to info `title`
/// * env `CARGO_PKG_VERSION` map to info `version`
/// * env `CARGO_PKG_DESCRIPTION` map info `description`
/// * env `CARGO_PKG_AUTHORS` map to contact `name` and `email` **only first author will be used**
/// * env `CARGO_PKG_LICENSE` map to info `license`
///
/// # `info(...)` attribute syntax
///
/// * `title = ...` Define title of the API. It can be [`str`] or an
///   expression such as [`include_str!`][include_str] or static [`const`][const] reference.
/// * `terms_of_service = ...` Define URL to the Terms of Service for the API. It can be [`str`] or an
///   expression such as [`include_str!`][include_str] or static [`const`][const] reference. Value
///   must be valid URL.
/// * `description = ...` Define description of the API. Markdown can be used for rich text
///   representation. It can be [`str`] or an expression such as [`include_str!`][include_str] or static
///   [`const`][const] reference.
/// * `version = ...` Override default version from _`Cargo.toml`_. Value can be [`str`] or an
///   expression such as [`include_str!`][include_str] or static [`const`][const] reference.
/// * `contact(...)` Used to override the whole contact generated from environment variables.
///     * `name = ...` Define identifying name of contact person / organization. It Can be a literal string.
///     * `email = ...` Define email address of the contact person / organization. It can be a literal string.
///     * `url = ...` Define URL pointing to the contact information. It must be in URL formatted string.
/// * `license(...)` Used to override the whole license generated from environment variables.
///     * `name = ...` License name of the API. It can be a literal string.
///     * `url = ...` Define optional URL of the license. It must be URL formatted string.
///
/// # `tags(...)` attribute syntax
///
/// * `name = ...` Must be provided, can be [`str`] or an expression such as [`include_str!`][include_str]
///   or static [`const`][const] reference.
/// * `description = ...` Optional description for the tag. Can be either or static [`str`]
///   or an expression e.g. _`include_str!(...)`_ macro call or reference to static [`const`][const].
/// * `external_docs(...)` Optional links to external documents.
///      * `url = ...` Mandatory URL for external documentation.
///      * `description = ...` Optional description for the _`url`_ link.
///
/// # `servers(...)` attribute syntax
///
/// * `url = ...` Define the url for server. It can be literal string.
/// * `description = ...` Define description for the server. It can be literal string.
/// * `variables(...)` Can be used to define variables for the url.
///     * `name = ...` Is the first argument within parentheses. It must be literal string.
///     * `default = ...` Defines a default value for the variable if nothing else will be
///       provided. If _`enum_values`_ is defined the _`default`_ must be found within the enum
///       options. It can be a literal string.
///     * `description = ...` Define the description for the variable. It can be a literal string.
///     * `enum_values(...)` Define list of possible values for the variable. Values must be
///       literal strings.
///
/// _**Example server variable definition.**_
/// ```text
/// ("username" = (default = "demo", description = "Default username for API")),
/// ("port" = (enum_values("8080", "5000", "4545")))
/// ```
///
/// # `nest(...)` attribute syntax
///
/// * `path = ...` Define mandatory path for nesting the [`OpenApi`][openapi_struct].
/// * `api = ...` Define mandatory path to struct that implements [`OpenApi`][openapi] trait.
///    The fully qualified path (_`path::to`_) will become the default _`tag`_ for the nested
///    `OpenApi` endpoints if provided.
/// * `tags = [...]` Define optional tags what are appended to the existing list of tags.
///
///  _**Example of nest definition**_
///  ```text
///  (path = "path/to/nest", api = path::to::NestableApi),
///  (path = "path/to/nest", api = path::to::NestableApi, tags = ["nestableapi", ...])
///  ```
///
/// # Examples
///
/// _**Define OpenApi schema with some paths and components.**_
/// ```rust
/// # use utoipa::{OpenApi, ToSchema};
/// #
/// #[derive(ToSchema)]
/// struct Pet {
///     name: String,
///     age: i32,
/// }
///
/// #[derive(ToSchema)]
/// enum Status {
///     Active, InActive, Locked,
/// }
///
/// #[utoipa::path(get, path = "/pet")]
/// fn get_pet() -> Pet {
///     Pet {
///         name: "bob".to_string(),
///         age: 8,
///     }
/// }
///
/// #[utoipa::path(get, path = "/status")]
/// fn get_status() -> Status {
///     Status::Active
/// }
///
/// #[derive(OpenApi)]
/// #[openapi(
///     paths(get_pet, get_status),
///     components(schemas(Pet, Status)),
///     security(
///         (),
///         ("my_auth" = ["read:items", "edit:items"]),
///         ("token_jwt" = [])
///     ),
///     tags(
///         (name = "pets::api", description = "All about pets",
///             external_docs(url = "http://more.about.pets.api", description = "Find out more"))
///     ),
///     external_docs(url = "http://more.about.our.apis", description = "More about our APIs")
/// )]
/// struct ApiDoc;
/// ```
///
/// _**Define servers to OpenApi.**_
/// ```rust
/// # use utoipa::OpenApi;
/// #[derive(OpenApi)]
/// #[openapi(
///     servers(
///         (url = "http://localhost:8989", description = "Local server"),
///         (url = "http://api.{username}:{port}", description = "Remote API",
///             variables(
///                 ("username" = (default = "demo", description = "Default username for API")),
///                 ("port" = (default = "8080", enum_values("8080", "5000", "3030"), description = "Supported ports for API"))
///             )
///         )
///     )
/// )]
/// struct ApiDoc;
/// ```
///
/// _**Define info attribute values used to override auto generated ones from Cargo environment
/// variables.**_
/// ```compile_fail
/// # use utoipa::OpenApi;
/// #[derive(OpenApi)]
/// #[openapi(info(
///     title = "title override",
///     description = include_str!("./path/to/content"), // fail compile cause no such file
///     contact(name = "Test")
/// ))]
/// struct ApiDoc;
/// ```
///
/// _**Create OpenAPI with reusable response.**_
/// ```rust
/// #[derive(utoipa::ToSchema)]
/// struct Person {
///     name: String,
/// }
///
/// /// Person list response
/// #[derive(utoipa::ToResponse)]
/// struct PersonList(Vec<Person>);
///
/// #[utoipa::path(
///     get,
///     path = "/person-list",
///     responses(
///         (status = 200, response = PersonList)
///     )
/// )]
/// fn get_persons() -> Vec<Person> {
///     vec![]
/// }
///
/// #[derive(utoipa::OpenApi)]
/// #[openapi(
///     components(
///         schemas(Person),
///         responses(PersonList)
///     )
/// )]
/// struct ApiDoc;
/// ```
///
/// _**Nest _`UserApi`_ to the current api doc instance.**_
/// ```rust
/// # use utoipa::OpenApi;
/// #
///  #[utoipa::path(get, path = "/api/v1/status")]
///  fn test_path_status() {}
///
///  #[utoipa::path(get, path = "/test")]
///  fn user_test_path() {}
///
///  #[derive(OpenApi)]
///  #[openapi(paths(user_test_path))]
///  struct UserApi;
///
///  #[derive(OpenApi)]
///  #[openapi(
///      paths(
///          test_path_status
///      ),
///      nest(
///          (path = "/api/v1/user", api = UserApi),
///      )
///  )]
///  struct ApiDoc;
/// ```
///
/// [openapi]: trait.OpenApi.html
/// [openapi_struct]: openapi/struct.OpenApi.html
/// [to_schema]: derive.ToSchema.html
/// [path]: attr.path.html
/// [modify]: trait.Modify.html
/// [info]: openapi/info/struct.Info.html
/// [security]: openapi/security/struct.SecurityRequirement.html
/// [path_security]: attr.path.html#security-requirement-attributes
/// [tags]: openapi/tag/struct.Tag.html
/// [to_response_trait]: trait.ToResponse.html
/// [servers]: openapi/server/index.html
/// [const]: https://doc.rust-lang.org/std/keyword.const.html
/// [tags_syntax]: #tags-attribute-syntax
/// [info_syntax]: #info-attribute-syntax
/// [servers_syntax]: #servers-attribute-syntax
/// [include_str]: https://doc.rust-lang.org/std/macro.include_str.html
pub fn openapi(input: TokenStream) -> TokenStream {
    let DeriveInput { attrs, ident, .. } = syn::parse_macro_input!(input);

    parse_openapi_attrs(&attrs)
        .map(|openapi_attr| OpenApi(openapi_attr, ident).to_token_stream())
        .map_or_else(syn::Error::into_compile_error, ToTokens::into_token_stream)
        .into()
}

#[proc_macro_derive(IntoParams, attributes(param, into_params))]
/// Generate [path parameters][path_params] from struct's
/// fields.
///
/// This is `#[derive]` implementation for [`IntoParams`][into_params] trait.
///
/// Typically path parameters need to be defined within [`#[utoipa::path(...params(...))]`][path_params] section
/// for the endpoint. But this trait eliminates the need for that when [`struct`][struct]s are used to define parameters.
/// Still [`std::primitive`] and [`String`] path parameters or [`tuple`] style path parameters need to be defined
/// within `params(...)` section if description or other than default configuration need to be given.
///
/// You can use the Rust's own `#[deprecated]` attribute on field to mark it as
/// deprecated and it will reflect to the generated OpenAPI spec.
///
/// `#[deprecated]` attribute supports adding additional details such as a reason and or since version
/// but this is is not supported in OpenAPI. OpenAPI has only a boolean flag to determine deprecation.
/// While it is totally okay to declare deprecated with reason
/// `#[deprecated  = "There is better way to do this"]` the reason would not render in OpenAPI spec.
///
/// Doc comment on struct fields will be used as description for the generated parameters.
/// ```rust
/// #[derive(utoipa::IntoParams)]
/// struct Query {
///     /// Query todo items by name.
///     name: String
/// }
/// ```
///
/// # IntoParams Container Attributes for `#[into_params(...)]`
///
/// The following attributes are available for use in on the container attribute `#[into_params(...)]` for the struct
/// deriving `IntoParams`:
///
/// * `names(...)` Define comma separated list of names for unnamed fields of struct used as a path parameter.
///    __Only__ supported on __unnamed structs__.
/// * `style = ...` Defines how all parameters are serialized by [`ParameterStyle`][style]. Default
///    values are based on _`parameter_in`_ attribute.
/// * `parameter_in = ...` =  Defines where the parameters of this field are used with a value from
///    [`openapi::path::ParameterIn`][in_enum]. There is no default value, if this attribute is not
///    supplied, then the value is determined by the `parameter_in_provider` in
///    [`IntoParams::into_params()`](trait.IntoParams.html#tymethod.into_params).
/// * `rename_all = ...` Can be provided to alternatively to the serde's `rename_all` attribute. Effectively provides same functionality.
///
/// Use `names` to define name for single unnamed argument.
/// ```rust
/// # use utoipa::IntoParams;
/// #
/// #[derive(IntoParams)]
/// #[into_params(names("id"))]
/// struct Id(u64);
/// ```
///
/// Use `names` to define names for multiple unnamed arguments.
/// ```rust
/// # use utoipa::IntoParams;
/// #
/// #[derive(IntoParams)]
/// #[into_params(names("id", "name"))]
/// struct IdAndName(u64, String);
/// ```
///
/// # IntoParams Field Attributes for `#[param(...)]`
///
/// The following attributes are available for use in the `#[param(...)]` on struct fields:
///
/// * `style = ...` Defines how the parameter is serialized by [`ParameterStyle`][style]. Default values are based on _`parameter_in`_ attribute.
///
/// * `explode` Defines whether new _`parameter=value`_ pair is created for each parameter within _`object`_ or _`array`_.
///
/// * `allow_reserved` Defines whether reserved characters _`:/?#[]@!$&'()*+,;=`_ is allowed within value.
///
/// * `example = ...` Can be method reference or _`json!(...)`_. Given example
///   will override any example in underlying parameter type.
///
/// * `value_type = ...` Can be used to override default type derived from type of the field used in OpenAPI spec.
///   This is useful in cases where the default type does not correspond to the actual type e.g. when
///   any third-party types are used which are not [`ToSchema`][to_schema]s nor [`primitive` types][primitive].
///   The value can be any Rust type what normally could be used to serialize to JSON, or either virtual type _`Object`_
///   or _`Value`_.
///   _`Object`_ will be rendered as generic OpenAPI object _(`type: object`)_.
///   _`Value`_ will be rendered as any OpenAPI value (i.e. no `type` restriction).
///
/// * `inline` If set, the schema for this field's type needs to be a [`ToSchema`][to_schema], and
///   the schema definition will be inlined.
///
/// * `default = ...` Can be method reference or _`json!(...)`_.
///
/// * `format = ...` May either be variant of the [`KnownFormat`][known_format] enum, or otherwise
///   an open value as a string. By default the format is derived from the type of the property
///   according OpenApi spec.
///
/// * `write_only` Defines property is only used in **write** operations *POST,PUT,PATCH* but not in *GET*.
///
/// * `read_only` Defines property is only used in **read** operations *GET* but not in *POST,PUT,PATCH*.
///
/// * `xml(...)` Can be used to define [`Xml`][xml] object properties applicable to named fields.
///    See configuration options at xml attributes of [`ToSchema`][to_schema_xml]
///
/// * `nullable` Defines property is nullable (note this is different to non-required).
///
/// * `required = ...` Can be used to enforce required status for the parameter. [See
///    rules][derive@IntoParams#field-nullability-and-required-rules]
///
/// * `rename = ...` Can be provided to alternatively to the serde's `rename` attribute. Effectively provides same functionality.
///
/// * `multiple_of = ...` Can be used to define multiplier for a value. Value is considered valid
///   division will result an `integer`. Value must be strictly above _`0`_.
///
/// * `maximum = ...` Can be used to define inclusive upper bound to a `number` value.
///
/// * `minimum = ...` Can be used to define inclusive lower bound to a `number` value.
///
/// * `exclusive_maximum = ...` Can be used to define exclusive upper bound to a `number` value.
///
/// * `exclusive_minimum = ...` Can be used to define exclusive lower bound to a `number` value.
///
/// * `max_length = ...` Can be used to define maximum length for `string` types.
///
/// * `min_length = ...` Can be used to define minimum length for `string` types.
///
/// * `pattern = ...` Can be used to define valid regular expression in _ECMA-262_ dialect the field value must match.
///
/// * `max_items = ...` Can be used to define maximum items allowed for `array` fields. Value must
///   be non-negative integer.
///
/// * `min_items = ...` Can be used to define minimum items allowed for `array` fields. Value must
///   be non-negative integer.
///
/// * `schema_with = ...` Use _`schema`_ created by provided function reference instead of the
///   default derived _`schema`_. The function must match to `fn() -> Into<RefOr<Schema>>`. It does
///   not accept arguments and must return anything that can be converted into `RefOr<Schema>`.
///
/// * `additional_properties = ...` Can be used to define free form types for maps such as
///   [`HashMap`](std::collections::HashMap) and [`BTreeMap`](std::collections::BTreeMap).
///   Free form type enables use of arbitrary types within map values.
///   Supports formats _`additional_properties`_ and _`additional_properties = true`_.
///
/// * `ignore` or `ignore = ...` Can be used to skip the field from being serialized to OpenAPI schema. It accepts either a literal `bool` value
///   or a path to a function that returns `bool` (`Fn() -> bool`).
///
/// #### Field nullability and required rules
///
/// Same rules for nullability and required status apply for _`IntoParams`_ field attributes as for
/// _`ToSchema`_ field attributes. [See the rules][`derive@ToSchema#field-nullability-and-required-rules`].
///
/// # Partial `#[serde(...)]` attributes support
///
/// IntoParams derive has partial support for [serde attributes]. These supported attributes will reflect to the
/// generated OpenAPI doc. The following attributes are currently supported:
///
/// * `rename_all = "..."` Supported at the container level.
/// * `rename = "..."` Supported **only** at the field level.
/// * `default` Supported at the container level and field level according to [serde attributes].
/// * `skip_serializing_if = "..."` Supported  **only** at the field level.
/// * `with = ...` Supported **only** at field level.
/// * `skip_serializing = "..."` Supported  **only** at the field or variant level.
/// * `skip_deserializing = "..."` Supported  **only** at the field or variant level.
/// * `skip = "..."` Supported  **only** at the field level.
///
/// Other _`serde`_ attributes will impact the serialization but will not be reflected on the generated OpenAPI doc.
///
/// # Examples
///
/// _**Demonstrate [`IntoParams`][into_params] usage with resolving `Path` and `Query` parameters
/// with _`actix-web`_**_.
/// ```rust
/// use actix_web::{get, HttpResponse, Responder};
/// use actix_web::web::{Path, Query};
/// use serde::Deserialize;
/// use serde_json::json;
/// use utoipa::IntoParams;
///
/// #[derive(Deserialize, IntoParams)]
/// struct PetPathArgs {
///     /// Id of pet
///     id: i64,
///     /// Name of pet
///     name: String,
/// }
///
/// #[derive(Deserialize, IntoParams)]
/// struct Filter {
///     /// Age filter for pets
///     #[deprecated]
///     #[param(style = Form, explode, allow_reserved, example = json!([10]))]
///     age: Option<Vec<i32>>,
/// }
///
/// #[utoipa::path(
///     params(PetPathArgs, Filter),
///     responses(
///         (status = 200, description = "success response")
///     )
/// )]
/// #[get("/pet/{id}/{name}")]
/// async fn get_pet(pet: Path<PetPathArgs>, query: Query<Filter>) -> impl Responder {
///     HttpResponse::Ok().json(json!({ "id": pet.id }))
/// }
/// ```
///
/// _**Demonstrate [`IntoParams`][into_params] usage with the `#[into_params(...)]` container attribute to
/// be used as a path query, and inlining a schema query field:**_
/// ```rust
/// use serde::Deserialize;
/// use utoipa::{IntoParams, ToSchema};
///
/// #[derive(Deserialize, ToSchema)]
/// #[serde(rename_all = "snake_case")]
/// enum PetKind {
///     Dog,
///     Cat,
/// }
///
/// #[derive(Deserialize, IntoParams)]
/// #[into_params(style = Form, parameter_in = Query)]
/// struct PetQuery {
///     /// Name of pet
///     name: Option<String>,
///     /// Age of pet
///     age: Option<i32>,
///     /// Kind of pet
///     #[param(inline)]
///     kind: PetKind
/// }
///
/// #[utoipa::path(
///     get,
///     path = "/get_pet",
///     params(PetQuery),
///     responses(
///         (status = 200, description = "success response")
///     )
/// )]
/// async fn get_pet(query: PetQuery) {
///     // ...
/// }
/// ```
///
/// _**Override `String` with `i64` using `value_type` attribute.**_
/// ```rust
/// # use utoipa::IntoParams;
/// #
/// #[derive(IntoParams)]
/// #[into_params(parameter_in = Query)]
/// struct Filter {
///     #[param(value_type = i64)]
///     id: String,
/// }
/// ```
///
/// _**Override `String` with `Object` using `value_type` attribute. _`Object`_ will render as `type: object` in OpenAPI spec.**_
/// ```rust
/// # use utoipa::IntoParams;
/// #
/// #[derive(IntoParams)]
/// #[into_params(parameter_in = Query)]
/// struct Filter {
///     #[param(value_type = Object)]
///     id: String,
/// }
/// ```
///
/// _**You can use a generic type to override the default type of the field.**_
/// ```rust
/// # use utoipa::IntoParams;
/// #
/// #[derive(IntoParams)]
/// #[into_params(parameter_in = Query)]
/// struct Filter {
///     #[param(value_type = Option<String>)]
///     id: String
/// }
/// ```
///
/// _**You can even override a [`Vec`] with another one.**_
/// ```rust
/// # use utoipa::IntoParams;
/// #
/// #[derive(IntoParams)]
/// #[into_params(parameter_in = Query)]
/// struct Filter {
///     #[param(value_type = Vec<i32>)]
///     id: Vec<String>
/// }
/// ```
///
/// _**We can override value with another [`ToSchema`][to_schema].**_
/// ```rust
/// # use utoipa::{IntoParams, ToSchema};
/// #
/// #[derive(ToSchema)]
/// struct Id {
///     value: i64,
/// }
///
/// #[derive(IntoParams)]
/// #[into_params(parameter_in = Query)]
/// struct Filter {
///     #[param(value_type = Id)]
///     id: String
/// }
/// ```
///
/// _**Example with validation attributes.**_
/// ```rust
/// #[derive(utoipa::IntoParams)]
/// struct Item {
///     #[param(maximum = 10, minimum = 5, multiple_of = 2.5)]
///     id: i32,
///     #[param(max_length = 10, min_length = 5, pattern = "[a-z]*")]
///     value: String,
///     #[param(max_items = 5, min_items = 1)]
///     items: Vec<String>,
/// }
/// ````
///
/// _**Use `schema_with` to manually implement schema for a field.**_
/// ```rust
/// # use utoipa::openapi::schema::{Object, ObjectBuilder};
/// fn custom_type() -> Object {
///     ObjectBuilder::new()
///         .schema_type(utoipa::openapi::schema::Type::String)
///         .format(Some(utoipa::openapi::SchemaFormat::Custom(
///             "email".to_string(),
///         )))
///         .description(Some("this is the description"))
///         .build()
/// }
///
/// #[derive(utoipa::IntoParams)]
/// #[into_params(parameter_in = Query)]
/// struct Query {
///     #[param(schema_with = custom_type)]
///     email: String,
/// }
/// ```
///
/// [to_schema]: trait.ToSchema.html
/// [known_format]: openapi/schema/enum.KnownFormat.html
/// [xml]: openapi/xml/struct.Xml.html
/// [into_params]: trait.IntoParams.html
/// [path_params]: attr.path.html#params-attributes
/// [struct]: https://doc.rust-lang.org/std/keyword.struct.html
/// [style]: openapi/path/enum.ParameterStyle.html
/// [in_enum]: openapi/path/enum.ParameterIn.html
/// [primitive]: https://doc.rust-lang.org/std/primitive/index.html
/// [serde attributes]: https://serde.rs/attributes.html
/// [to_schema_xml]: macro@ToSchema#xml-attribute-configuration-options
pub fn into_params(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs,
        ident,
        generics,
        data,
        ..
    } = syn::parse_macro_input!(input);

    let into_params = IntoParams {
        attrs,
        generics,
        data,
        ident,
    };

    into_params.to_token_stream().into()
}

#[proc_macro_derive(ToResponse, attributes(response, content, to_schema))]
/// Generate reusable OpenAPI response that can be used
/// in [`utoipa::path`][path] or in [`OpenApi`][openapi].
///
/// This is `#[derive]` implementation for [`ToResponse`][to_response] trait.
///
///
/// _`#[response]`_ attribute can be used to alter and add [response attributes](#toresponse-response-attributes).
///
/// _`#[content]`_ attributes is used to make enum variant a content of a specific type for the
/// response.
///
/// _`#[to_schema]`_ attribute is used to inline a schema for a response in unnamed structs or
/// enum variants with `#[content]` attribute. **Note!** [`ToSchema`] need to be implemented for
/// the field or variant type.
///
/// Type derived with _`ToResponse`_ uses provided doc comment as a description for the response. It
/// can alternatively be overridden with _`description = ...`_ attribute.
///
/// _`ToResponse`_ can be used in four different ways to generate OpenAPI response component.
///
/// 1. By decorating `struct` or `enum` with [`derive@ToResponse`] derive macro. This will create a
///    response with inlined schema resolved from the fields of the `struct` or `variants` of the
///    enum.
///
///    ```rust
///     # use utoipa::ToResponse;
///     #[derive(ToResponse)]
///     #[response(description = "Person response returns single Person entity")]
///     struct Person {
///         name: String,
///     }
///    ```
///
/// 2. By decorating unnamed field `struct` with [`derive@ToResponse`] derive macro. Unnamed field struct
///    allows users to use new type pattern to define one inner field which is used as a schema for
///    the generated response. This allows users to define `Vec` and `Option` response types.
///    Additionally these types can also be used with `#[to_schema]` attribute to inline the
///    field's type schema if it implements [`ToSchema`] derive macro.
///
///    ```rust
///     # #[derive(utoipa::ToSchema)]
///     # struct Person {
///     #     name: String,
///     # }
///     /// Person list response
///     #[derive(utoipa::ToResponse)]
///     struct PersonList(Vec<Person>);
///    ```
///
/// 3. By decorating unit struct with [`derive@ToResponse`] derive macro. Unit structs will produce a
///    response without body.
///
///    ```rust
///     /// Success response which does not have body.
///     #[derive(utoipa::ToResponse)]
///     struct SuccessResponse;
///    ```
///
/// 4. By decorating `enum` with variants having `#[content(...)]` attribute. This allows users to
///    define multiple response content schemas to single response according to OpenAPI spec.
///    **Note!** Enum with _`content`_ attribute in variants cannot have enum level _`example`_ or
///    _`examples`_ defined. Instead examples need to be defined per variant basis. Additionally
///    these variants can also be used with `#[to_schema]` attribute to inline the variant's type schema
///    if it implements [`ToSchema`] derive macro.
///
///    ```rust
///     #[derive(utoipa::ToSchema)]
///     struct Admin {
///         name: String,
///     }
///     #[derive(utoipa::ToSchema)]
///     struct Admin2 {
///         name: String,
///         id: i32,
///     }
///
///     #[derive(utoipa::ToResponse)]
///     enum Person {
///         #[response(examples(
///             ("Person1" = (value = json!({"name": "name1"}))),
///             ("Person2" = (value = json!({"name": "name2"})))
///         ))]
///         Admin(#[content("application/vnd-custom-v1+json")] Admin),
///
///         #[response(example = json!({"name": "name3", "id": 1}))]
///         Admin2(#[content("application/vnd-custom-v2+json")] #[to_schema] Admin2),
///     }
///    ```
///
/// # ToResponse `#[response(...)]` attributes
///
/// * `description = "..."` Define description for the response as str. This can be used to
///   override the default description resolved from doc comments if present.
///
/// * `content_type = "..."` Can be used to override the default behavior
///   of auto resolving the content type from the `body` attribute. If defined the value should be valid
///   content type such as _`application/json`_ . By default the content type is _`text/plain`_
///   for [primitive Rust types][primitive], `application/octet-stream` for _`[u8]`_ and _`application/json`_
///   for struct and mixed enum types.
///
/// * `headers(...)` Slice of response headers that are returned back to a caller.
///
/// * `example = ...` Can be _`json!(...)`_. _`json!(...)`_ should be something that
///   _`serde_json::json!`_ can parse as a _`serde_json::Value`_.
///
/// * `examples(...)` Define multiple examples for single response. This attribute is mutually
///   exclusive to the _`example`_ attribute and if both are defined this will override the _`example`_.
///     * `name = ...` This is first attribute and value must be literal string.
///     * `summary = ...` Short description of example. Value must be literal string.
///     * `description = ...` Long description of example. Attribute supports markdown for rich text
///       representation. Value must be literal string.
///     * `value = ...` Example value. It must be _`json!(...)`_. _`json!(...)`_ should be something that
///       _`serde_json::json!`_ can parse as a _`serde_json::Value`_.
///     * `external_value = ...` Define URI to literal example value. This is mutually exclusive to
///       the _`value`_ attribute. Value must be literal string.
///
///      _**Example of example definition.**_
///     ```text
///      ("John" = (summary = "This is John", value = json!({"name": "John"})))
///     ```
///
/// # Examples
///
/// _**Use reusable response in operation handler.**_
/// ```rust
/// #[derive(utoipa::ToResponse)]
/// struct PersonResponse {
///    value: String
/// }
///
/// #[derive(utoipa::OpenApi)]
/// #[openapi(components(responses(PersonResponse)))]
/// struct Doc;
///
/// #[utoipa::path(
///     get,
///     path = "/api/person",
///     responses(
///         (status = 200, response = PersonResponse)
///     )
/// )]
/// fn get_person() -> PersonResponse {
///     PersonResponse { value: "person".to_string() }
/// }
/// ```
///
/// _**Create a response from named struct.**_
/// ```rust
///  /// This is description
///  ///
///  /// It will also be used in `ToSchema` if present
///  #[derive(utoipa::ToSchema, utoipa::ToResponse)]
///  #[response(
///      description = "Override description for response",
///      content_type = "text/xml"
///  )]
///  #[response(
///      example = json!({"name": "the name"}),
///      headers(
///          ("csrf-token", description = "response csrf token"),
///          ("random-id" = i32)
///      )
///  )]
///  struct Person {
///      name: String,
///  }
/// ```
///
/// _**Create inlined person list response.**_
/// ```rust
///  # #[derive(utoipa::ToSchema)]
///  # struct Person {
///  #     name: String,
///  # }
///  /// Person list response
///  #[derive(utoipa::ToResponse)]
///  struct PersonList(#[to_schema] Vec<Person>);
/// ```
///
/// _**Create enum response from variants.**_
/// ```rust
///  #[derive(utoipa::ToResponse)]
///  enum PersonType {
///      Value(String),
///      Foobar,
///  }
/// ```
///
/// [to_response]: trait.ToResponse.html
/// [primitive]: https://doc.rust-lang.org/std/primitive/index.html
/// [path]: attr.path.html
/// [openapi]: derive.OpenApi.html
pub fn to_response(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs,
        ident,
        generics,
        data,
        ..
    } = syn::parse_macro_input!(input);

    ToResponse::new(attrs, &data, generics, ident)
        .as_ref()
        .map_or_else(Diagnostics::to_token_stream, ToResponse::to_token_stream)
        .into()
}

#[proc_macro_derive(
    IntoResponses,
    attributes(response, to_schema, ref_response, to_response)
)]
/// Generate responses with status codes what
/// can be attached to the [`utoipa::path`][path_into_responses].
///
/// This is `#[derive]` implementation of [`IntoResponses`][into_responses] trait. [`derive@IntoResponses`]
/// can be used to decorate _`structs`_ and _`enums`_ to generate response maps that can be used in
/// [`utoipa::path`][path_into_responses]. If _`struct`_ is decorated with [`derive@IntoResponses`] it will be
/// used to create a map of responses containing single response. Decorating _`enum`_ with
/// [`derive@IntoResponses`] will create a map of responses with a response for each variant of the _`enum`_.
///
/// Named field _`struct`_ decorated with [`derive@IntoResponses`] will create a response with inlined schema
/// generated from the body of the struct. This is a conveniency which allows users to directly
/// create responses with schemas without first creating a separate [response][to_response] type.
///
/// Unit _`struct`_ behaves similarly to then named field struct. Only difference is that it will create
/// a response without content since there is no inner fields.
///
/// Unnamed field _`struct`_ decorated with [`derive@IntoResponses`] will by default create a response with
/// referenced [schema][to_schema] if field is object or schema if type is [primitive
/// type][primitive]. _`#[to_schema]`_ attribute at field of unnamed _`struct`_ can be used to inline
/// the schema if type of the field implements [`ToSchema`][to_schema] trait. Alternatively
/// _`#[to_response]`_ and _`#[ref_response]`_ can be used at field to either reference a reusable
/// [response][to_response] or inline a reusable [response][to_response]. In both cases the field
/// type is expected to implement [`ToResponse`][to_response] trait.
///
///
/// Enum decorated with [`derive@IntoResponses`] will create a response for each variant of the _`enum`_.
/// Each variant must have it's own _`#[response(...)]`_ definition. Unit variant will behave same
/// as unit _`struct`_ by creating a response without content. Similarly named field variant and
/// unnamed field variant behaves the same as it was named field _`struct`_ and unnamed field
/// _`struct`_.
///
/// _`#[response]`_ attribute can be used at named structs, unnamed structs, unit structs and enum
/// variants to alter [response attributes](#intoresponses-response-attributes) of responses.
///
/// Doc comment on a _`struct`_ or _`enum`_ variant will be used as a description for the response.
/// It can also be overridden with _`description = "..."`_ attribute.
///
/// # IntoResponses `#[response(...)]` attributes
///
/// * `status = ...` Must be provided. Is either a valid http status code integer. E.g. _`200`_ or a
///   string value representing a range such as _`"4XX"`_ or `"default"` or a valid _`http::status::StatusCode`_.
///   _`StatusCode`_ can either be use path to the status code or _status code_ constant directly.
///
/// * `description = "..."` Define description for the response as str. This can be used to
///   override the default description resolved from doc comments if present.
///
/// * `content_type = "..."` Can be used to override the default behavior
///   of auto resolving the content type from the `body` attribute. If defined the value should be valid
///   content type such as _`application/json`_ . By default the content type is _`text/plain`_
///   for [primitive Rust types][primitive], `application/octet-stream` for _`[u8]`_ and _`application/json`_
///   for struct and mixed enum types.
///
/// * `headers(...)` Slice of response headers that are returned back to a caller.
///
/// * `example = ...` Can be _`json!(...)`_. _`json!(...)`_ should be something that
///   _`serde_json::json!`_ can parse as a _`serde_json::Value`_.
///
/// * `examples(...)` Define multiple examples for single response. This attribute is mutually
///   exclusive to the _`example`_ attribute and if both are defined this will override the _`example`_.
///     * `name = ...` This is first attribute and value must be literal string.
///     * `summary = ...` Short description of example. Value must be literal string.
///     * `description = ...` Long description of example. Attribute supports markdown for rich text
///       representation. Value must be literal string.
///     * `value = ...` Example value. It must be _`json!(...)`_. _`json!(...)`_ should be something that
///       _`serde_json::json!`_ can parse as a _`serde_json::Value`_.
///     * `external_value = ...` Define URI to literal example value. This is mutually exclusive to
///       the _`value`_ attribute. Value must be literal string.
///
///      _**Example of example definition.**_
///     ```text
///      ("John" = (summary = "This is John", value = json!({"name": "John"})))
///     ```
///
/// # Examples
///
/// _**Use `IntoResponses` to define [`utoipa::path`][path] responses.**_
/// ```rust
/// #[derive(utoipa::ToSchema)]
/// struct BadRequest {
///     message: String,
/// }
///
/// #[derive(utoipa::IntoResponses)]
/// enum UserResponses {
///     /// Success response
///     #[response(status = 200)]
///     Success { value: String },
///
///     #[response(status = 404)]
///     NotFound,
///
///     #[response(status = 400)]
///     BadRequest(BadRequest),
/// }
///
/// #[utoipa::path(
///     get,
///     path = "/api/user",
///     responses(
///         UserResponses
///     )
/// )]
/// fn get_user() -> UserResponses {
///    UserResponses::NotFound
/// }
/// ```
/// _**Named struct response with inlined schema.**_
/// ```rust
/// /// This is success response
/// #[derive(utoipa::IntoResponses)]
/// #[response(status = 200)]
/// struct SuccessResponse {
///     value: String,
/// }
/// ```
///
/// _**Unit struct response without content.**_
/// ```rust
/// #[derive(utoipa::IntoResponses)]
/// #[response(status = NOT_FOUND)]
/// struct NotFound;
/// ```
///
/// _**Unnamed struct response with inlined response schema.**_
/// ```rust
/// # #[derive(utoipa::ToSchema)]
/// # struct Foo;
/// #[derive(utoipa::IntoResponses)]
/// #[response(status = 201)]
/// struct CreatedResponse(#[to_schema] Foo);
/// ```
///
/// _**Enum with multiple responses.**_
/// ```rust
/// # #[derive(utoipa::ToResponse)]
/// # struct Response {
/// #     message: String,
/// # }
/// # #[derive(utoipa::ToSchema)]
/// # struct BadRequest {}
/// #[derive(utoipa::IntoResponses)]
/// enum UserResponses {
///     /// Success response description.
///     #[response(status = 200)]
///     Success { value: String },
///
///     #[response(status = 404)]
///     NotFound,
///
///     #[response(status = 400)]
///     BadRequest(BadRequest),
///
///     #[response(status = 500)]
///     ServerError(#[ref_response] Response),
///
///     #[response(status = 418)]
///     TeaPot(#[to_response] Response),
/// }
/// ```
///
/// [into_responses]: trait.IntoResponses.html
/// [to_schema]: trait.ToSchema.html
/// [to_response]: trait.ToResponse.html
/// [path_into_responses]: attr.path.html#responses-from-intoresponses
/// [primitive]: https://doc.rust-lang.org/std/primitive/index.html
/// [path]: macro@crate::path
pub fn into_responses(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs,
        ident,
        generics,
        data,
        ..
    } = syn::parse_macro_input!(input);

    let into_responses = IntoResponses {
        attributes: attrs,
        ident,
        generics,
        data,
    };

    into_responses.to_token_stream().into()
}

/// Create OpenAPI Schema from arbitrary type.
///
/// This macro provides a quick way to render arbitrary types as OpenAPI Schema Objects. It
/// supports two call formats.
/// 1. With type only
/// 2. With _`#[inline]`_ attribute to inline the referenced schemas.
///
/// By default the macro will create references `($ref)` for non primitive types like _`Pet`_.
/// However when used with _`#[inline]`_ the non [`primitive`][primitive] type schemas will
/// be inlined to the schema output.
///
/// ```rust
/// # use utoipa::openapi::{RefOr, schema::Schema};
/// # #[derive(utoipa::ToSchema)]
/// # struct Pet {id: i32};
/// let schema: RefOr<Schema> = utoipa::schema!(Vec<Pet>).into();
///
/// // with inline
/// let schema: RefOr<Schema> = utoipa::schema!(#[inline] Vec<Pet>).into();
/// ```
///
/// # Examples
///
/// _**Create vec of pets schema.**_
/// ```rust
/// # use utoipa::openapi::schema::{Schema, Array, Object, ObjectBuilder, SchemaFormat,
/// # KnownFormat, Type};
/// # use utoipa::openapi::RefOr;
/// #[derive(utoipa::ToSchema)]
/// struct Pet {
///     id: i32,
///     name: String,
/// }
///
/// let schema: RefOr<Schema> = utoipa::schema!(#[inline] Vec<Pet>).into();
/// // will output
/// let generated = RefOr::T(Schema::Array(
///     Array::new(
///         ObjectBuilder::new()
///             .property("id", ObjectBuilder::new()
///                 .schema_type(Type::Integer)
///                 .format(Some(SchemaFormat::KnownFormat(KnownFormat::Int32)))
///                 .build())
///             .required("id")
///             .property("name", Object::with_type(Type::String))
///             .required("name")
///     )
/// ));
/// # insta::assert_json_snapshot!("schema", &schema);
/// ```
///
/// [primitive]: https://doc.rust-lang.org/std/primitive/index.html
#[proc_macro]
pub fn schema(input: TokenStream) -> TokenStream {
    struct Schema {
        inline: bool,
        ty: syn::Type,
    }
    impl Parse for Schema {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let inline = if input.peek(Token![#]) && input.peek2(Bracket) {
                input.parse::<Token![#]>()?;

                let inline;
                bracketed!(inline in input);
                let i = inline.parse::<Ident>()?;
                i == "inline"
            } else {
                false
            };

            let ty = input.parse()?;

            Ok(Self { inline, ty })
        }
    }

    let schema = syn::parse_macro_input!(input as Schema);
    let type_tree = match TypeTree::from_type(&schema.ty) {
        Ok(type_tree) => type_tree,
        Err(diagnostics) => return diagnostics.into_token_stream().into(),
    };

    let generics = match type_tree.get_path_generics() {
        Ok(generics) => generics,
        Err(error) => return error.into_compile_error().into(),
    };

    let schema = ComponentSchema::new(ComponentSchemaProps {
        features: vec![Feature::Inline(schema.inline.into())],
        type_tree: &type_tree,
        description: None,
        container: &component::Container {
            generics: &generics,
        },
    });

    let schema = match schema {
        Ok(schema) => schema.to_token_stream(),
        Err(diagnostics) => return diagnostics.to_token_stream().into(),
    };

    quote! {
        {
            let mut generics: Vec<utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>> = Vec::new();
            #schema
        }
    }
    .into()
}

/// Tokenizes slice or Vec of tokenizable items as array either with reference (`&[...]`)
/// or without correctly to OpenAPI JSON.
#[cfg_attr(feature = "debug", derive(Debug))]
enum Array<'a, T>
where
    T: Sized + ToTokens,
{
    Owned(Vec<T>),
    #[allow(dead_code)]
    Borrowed(&'a [T]),
}

impl<V> FromIterator<V> for Array<'_, V>
where
    V: Sized + ToTokens,
{
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        Self::Owned(iter.into_iter().collect())
    }
}

impl<'a, T> Deref for Array<'a, T>
where
    T: Sized + ToTokens,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(vec) => vec.as_slice(),
            Self::Borrowed(slice) => slice,
        }
    }
}

impl<T> ToTokens for Array<'_, T>
where
    T: Sized + ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let values = match self {
            Self::Owned(values) => values.iter(),
            Self::Borrowed(values) => values.iter(),
        };

        tokens.append(Group::new(
            proc_macro2::Delimiter::Bracket,
            values
                .fold(Punctuated::new(), |mut punctuated, item| {
                    punctuated.push_value(item);
                    punctuated.push_punct(Punct::new(',', proc_macro2::Spacing::Alone));

                    punctuated
                })
                .to_token_stream(),
        ));
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
enum Deprecated {
    True,
    False,
}

impl From<bool> for Deprecated {
    fn from(bool: bool) -> Self {
        if bool {
            Self::True
        } else {
            Self::False
        }
    }
}

impl ToTokens for Deprecated {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(match self {
            Self::False => quote! { utoipa::openapi::Deprecated::False },
            Self::True => quote! { utoipa::openapi::Deprecated::True },
        })
    }
}

#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
enum Required {
    True,
    False,
}

impl From<bool> for Required {
    fn from(bool: bool) -> Self {
        if bool {
            Self::True
        } else {
            Self::False
        }
    }
}

impl From<features::attributes::Required> for Required {
    fn from(value: features::attributes::Required) -> Self {
        let features::attributes::Required(required) = value;
        crate::Required::from(required)
    }
}

impl ToTokens for Required {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(match self {
            Self::False => quote! { utoipa::openapi::Required::False },
            Self::True => quote! { utoipa::openapi::Required::True },
        })
    }
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
struct ExternalDocs {
    url: String,
    description: Option<String>,
}

impl Parse for ExternalDocs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE: &str = "unexpected attribute, expected any of: url, description";

        let mut external_docs = ExternalDocs::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>().map_err(|error| {
                syn::Error::new(error.span(), format!("{EXPECTED_ATTRIBUTE}, {error}"))
            })?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "url" => {
                    external_docs.url = parse_utils::parse_next_literal_str(input)?;
                }
                "description" => {
                    external_docs.description = Some(parse_utils::parse_next_literal_str(input)?);
                }
                _ => return Err(syn::Error::new(ident.span(), EXPECTED_ATTRIBUTE)),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(external_docs)
    }
}

impl ToTokens for ExternalDocs {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let url = &self.url;
        tokens.extend(quote! {
            utoipa::openapi::external_docs::ExternalDocsBuilder::new()
                .url(#url)
        });

        if let Some(ref description) = self.description {
            tokens.extend(quote! {
                .description(Some(#description))
            });
        }

        tokens.extend(quote! { .build() })
    }
}

/// Represents OpenAPI Any value used in example and default fields.
#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
enum AnyValue {
    String(TokenStream2),
    Json(TokenStream2),
    DefaultTrait {
        struct_ident: Ident,
        field_ident: Member,
    },
}

impl AnyValue {
    /// Parse `json!(...)` as [`AnyValue::Json`]
    fn parse_json(input: ParseStream) -> syn::Result<Self> {
        parse_utils::parse_json_token_stream(input).map(AnyValue::Json)
    }

    fn parse_any(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Lit) {
            let punct = input.parse::<Option<Token![-]>>()?;
            let lit = input.parse::<Lit>().unwrap();

            Ok(AnyValue::Json(quote! { #punct #lit}))
        } else {
            let fork = input.fork();
            let is_json = if fork.peek(syn::Ident) && fork.peek2(Token![!]) {
                let ident = fork.parse::<Ident>().unwrap();
                ident == "json"
            } else {
                false
            };

            if is_json {
                let json = parse_utils::parse_json_token_stream(input)?;

                Ok(AnyValue::Json(json))
            } else {
                let method = input.parse::<ExprPath>().map_err(|error| {
                    syn::Error::new(
                        error.span(),
                        "expected literal value, json!(...) or method reference",
                    )
                })?;

                Ok(AnyValue::Json(quote! { #method() }))
            }
        }
    }

    fn parse_lit_str_or_json(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            Ok(AnyValue::String(
                input.parse::<LitStr>().unwrap().to_token_stream(),
            ))
        } else {
            Ok(AnyValue::Json(parse_utils::parse_json_token_stream(input)?))
        }
    }

    fn new_default_trait(struct_ident: Ident, field_ident: Member) -> Self {
        Self::DefaultTrait {
            struct_ident,
            field_ident,
        }
    }
}

impl ToTokens for AnyValue {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Json(json) => tokens.extend(quote! {
                utoipa::gen::serde_json::json!(#json)
            }),
            Self::String(string) => string.to_tokens(tokens),
            Self::DefaultTrait {
                struct_ident,
                field_ident,
            } => tokens.extend(quote! {
                utoipa::gen::serde_json::to_value(#struct_ident::default().#field_ident).unwrap()
            }),
        }
    }
}

trait OptionExt<T> {
    fn map_try<F, U, E>(self, f: F) -> Result<Option<U>, E>
    where
        F: FnOnce(T) -> Result<U, E>;
    fn and_then_try<F, U, E>(self, f: F) -> Result<Option<U>, E>
    where
        F: FnOnce(T) -> Result<Option<U>, E>;
    fn or_else_try<F, U>(self, f: F) -> Result<Option<T>, U>
    where
        F: FnOnce() -> Result<Option<T>, U>;
}

impl<T> OptionExt<T> for Option<T> {
    fn map_try<F, U, E>(self, f: F) -> Result<Option<U>, E>
    where
        F: FnOnce(T) -> Result<U, E>,
    {
        if let Some(v) = self {
            f(v).map(Some)
        } else {
            Ok(None)
        }
    }

    fn and_then_try<F, U, E>(self, f: F) -> Result<Option<U>, E>
    where
        F: FnOnce(T) -> Result<Option<U>, E>,
    {
        if let Some(v) = self {
            match f(v) {
                Ok(inner) => Ok(inner),
                Err(error) => Err(error),
            }
        } else {
            Ok(None)
        }
    }

    fn or_else_try<F, U>(self, f: F) -> Result<Option<T>, U>
    where
        F: FnOnce() -> Result<Option<T>, U>,
    {
        if self.is_none() {
            f()
        } else {
            Ok(self)
        }
    }
}

trait GenericsExt {
    /// Get index of `GenericParam::Type` ignoring other generic param types.
    fn get_generic_type_param_index(&self, type_tree: &TypeTree) -> Option<usize>;
}

impl<'g> GenericsExt for &'g syn::Generics {
    fn get_generic_type_param_index(&self, type_tree: &TypeTree) -> Option<usize> {
        let ident = &type_tree
            .path
            .as_ref()
            .expect("TypeTree of generic object must have a path")
            .segments
            .last()
            .expect("Generic object path must have at least one segment")
            .ident;

        self.params
            .iter()
            .filter(|generic| matches!(generic, GenericParam::Type(_)))
            .enumerate()
            .find_map(|(index, generic)| {
                if matches!(generic, GenericParam::Type(ty) if ty.ident == *ident) {
                    Some(index)
                } else {
                    None
                }
            })
    }
}

trait ToTokensDiagnostics {
    fn to_tokens(&self, tokens: &mut TokenStream2) -> Result<(), Diagnostics>;

    #[allow(unused)]
    fn into_token_stream(self) -> TokenStream2
    where
        Self: std::marker::Sized,
    {
        ToTokensDiagnostics::to_token_stream(&self)
    }

    fn to_token_stream(&self) -> TokenStream2 {
        let mut tokens = TokenStream2::new();
        match ToTokensDiagnostics::to_tokens(self, &mut tokens) {
            Ok(_) => tokens,
            Err(error_stream) => Into::<Diagnostics>::into(error_stream).into_token_stream(),
        }
    }

    fn try_to_token_stream(&self) -> Result<TokenStream2, Diagnostics> {
        let mut tokens = TokenStream2::new();
        match ToTokensDiagnostics::to_tokens(self, &mut tokens) {
            Ok(_) => Ok(tokens),
            Err(diagnostics) => Err(diagnostics),
        }
    }
}

macro_rules! as_tokens_or_diagnostics {
    ( $type:expr ) => {{
        let mut _tokens = proc_macro2::TokenStream::new();
        match crate::ToTokensDiagnostics::to_tokens($type, &mut _tokens) {
            Ok(_) => _tokens,
            Err(diagnostics) => return Err(diagnostics),
        }
    }};
}

use as_tokens_or_diagnostics;

#[derive(Debug)]
struct Diagnostics {
    diagnostics: Vec<DiangosticsInner>,
}

#[derive(Debug)]
struct DiangosticsInner {
    span: Span,
    message: Cow<'static, str>,
    suggestions: Vec<Suggestion>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Suggestion {
    Help(Cow<'static, str>),
    Note(Cow<'static, str>),
}

impl Display for Diagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl Display for Suggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Help(help) => {
                let s: &str = help.borrow();
                write!(f, "help = {}", s)
            }
            Self::Note(note) => {
                let s: &str = note.borrow();
                write!(f, "note = {}", s)
            }
        }
    }
}

impl Diagnostics {
    fn message(&self) -> Cow<'static, str> {
        self.diagnostics
            .first()
            .as_ref()
            .map(|diagnostics| diagnostics.message.clone())
            .unwrap_or_else(|| Cow::Borrowed(""))
    }

    pub fn new<S: Into<Cow<'static, str>>>(message: S) -> Self {
        Self::with_span(Span::call_site(), message)
    }

    pub fn with_span<S: Into<Cow<'static, str>>>(span: Span, message: S) -> Self {
        Self {
            diagnostics: vec![DiangosticsInner {
                span,
                message: message.into(),
                suggestions: Vec::new(),
            }],
        }
    }

    pub fn help<S: Into<Cow<'static, str>>>(mut self, help: S) -> Self {
        if let Some(diagnostics) = self.diagnostics.first_mut() {
            diagnostics.suggestions.push(Suggestion::Help(help.into()));
            diagnostics.suggestions.sort();
        }

        self
    }

    pub fn note<S: Into<Cow<'static, str>>>(mut self, note: S) -> Self {
        if let Some(diagnostics) = self.diagnostics.first_mut() {
            diagnostics.suggestions.push(Suggestion::Note(note.into()));
            diagnostics.suggestions.sort();
        }

        self
    }
}

impl From<syn::Error> for Diagnostics {
    fn from(value: syn::Error) -> Self {
        Self::with_span(value.span(), value.to_string())
    }
}

impl ToTokens for Diagnostics {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for diagnostics in &self.diagnostics {
            let span = diagnostics.span;
            let message: &str = diagnostics.message.borrow();

            let suggestions = diagnostics
                .suggestions
                .iter()
                .map(Suggestion::to_string)
                .collect::<Vec<_>>()
                .join("\n");

            let diagnostics = if !suggestions.is_empty() {
                Cow::Owned(format!("{message}\n\n{suggestions}"))
            } else {
                Cow::Borrowed(message)
            };

            tokens.extend(quote_spanned! {span=>
                ::core::compile_error!(#diagnostics);
            })
        }
    }
}

impl Error for Diagnostics {}

impl FromIterator<Diagnostics> for Option<Diagnostics> {
    fn from_iter<T: IntoIterator<Item = Diagnostics>>(iter: T) -> Self {
        iter.into_iter().reduce(|mut acc, diagnostics| {
            acc.diagnostics.extend(diagnostics.diagnostics);
            acc
        })
    }
}

trait AttributesExt {
    fn has_deprecated(&self) -> bool;
}

impl AttributesExt for Vec<syn::Attribute> {
    fn has_deprecated(&self) -> bool {
        let this = &**self;
        this.has_deprecated()
    }
}

impl<'a> AttributesExt for &'a [syn::Attribute] {
    fn has_deprecated(&self) -> bool {
        self.iter().any(|attr| {
            matches!(attr.path().get_ident(), Some(ident) if &*ident.to_string() == "deprecated")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_ordering_help_comes_before_note() {
        let diagnostics = Diagnostics::new("this an error")
            .note("you could do this to solve the error")
            .help("try this thing");

        let tokens = diagnostics.into_token_stream();

        let expected_tokens = quote::quote!(::core::compile_error!(
            "this an error\n\nhelp = try this thing\nnote = you could do this to solve the error"
        ););

        assert_eq!(tokens.to_string(), expected_tokens.to_string());
    }
}

/// Parsing utils
mod parse_utils {
    use std::fmt::Display;

    use proc_macro2::{Group, Ident, TokenStream};
    use quote::{quote, ToTokens};
    use syn::{
        parenthesized,
        parse::{Parse, ParseStream},
        punctuated::Punctuated,
        spanned::Spanned,
        token::Comma,
        Error, Expr, ExprPath, LitBool, LitStr, Token,
    };

    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub enum LitStrOrExpr {
        LitStr(LitStr),
        Expr(Expr),
    }

    impl From<String> for LitStrOrExpr {
        fn from(value: String) -> Self {
            Self::LitStr(LitStr::new(&value, proc_macro2::Span::call_site()))
        }
    }

    impl LitStrOrExpr {
        pub(crate) fn is_empty_litstr(&self) -> bool {
            matches!(self, Self::LitStr(s) if s.value().is_empty())
        }
    }

    impl Default for LitStrOrExpr {
        fn default() -> Self {
            Self::LitStr(LitStr::new("", proc_macro2::Span::call_site()))
        }
    }

    impl Parse for LitStrOrExpr {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            if input.peek(LitStr) {
                Ok::<LitStrOrExpr, Error>(LitStrOrExpr::LitStr(input.parse::<LitStr>()?))
            } else {
                Ok(LitStrOrExpr::Expr(input.parse::<Expr>()?))
            }
        }
    }

    impl ToTokens for LitStrOrExpr {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            match self {
                Self::LitStr(str) => str.to_tokens(tokens),
                Self::Expr(expr) => expr.to_tokens(tokens),
            }
        }
    }

    impl Display for LitStrOrExpr {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::LitStr(str) => write!(f, "{str}", str = str.value()),
                Self::Expr(expr) => write!(f, "{expr}", expr = expr.into_token_stream()),
            }
        }
    }

    pub fn parse_next<T: FnOnce() -> Result<R, syn::Error>, R: Sized>(
        input: ParseStream,
        next: T,
    ) -> Result<R, syn::Error> {
        input.parse::<Token![=]>()?;
        next()
    }

    pub fn parse_next_literal_str(input: ParseStream) -> syn::Result<String> {
        Ok(parse_next(input, || input.parse::<LitStr>())?.value())
    }

    pub fn parse_next_literal_str_or_expr(input: ParseStream) -> syn::Result<LitStrOrExpr> {
        parse_next(input, || LitStrOrExpr::parse(input)).map_err(|error| {
            syn::Error::new(
                error.span(),
                format!("expected literal string or expression argument: {error}"),
            )
        })
    }

    pub fn parse_groups_collect<T, R>(input: ParseStream) -> syn::Result<R>
    where
        T: Sized,
        T: Parse,
        R: FromIterator<T>,
    {
        Punctuated::<Group, Comma>::parse_terminated(input).and_then(|groups| {
            groups
                .into_iter()
                .map(|group| syn::parse2::<T>(group.stream()))
                .collect::<syn::Result<R>>()
        })
    }

    pub fn parse_parethesized_terminated<T: Parse, S: Parse>(
        input: ParseStream,
    ) -> syn::Result<Punctuated<T, S>> {
        let group;
        syn::parenthesized!(group in input);
        Punctuated::parse_terminated(&group)
    }

    pub fn parse_comma_separated_within_parethesis_with<T>(
        input: ParseStream,
        with: fn(ParseStream) -> syn::Result<T>,
    ) -> syn::Result<Punctuated<T, Comma>>
    where
        T: Parse,
    {
        let content;
        parenthesized!(content in input);
        Punctuated::<T, Comma>::parse_terminated_with(&content, with)
    }

    pub fn parse_comma_separated_within_parenthesis<T>(
        input: ParseStream,
    ) -> syn::Result<Punctuated<T, Comma>>
    where
        T: Parse,
    {
        let content;
        parenthesized!(content in input);
        Punctuated::<T, Comma>::parse_terminated(&content)
    }

    pub fn parse_bool_or_true(input: ParseStream) -> syn::Result<bool> {
        if input.peek(Token![=]) && input.peek2(LitBool) {
            input.parse::<Token![=]>()?;

            Ok(input.parse::<LitBool>()?.value())
        } else {
            Ok(true)
        }
    }

    /// Parse `json!(...)` as a [`TokenStream`].
    pub fn parse_json_token_stream(input: ParseStream) -> syn::Result<TokenStream> {
        if input.peek(syn::Ident) && input.peek2(Token![!]) {
            input.parse::<Ident>().and_then(|ident| {
                if ident != "json" {
                    return Err(Error::new(
                        ident.span(),
                        format!("unexpected token {ident}, expected: json!(...)"),
                    ));
                }

                Ok(ident)
            })?;
            input.parse::<Token![!]>()?;

            Ok(input.parse::<Group>()?.stream())
        } else {
            Err(Error::new(
                input.span(),
                "unexpected token, expected json!(...)",
            ))
        }
    }

    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub enum LitBoolOrExprPath {
        LitBool(LitBool),
        ExprPath(ExprPath),
    }

    impl From<bool> for LitBoolOrExprPath {
        fn from(value: bool) -> Self {
            Self::LitBool(LitBool::new(value, proc_macro2::Span::call_site()))
        }
    }

    impl Default for LitBoolOrExprPath {
        fn default() -> Self {
            Self::LitBool(LitBool::new(false, proc_macro2::Span::call_site()))
        }
    }

    impl Parse for LitBoolOrExprPath {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            if input.peek(LitBool) {
                Ok(LitBoolOrExprPath::LitBool(input.parse::<LitBool>()?))
            } else {
                let expr = input.parse::<Expr>()?;

                match expr {
                    Expr::Path(expr_path) => Ok(LitBoolOrExprPath::ExprPath(expr_path)),
                    _ => Err(syn::Error::new(
                        expr.span(),
                        format!(
                            "expected literal bool or path to a function that returns bool, found: {}",
                            quote! {#expr}
                        ),
                    )),
                }
            }
        }
    }

    impl ToTokens for LitBoolOrExprPath {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            match self {
                Self::LitBool(bool) => bool.to_tokens(tokens),
                Self::ExprPath(call) => call.to_tokens(tokens),
            }
        }
    }

    pub fn parse_next_literal_bool_or_call(input: ParseStream) -> syn::Result<LitBoolOrExprPath> {
        if input.peek(Token![=]) {
            parse_next(input, || LitBoolOrExprPath::parse(input))
        } else {
            Ok(LitBoolOrExprPath::from(true))
        }
    }
}
