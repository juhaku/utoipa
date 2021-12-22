use std::{env, fs, str::FromStr};

use proc_macro2::{Group, Ident, Punct, TokenStream as TokenStream2};
use proc_macro_error::{abort, abort_call_site, ResultExt};
use quote::quote;
use syn::{
    parse::Parse, punctuated::Punctuated, Attribute, Error, ItemFn, LitInt, LitStr, MetaNameValue,
    Token,
};

const API_OPERATION_IDENT: &str = "api_operation";

#[derive(Debug, Default, Clone)]
struct Path {
    comments: Vec<String>,
    operation: Operation,
    responses: Vec<ApiOperationResponse>,
    operation_id: String,
    // TODO missing request path, request body, request params, response body
}

impl Path {
    fn with_comment<S: Into<String>>(mut self, comment: S) -> Self {
        self.comments.push(comment.into());

        self
    }

    fn with_responses(self, responses: Vec<ApiOperationResponse>) -> Self {
        Self { responses, ..self }
    }

    fn with_operation(self, operation: Operation) -> Self {
        Self { operation, ..self }
    }

    fn with_operation_id<S: Into<String>>(self, operation_id: S) -> Self {
        Self {
            operation_id: operation_id.into(),
            ..self
        }
    }
}

#[derive(Debug, Clone)]
enum Operation {
    Get,
    Delete,
    Put,
    Post,
    Head,
    Patch,
}

impl Default for Operation {
    fn default() -> Self {
        Self::Get
    }
}

impl FromStr for Operation {
    type Err = syn::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "get" => Ok(Self::Get),
            "delete" => Ok(Self::Delete),
            "put" => Ok(Self::Put),
            "post" => Ok(Self::Post),
            "head" => Ok(Self::Head),
            "patch" => Ok(Self::Patch),
            _ => Err(Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "invalid operation: {}, expected one of: {}",
                    s, "get,delete,put,post,head,patch"
                ),
            )),
        }
    }
}

#[derive(Debug, Default, Clone)]
struct ApiOperationResponse {
    code: i32,
    description: String,
    response_item: Option<syn::Ident>,
}

impl Parse for ApiOperationResponse {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let code = input.parse::<LitInt>()?;
        println!("parsed response code: {:?}", code);

        input.parse::<Punct>()?;

        let description = input.parse::<LitStr>()?;
        println!("parsed response description: {:?}", description);

        if input.is_empty() {
            Ok(ApiOperationResponse {
                code: code.base10_parse()?,
                description: description.value(),
                response_item: None,
            })
        } else {
            input.parse::<Punct>()?;

            let response_type = input.parse::<Ident>()?;
            println!("parsed response type: {:?}", response_type);

            Ok(ApiOperationResponse {
                code: code.base10_parse()?,
                description: description.value(),
                response_item: Some(response_type),
            })
        }
    }
}

enum ApiOperationItem {
    Operation(Operation),
    Responses(Vec<ApiOperationResponse>),
}

impl Parse for ApiOperationItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;

        let name = &*ident.to_string();

        println!("ident: {:#?}", ident);

        match name {
            operation if input.peek(syn::Token![,]) && input.peek2(syn::Ident) => {
                // input.parse::<Token![,]>()?;
                // input.parse::<syn::Ident>()?;

                // input.parse::<Token![,]>()?; // parse next comma (,)

                Ok(ApiOperationItem::Operation(operation.parse()?))
            }
            "responses" if input.peek(syn::Token![=]) => {
                input.parse::<Token![=]>()?;

                let content;
                syn::bracketed!(content in input);
                let response_groups = Punctuated::<Group, Token![,]>::parse_terminated(&content)?;

                Ok(ApiOperationItem::Responses(
                    response_groups
                        .iter()
                        .map(|group| {
                            syn::parse2::<ApiOperationResponse>(group.stream()).unwrap_or_else(
                                |error| {
                                    abort!(
                                        group,
                                        "parse responses api operation error: {:?}",
                                        error
                                    )
                                },
                            )
                        })
                        .collect::<Vec<_>>(),
                ))
            }
            _ => abort!(input.span(), "unexpected attribute value: {}", name),
        }
    }
}

pub fn impl_paths(files: &[String]) -> TokenStream2 {
    parse_files(files);

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
        .unwrap_or_else(|error| abort_call_site!("could not define current dir: {}", error));

    let path = current_dir.join(file.as_ref());

    let content = fs::read_to_string(&path.as_path()).unwrap_or_else(|error| {
        abort_call_site!("read file from path: {:?}, error: {}", path, error)
    });

    let file_content = syn::parse_file(&content).unwrap();

    // file_content.items.iter().for_each(|item| match item {
    //     syn::Item::Fn(func) => {
    //         if let Some(span) = has_attribute(&func.attrs, API_OPERATION_IDENT) {
    //             abort!(span, "missing api_operation");
    //         } else {
    //             abort_call_site!("Missing attribute open_apifffff");
    //         }
    //     }
    //     _ => (),
    // });

    file_content.items.iter().for_each(|item| {
        // TODO
        match item {
            syn::Item::Fn(func) if has_attribute(&func.attrs, API_OPERATION_IDENT) => {
                process_function(func);
            }
            _ => (),
        }
    });
    // println!("{:#?}", &file_content);
}

fn has_attribute<S: AsRef<str>>(attributes: &[Attribute], name: S) -> bool {
    attributes.iter().any(|attribute| {
        let ident = &attribute
            .path
            .segments
            .first()
            .unwrap_or_else(|| {
                abort!(
                    attribute.path,
                    "incorrect path, expected to have a one segment"
                )
            })
            .ident;

        *ident == name.as_ref()
    })
}

fn process_function(function: &ItemFn) {
    // TODO this should return the resolved path token stream

    let fn_name = function.sig.ident.to_string();
    println!("fn: {:#?}", function);

    // println!("{:?}", function.sig.ident);

    // emit_error!(function.sig.ident.span(), "missing foo bar");

    function
        .attrs
        .iter()
        .fold(Path::default(), |api_operation, attribute| {
            let attribute_ident = &attribute
                .path
                .segments
                .first()
                .unwrap_or_else(|| {
                    abort!(
                        attribute.path,
                        "incorrect path, expected to have a one segment"
                    )
                })
                .ident;

            let name = attribute_ident.to_string();

            match &*name {
                "doc" => api_operation.with_comment(parse_doc_attribute(attribute)), // TODO parse doc
                API_OPERATION_IDENT => {
                    // println!("attribute: {:#?}", attribute);
                    parse_api_operation_attribute(attribute).into_iter().fold(
                        api_operation,
                        |op, item| match item {
                            ApiOperationItem::Operation(operation) => op.with_operation(operation),
                            ApiOperationItem::Responses(responses) => op.with_responses(responses),
                        },
                    )
                } // TODO parse api_operation
                _rest => api_operation, // TODO do custom manipulation based on enabled library
            }
        })
        .with_operation_id(fn_name);
}

fn parse_doc_attribute(attribute: &Attribute) -> String {
    let meta = attribute.parse_meta().unwrap_or_else(|error| {
        abort!(
            attribute,
            "parse attribute meta: {:?} failed: {}",
            attribute,
            error
        )
    });

    match meta {
        syn::Meta::NameValue(MetaNameValue { lit, .. }) => match lit {
            syn::Lit::Str(lit_str) => lit_str.value(),
            _ => abort!(lit, "unexpecte lit type: {:?}", lit),
        },
        _ => abort!(meta, "unexpected meta type: {:?}", meta),
    }
}

fn parse_api_operation_attribute(attribute: &Attribute) -> Punctuated<ApiOperationItem, Token![,]> {
    attribute
        .parse_args_with(Punctuated::<ApiOperationItem, Token![,]>::parse_terminated)
        .unwrap_or_abort()
}
