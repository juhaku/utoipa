//! This is private utoipa codegen library and is not used alone

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};

use proc_macro2::Ident;
use syn::{
    bracketed,
    parse::{Parse, ParseBuffer},
    punctuated::Punctuated,
    token::Token,
    Attribute, DeriveInput, LitStr, Token,
};

mod attribute;
mod component;
mod component_type;
mod info;
mod path2;
mod paths;

use proc_macro_error::*;

use crate::{component::impl_component, path2::PathAttr};

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
    println!("attr: {:#?}", attr);

    let path_attribute = syn::parse_macro_input!(attr as PathAttr);

    println!("parsed path attribute: {:#?}", path_attribute);

    item
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

    let files = openapi_args
        .iter()
        .filter(|args| matches!(args, OpenApiArgs::HandlerFiles(_)))
        .flat_map(|args| match args {
            OpenApiArgs::HandlerFiles(files) => files,
            _ => unreachable!(),
        });

    let components = openapi_args
        .iter()
        .filter(|args| matches!(args, OpenApiArgs::Components(_)))
        .flat_map(|args| match args {
            OpenApiArgs::Components(components) => components,
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();

    let info = info::impl_info();
    let paths = paths::impl_paths(&files.map(String::to_owned).collect::<Vec<_>>());

    let span = ident.span();
    let mut quote = quote! {};
    let mut schema = quote! {
        utoipa::openapi::Schema::new()
    };

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
                utoipa::openapi::OpenApi::new(#info, #paths)
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

enum OpenApiArgs {
    HandlerFiles(Vec<String>),
    Components(Vec<Ident>),
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
            _ => Err(syn::Error::new(
                input.span(),
                "unexpected token expected either handler_files or components",
            )),
        }
    }
}
