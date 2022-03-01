//! This is **private** utoipa codegen library and is not used alone
//!
//! The library contains macro implementations for utoipa library. Content
//! of the libarary documentation is available through **utoipa** library itself.
//! Consider browsing via the **utoipa** crate so all links will work correctly.

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

use component::Component;
use doc_comment::CommentAttributes;

use ext::{ArgumentResolver, PathOperationResolver, PathOperations, PathResolver};
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

use crate::path::{Path, PathAttr, PathOperation};

#[proc_macro_error]
#[proc_macro_derive(Component, attributes(component))]
/// Component dervice macro
///
/// This is `#[derive]` implementation for [`Component`][c] trait. The macro accepts one `component`
/// attribute optionally which can be used to enhance generated documentation. The attribute can be placed
/// at item level or field level in struct and enums. Currently placing this attribute to unnamed field does
/// not have any effect.
///
/// # Struct Optional Configuration Options
/// * **example** Can be either `json!(...)` or literal string that can be parsed to json. `json!`
///   should be something that `serde_json::json!` can parse as a `serde_json::Value`. [^json]
///  
/// [^json]: **json** feature need to be enabled for `json!(...)` type to work.
///
/// # Enum & Unnamed Field Struct Optional Configuration Options
/// * **example** Can be method reference or literal value. [^json2]
/// * **default** Can be method reference or literal value. [^json2]
///
/// # Named Fields Optional Configuration Options
/// * **example** Can be method reference or literal value. [^json2]
/// * **default** Can be method reference or literal value. [^json2]
/// * **format**  [`ComponentFormat`][format] to use for the property. By default the format is derived from
///   the type of the property according OpenApi spec.
/// * **write_only** Defines property is only used in **write** operations *POST,PUT,PATCH* but not in *GET*
/// * **read_only** Defines property is only used in **read** operations *GET* but not in *POST,PUT,PATCH*
///
/// [^json2]: Values are converted to string if **json** feature is not enabled.
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
///     #[component(example = 1, default = 0))]
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
///     #[component(example = u64::default, default = u64::default))]
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
/// [c]: trait.Component.html
/// [format]: schema/enum.ComponentFormat.html
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
/// # Path Attributes
///
/// * **operation** _**Must be first parameter!**_ Accepted values are known http operations suchs as
///   _`get, post, put, delete, head, options, connect, patch, trace`_.
/// * **path** Must be OpenAPI format compatible str with arguments withing curly braces. E.g _`{id}`_
/// * **operation_id** Unique operation id for the enpoint. By default this is mapped to function name.
/// * **tag** Can be used to group operations. Operations with same tag are groupped together. By default
///   this is derived from the handler that is given to [`OpenApi`][openapi]. If derive results empty str
///   then default value _`crate`_ is used instead.
/// * **request_body** Defining request body indicates that the request is expecting request body within
///   the performed request.
/// * **responses** Slice of responses the endpoint is going to possibly return to the caller.
/// * **params** Slice of params that the endpoint accepts.
/// * **security** List of [`SecurityRequirement`][security]s local to the path operation.
///
/// # Request Body Attributes
///
/// * **content** Can be used to define the content object. Should be an identifier, slice or option
///   E.g. _`Pet`_ or _`[Pet]`_ or _`Option<Pet>`_.
/// * **description** Define the description for the request body object as str.
/// * **content_type** Can be used to override the default behaviour of auto resolving the content type
///   from the `content` attribute. If defined the value should be valid content type such as
///   _`application/json`_. By default the content type is _`text/plain`_ for
///   [primitive Rust types][primitive] and _`application/json`_ for struct and complex enum types.
///
/// **Request body supports following formats:**
///
/// ```text
/// request_body = (content = String, description = "Xml as string request", content_type = "text/xml"),
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
/// * **status** Is valid http status code. E.g. _`200`_
/// * **description** Define description for the response as str.
/// * **body** Optional response body object type. When left empty response does not expect to send any
///   response body. Should be an identifier or slice. E.g _`Pet`_ or _`[Pet]`_
/// * **content_type** Can be used to override the default behaviour of auto resolving the content type
///   from the `body` attribute. If defined the value should be valid content type such as
///   _`application/json`_. By default the content type is _`text/plain`_ for
///   [primitive Rust types][primitive] and _`application/json`_ for struct and complex enum types.
/// * **headers** Slice of response headers that are returned back to a caller.
/// * **example** Can be either `json!(...)` or literal str that can be parsed to json. `json!`
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
///     headers = [...],
///     example = json!({"id": 1, "name": "bob the cat"})
/// )
/// ```
///
/// # Response Header Attributes
///
/// * **name** Name of the header. E.g. _`x-csrf-token`_
/// * **type** Addtional type of the header value. Type is defined after `name` with equals sign before the type.
///   Type should be identifer or slice of identifiers. E.g. _`String`_ or _`[String]`_
/// * **description** Can be used to define optional description for the response header as str.
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
/// * **name** _**Must be the first argument**_. Define the name for parameter.
/// * **parameter_type** Define possible type for the parameter. Type should be an identifer, slice or option.
///   E.g. _`String`_ or _`[String]`_ or _`Option<String>`_. Parameter type is placed after `name` with
///   equals sign E.g. _`"id" = String`_
/// * **in** _**Must be placed after name or parameter_type**_. Define the place of the parameter.
///   E.g. _`path, query, header, cookie`_
/// * **deprecated** Define whether the parameter is deprecated or not.
/// * **description** Define possible description for the parameter as str.
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
/// * **name** Define the name for security requirement. This must match to name of existing
///   [`SecuritySchema`][security_schema].
/// * **scopes** Define the list of scopes needed. These must be scopes defined already in
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
///    tag = "pet_handlers"
///    request_body = (content = Pet, description = "Pet to store the database", content_type = "application/json")
///    responses = [
///         (status = 200, description = "Pet stored successfully", body = Pet, content_type = "application/json",
///             headers = [
///                 ("x-cache-len" = String, description = "Cache length")
///             ],
///             example = json!({"id": 1, "name": "bob the cat"})
///         ),
///    ],
///    params = [
///      ("x-csrf-token" = String, header, deprecated, description = "Current csrf token of user"),
///    ],
///    security = [
///        (),
///        ("my_auth" = ["read:items", "edit:items"]),
///        ("token_jwt" = [])
///    ]
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
///    request_body = Pet
///    responses = [
///         (status = 200, description = "Pet stored successfully", body = Pet,
///             headers = [
///                 ("x-cache-len", description = "Cache length")
///             ]
///         ),
///    ],
///    params = [
///      ("x-csrf-token", header, description = "Current csrf token of user"),
///    ]
/// )]
/// fn post_pet(pet: Pet) -> Pet {
///     Pet {
///         id: 4,
///         name: "bob the cat".to_string(),
///     }
/// }
/// ```
///
/// [path]: trait.Path.html
/// [openapi]: derive.OpenApi.html
/// [security]: openapi/security/struct.SecurityRequirement.html
/// [security_schema]: openapi/security/struct.SecuritySchema.html
/// [primitive]: https://doc.rust-lang.org/std/primitive/index.html
/// [^json]: **json** feature need to be enabled for `json!(...)` type to work.
pub fn path(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut path_attribute = syn::parse_macro_input!(attr as PathAttr);
    let ast_fn = syn::parse::<ItemFn>(item).unwrap_or_abort();
    let fn_name = &*ast_fn.sig.ident.to_string();

    let arguments = PathOperations::resolve_path_arguments(&ast_fn.sig.inputs);

    #[cfg(feature = "actix_extras")]
    path_attribute.update_parameters(arguments);

    let operation_attribute = &PathOperations::resolve_attribute(&ast_fn);
    let path_provider = || PathOperations::resolve_path(operation_attribute);

    let path = Path::new(path_attribute, fn_name)
        .with_path_operation(operation_attribute.map(|attribute| {
            let ident = attribute.path.get_ident().unwrap();
            PathOperation::from_ident(ident)
        }))
        .with_path(path_provider)
        .with_doc_comments(CommentAttributes::from_attributes(&ast_fn.attrs).0)
        .with_deprecated(ast_fn.attrs.iter().find_map(|attr| {
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
/// * **handlers**  List of method references having attribute [`#[utoipa::path]`][path] macro.
/// * **components**  List of [`Component`][component]s in OpenAPI schema.
/// * **modifiers** List of items implemeting [`Modify`][modify] trait for runtime OpenApi modification.
///   See the [trait documentation][modify] for more details.
/// * **security** List of [`SecurityRequirement`][security]s global to all operations.
///   See more details in [`#[utoipa::path(...)]`][path] [attribute macro security options][path_security].
///
/// OpenApi derive macro will also derive [`Info`][info] for OpenApi specification using Cargo
/// environment variables.
///
/// * env `CARGO_PKG_NAME` map to info `title`
/// * env `CARGO_PKG_VERSION` map to info `version`
/// * env `CARGO_PKG_DESCRIPTION` map info `description`
/// * env `CARGO_PKG_AUTHORS` map to contact `name` and `email` **only first author will be used**
/// * env `CARGO_PKG_LICENSE` map to info `licence`
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
/// fn get_pet() - Pet {
///     Pet {
///         name: "bob".to_string(),
///         age: 8,
///     }
/// }
///
/// #[utoipa::path(get, path = "/status")]
/// fn get_status() - Status {
///     Status::Active
/// }
///
/// #[derive(OpenApi)]
/// #[openapi(
///     handlers = [get_pet, get_status],
///     components = [Pet, Status],
///     security = [
///         (),
///         ("my_auth" = ["read:items", "edit:items"]),
///         ("token_jwt" = [])
///     ]
/// )]
/// struct ApiDoc;
/// ```
///
/// [openapi]: trait.OpenApi.html
/// [component]: derive.Component.html
/// [path]: attr.path.html
/// [modify]: trait.Modify.html
/// [info]: openapi/info/struct.Info.html
/// [security]: openapi/security/struct.SecurityRequirement.html
/// [path_security]: attr.path.html#security-requirement-attributes
pub fn openapi(input: TokenStream) -> TokenStream {
    let DeriveInput { attrs, ident, .. } = syn::parse_macro_input!(input);

    let openapi_attributes = openapi::parse_openapi_attributes_from_attributes(&attrs)
        .expect_or_abort(
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
enum Array<T>
where
    T: Sized + ToTokens,
{
    Owned(Vec<T>),
    Referenced(Vec<T>),
}

impl<T> Array<T>
where
    T: ToTokens + Sized,
{
    fn into_referenced_array(self) -> Self {
        match self {
            Array::Owned(values) => Self::Referenced(values),
            Array::Referenced(_) => self,
        }
    }
}

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
        let (add_and, values) = match self {
            Array::Owned(values) => (false, values),
            Array::Referenced(values) => (true, values),
        };

        if add_and {
            tokens.append(Punct::new('&', proc_macro2::Spacing::Joint));
        }

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

/// Parses a type information in uotapi macro parameters.
///
/// Supports formats:
///   * `type` type is just a simple type identifier
///   * `[type]` type is an array of types
///   * `Option<type>` type is option of type
///   * `Option<[type]>` type is an option of array of types
#[cfg_attr(feature = "debug", derive(Debug))]
struct Type {
    ty: Ident,
    is_array: bool,
    is_option: bool,
}

impl Type {
    pub fn new(ident: Ident) -> Self {
        Self {
            ty: ident,
            is_array: false,
            is_option: false,
        }
    }
}

impl Parse for Type {
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
            let mut ident: Ident = input.parse().unwrap();

            // is option of type or [type]
            if (ident == "Option" && input.peek(Token![<]))
                && (input.peek2(syn::Ident) || input.peek2(Bracket))
            {
                is_option = true;

                input.parse::<Token![<]>().unwrap();

                if input.peek(syn::Ident) {
                    ident = input.parse::<Ident>().unwrap();
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
            ty,
            is_array,
            is_option,
        })
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
    use syn::{parse::ParseStream, Error, LitBool, LitStr, Token};

    use crate::Example;

    pub fn parse_next<T: Sized>(input: ParseStream, next: impl FnOnce() -> T) -> T {
        input
            .parse::<Token![=]>()
            .expect_or_abort("expected equals token (=) before value assigment");
        next()
    }

    pub fn parse_next_lit_str(input: ParseStream, error_message: &str) -> String {
        parse_next(input, || {
            input
                .parse::<LitStr>()
                .expect_or_abort(error_message)
                .value()
        })
    }

    pub fn parse_bool_or_true(input: ParseStream) -> bool {
        if input.peek(Token![=]) && input.peek2(LitBool) {
            input.parse::<Token![=]>().unwrap();

            input.parse::<LitBool>().unwrap().value()
        } else {
            true
        }
    }

    pub fn parse_json_token_stream(input: ParseStream) -> Result<TokenStream, Error> {
        if input.peek(syn::Ident) && input.peek2(Token![!]) {
            input.parse::<Ident>().unwrap();
            input.parse::<Token![!]>().unwrap();

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
