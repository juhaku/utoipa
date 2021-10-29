use proc_macro::{Span, TokenStream};
use quote::quote;

use proc_macro2::Ident;
use syn::{bracketed, parse::Parse, punctuated::Punctuated, Attribute, DeriveInput, LitStr, Token};

mod info;
mod paths;

use proc_macro_error::*;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn api_operation(_attr: TokenStream, item: TokenStream) -> TokenStream {
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

    let span = Span::call_site();

    let files = resolve_handler_files(&attrs).unwrap_or_else(|| {
        abort!(
            span,
            "Expected at least one handler file as #openapi[...] argument"
        )
    });

    // println!("attributes: {:#?}", &attrs);
    // println!("data: {:#?}", &data);
    // println!("ident: {:#?}", &ident);
    // println!("generics: {:#?}", &generics);

    let info = info::impl_info();
    let paths = paths::impl_paths(&files);

    let quote = quote! {
        impl utoipa::OpenApi for #ident {
            fn openapi() -> utoipa::openapi::OpenApi {
                utoipa::openapi::OpenApi::new(#info, #paths)
            }
        }
    };

    println!("{:#?}", &quote);

    quote.into()
}

fn resolve_handler_files(attributes: &[Attribute]) -> Option<Vec<String>> {
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
        })
        .map(|args| {
            args.into_iter()
                .map(|arg| match arg {
                    OpenApiArgs::HandlerFiles(files) => files,
                })
                .flatten()
                .collect::<Vec<_>>()
        })
}

enum OpenApiArgs {
    HandlerFiles(Vec<String>),
}

impl Parse for OpenApiArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;

        println!("parsed ident: {:?}", name);

        if !input.peek(Token![=]) {
            abort!(
                input.span(),
                "Expected token = after {} but did not find one",
                &name
            );
        }
        input.parse::<Token![=]>()?;

        if !input.peek(syn::token::Bracket) {
            abort!(
                input.span(),
                "Expected group [...], but dit not find brackets"
            );
        }
        let content;
        bracketed!(content in input);
        let tokens = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;

        println!("tokens: {:#?}", &tokens);

        let name_str = name.to_string();

        match &*name_str {
            "handler_files" => Ok(Self::HandlerFiles(
                tokens.iter().map(LitStr::value).collect::<Vec<_>>(),
            )),
            _ => abort!(
                input.span(),
                "unexpected token: {}, call with #[openapi(handler_files = [...])]",
                &name_str
            ),
        }
    }
}
