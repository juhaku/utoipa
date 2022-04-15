//! This is **private** utoipa codegen library and is not used alone.
//!
//! The library contains macro implementations for utoipa library. Content
//! of the libarary documentation is available through **utoipa** library itself.
//! Consider browsing via the **utoipa** crate so all links will work correctly.

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

use std::{borrow::Cow, mem};

use component::Component;
use doc_comment::CommentAttributes;

use ext::{PathOperationResolver, PathOperations, PathResolver};
use openapi::OpenApi;
use proc_macro::TokenStream;
use proc_macro_error::{proc_macro_error, OptionExt, ResultExt};
use quote::{quote, ToTokens, TokenStreamExt};

use proc_macro2::{Group, Ident, Punct, TokenStream as TokenStream2};
use syn::{
    bracketed,
    parse::{Parse, ParseBuffer, ParseStream},
    punctuated::Punctuated,
    token::Bracket,
    DeriveInput, ItemFn, Token,
};

mod component;
mod component_type;
mod doc_comment;
mod ext;
mod openapi;
mod path;
mod security_requirement;

use crate::path::{Path, PathAttr};

#[cfg(any(feature = "actix_extras", feature = "rocket_extras"))]
use ext::ArgumentResolver;

#[proc_macro_error]
#[proc_macro_derive(Component, attributes(component))]
/// Component dervice macro
///
/// This is `#[derive]` implementation for [`Component`][c] trait. The macro accepts one `component`
/// attribute optionally which can be used to enhance generated documentation. The attribute can be placed
/// at item level or field level in struct and enums. Currently placing this attribute to unnamed field does
/// not have any effect.
///
/// You can use the Rust's own `#[deprecated]` attribute on any struct, emun or field to mark it as deprecated and it will
/// reflect to the generated OpenAPI spec.
///
/// `#[deprecated]` attribute supports adding addtional details such as a reason and or since version but this is is not supported in
/// OpenAPI. OpenAPI has only a boolean flag to determine deprecation. While it is totally okay to declare deprecated with reason
/// `#[deprecated  = "There is better way to do this"]` the reason would not render in OpenAPI spec.
///
/// # Struct Optional Configuration Options
/// * `example = ...` Can be either `json!(...)` or literal string that can be parsed to json. `json!`
///   should be something that `serde_json::json!` can parse as a `serde_json::Value`. [^json]
/// * `xml(...)` Can be used to define [`Xml`][xml] object properties applicable to Structs.
///  
/// [^json]: **json** feature need to be enabled for `json!(...)` type to work.
///
/// # Enum Optional Configuration Options
/// * `example = ...` Can be method reference or literal value. [^json2]
/// * `default = ...` Can be method reference or literal value. [^json2]
///
/// # Unnamed Field Struct Optional Configuration Options
/// * `example = ...` Can be method reference or literal value. [^json2]
/// * `default = ...` Can be method reference or literal value. [^json2]
/// * `format = ...` [`ComponentFormat`][format] to use for the property. By default the format is derived from
///   the type of the property according OpenApi spec.
/// * `value_type = ...` Can be used to override default type derived from type of the field used in OpenAPI spec.
///   This is useful in cases the where default type does not correspond to the actual type e.g. when
///   any thrid-party types are used which are not components nor primitive types. With **value_type** we can enforce
///   type used to certain type. Value type may only be [`primitive`][primitive] type or [`String`]. Generic types are not allowed.
///
/// # Named Fields Optional Configuration Options
/// * `example = ...` Can be method reference or literal value. [^json2]
/// * `default = ...` Can be method reference or literal value. [^json2]
/// * `format = ...` [`ComponentFormat`][format] to use for the property. By default the format is derived from
///   the type of the property according OpenApi spec.
/// * `write_only` Defines property is only used in **write** operations *POST,PUT,PATCH* but not in *GET*
/// * `read_only` Defines property is only used in **read** operations *GET* but not in *POST,PUT,PATCH*
/// * `xml(...)` Can be used to define [`Xml`][xml] object properties applicable to named fields.
/// * `value_type = ...` Can be used to override default type derived from type of the field used in OpenAPI spec.
///   This is useful in cases the where default type does not correspond to the actual type e.g. when
///   any thrid-party types are used which are not components nor primitive types. With **value_type** we can enforce
///   type used to certain type. Value type may only be [`primitive`][primitive] type or [`String`]. Generic types are not allowed.
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
/// # Examples
///
/// Example struct with struct level example.
/// ```rust
/// # use utoipa::Component;
/// #[derive(Component)]
/// #[component(example = json!({"name": "bob the cat", "id": 0}))]
/// struct Pet {
///     id: u64,
///     name: String,
///     age: Option<i32>,
/// }
/// ```
///
/// The `component` attribute can also be placed at field level as follows.
/// ```rust
/// # use utoipa::Component;
/// #[derive(Component)]
/// struct Pet {
///     #[component(example = 1, default = 0)]
///     id: u64,
///     name: String,
///     age: Option<i32>,
/// }
/// ```
///
/// You can also use method reference for attribute values.
/// ```rust
/// # use utoipa::Component;
/// #[derive(Component)]
/// struct Pet {
///     #[component(example = u64::default, default = u64::default)]
///     id: u64,
///     #[component(default = default_name)]
///     name: String,
///     age: Option<i32>,
/// }
///
/// fn default_name() -> String {
///     "bob".to_string()
/// }
/// ```
///
/// For enums and unnamed field structs you can define `component` at type level.
/// ```rust
/// # use utoipa::Component;
/// #[derive(Component)]
/// #[component(example = "Bus")]
/// enum VechileType {
///     Rocket, Car, Bus, Submarine
/// }
/// ```
///
/// Also you write complex enum combining all above types.
/// ```rust
/// # use utoipa::Component;
/// #[derive(Component)]
/// enum ErrorResponse {
///     InvalidCredentials,
///     #[component(default = String::default, example = "Pet not found")]
///     NotFound(String),
///     System {
///         #[component(example = "Unknown system failure")]
///         details: String,
///     }
/// }
/// ```
///
/// Use `xml` attribute to manipulate xml output.
/// ```rust
/// # use utoipa::Component;
/// #[derive(Component)]
/// #[component(xml(name = "user", prefix = "u", namespace = "https://user.xml.schema.test"))]
/// struct User {
///     #[component(xml(attribute, prefix = "u"))]
///     id: i64,
///     #[component(xml(name = "user_name", prefix = "u"))]
///     username: String,
///     #[component(xml(wrapped(name = "linkList"), name = "link"))]
///     links: Vec<String>,
///     #[component(xml(wrapped, name = "photo_url"))]
///     photos_urls: Vec<String>
/// }
/// ```
///
/// Use of Rust's own `#[deprecated]` attribute will reflect to generated OpenAPI spec.
/// ```rust
/// # use utoipa::Component;
/// #[derive(Component)]
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
/// Enforce type being used in OpenAPI spec to String with `value_type` and set format to octect stream
/// with [`ComponentFormat::Binary`][binary].
/// ```rust
/// # use utoipa::Component;
/// #[derive(Component)]
/// struct Post {
///     id: i32,
///     #[component(value_type = String, format = ComponentFormat::Binary)]
///     value: Vec<u8>,
/// }
/// ```
///
/// Enforce type being used in OpenAPI spec to String with `value_type` option.
/// ```rust
/// # use utoipa::Component;
/// #[derive(Component)]
/// #[component(value_type = String)]
/// struct Value(i64);
/// ```
///
/// [c]: trait.Component.html
/// [format]: openapi/schema/enum.ComponentFormat.html
/// [binary]: openapi/schema/enum.ComponentFormat.html#variant.Binary
/// [xml]: openapi/xml/struct.Xml.html
/// [primitive]: https://doc.rust-lang.org/std/primitive/index.html
pub fn derive_component(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs,
        ident,
        data,
        generics,
        ..
    } = syn::parse_macro_input!(input);

    let component = Component::new(&data, &attrs, &ident, &generics);

    component.to_token_stream().into()
}

#[proc_macro_error]
#[proc_macro_attribute]
/// Path attribute macro
///
/// This is a `#[derive]` implementation for [`Path`][path] trait. Macro accepts set of attributes that can
/// be used to configure and override default values what are resolved automatically.
///
/// You can use the Rust's own `#[deprecated]` attribute on functions to mark it as deprecated and it will
/// reflect to the generated OpenAPI spec. Only **parameters** has a special **deprecated** attribute to define them as deprecated.
///
/// `#[deprecated]` attribute supports adding addtional details such as a reason and or since version but this is is not supported in
/// OpenAPI. OpenAPI has only a boolean flag to determine deprecation. While it is totally okay to declare deprecated with reason
/// `#[deprecated  = "There is better way to do this"]` the reason would not render in OpenAPI spec.
///
/// # Path Attributes
///
/// * `operation` _**Must be first parameter!**_ Accepted values are known http operations suchs as
///   _`get, post, put, delete, head, options, connect, patch, trace`_.
/// * `path = "..."` Must be OpenAPI format compatible str with arguments withing curly braces. E.g _`{id}`_
/// * `operation_id = "..."` Unique operation id for the enpoint. By default this is mapped to function name.
/// * `context_path = "..."` Can add optional scope for **path**. The **context_path** will be prepended to begining of **path**.
///   This is particularly useful when **path** does not contain the full path to the endpoint. For example if web framework
///   allows operation to be defined under some context path or scope which does not reflect to the resolved path then this
///   **context_path** can become handy to alter the path.
/// * `tag = "..."` Can be used to group operations. Operations with same tag are groupped together. By default
///   this is derived from the handler that is given to [`OpenApi`][openapi]. If derive results empty str
///   then default value _`crate`_ is used instead.
/// * `request_body = ... | request_body(...)` Defining request body indicates that the request is expecting request body within
///   the performed request.
/// * `responses(...)` Slice of responses the endpoint is going to possibly return to the caller.
/// * `params(...)` Slice of params that the endpoint accepts.
/// * `security(...)` List of [`SecurityRequirement`][security]s local to the path operation.
///
/// > **Note!** when **actix_extras** feature is enabled the **operation**, **path** and **params** declaration
/// > may be omitted since they are resolved from **actix-web** attributes namely **path** and function arguments.
/// > To define description or other parameter info then **params** still need to be defined manually. See the example
/// > in [examples section](#examples).
///
/// # Request Body Attributes
///
/// * `content = ...` Can be used to define the content object. Should be an identifier, slice or option
///   E.g. _`Pet`_ or _`[Pet]`_ or _`Option<Pet>`_.
/// * `description = "..."` Define the description for the request body object as str.
/// * `content_type = "..."` Can be used to override the default behaviour of auto resolving the content type
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
/// * `status = ...` Is valid http status code. E.g. _`200`_
/// * `description = "..."` Define description for the response as str.
/// * `body = ...` Optional response body object type. When left empty response does not expect to send any
///   response body. Should be an identifier or slice. E.g _`Pet`_ or _`[Pet]`_
/// * `content_type = "..." | content_type = [...]` Can be used to override the default behaviour of auto resolving the content type
///   from the `body` attribute. If defined the value should be valid content type such as
///   _`application/json`_. By default the content type is _`text/plain`_ for
///   [primitive Rust types][primitive] and _`application/json`_ for struct and complex enum types.
///   Content type can also be slice of **content_type** values if the endpoint support returning multiple
///  response content types. E.g _`["application/json", "text/xml"]`_ would indicate that endpoint can return both
///  _`json`_ and _`xml`_ formats.
/// * `headers(...)` Slice of response headers that are returned back to a caller.
/// * `example = ...` Can be either `json!(...)` or literal str that can be parsed to json. `json!`
///   should be something that `serde_json::json!` can parse as a `serde_json::Value`. [^json]
///
/// **Minimal response format:**
/// ```text
/// (status = 200, description = "success response")
/// ```
///
/// **Response with all possible values:**
/// ```text
/// (status = 200, description = "Success response", body = Pet, content_type = "application/json",
///     headers(...),
///     example = json!({"id": 1, "name": "bob the cat"})
/// )
/// ```
///
/// **Response with multiple response content types:**
/// ```text
/// (status = 200, description = "Success response", body = Pet, content_type = ["application/json", "text/xml"])
/// ```
///
/// # Response Header Attributes
///
/// * `name` Name of the header. E.g. _`x-csrf-token`_
/// * `type` Addtional type of the header value. Type is defined after `name` with equals sign before the type.
///   Type should be identifer or slice of identifiers. E.g. _`String`_ or _`[String]`_
/// * `description = "..."` Can be used to define optional description for the response header as str.
///
/// **Header supported formats:**
///
/// ```text
/// ("x-csfr-token"),
/// ("x-csrf-token" = String, description = "New csfr token"),
/// ```
///
/// # Params Attributes
///
/// * `name` _**Must be the first argument**_. Define the name for parameter.
/// * `parameter_type` Define possible type for the parameter. Type should be an identifer, slice or option.
///   E.g. _`String`_ or _`[String]`_ or _`Option<String>`_. Parameter type is placed after `name` with
///   equals sign E.g. _`"id" = String`_
/// * `in` _**Must be placed after name or parameter_type**_. Define the place of the parameter.
///   E.g. _`path, query, header, cookie`_
/// * `deprecated` Define whether the parameter is deprecated or not.
/// * `description = "..."` Define possible description for the parameter as str.
///
/// **Params supports following representation formats:**
///
/// ```text
/// ("id" = String, path, deprecated, description = "Pet database id"),
/// ("id", path, deprecated, description = "Pet database id"),
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
///      ("x-csrf-token" = String, header, deprecated, description = "Current csrf token of user"),
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
///      ("x-csrf-token", header, description = "Current csrf token of user"),
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
/// With **actix_extras** feature enabled the you can leave out definitions for **path**, **operation** and **parmater types** [^actix_extras].
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
/// With **actix_extras** you may also not to list any _**parmas**_ if you do not want to specify any description for them. Params are resolved from
/// path and the argument types of handler.
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
/// Use of Rust's own `#[deprecated]` attribute will refect to the generated OpenAPI spec and mark this operation as deprecated.
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
/// [path]: trait.Path.html
/// [openapi]: derive.OpenApi.html
/// [security]: openapi/security/struct.SecurityRequirement.html
/// [security_schema]: openapi/security/struct.SecuritySchema.html
/// [primitive]: https://doc.rust-lang.org/std/primitive/index.html
///
/// [^json]: **json** feature need to be enabled for `json!(...)` type to work.
///
/// [^actix_extras]: **actix_extras** feature need to be enabled and **actix-web** framework must be declared in your `Cargo.toml`.
pub fn path(attr: TokenStream, item: TokenStream) -> TokenStream {
    let path_attribute = syn::parse_macro_input!(attr as PathAttr);

    #[cfg(any(feature = "actix_extras", feature = "rocket_extras"))]
    let mut path_attribute = path_attribute;

    let ast_fn = syn::parse::<ItemFn>(item).unwrap_or_abort();
    let fn_name = &*ast_fn.sig.ident.to_string();

    let mut resolved_operation = PathOperations::resolve_operation(&ast_fn);

    let mut resolved_path = PathOperations::resolve_path(
        &resolved_operation
            .as_mut()
            .map(|operation| mem::take(&mut operation.path))
            .or_else(|| path_attribute.path.as_ref().map(String::to_string)), // cannot use mem take because we need this later
    );

    #[cfg(any(feature = "actix_extras", feature = "rocket_extras"))]
    {
        let args = resolved_path.as_mut().map(|path| mem::take(&mut path.args));
        let arguments = PathOperations::resolve_path_arguments(&ast_fn.sig.inputs, args);

        path_attribute.update_parameters(arguments);
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
/// OpenApi derive macro
///
/// This is `#[derive]` implementation for [`OpenApi`][openapi] trait. The macro accepts one `openapi` argument.
///
/// **Accepted argument attributes:**
///
/// * `handlers(...)`  List of method references having attribute [`#[utoipa::path]`][path] macro.
/// * `components(...)`  List of [`Component`][component]s in OpenAPI schema.
/// * `modifiers(...)` List of items implemeting [`Modify`][modify] trait for runtime OpenApi modification.
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
/// # use utoipa::{OpenApi, Component};
/// #
/// #[derive(Component)]
/// struct Pet {
///     name: String,
///     age: i32,
/// }
///
/// #[derive(Component)]
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
///     handlers(get_pet, get_status),
///     components(Pet, Status),
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
/// [component]: derive.Component.html
/// [path]: attr.path.html
/// [modify]: trait.Modify.html
/// [info]: openapi/info/struct.Info.html
/// [security]: openapi/security/struct.SecurityRequirement.html
/// [path_security]: attr.path.html#security-requirement-attributes
/// [tags]: openapi/tag/struct.Tag.html
pub fn openapi(input: TokenStream) -> TokenStream {
    let DeriveInput { attrs, ident, .. } = syn::parse_macro_input!(input);

    let openapi_attributes = openapi::parse_openapi_attrs(&attrs).expect_or_abort(
        "expected #[openapi(...)] attribute to be present when used with OpenApi derive trait",
    );

    let openapi = OpenApi(openapi_attributes, ident);
    quote! {
        #openapi
    }
    .into()
}

/// Tokenizes slice or Vec of tokenizable items as array either with reference (`&[...]`)
/// or without correctly to OpenAPI JSON.
#[cfg_attr(feature = "debug", derive(Debug))]
enum Array<T>
where
    T: Sized + ToTokens,
{
    Owned(Vec<T>),
}

impl<T> Array<T> where T: ToTokens + Sized {}

impl<V> FromIterator<V> for Array<V>
where
    V: Sized + ToTokens,
{
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        Self::Owned(iter.into_iter().collect())
    }
}

impl<T> ToTokens for Array<T>
where
    T: Sized + ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Array::Owned(values) = self;

        tokens.append(Group::new(
            proc_macro2::Delimiter::Bracket,
            values
                .iter()
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
struct Type<'a> {
    ty: Cow<'a, Ident>,
    is_array: bool,
    is_option: bool,
}

impl<'a> Type<'a> {
    #[cfg(any(feature = "actix_extras", feature = "rocket_extras"))]
    pub fn new(ident: Cow<'a, Ident>, is_array: bool, is_option: bool) -> Self {
        Self {
            ty: ident,
            is_array,
            is_option,
        }
    }
}

impl Parse for Type<'_> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut is_array = false;
        let mut is_option = false;

        let mut parse_array = |input: &ParseBuffer| {
            is_array = true;
            let group;
            bracketed!(group in input);
            group.parse::<Ident>()
        };

        let ty = if input.peek(syn::Ident) {
            let mut ident: Ident = input.parse()?;

            // is option of type or [type]
            if (ident == "Option" && input.peek(Token![<]))
                && (input.peek2(syn::Ident) || input.peek2(Bracket))
            {
                is_option = true;

                input.parse::<Token![<]>()?;

                if input.peek(syn::Ident) {
                    ident = input.parse::<Ident>()?;
                } else {
                    ident = parse_array(input)?;
                }
                input.parse::<Token![>]>()?;
            }
            Ok(ident)
        } else {
            parse_array(input)
        }?;

        Ok(Type {
            ty: Cow::Owned(ty),
            is_array,
            is_option,
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

#[cfg_attr(feature = "debug", derive(Debug))]
enum Example {
    String(TokenStream2),
    Json(TokenStream2),
}

impl ToTokens for Example {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Json(json) => tokens.extend(quote! {
                serde_json::json!(#json)
            }),
            Self::String(string) => tokens.extend(string.to_owned()),
        }
    }
}

/// Parsing utils
mod parse_utils {
    use proc_macro2::{Group, Ident, TokenStream};
    use proc_macro_error::{abort, ResultExt};
    use quote::ToTokens;
    use syn::{
        parenthesized,
        parse::{Parse, ParseStream},
        punctuated::Punctuated,
        token::Comma,
        Error, LitBool, LitStr, Token,
    };

    use crate::Example;

    pub fn parse_next<T: Sized>(input: ParseStream, next: impl FnOnce() -> T) -> T {
        input
            .parse::<Token![=]>()
            .expect_or_abort("expected equals token before value assigment");
        next()
    }

    pub fn parse_next_literal_str(input: ParseStream) -> Result<String, Error> {
        Ok(parse_next(input, || input.parse::<LitStr>())?.value())
    }

    pub fn parse_groups<T, R>(input: ParseStream) -> Result<R, Error>
    where
        T: Sized,
        T: Parse,
        R: FromIterator<T>,
    {
        Punctuated::<Group, Comma>::parse_terminated(input).and_then(|groups| {
            groups
                .into_iter()
                .map(|group| syn::parse2::<T>(group.stream()))
                .collect::<Result<R, Error>>()
        })
    }

    pub fn parse_punctuated_within_parenthesis<T>(
        input: ParseStream,
    ) -> Result<Punctuated<T, Comma>, Error>
    where
        T: Parse,
    {
        let content;
        parenthesized!(content in input);
        Punctuated::<T, Comma>::parse_terminated(&content)
    }

    pub fn parse_bool_or_true(input: ParseStream) -> Result<bool, syn::Error> {
        if input.peek(Token![=]) && input.peek2(LitBool) {
            input.parse::<Token![=]>()?;

            Ok(input.parse::<LitBool>()?.value())
        } else {
            Ok(true)
        }
    }

    pub fn parse_json_token_stream(input: ParseStream) -> Result<TokenStream, Error> {
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

    fn parse_next_lit_str_or_json(input: ParseStream, abort_op: impl FnOnce(&Error)) -> Example {
        if input.peek2(LitStr) {
            Example::String(parse_next(input, || {
                input.parse::<LitStr>().unwrap().to_token_stream()
            }))
        } else {
            Example::Json(parse_next(input, || {
                parse_json_token_stream(input).unwrap_or_else(|error| {
                    abort_op(&error);
                    // hacky way to tell rust that we are having a "never" type here
                    unreachable!("oops! unreachable code we should have aborted here");
                })
            }))
        }
    }

    pub(crate) fn parse_next_lit_str_or_json_example(input: ParseStream, ident: &Ident) -> Example {
        parse_next_lit_str_or_json(input, |error| {
            abort! {ident, "unparseable example, expected json!(), {}", error;
            help = r#"Try defining example = json!({{"key": "value"}})"#;
            }
        })
    }
}
