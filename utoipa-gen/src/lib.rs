//! This is private utoipa codegen library and is not used alone

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

#[cfg(feature = "actix_extras")]
use ext::actix::update_parameters_from_arguments;

use ext::{ArgumentResolver, PathOperationResolver, PathOperations, PathResolver};
use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens, TokenStreamExt};

use proc_macro2::{Group, Ident, Punct, TokenStream as TokenStream2};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Bracket,
    Attribute, DeriveInput, ExprPath, LitStr, Token,
};

mod attribute;
mod component;
mod component_type;
mod ext;
mod info;
mod path;
mod property;
mod request_body;
mod response;

use proc_macro_error::*;

use crate::{
    attribute::CommentAttributes,
    component::impl_component,
    path::{Path, PathAttr, PathOperation},
};

const PATH_STRUCT_PREFIX: &str = "__path_";

#[proc_macro_error]
#[proc_macro_derive(Component, attributes(component))]
/// Component dervice
pub fn derive_component(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs, ident, data, ..
    } = syn::parse_macro_input!(input);

    let component_quote = impl_component(data, attrs);

    let component = quote! {
        impl utoipa::Component for #ident {
            fn component() -> utoipa::openapi::schema::Component {
                #component_quote
            }
        }
    };

    component.into()
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
    update_parameters_from_arguments(arguments, &mut path_attribute.params);

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
pub fn openapi(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs,
        // data,
        // generics,
        ident,
        ..
    } = syn::parse_macro_input!(input);

    let openapi_args =
        parse_openapi_attributes(&attrs).expect_or_abort("Expected #openapi[...] attribute");

    // let files = openapi_args
    //     .iter()
    //     .filter(|args| matches!(args, OpenApiArgs::HandlerFiles(_)))
    //     .flat_map(|args| match args {
    //         OpenApiArgs::HandlerFiles(files) => files,
    //         _ => unreachable!(),
    //     });

    let components = openapi_args
        .iter()
        .filter(|args| matches!(args, OpenApiArgs::Components(_)))
        .flat_map(|args| match args {
            OpenApiArgs::Components(components) => components,
            _ => unreachable!(),
            // TODO enabed if argument resolving is enabled
        })
        .collect::<Vec<_>>();

    let info = info::impl_info();
    // let paths = paths::impl_paths(&files.map(String::to_owned).collect::<Vec<_>>());

    let span = ident.span();
    let mut quote = quote! {};
    let mut schema = quote! {
        utoipa::openapi::Schema::new()
    };

    let handlers = openapi_args
        .iter()
        .filter_map(|args| match args {
            OpenApiArgs::Handlers(handlers) => Some(handlers.clone()),
            _ => None,
        })
        .flatten()
        .collect::<Vec<_>>();

    // println!("handlers: {:#?}", &handlers);

    let path_items = impl_paths(handlers.into_iter(), &mut quote);

    for component in components {
        let component_name = &*component.to_string();
        let assert_ident = format_ident!("_AssertComponent{}", component_name);
        quote.extend(quote_spanned! {span=>
            struct #assert_ident where #component: utoipa::Component;
        });

        schema.extend(quote! {
            .with_component(#component_name, #component::component())
        });
    }

    quote.extend(quote! {
        use utoipa::openapi::schema::ToArray;
        impl utoipa::OpenApi for #ident {
            fn openapi() -> utoipa::openapi::OpenApi {
                utoipa::openapi::OpenApi::new(#info, #path_items)
                    .with_components(#schema)
            }
        }
    });

    quote.into()
}

fn parse_openapi_attributes(attributes: &[Attribute]) -> Option<Vec<OpenApiArgs>> {
    if attributes.len() > 1 {
        panic!(
            "Expected at most 1 attribute, but found: {}",
            &attributes.len()
        );
    }

    attributes
        .iter()
        .next()
        .map(|attribute| {
            if !attribute.path.is_ident("openapi") {
                abort_call_site!("Expected #[openapi(...)]");
            } else {
                attribute
            }
        })
        .map(|att| {
            att.parse_args_with(Punctuated::<OpenApiArgs, Token![,]>::parse_terminated)
                .unwrap_or_abort()
                .into_iter()
                .collect()
        })
}

#[cfg_attr(feature = "debug", derive(Debug))]
enum OpenApiArgs {
    HandlerFiles(Vec<String>),
    Components(Vec<Ident>),
    Handlers(Vec<syn::ExprPath>),
}

impl Parse for OpenApiArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name_ident = input.parse::<Ident>()?;
        let name_str = &*name_ident.to_string();

        match name_str {
            "handler_files" => {
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                }

                if input.peek(syn::token::Bracket) {
                    let content;
                    bracketed!(content in input);
                    let tokens = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;

                    Ok(Self::HandlerFiles(
                        tokens.iter().map(LitStr::value).collect::<Vec<_>>(),
                    ))
                } else {
                    Err(syn::Error::new(
                        input.span(),
                        "Expected handler_files = [...]",
                    ))
                }
            }
            "components" => {
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                }

                if input.peek(syn::token::Bracket) {
                    let content;
                    bracketed!(content in input);
                    let tokens = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?;

                    Ok(Self::Components(tokens.into_iter().collect::<Vec<_>>()))
                } else {
                    Err(syn::Error::new(input.span(), "Expected components = [...]"))
                }
            }
            "handlers" => {
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                }

                if input.peek(syn::token::Bracket) {
                    let content;
                    bracketed!(content in input);
                    let tokens =
                        Punctuated::<syn::ExprPath, Token![,]>::parse_terminated(&content)?;

                    Ok(Self::Handlers(tokens.into_iter().collect::<Vec<_>>()))
                } else {
                    Err(syn::Error::new(input.span(), "Expected handlers = [...]"))
                }
            }
            _ => Err(syn::Error::new(
                input.span(),
                "unexpected token expected either handler_files or components",
            )),
        }
    }
}

fn impl_paths<I: IntoIterator<Item = ExprPath>>(
    handler_paths: I,
    quote: &mut TokenStream2,
) -> TokenStream2 {
    quote.extend(quote! {
        use utoipa::Path as OpenApiPath;
    });
    handler_paths.into_iter().fold(
        quote! { utoipa::openapi::path::Paths::new() },
        |mut paths, handler| {
            let segments = handler.path.segments.iter().collect::<Vec<_>>();
            let handler_fn_name = &*segments.last().unwrap().ident.to_string();

            let tag = segments
                .iter()
                .take(segments.len() - 1)
                .map(|part| part.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            let handler_ident = format_ident!("{}{}", PATH_STRUCT_PREFIX, handler_fn_name);
            let handler_ident_name = &*handler_ident.to_string();

            let usage = syn::parse_str::<ExprPath>(
                &vec![
                    if tag.starts_with("crate") {
                        None
                    } else {
                        Some("crate")
                    },
                    if tag.is_empty() { None } else { Some(&tag) },
                    Some(handler_ident_name),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join("::"),
            )
            .unwrap();

            let assert_handler_ident = format_ident!("__assert_{}", handler_ident_name);
            quote.extend(quote! {
                struct #assert_handler_ident where #handler_ident : utoipa::Path;
                use #usage;
                impl utoipa::DefaultTag for #handler_ident {
                    fn tag() -> &'static str {
                        #tag
                    }
                }
            });
            paths.extend(quote! {
                .append(#handler_ident::path(), #handler_ident::path_item())
            });

            paths
        },
    )
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
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
struct MediaType {
    ty: Option<Ident>,
    is_array: bool,
}

impl Parse for MediaType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut is_array = false;
        let ty = if input.peek(Bracket) {
            is_array = true;
            let group;
            bracketed!(group in input);
            group.parse::<Ident>().unwrap()
        } else {
            input.parse::<Ident>().unwrap()
        };

        Ok(MediaType {
            ty: Some(ty),
            is_array,
        })
    }
}
