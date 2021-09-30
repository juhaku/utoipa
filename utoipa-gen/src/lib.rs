// #![allow(reso)]
use std::{env, fs};

use proc_macro::TokenStream;
use quote::quote;

use proc_macro2::{Ident, TokenStream as TokenStream2};
use syn::{bracketed, parse::Parse, punctuated::Punctuated, Attribute, DeriveInput, LitStr, Token};

#[proc_macro]
pub fn openapi_spec(input: TokenStream) -> TokenStream {
    let current_dir = env::current_dir()
        .unwrap_or_else(|error| panic!("Could not define current dir: {}", error));
    let parsed = syn::parse_macro_input!(input as syn::LitStr);

    println!("{:#?}", parsed);

    let path = current_dir.join(&parsed.value());

    let content = fs::read_to_string(&path.as_path())
        .unwrap_or_else(|error| panic!("Read file from path: {:?}, error: {}", path, error));

    println!("{:#?}", syn::parse_file(&content).unwrap());

    let pkg_info = get_pkg_info();

    let tokens = quote! {
        #pkg_info
    };

    tokens.into()
}

fn get_pkg_info() -> TokenStream2 {
    let info = quote! {
        fn get_pkg_info() {
            let description = env!("CARGO_PKG_DESCRIPTION");
            let name = env!("CARGO_PKG_NAME");
            let version = env!("CARGO_PKG_VERSION");
            let authors = env!("CARGO_PKG_AUTHORS");
            let licence = env!("CARGO_PKG_LICENSE");

            println!(
                "description: {:?}, name: {:?}, version: {:?}, authors: {:?}, licence: {:?}",
                description, name, version, authors, licence
            );
        }
    };

    info
}

#[proc_macro_attribute]
pub fn api(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_derive(OpenApi, attributes(openapi))]
pub fn openapi(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs,
        data,
        generics,
        ident,
        ..
    } = syn::parse_macro_input!(input);

    if let Some(files) = resolve_handler_files(&attrs) {
        parse_files(&files)
    }

    println!("attributes: {:#?}", &attrs);
    println!("data: {:#?}", &data);
    println!("ident: {:#?}", &ident);
    println!("generics: {:#?}", &generics);

    let info = impl_info();
    let paths = impl_paths();

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
                panic!("Expected #[openapi(...)], but was: {:?}", attribute);
            } else {
                attribute
            }
        })
        .map(|att| {
            att.parse_args_with(Punctuated::<OpenApiArgs, Token![,]>::parse_terminated)
                .unwrap_or_else(|error| {
                    panic!("Parse attribute: {:?} failed, err: {:?}", att, error)
                })
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
            panic!("Expected token = after {} but did not find one", &name);
        }
        input.parse::<Token![=]>()?;

        if !input.peek(syn::token::Bracket) {
            panic!("Expected group [...], but dit not find brackets");
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
            _ => panic!(
                "unexpected token: {}, call with #[openapi(handler_files = [...])]",
                &name_str
            ),
        }
    }
}

fn impl_info() -> TokenStream2 {
    let name = std::env::var("CARGO_PKG_NAME").unwrap_or_default();
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let description = std::env::var("CARGO_PKG_DESCRIPTION").unwrap_or_default();
    let authors = std::env::var("CARGO_PKG_AUTHORS").unwrap_or_default();
    let licence = std::env::var("CARGO_PKG_LICENSE").unwrap_or_default();

    let contact = get_contact(&authors);

    let info = quote! {
        utoipa::openapi::Info::new(#name, #version)
            .with_description(#description)
            .with_licence(utoipa::openapi::Licence::new(#licence))
            .with_contact(#contact)
    };

    info
}

fn get_parsed_author(author: Option<&str>) -> Option<(&str, String)> {
    author.map(|author| {
        if author.contains('<') && author.contains('>') {
            let mut author_iter = author.split('<');

            let name = author_iter.next().unwrap_or_default();
            let mut email = author_iter.next().unwrap_or_default().to_string();
            email = email.replace("<", "").replace(">", "");

            (name.trim_end(), email)
        } else {
            (author, "".to_string())
        }
    })
}

fn get_contact(authors: &str) -> TokenStream2 {
    if let Some((name, email)) = get_parsed_author(authors.split(',').into_iter().next()) {
        quote! {
            utoipa::openapi::Contact::new()
            .with_name(#name)
            .with_email(#email)
        }
    } else {
        quote! {
            utoipa::openapi::Contact::default()
        }
    }
}

fn impl_paths() -> TokenStream2 {
    let paths = quote! {
        utoipa::openapi::Paths::new()
    };

    paths
}

fn parse_files(files: &[String]) {
    files.iter().for_each(parse_file)
}

fn parse_file<S: AsRef<str>>(file: S) {
    let current_dir = env::current_dir()
        .unwrap_or_else(|error| panic!("could not define current dir: {}", error));

    let path = current_dir.join(file.as_ref());

    let content = fs::read_to_string(&path.as_path())
        .unwrap_or_else(|error| panic!("read file from path: {:?}, error: {}", path, error));

    let file_content = syn::parse_file(&content).unwrap();
    println!("{:#?}", &file_content);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_author_with_email_success() {
        let author = "Tessu Tester <tessu@steps.com>";

        if let Some((name, email)) = get_parsed_author(Some(author)) {
            assert_eq!(
                name, "Tessu Tester",
                "expected name {} != {}",
                "Tessu Tester", name
            );
            assert_eq!(
                email, "tessu@steps.com",
                "expected email {} != {}",
                "tessu@steps.com", email
            );
        } else {
            panic!("Expected Some(Tessu Tester, tessu@steps.com), but was none")
        }
    }

    #[test]
    fn parse_author_only_name() {
        let author = "Tessu Tester";

        if let Some((name, email)) = get_parsed_author(Some(author)) {
            assert_eq!(
                name, "Tessu Tester",
                "expected name {} != {}",
                "Tessu Tester", name
            );
            assert_eq!(email, "", "expected email {} != {}", "", email);
        } else {
            panic!("Expected Some(Tessu Tester, ), but was none")
        }
    }
}
