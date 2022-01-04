use std::{io::Error, str::FromStr};

use proc_macro2::{Group, Ident, TokenStream as TokenStream2};
use proc_macro_error::{abort_call_site, OptionExt, ResultExt};
use quote::{format_ident, quote, ToTokens};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    LitInt, LitStr, Token,
};

const PATH_STRUCT_PREFIX: &str = "__path_";

// #[api_operation(delete,
//    operation_id = "custom_operation_id",
//    path = "custom_path",
//    tag = "groupping_tag"
//    responses = [
//     (200, "success", String),
//     (400, "my bad error", u64),
//     (404, "vault not found"),
//     (500, "internal server error")
// ])]

/// PathAttr is parsed #[path(...)] proc macro and its attributes.
/// Parsed attributes can be used to override or append OpenAPI Path
/// options.
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct PathAttr {
    path_operation: Option<PathOperation>,
    responses: Vec<PathResponse>,
    path: Option<String>,
    operation_id: Option<String>,
    tag: Option<String>,
}

/// Parse implementation for PathAttr will parse arguments
/// exhaustively.
impl Parse for PathAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut path_attr = PathAttr::default();

        loop {
            let ident = input.parse::<Ident>().unwrap();
            let ident_name = &*ident.to_string();

            let parse_lit_str = |input: &ParseStream, error_message: &str| -> String {
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>().unwrap();
                }

                input
                    .parse::<LitStr>()
                    .expect_or_abort(error_message)
                    .value()
            };

            match ident_name {
                "operation_id" => {
                    path_attr.operation_id = Some(parse_lit_str(
                        &input,
                        "expected literal string for operation id",
                    ));
                }
                "path" => {
                    path_attr.path =
                        Some(parse_lit_str(&input, "expected literal string for path"));
                }
                "responses" => {
                    if input.peek(Token![=]) {
                        input.parse::<Token![=]>().unwrap();
                    }

                    let content;
                    bracketed!(content in input);
                    let groups = Punctuated::<Group, Token![,]>::parse_terminated(&content)
                        .expect_or_abort("expected responses to be group separated by comma (,)");

                    path_attr.responses = groups
                        .iter()
                        .map(|group| parse2::<PathResponse>(group.stream()).unwrap_or_abort())
                        .collect::<Vec<_>>();
                }
                "tag" => {
                    path_attr.tag = Some(parse_lit_str(&input, "expected literal string for tag"));
                }
                _ => {
                    // any other case it is expected to be path operation
                    if let Some(path_operation) =
                        ident_name.parse::<PathOperation>().into_iter().next()
                    {
                        path_attr.path_operation = Some(path_operation)
                    }
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>().unwrap();
            }
            if input.is_empty() {
                break;
            }
        }

        Ok(path_attr)
    }
}

/// Path operation type of response
///
/// Instance of path operation can be formed from str parsing with following supported values:
///   * "get"
///   * "post"
///   * "put"
///   * "delete"
///   * "options"
///   * "head"
///   * "patch"
///   * "trace"
///
/// # Examples
///
/// Basic usage:
/// ```
/// let operation = "get".parse::<PathOperation>().unwrap();
/// assert_eq!(operation, PathOperation::Get)
/// ```
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum PathOperation {
    Get,
    Post,
    Put,
    Delete,
    Options,
    Head,
    Patch,
    Trace,
}

impl PathOperation {
    /// Create path operation from ident
    ///
    /// Ident must have value of http request type as lower case string such as `get`.
    pub fn from_ident(ident: &Ident) -> Self {
        match ident.to_string().as_str().parse::<PathOperation>() {
            Ok(operation) => operation,
            Err(error) => abort_call_site!("{}", error),
        }
    }
}

impl FromStr for PathOperation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "get" => Ok(Self::Get),
            "post" => Ok(Self::Post),
            "put" => Ok(Self::Put),
            "delete" => Ok(Self::Delete),
            "options" => Ok(Self::Options),
            "head" => Ok(Self::Head),
            "patch" => Ok(Self::Patch),
            "trace" => Ok(Self::Trace),
            _ => Err(Error::new(
                std::io::ErrorKind::Other,
                "invalid PathOperation expected one of: [get, post, put, delete, options, head, patch, trace]",
            )),
        }
    }
}

impl ToTokens for PathOperation {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let path_item_type = match self {
            Self::Get => quote! { utoipa::openapi::PathItemType::Get },
            Self::Post => quote! { utoipa::openapi::PathItemType::Post },
            Self::Put => quote! { utoipa::openapi::PathItemType::Put },
            Self::Delete => quote! { utoipa::openapi::PathItemType::Delete },
            Self::Options => quote! { utoipa::openapi::PathItemType::Options },
            Self::Head => quote! { utoipa::openapi::PathItemType::Head },
            Self::Patch => quote! { utoipa::openapi::PathItemType::Patch },
            Self::Trace => quote! { utoipa::openapi::PathItemType::Trace },
        };

        tokens.extend(path_item_type);
    }
}

/// Parsed representation of response argument within `#[path(...)]` macro attribute.
/// Response is typically formed from group such like (200, "success", String) where
///   * 200 number represents http status code
///   * "success" stands for response description included in documentation
///   * String represents type of response body
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
struct PathResponse {
    status_code: i32,
    message: String,
    response_type: Option<Ident>,
}

impl Parse for PathResponse {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut response = PathResponse::default();

        loop {
            let next_type = input.lookahead1();
            if next_type.peek(LitInt) {
                response.status_code = input
                    .parse::<LitInt>()
                    .unwrap()
                    .base10_parse()
                    .unwrap_or_abort();
            } else if next_type.peek(LitStr) {
                response.message = input.parse::<LitStr>().unwrap().value();
            } else if next_type.peek(syn::Ident) {
                response.response_type = Some(input.parse::<syn::Ident>().unwrap());
            } else {
                return Err(next_type.error());
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>().unwrap();
            }

            if input.is_empty() {
                break;
            }
        }

        Ok(response)
    }
}

pub struct Path {
    path_attr: PathAttr,
    fn_name: String,
    path_operation: Option<PathOperation>,
    path: Option<String>,
    doc_comments: Option<Vec<String>>,
    deprecated: Option<bool>,
}

impl Path {
    pub fn new(path_attr: PathAttr, fn_name: &str) -> Self {
        Self {
            path_attr,
            fn_name: fn_name.to_string(),
            path_operation: None,
            path: None,
            doc_comments: None,
            deprecated: None,
        }
    }

    pub fn with_path_operation(mut self, path_operation: Option<PathOperation>) -> Self {
        self.path_operation = path_operation;

        self
    }

    pub fn with_path(mut self, path_provider: impl FnOnce() -> Option<String>) -> Self {
        self.path = path_provider();

        self
    }

    pub fn with_doc_comments(mut self, doc_commens: Vec<String>) -> Self {
        self.doc_comments = Some(doc_commens);

        self
    }

    pub fn with_deprecated(mut self, deprecated: Option<bool>) -> Self {
        self.deprecated = deprecated;

        self
    }
}

impl ToTokens for Path {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let path_struct = format_ident!("{}{}", PATH_STRUCT_PREFIX, self.fn_name);
        let operation_id = self
            .path_attr
            .operation_id
            .as_ref()
            .or(Some(&self.fn_name))
            .expect_or_abort("expected to find operation id but was None");
        let tag = self
            .path_attr
            .tag
            .as_ref()
            .map(ToOwned::to_owned)
            .unwrap_or_default();
        let path_operation = self
            .path_attr
            .path_operation
            .as_ref()
            .or_else(|| self.path_operation.as_ref())
            .expect_or_abort("expected to find path operation but was None");
        let path = self
            .path_attr
            .path
            .as_ref()
            .or_else(|| self.path.as_ref())
            .expect_or_abort("expected to find path but was None");

        let operation = Operation {
            fn_name: &self.fn_name,
            deprecated: &self.deprecated,
            operation_id,
            summary: self
                .doc_comments
                .as_ref()
                .and_then(|comments| comments.iter().next()),
            description: self.doc_comments.as_ref(),
        };

        tokens.extend(quote! {
            #[allow(non_camel_case_types)]
            pub struct #path_struct;

            impl utoipa::Tag for #path_struct {
                fn tag() -> &'static str {
                    #tag
                }
            }

            impl utoipa::Path for  #path_struct {
                fn path() -> &'static str {
                    #path
                }

                fn path_item() -> utoipa::openapi::path::PathItem {
                    utoipa::openapi::PathItem::new(
                        #path_operation,
                        #operation
                    )
                }
            }
        })
    }
}

struct Operation<'a> {
    fn_name: &'a String,
    operation_id: &'a String,
    summary: Option<&'a String>,
    description: Option<&'a Vec<String>>,
    deprecated: &'a Option<bool>,
}

impl ToTokens for Operation<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(quote! { utoipa::openapi::path::Operation::new() });

        // impl dummy responses
        tokens.extend(quote! {
            .with_response(
                "200", // TODO resolve this status
                utoipa::openapi::response::Response::new("this is response message")
            )
        });
        //         // .with_parameters()
        //         // .with_request_body()
        //         // .with_security()
        let path_struct = format_ident!("{}{}", PATH_STRUCT_PREFIX, self.fn_name);
        let operation_id = self.operation_id;
        tokens.extend(quote! {
            .with_tag(
                vec![<#path_struct as utoipa::Tag>::tag(),
                    <#path_struct as utoipa::DefaultTag>::tag()
                ]
                .into_iter().find(|s| !s.is_empty()).unwrap_or_else(|| "crate")
            )
            .with_operation_id(
                #operation_id
            )
        });

        let deprecated = get_deprecated_token_stream(self.deprecated);
        tokens.extend(quote! {
           .with_deprecated(#deprecated)
        });

        if let Some(summary) = self.summary {
            tokens.extend(quote! {
                .with_summary(#summary)
            })
        }

        if let Some(description) = self.description {
            let description = description
                .iter()
                .map(|comment| format!("{}\n", comment))
                .collect::<Vec<String>>()
                .join("");

            tokens.extend(quote! {
                .with_description(#description)
            })
        }
    }
}

fn get_deprecated_token_stream(deprecated: &Option<bool>) -> TokenStream2 {
    let get_deprecated_false = || {
        quote! {
            utoipa::openapi::Deprecated::False
        }
    };

    deprecated
        .as_ref()
        .map(|deprecated| {
            println!("is deprecated: {:?}", deprecated);
            if *deprecated {
                quote! {
                    utoipa::openapi::Deprecated::True
                }
            } else {
                get_deprecated_false()
            }
        })
        .unwrap_or_else(get_deprecated_false)
}
