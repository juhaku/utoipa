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
    DeriveInput, Error,
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
/// Component dervice
pub fn derive_component(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs, ident, data, ..
    } = syn::parse_macro_input!(input);

    let component = Component::new(data, &attrs, &ident);

    quote! {
        #component
    }
    .into()
}

#[proc_macro_error]
#[proc_macro_attribute]
/// Path attribute macro
pub fn path(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut path_attribute = syn::parse_macro_input!(attr as PathAttr);
    let ast_fn = syn::parse::<syn::ItemFn>(item).unwrap_or_abort();
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
        use utoipa::openapi::schema::ToArray;
        #path
        #ast_fn
    }
    .into()
}

#[proc_macro_error]
#[proc_macro_derive(OpenApi, attributes(openapi))]
/// Derive OpenApi macro
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

/// Media type is wrapper around type and information is type an array
// #[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
struct MediaType {
    ty: Ident,
    is_array: bool,
}

impl MediaType {
    pub fn new(ident: Ident) -> Self {
        Self {
            ty: ident,
            is_array: false,
        }
    }
}

impl Parse for MediaType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut is_array = false;

        let parse_ident = |group: &ParseBuffer, error_msg: &str| {
            if group.peek(syn::Ident) {
                group.parse::<Ident>()
            } else {
                Err(Error::new(input.span(), error_msg))
            }
        };

        let ty = if input.peek(syn::Ident) {
            parse_ident(input, "unparseable MediaType, expected Ident")
        } else {
            is_array = true;

            let group;
            bracketed!(group in input);

            parse_ident(
                &group,
                "unparseable MediaType, expected Ident within Bracket Group",
            )
        }?;

        Ok(MediaType { ty, is_array })
    }
}

/// Parsing utils
mod parse_utils {
    use proc_macro_error::ResultExt;
    use syn::{parse::ParseStream, LitBool, LitStr, Token};

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
}
