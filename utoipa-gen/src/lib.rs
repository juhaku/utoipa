//! This is private utoipa codegen library and is not used alone

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
/// This is `#[derive]` implementation for [OpenApi][openapi] trait. The macro accepts one `openapi` argument.
///
/// **Accepted `openapi` argument attributes**
///
/// * **handlers**  List of method references having attribute [`#[utoipa::path]`][path] macro.
/// * **components**  List of [Component][component]s in OpenAPI schema.
///
/// OpenApi derive macro will also derive [Info][info] for OpenApi specification using Cargo
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
/// #[openapi(handlers = [get_pet, get_status], components = [Pet, Status])]
/// struct ApiDoc;
/// ```
///
/// [openapi]: trait.OpenApi.html
/// [component]: derive.Component.html
/// [path]: attr.path.html
/// [info]: openapi/info/struct.Info.html
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

/// Tokenizes slice or Vec of tokenizable items as slice reference (`&[...]`) correctly to OpenAPI JSON.
struct ValueArray<V>(Vec<V>)
where
    V: Sized + ToTokens;

impl<V> FromIterator<V> for ValueArray<V>
where
    V: Sized + ToTokens,
{
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        Self {
            0: iter.into_iter().collect::<Vec<_>>(),
        }
    }
}

impl<T> ToTokens for ValueArray<T>
where
    T: Sized + ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.append(Punct::new('&', proc_macro2::Spacing::Joint));

        tokens.append(Group::new(
            proc_macro2::Delimiter::Bracket,
            self.0
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
