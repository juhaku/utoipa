//! This is **private** utoipa codegen library and is not used alone.
//!
//! The library contains macro implementations for utoipa library. Content
//! of the library documentation is available through **utoipa** library itself.
//! Consider browsing via the **utoipa** crate so all links will work correctly.

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

use std::{borrow::Cow, mem, ops::Deref};

use component::schema::Schema;
use doc_comment::CommentAttributes;

use component::into_params::IntoParams;
use ext::{PathOperationResolver, PathOperations, PathResolver};
use openapi::OpenApi;
use proc_macro::TokenStream;
use proc_macro_error::{proc_macro_error, OptionExt, ResultExt};
use quote::{quote, ToTokens, TokenStreamExt};

use proc_macro2::{Group, Ident, Punct, TokenStream as TokenStream2};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    AngleBracketedGenericArguments, DeriveInput, ExprPath, GenericArgument, ItemFn, Lit, LitStr,
    PathArguments, PathSegment, Token, TypePath,
};

mod component;
mod doc_comment;
mod ext;
mod openapi;
mod path;
mod schema_type;
mod security_requirement;

use crate::path::{Path, PathAttr};

#[cfg(any(
    feature = "actix_extras",
    feature = "rocket_extras",
    feature = "axum_extras"
))]
use ext::ArgumentResolver;

#[proc_macro_error]
#[proc_macro_derive(ToSchema, attributes(schema, aliases))]
/// ToSchema derive macro.
///
/// This is `#[derive]` implementation for [`ToSchema`][to_schema] trait. The macro accepts one
/// `schema`
/// attribute optionally which can be used to enhance generated documentation. The attribute can be placed
/// at item level or field level in struct and enums. Currently placing this attribute to unnamed field does
/// not have any effect.
///
/// You can use the Rust's own `#[deprecated]` attribute on any struct, enum or field to mark it as deprecated and it will
/// reflect to the generated OpenAPI spec.
///
/// `#[deprecated]` attribute supports adding additional details such as a reason and or since version but this is is not supported in
/// OpenAPI. OpenAPI has only a boolean flag to determine deprecation. While it is totally okay to declare deprecated with reason
/// `#[deprecated  = "There is better way to do this"]` the reason would not render in OpenAPI spec.
///
/// # Struct Optional Configuration Options for `#[schema(...)]`
/// * `example = ...` Can be either _`json!(...)`_ or literal string that can be parsed to json. _`json!`_
///   should be something that _`serde_json::json!`_ can parse as a _`serde_json::Value`_. [^json]
/// * `xml(...)` Can be used to define [`Xml`][xml] object properties applicable to Structs.
///  
/// [^json]: **json** feature need to be enabled for _`json!(...)`_ type to work.
///
/// # Enum Optional Configuration Options for `#[schema(...)]`
/// * `example = ...` Can be literal value, method reference or _`json!(...)`_. [^json2]
/// * `default = ...` Can be literal value, method reference or _`json!(...)`_. [^json2]
///
/// # Unnamed Field Struct Optional Configuration Options for `#[schema(...)]`
/// * `example = ...` Can be literal value, method reference or _`json!(...)`_. [^json2]
/// * `default = ...` Can be literal value, method reference or _`json!(...)`_. [^json2]
/// * `format = ...` Any variant of a [`SchemaFormat`][format] to use for the property. By default the format is derived from
///   the type of the property according OpenApi spec.
/// * `value_type = ...` Can be used to override default type derived from type of the field used in OpenAPI spec.
///   This is useful in cases where the default type does not correspond to the actual type e.g. when
///   any third-party types are used which are not [`ToSchema`][to_schema]s nor [`primitive` types][primitive].
///    Value can be any Rust type what normally could be used to serialize to JSON or custom type such as _`Object`_.
///
/// # Named Fields Optional Configuration Options for `#[schema(...)]`
/// * `example = ...` Can be literal value, method reference or _`json!(...)`_. [^json2]
/// * `default = ...` Can be literal value, method reference or _`json!(...)`_. [^json2]
/// * `format = ...` Any variant of a [`SchemaFormat`][format] to use for the property. By default the format is derived from
///   the type of the property according OpenApi spec.
/// * `write_only` Defines property is only used in **write** operations *POST,PUT,PATCH* but not in *GET*
/// * `read_only` Defines property is only used in **read** operations *GET* but not in *POST,PUT,PATCH*
/// * `xml(...)` Can be used to define [`Xml`][xml] object properties applicable to named fields.
/// * `value_type = ...` Can be used to override default type derived from type of the field used in OpenAPI spec.
///   This is useful in cases where the default type does not correspond to the actual type e.g. when
///   any third-party types are used which are not [`ToSchema`][to_schema]s nor [`primitive` types][primitive].
///    Value can be any Rust type what normally could be used to serialize to JSON or custom type such as _`Object`_.
/// * `inline` If the type of this field implements [`ToSchema`][to_schema], then the schema definition
///   will be inlined. **warning:** Don't use this for recursive data types!
///
/// [^json2]: Values are converted to string if **json** feature is not enabled.
///
/// # Xml attribute Configuration Options
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
/// # Partial `#[serde(...)]` attributes support
///
/// ToSchema derive has partial support for [serde attributes]. These supported attributes will reflect to the
/// generated OpenAPI doc. For example if _`#[serde(skip)]`_ is defined the attribute will not show up in the OpenAPI spec at all since it will not never
/// be serialized anyway. Similarly the _`rename`_ and _`rename_all`_ will reflect to the generated OpenAPI doc.
///
/// * `rename_all = "..."` Supported in container level.
/// * `rename = "..."` Supported **only** in field or variant level.
/// * `skip = "..."` Supported  **only** in field or variant level.
/// * `tag = "..."` Supported in container level. `tag` attribute also works as a [discriminator field][discriminator] for an enum.
/// * `default` Supported in container level and field level according to [serde attributes].
///
/// Other _`serde`_ attributes works as is but does not have any effect on the generated OpenAPI doc.
///
/// **Note!** `tag` attribute has some limitations like it cannot be used
/// with **unnamed field structs** and **tuple types**.  See more at
/// [enum representation docs](https://serde.rs/enum-representations.html).
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
/// Add custom `tag` to change JSON representation to be internally tagged.
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
/// Add serde `default` attribute for MyValue struct. Similarly `default` could be added to
/// individual fields as well. If `default` is given the field's affected will be treated
/// as optional.
/// ```rust
///  #[derive(utoipa::ToSchema, serde::Deserialize, Default)]
///  #[serde(default)]
///  struct MyValue {
///      field: String
///  }
/// ```
///
/// # `#[repr(...)]` attribute support
/// ToSchema derive has support for `repr(u*)` and `repr(i*)` attributes for fieldless enums.
/// This allows you to create enums from thier discriminant values.
/// **repr** feature need to be enabled.
/// Otherwise, string representations of the fields will be used as values.
/// ```rust
/// # use serde::{Deserialize, Serialize};
/// # use utoipa::ToSchema;
/// #[derive(ToSchema, Deserialize, Serialize)]
/// #[repr(u8)]
/// enum ApiVersion {
///     One = 1,
///     Two,
///     Three,
/// }
/// ```
/// You can use `skip` and `tag` attributes from serde.
/// ```rust
/// # use serde::{Deserialize, Serialize};
/// # use utoipa::ToSchema;
/// #[derive(ToSchema, Deserialize, Serialize)]
/// #[repr(i8)]
/// #[serde(tag = "code")]
/// enum ExitCode {
///     Error = -1,
///     #[serde(skip)]
///     Unknown = 0,
///     Ok = 1,
///  }
/// ```
/// As well as [`schema attributes`][enum_schema] for enums.
/// ```rust
/// # use serde::{Deserialize, Serialize};
/// # use utoipa::ToSchema;
/// #[derive(ToSchema, Deserialize, Serialize)]
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
/// # Generic schemas with aliases
///
/// Schemas can also be generic which allows reusing types. This enables certain behaviour patters
/// where super type delcares common code for type aliases.
///
/// In this example we have common `Status` type which accepts one generic type. It is then defined
/// with `#[aliases(...)]` that it is going to be used with [`std::string::String`] and [`i32`] values.
/// The generic argument could also be another [`ToSchema`][to_schema] as well.
/// ```rust
/// # use utoipa::{ToSchema, OpenApi};
/// #[derive(ToSchema)]
/// #[aliases(StatusMessage = Status<String>, StatusNumber = Status<i32>)]
/// struct Status<T> {
///     value: T
/// }
///
/// #[derive(OpenApi)]
/// #[openapi(
///     components(schemas(StatusMessage, StatusNumber))
/// )]
/// struct ApiDoc;
/// ```
///
/// The `#[aliases(...)]` is just syntatic sugar and will create Rust [type aliases](https://doc.rust-lang.org/reference/items/type-aliases.html)
/// behind the scenes which then can be later referenced anywhere in code.
///
/// **Note!** You should never register generic type itself in `components(...)` so according above example `Status<...>` should not be registered
/// because it will not render the type correctly and will cause an error in generated OpenAPI spec.
///
/// # Examples
///
/// Example struct with struct level example.
/// ```rust
/// # use utoipa::ToSchema;
/// #[derive(ToSchema)]
/// #[schema(example = json!({"name": "bob the cat", "id": 0}))]
/// struct Pet {
///     id: u64,
///     name: String,
///     age: Option<i32>,
/// }
/// ```
///
/// The `schema` attribute can also be placed at field level as follows.
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
/// You can also use method reference for attribute values.
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
/// For enums and unnamed field structs you can define `schema` at type level.
/// ```rust
/// # use utoipa::ToSchema;
/// #[derive(ToSchema)]
/// #[schema(example = "Bus")]
/// enum VehicleType {
///     Rocket, Car, Bus, Submarine
/// }
/// ```
///
/// Also you write complex enum combining all above types.
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
/// It is possible to specify the title of each variant to help generators create named structures.
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
/// Use `xml` attribute to manipulate xml output.
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
/// Use of Rust's own `#[deprecated]` attribute will reflect to generated OpenAPI spec.
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
/// Enforce type being used in OpenAPI spec to [`String`] with `value_type` and set format to octet stream
/// with [`SchemaFormat::Binary`][binary].
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
/// Enforce type being used in OpenAPI spec to [`String`] with `value_type` option.
/// ```rust
/// # use utoipa::ToSchema;
/// #[derive(ToSchema)]
/// #[schema(value_type = String)]
/// struct Value(i64);
/// ```
///
/// Override the `Bar` reference with a `custom::NewBar` reference.
/// ```rust
/// # use utoipa::ToSchema;
/// #  mod custom {
/// #      struct NewBar;
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
/// Use a virtual `Object` type to render generic `object` in OpenAPI spec.
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
///
/// More examples for _`value_type`_ in [`IntoParams` derive docs][into_params].
///
/// [to_schema]: trait.ToSchema.html
/// [format]: openapi/schema/enum.SchemaFormat.html
/// [binary]: openapi/schema/enum.SchemaFormat.html#variant.Binary
/// [xml]: openapi/xml/struct.Xml.html
/// [into_params]: derive.IntoParams.html
/// [primitive]: https://doc.rust-lang.org/std/primitive/index.html
/// [serde attributes]: https://serde.rs/attributes.html
/// [discriminator]: openapi/schema/struct.Discriminator.html
/// [enum_schema]: derive.ToSchema.html#enum-optional-configuration-options-for-schema
pub fn derive_to_schema(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs,
        ident,
        data,
        generics,
        vis,
    } = syn::parse_macro_input!(input);

    let schema = Schema::new(&data, &attrs, &ident, &generics, &vis);

    schema.to_token_stream().into()
}

#[proc_macro_error]
#[proc_macro_attribute]
/// Path attribute macro.
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
/// # Path Attributes
///
/// * `operation` _**Must be first parameter!**_ Accepted values are known http operations such as
///   _`get, post, put, delete, head, options, connect, patch, trace`_.
/// * `path = "..."` Must be OpenAPI format compatible str with arguments withing curly braces. E.g _`{id}`_
/// * `operation_id = "..."` Unique operation id for the endpoint. By default this is mapped to function name.
/// * `context_path = "..."` Can add optional scope for **path**. The **context_path** will be prepended to beginning of **path**.
///   This is particularly useful when **path** does not contain the full path to the endpoint. For example if web framework
///   allows operation to be defined under some context path or scope which does not reflect to the resolved path then this
///   **context_path** can become handy to alter the path.
/// * `tag = "..."` Can be used to group operations. Operations with same tag are grouped together. By default
///   this is derived from the handler that is given to [`OpenApi`][openapi]. If derive results empty str
///   then default value _`crate`_ is used instead.
/// * `request_body = ... | request_body(...)` Defining request body indicates that the request is expecting request body within
///   the performed request.
/// * `responses(...)` Slice of responses the endpoint is going to possibly return to the caller.
/// * `params(...)` Slice of params that the endpoint accepts.
/// * `security(...)` List of [`SecurityRequirement`][security]s local to the path operation.
///
///
/// # Request Body Attributes
///
/// * `content = ...` Can be used to define the content object. Should be an identifier, slice or option
///   E.g. _`Pet`_ or _`[Pet]`_ or _`Option<Pet>`_. Where the type implments [`ToSchema`][to_schema],
///   it can also be  wrapped in `inline(...)` in order to inline the schema definition.
///   E.g. _`inline(Pet)`_.
/// * `description = "..."` Define the description for the request body object as str.
/// * `content_type = "..."` Can be used to override the default behavior of auto resolving the content type
///   from the `content` attribute. If defined the value should be valid content type such as
///   _`application/json`_. By default the content type is _`text/plain`_ for
///   [primitive Rust types][primitive] and _`application/json`_ for struct and complex enum types.
///
/// **Request body supports following formats:**
///
/// ```text
/// request_body(content = String, description = "Xml as string request", content_type = "text/xml"),
/// request_body = Pet,
/// request_body = Option<[Pet]>,
/// ```
///
/// 1. First is the long representation of the request body definition.
/// 2. Second is the quick format which only defines the content object type.
/// 3. Last one is same quick format but only with optional request body.
///
/// # Responses Attributes
///
/// * `status = ...` Is either a valid http status code integer. E.g. _`200`_ or a string value representing a range such as `"4XX"` or `"default"`.
/// * `description = "..."` Define description for the response as str.
/// * `body = ...` Optional response body object type. When left empty response does not expect to send any
///   response body. Should be an identifier or slice. E.g _`Pet`_ or _`[Pet]`_. Where the type implments [`ToSchema`][to_schema],
///   it can also be wrapped in `inline(...)` in order to inline the schema definition. E.g. _`inline(Pet)`_.
/// * `content_type = "..." | content_type = [...]` Can be used to override the default behavior of auto resolving the content type
///   from the `body` attribute. If defined the value should be valid content type such as
///   _`application/json`_. By default the content type is _`text/plain`_ for
///   [primitive Rust types][primitive] and _`application/json`_ for struct and complex enum types.
///   Content type can also be slice of **content_type** values if the endpoint support returning multiple
///  response content types. E.g _`["application/json", "text/xml"]`_ would indicate that endpoint can return both
///  _`json`_ and _`xml`_ formats. **The order** of the content types define the default example show first in
///  the Swagger UI. Swagger UI wil use the first _`content_type`_ value as a default example.
/// * `headers(...)` Slice of response headers that are returned back to a caller.
/// * `example = ...` Can be either `json!(...)` or literal str that can be parsed to json. `json!`
///   should be something that `serde_json::json!` can parse as a `serde_json::Value`. [^json]
/// * `response = ...` Type what implements [`ToResponse`][to_response_trait] trait. This can alternatively be used to
///    define response attributes. _`response`_ attribute cannot co-exist with other than _`status`_ attribute.
///
/// **Minimal response format:**
/// ```text
/// responses(
///     (status = 200, description = "success response"),
///     (status = 404, description = "resource missing"),
///     (status = "5XX", description = "server error"),
/// )
/// ```
///
/// **Response with all possible values:**
/// ```text
/// responses(
///     (status = 200, description = "Success response", body = Pet, content_type = "application/json",
///         headers(...),
///         example = json!({"id": 1, "name": "bob the cat"})
///     )
/// )
/// ```
///
/// **Response with multiple response content types:**
/// ```text
/// responses(
///     (status = 200, description = "Success response", body = Pet, content_type = ["application/json", "text/xml"])
/// )
/// ```
///
/// **Reference a reusable response type:**
///
/// `ReusableResponse` must be a type that implements [`ToResponse`][to_response_trait]
///
/// ```text
/// responses(
///     (status = 200, response = ReusableResponse)
/// )
/// ```
///
/// ## Responses from `IntoResponses`
///
/// Responses for a path can be specified with one or more types that implement
/// [`IntoResponses`][into_responses_trait]:
///
/// ```text
/// responses(MyResponse)
/// ```
///
/// # Response Header Attributes
///
/// * `name` Name of the header. E.g. _`x-csrf-token`_
/// * `type` Additional type of the header value. Type is defined after `name` with equals sign before the type.
///   Type should be identifier or slice of identifiers. E.g. _`String`_ or _`[String]`_
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
/// tuples seperated by commas:
///
/// * `name` _**Must be the first argument**_. Define the name for parameter.
/// * `parameter_type` Define possible type for the parameter. Type should be an identifier, slice `[Type]`,
///   option `Option<Type>`. Where the type implments [`ToSchema`][to_schema], it can also be wrapped in `inline(MySchema)`
///   in order to inline the schema definition.
///   E.g. _`String`_ or _`[String]`_ or _`Option<String>`_. Parameter type is placed after `name` with
///   equals sign E.g. _`"id" = String`_
/// * `in` _**Must be placed after name or parameter_type**_. Define the place of the parameter.
///   This must be one of the variants of [`openapi::path::ParameterIn`][in_enum].
///   E.g. _`Path, Query, Header, Cookie`_
/// * `deprecated` Define whether the parameter is deprecated or not.
/// * `description = "..."` Define possible description for the parameter as str.
/// * `style = ...` Defines how parameters are serialized by [`ParameterStyle`][style]. Default values are based on _`in`_ attribute.
/// * `explode` Defines whether new _`parameter=value`_ is created for each parameter withing _`object`_ or _`array`_.
/// * `allow_reserved` Defines whether reserved characters _`:/?#[]@!$&'()*+,;=`_ is allowed within value.
/// * `example = ...` Can be literal value, method reference or _`json!(...)`_. [^json]. Given example
///   will override any example in underlying parameter type.
///
/// **For example:**
///
/// ```text
/// params(
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
///         example = json!(["Value"]))
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
/// [into_params]: ./trait.IntoParams.html
/// **For example:**
///
/// ```text
/// params(MyParameters)
/// ```
///
/// Note that `MyParameters` can also be used in combination with the [tuples
/// representation](#tuples) or other structs. **For example:**
///
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
///   [`SecuritySchema`][security_schema].
/// * `scopes = [...]` Define the list of scopes needed. These must be scopes defined already in
///   existing [`SecuritySchema`][security_schema].
///
/// **Security Requirement supported formats:**
///
/// ```text
/// (),
/// ("name" = []),
/// ("name" = ["scope1", "scope2"]),
/// ```
///
/// Leaving empty _`()`_ creates an empty [`SecurityRequirement`][security] this is useful when
/// security requirement is optional for operation.
///
/// # actix_extras support for actix-web
///
/// **actix_extras** feature gives **utoipa** ability to parse path operation information from **actix-web** types and macros.
///
/// 1. Ability to parse `path` from **actix-web** path attribute macros e.g. _`#[get(...)]`_.
/// 2. Ability to parse [`std::primitive`]  or [`String`] or [`tuple`] typed `path` parameters from **actix-web** _`web::Path<...>`_.
/// 3. Ability to parse `path` and `query` parameters form **actix-web** _`web::Path<...>`_, _`web::Query<...>`_ types
///    with [`IntoParams`][into_params] trait.
///
/// See the **actix_extras** in action in examples [todo-actix](https://github.com/juhaku/utoipa/tree/master/examples/todo-actix).
///
/// With **actix_extras** feature enabled the you can leave out definitions for **path**, **operation**
/// and **parameter types** [^actix_extras].
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
/// resolved from path and the argument types of handler. [^actix_extras]
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
/// # rocket_extras support for rocket
///
/// **rocket_extras** feature enahances path operation parameter support. It gives **utoipa** ability to parse `path`, `path parameters`
/// and `query parameters` based on arguments given to **rocket**  proc macros such as _**`#[get(...)]`**_.  
///
/// 1. It is able to parse parameter types for [primitive types][primitive], [`String`], [`Vec`], [`Option`] or [`std::path::PathBuf`]
///    type.
/// 2. It is able to determine `parameter_in` for [`IntoParams`][into_params] trait used for `FromForm` type of query parameters.
///
/// See the **rocket_extras** in action in examples [rocket-todo](https://github.com/juhaku/utoipa/tree/master/examples/rocket-todo).
///
///
/// # axum_extras suppport for axum
///
/// **axum_extras** feature enhances [`IntoParams` derive][into_params_derive] functionality by automatically resolving _`parameter_in`_ from
/// _`Path<...>`_ or _`Query<...>`_ handler function arguments.
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
/// # Examples
///
/// Example with all possible arguments.
/// ```rust
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
/// More minimal example with the defaults.
/// ```rust
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
/// Use of Rust's own `#[deprecated]` attribute will reflect to the generated OpenAPI spec and mark this operation as deprecated.
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
/// Define context path for endpoint. The resolved **path** shown in OpenAPI doc will be `/api/pet/{id}`.
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
/// [in_enum]: utoipa/openapi/path/enum.ParameterIn.html
/// [path]: trait.Path.html
/// [to_schema]: trait.ToSchema.html
/// [openapi]: derive.OpenApi.html
/// [security]: openapi/security/struct.SecurityRequirement.html
/// [security_schema]: openapi/security/struct.SecuritySchema.html
/// [primitive]: https://doc.rust-lang.org/std/primitive/index.html
/// [into_params]: trait.IntoParams.html
/// [style]: openapi/path/enum.ParameterStyle.html
/// [into_responses_trait]: trait.IntoResponses.html
/// [into_params_derive]: derive.IntoParams.html
/// [to_response_trait]: trait.ToResponse.html
///
/// [^json]: **json** feature need to be enabled for `json!(...)` type to work.
///
/// [^actix_extras]: **actix_extras** feature need to be enabled and **actix-web** framework must be declared in your `Cargo.toml`.
pub fn path(attr: TokenStream, item: TokenStream) -> TokenStream {
    let path_attribute = syn::parse_macro_input!(attr as PathAttr);

    #[cfg(any(
        feature = "actix_extras",
        feature = "rocket_extras",
        feature = "axum_extras"
    ))]
    let mut path_attribute = path_attribute;

    let ast_fn = syn::parse::<ItemFn>(item).unwrap_or_abort();
    let fn_name = &*ast_fn.sig.ident.to_string();

    let mut resolved_operation = PathOperations::resolve_operation(&ast_fn);

    let resolved_path = PathOperations::resolve_path(
        &resolved_operation
            .as_mut()
            .map(|operation| mem::take(&mut operation.path))
            .or_else(|| path_attribute.path.as_ref().map(String::to_string)), // cannot use mem take because we need this later
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
        let args = resolved_path.as_mut().map(|path| mem::take(&mut path.args));
        let (arguments, into_params_types) =
            PathOperations::resolve_arguments(&ast_fn.sig.inputs, args);

        path_attribute.update_parameters(arguments);
        path_attribute.update_parameters_parameter_in(into_params_types);
    }

    let path = Path::new(path_attribute, fn_name)
        .path_operation(resolved_operation.map(|operation| operation.path_operation))
        .path(|| resolved_path.map(|path| path.path))
        .doc_comments(CommentAttributes::from_attributes(&ast_fn.attrs).0)
        .deprecated(ast_fn.attrs.iter().find_map(|attr| {
            if !matches!(attr.path.get_ident(), Some(ident) if &*ident.to_string() == "deprecated")
            {
                None
            } else {
                Some(true)
            }
        }));

    quote! {
        #path
        #ast_fn
    }
    .into()
}

#[proc_macro_error]
#[proc_macro_derive(OpenApi, attributes(openapi))]
/// OpenApi derive macro.
///
/// This is `#[derive]` implementation for [`OpenApi`][openapi] trait. The macro accepts one `openapi` argument.
///
/// **Accepted argument attributes:**
///
/// * `paths(...)`  List of method references having attribute [`#[utoipa::path]`][path] macro.
/// * `components(schemas(...), responses(...))` Takes available _`component`_ configurations. Currently only
///    _`schema`_ and _`response`_ components are supported.
///    * `schemas(...)` List of [`ToSchema`][to_schema]s in OpenAPI schema.
///    * `responses(...)` List of types that implement
/// [`ToResponse`][to_response_trait].
/// * `modifiers(...)` List of items implementing [`Modify`][modify] trait for runtime OpenApi modification.
///   See the [trait documentation][modify] for more details.
/// * `security(...)` List of [`SecurityRequirement`][security]s global to all operations.
///   See more details in [`#[utoipa::path(...)]`][path] [attribute macro security options][path_security].
/// * `tags(...)` List of [`Tag`][tags] which must match the tag _**path operation**_. By default
///   the tag is derived from path given to **handlers** list or if undefined then `crate` is used by default.
///   Alternatively the tag name can be given to path operation via [`#[utoipa::path(...)]`][path] macro.
///   Tag can be used to define extra information for the api to produce richer documentation.
/// * `external_docs(...)` Can be used to reference external resource to the OpenAPI doc for extended documentation.
///   External docs can be in [`OpenApi`][openapi_struct] or in [`Tag`][tags] level.
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
/// # Examples
///
/// Define OpenApi schema with some paths and components.
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
pub fn openapi(input: TokenStream) -> TokenStream {
    let DeriveInput { attrs, ident, .. } = syn::parse_macro_input!(input);

    let openapi_attributes = openapi::parse_openapi_attrs(&attrs).expect_or_abort(
        "expected #[openapi(...)] attribute to be present when used with OpenApi derive trait",
    );

    let openapi = OpenApi(openapi_attributes, ident);

    openapi.to_token_stream().into()
}

#[proc_macro_error]
#[proc_macro_derive(IntoParams, attributes(param, into_params))]
/// IntoParams derive macro.
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
/// # IntoParams Container Attributes for `#[into_params(...)]`
///
/// The following attributes are available for use in on the container attribute `#[into_params(...)]` for the struct
/// deriving `IntoParams`:
///
/// * `names(...)` Define comma seprated list of names for unnamed fields of struct used as a path parameter.
/// * `style = ...` Defines how all parameters are serialized by [`ParameterStyle`][style]. Default
///    values are based on _`parameter_in`_ attribute.
/// * `parameter_in = ...` =  Defines where the parameters of this field are used with a value from
///    [`openapi::path::ParameterIn`][in_enum]. There is no default value, if this attribute is not
///    supplied, then the value is determined by the `parameter_in_provider` in
///    [`IntoParams::into_params()`](trait.IntoParams.html#tymethod.into_params).
///
/// # IntoParams Field Attributes for `#[param(...)]`
///
/// The following attributes are available for use in the `#[param(...)]` on struct fields:
///
/// * `style = ...` Defines how the parameter is serialized by [`ParameterStyle`][style]. Default values are based on _`parameter_in`_ attribute.
/// * `explode` Defines whether new _`parameter=value`_ pair is created for each parameter withing _`object`_ or _`array`_.
/// * `allow_reserved` Defines whether reserved characters _`:/?#[]@!$&'()*+,;=`_ is allowed within value.
/// * `example = ...` Can be literal value, method reference or _`json!(...)`_. [^json] Given example
///   will override any example in underlying parameter type.
/// * `value_type = ...` Can be used to override default type derived from type of the field used in OpenAPI spec.
///   This is useful in cases where the default type does not correspond to the actual type e.g. when
///   any third-party types are used which are not [`ToSchema`][to_schema]s nor [`primitive` types][primitive].
///    Value can be any Rust type what normally could be used to serialize to JSON or custom type such as _`Object`_.
///    _`Object`_ will be rendered as generic OpenAPI object.
/// * `inline` If set, the schema for this field's type needs to be a [`ToSchema`][to_schema], and
///   the schema definition will be inlined.
///
/// **Note!** `#[into_params(...)]` is only supported on unnamed struct types to declare names for the arguments.
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
/// # Examples
///
/// Demonstrate [`IntoParams`][into_params] usage with resolving `Path` and `Query` parameters
/// for `get_pet` endpoint. [^actix]
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
/// Demonstrate [`IntoParams`][into_params] usage with the `#[into_params(...)]` container attribute to
/// be used as a path query, and inlining a schema query field:
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
/// Override `String` with `i64` using `value_type` attribute.
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
/// Override `String` with `Object` using `value_type` attribute. _`Object`_ will render as `type: object` in OpenAPI spec.
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
/// You can use a generic type to override the default type of the field.
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
/// You can even overide a [`Vec`] with another one.
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
/// We can override value with another [`ToSchema`][to_schema].
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
/// [to_schema]: trait.ToSchema.html
/// [into_params]: trait.IntoParams.html
/// [path_params]: attr.path.html#params-attributes
/// [struct]: https://doc.rust-lang.org/std/keyword.struct.html
/// [style]: openapi/path/enum.ParameterStyle.html
/// [in_enum]: utoipa/openapi/path/enum.ParameterIn.html
/// [primitive]: https://doc.rust-lang.org/std/primitive/index.html
///
/// [^actix]: Feature **actix_extras** need to be enabled
///
/// [^json]: **json** feature need to be enabled for `json!(...)` type to work.
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

/// Tokenizes slice or Vec of tokenizable items as array either with reference (`&[...]`)
/// or without correctly to OpenAPI JSON.
#[cfg_attr(feature = "debug", derive(Debug))]
enum Array<'a, T>
where
    T: Sized + ToTokens,
{
    Owned(Vec<T>),
    Borrowed(&'a [T]),
}

impl<T> Array<'_, T> where T: ToTokens + Sized {}

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
            Self::Borrowed(slice) => *slice,
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

impl ToTokens for Required {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(match self {
            Self::False => quote! { utoipa::openapi::Required::False },
            Self::True => quote! { utoipa::openapi::Required::True },
        })
    }
}

/// Parses a type information in utoipa macro parameters.
///
/// Supports formats:
///   * `type` type is just a simple type identifier
///   * `[type]` type is an array of types
///   * `Option<type>` type is option of type
///   * `Option<[type]>` type is an option of array of types
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
struct Type<'a> {
    ty: Cow<'a, syn::Path>,
    is_array: bool,
    is_option: bool,
    is_inline: bool,
}

impl<'a> Type<'a> {
    #[cfg(any(
        feature = "actix_extras",
        feature = "rocket_extras",
        feature = "axum_extras"
    ))]
    pub fn new(path: Cow<'a, syn::Path>, is_array: bool, is_option: bool) -> Self {
        Self {
            ty: path,
            is_array,
            is_option,
            is_inline: false,
        }
    }
}

/// A parser for [`Type`] to parse as as `inline(Type)` where `Type` is anything parsed by
/// [`ArrayOrOptionType`].
struct InlineType<'a>(Type<'a>);

impl Parse for InlineType<'_> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const EXPECTED_TYPE_DEFINITION: &str = "unexpected attribute, expected any of inline(Type)";
        let ident: Ident = input.parse().map_err(|error| {
            syn::Error::new(
                error.span(),
                format!("{}: {}", EXPECTED_TYPE_DEFINITION, error),
            )
        })?;

        match &*ident.to_string() {
            "inline" => {
                let content;
                syn::parenthesized!(content in input);

                let mut t: Type = content
                    .parse::<ArrayOrOptionType>()
                    .map_err(|error| {
                        syn::Error::new(
                            error.span(),
                            format!("{}: {}", EXPECTED_TYPE_DEFINITION, error),
                        )
                    })?
                    .0;

                t.is_inline = true;

                Ok(Self(t))
            }
            _ => Err(syn::Error::new(ident.span(), EXPECTED_TYPE_DEFINITION)),
        }
    }
}

/// A parser for [`Type`] to parse as
///  * `Type`
///  * `[Type]`
///  * `Option<Type>`
///  * `Option<[Type]>`
struct ArrayOrOptionType<'a>(Type<'a>);

impl Parse for ArrayOrOptionType<'_> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const EXPECTED_TYPE_MESSAGE: &str =
            "Expected a type/path such as path::to::Foo, or Foo. May also be Option<Foo> or [Foo].";

        fn parse_type<'a>(t: syn::Type) -> syn::Result<Type<'a>> {
            let mut is_option: bool = false;
            let mut is_array: bool = false;
            let path: TypePath = match t {
                syn::Type::Path(mut path) => {
                    let type_segment: &PathSegment =
                        path.path.segments.last().ok_or_else(|| {
                            syn::Error::new(path.path.span(), "No last path segment")
                        })?;
                    let ident = &type_segment.ident;

                    // is option of type or [type]
                    if ident == "Option" {
                        is_option = true;

                        let angle_bracketed: &AngleBracketedGenericArguments = match &type_segment
                            .arguments
                        {
                            PathArguments::AngleBracketed(angle_bracketed) => angle_bracketed,
                            _ => {
                                return Err(syn::Error::new(type_segment.span(), "Option must have its generic type parameter specified. e.g. Option<String>"));
                            }
                        };

                        if angle_bracketed.args.len() != 1 {
                            return Err(syn::Error::new(type_segment.span(), "Option must have only a single generic parameter specified. e.g. Option<String>"));
                        }

                        let argument: &GenericArgument = angle_bracketed.args.first().expect(
                            "Expected there to be 1 angle bracketed argument for Option<...>",
                        );

                        let argument_path: &TypePath = match argument {
                            GenericArgument::Type(syn::Type::Path(path)) => path,
                            GenericArgument::Type(syn::Type::Slice(slice)) => {
                                is_array = true;
                                match &*slice.elem {
                                    syn::Type::Path(path) => path,
                                    unsupported_type => {
                                        return Err(syn::Error::new(
                                            unsupported_type.span(),
                                            format!(
                                                "Unsupported slice type. {}",
                                                EXPECTED_TYPE_MESSAGE
                                            ),
                                        ))
                                    }
                                }
                            }
                            unsupported_type => {
                                return Err(syn::Error::new(
                                    unsupported_type.span(),
                                    format!("Unsupported argument type. {}", EXPECTED_TYPE_MESSAGE),
                                ))
                            }
                        };

                        path = argument_path.clone();
                    }

                    path
                }
                syn::Type::Slice(type_slice) => {
                    is_array = true;
                    match &*type_slice.elem {
                        syn::Type::Path(path) => path.clone(),
                        unsupported_type => {
                            return Err(syn::Error::new(
                                unsupported_type.span(),
                                format!("Unsupported slice type. {}", EXPECTED_TYPE_MESSAGE),
                            ))
                        }
                    }
                }
                syn::Type::Group(group) => {
                    return parse_type(*group.elem);
                }
                unsupported_type => {
                    return Err(syn::Error::new(
                        unsupported_type.span(),
                        format!(
                            "Unsupported type {}. {}",
                            unsupported_type.to_token_stream(),
                            EXPECTED_TYPE_MESSAGE
                        ),
                    ))
                }
            };

            Ok(Type {
                ty: Cow::Owned(path.path),
                is_array,
                is_option,
                is_inline: false,
            })
        }

        let t: syn::Type = input
            .parse::<syn::Type>()
            .map_err(|error| syn::Error::new(error.span(), EXPECTED_TYPE_MESSAGE))?;

        parse_type(t).map(ArrayOrOptionType)
    }
}

impl Parse for Type<'_> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const EXPECTED_TYPE_DEFINITION: &str =
            "unexpected attribute, expected `inline(Type)` or `Type`, where `Type` can be `Type`, `[Type]` or `Option<Type>`";

        // Try parsing as `inline(Type)`
        if input.fork().parse::<InlineType>().is_ok() {
            let t: Self = input.parse::<InlineType>()?.0;
            return Ok(t);
        }

        // Try parsing as `Type`, `[Type]` or `Option<Type>`)
        let t: Type = input
            .parse::<ArrayOrOptionType>()
            .map_err(|error| {
                syn::Error::new(
                    error.span(),
                    format!("{}: {}", EXPECTED_TYPE_DEFINITION, error),
                )
            })?
            .0;

        Ok(t)
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
                syn::Error::new(error.span(), &format!("{}, {}", EXPECTED_ATTRIBUTE, error))
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
#[cfg_attr(feature = "debug", derive(Debug))]
pub(self) enum AnyValue {
    String(TokenStream2),
    Json(TokenStream2),
    #[cfg(not(feature = "json"))]
    Literal(TokenStream2),
}

impl AnyValue {
    fn parse_any(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Lit) {
            if input.peek(LitStr) {
                let lit_str = input.parse::<LitStr>().unwrap().to_token_stream();

                #[cfg(feature = "json")]
                {
                    Ok(AnyValue::Json(lit_str))
                }
                #[cfg(not(feature = "json"))]
                {
                    Ok(AnyValue::String(lit_str))
                }
            } else {
                let lit = input.parse::<Lit>().unwrap().to_token_stream();

                #[cfg(feature = "json")]
                {
                    Ok(AnyValue::Json(lit))
                }
                #[cfg(not(feature = "json"))]
                {
                    Ok(AnyValue::Literal(lit))
                }
            }
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

                #[cfg(feature = "json")]
                {
                    Ok(AnyValue::Json(json))
                }
                #[cfg(not(feature = "json"))]
                {
                    Ok(AnyValue::Literal(json))
                }
            } else {
                let method = input.parse::<ExprPath>().map_err(|error| {
                    syn::Error::new(
                        error.span(),
                        "expected literal value, json!(...) or method reference",
                    )
                })?;

                #[cfg(feature = "json")]
                {
                    Ok(AnyValue::Json(quote! { #method() }))
                }
                #[cfg(not(feature = "json"))]
                {
                    Ok(AnyValue::Literal(quote! { #method() }))
                }
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
}

impl ToTokens for AnyValue {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Json(json) => tokens.extend(quote! {
                serde_json::json!(#json)
            }),
            Self::String(string) => tokens.extend(string.to_owned()),
            #[cfg(not(feature = "json"))]
            Self::Literal(literal) => tokens.extend(quote! { format!("{}", #literal) }),
        }
    }
}

/// Parsing utils
mod parse_utils {
    use proc_macro2::{Group, Ident, TokenStream, TokenTree};
    use proc_macro_error::ResultExt;
    use syn::{
        parenthesized,
        parse::{Parse, ParseStream},
        punctuated::Punctuated,
        token::Comma,
        Error, LitBool, LitStr, Token,
    };

    pub fn skip_past_next_comma(input: ParseStream) -> syn::Result<()> {
        input.step(|cursor| {
            let mut rest = *cursor;
            while let Some((tt, next)) = rest.token_tree() {
                match &tt {
                    TokenTree::Punct(punct) if punct.as_char() == ',' => {
                        return Ok(((), next));
                    }
                    _ => rest = next,
                }
            }
            Ok(((), rest))
        })
    }

    pub fn parse_next<T: Sized>(input: ParseStream, next: impl FnOnce() -> T) -> T {
        input
            .parse::<Token![=]>()
            .expect_or_abort("expected equals token before value assignment");
        next()
    }

    pub fn parse_next_literal_str(input: ParseStream) -> syn::Result<String> {
        Ok(parse_next(input, || input.parse::<LitStr>())?.value())
    }

    pub fn parse_groups<T, R>(input: ParseStream) -> syn::Result<R>
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

    pub fn parse_punctuated_within_parenthesis<T>(
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

    pub fn parse_json_token_stream(input: ParseStream) -> syn::Result<TokenStream> {
        if input.peek(syn::Ident) && input.peek2(Token![!]) {
            input.parse::<Ident>().and_then(|ident| {
                if ident != "json" {
                    return Err(Error::new(ident.span(), "unexpected token, expected: json"));
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
}
