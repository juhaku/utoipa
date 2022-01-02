//! This is private utoipa codegen library and is not used alone

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};

use proc_macro2::{Ident, TokenStream as TokenStream2};
use syn::{
    bracketed, parse::Parse, punctuated::Punctuated, Attribute, DeriveInput, ExprPath, LitStr,
    Token,
};

mod attribute;
mod component;
mod component_type;
mod info;
mod path;
mod paths;

use proc_macro_error::*;

use crate::{
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
pub fn api_operation(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("Attr: {:#?}", &attr);
    // let input = syn::parse_macro_input!(attr as PathAttr);

    item
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn path(attr: TokenStream, item: TokenStream) -> TokenStream {
    let path_attribute = syn::parse_macro_input!(attr as PathAttr);

    // println!("parsed path attribute: {:#?}", &path_attribute);

    let ast_fn = syn::parse::<syn::ItemFn>(item).unwrap_or_abort();

    // println!("item attrs: {:#?}", &attrs);
    // println!("item block: {:#?}", &block);
    // println!("item sig: {:#?}", &sig);
    // println!("item vis: {:#?}", &vis);

    let fn_name = &*ast_fn.sig.ident.to_string();

    let attribute = &ast_fn.attrs.iter().find_map(|attribute| {
        if is_valid_request_type(
            &attribute
                .path
                .get_ident()
                .map(ToString::to_string)
                .unwrap_or_default(),
        ) {
            Some(attribute)
        } else {
            None
        }
    });

    #[cfg(feature = "actix_gen")]
    let path_provider = || {
        attribute.as_ref().map(|attribute| {
            let lit = attribute.parse_args::<LitStr>().unwrap();
            lit.value() // TODO format path according OpenAPI specs
        })
    };

    #[cfg(not(feature = "actix_gen"))]
    let path_provider = || None::<String>;

    // TODO validate that path is provided one way or the other

    // println!("path provider: {:#?}", path_provider());

    let path = Path::new(path_attribute, fn_name)
        .with_path_operation(attribute.as_ref().map(|attribute| {
            let ident = attribute.path.get_ident().unwrap();
            PathOperation::from_ident(ident)
        }))
        .with_path(path_provider);

    quote! {
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
                abort_call_site!("Expected #[openapi(...)], but was: {:?}", attribute);
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

fn is_valid_request_type(s: &str) -> bool {
    match s {
        "get" | "post" | "put" | "delete" | "head" | "connect" | "options" | "trace" | "patch" => {
            true
        }
        _ => false,
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
