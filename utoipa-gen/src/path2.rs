use std::{io::Error, str::FromStr};

use proc_macro2::{Group, Ident};
use proc_macro_error::{abort_call_site, ResultExt};
use quote::{quote, ToTokens};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    LitInt, LitStr, Token,
};

// #[api_operation(delete,
//    operation_id = "custom_operation_id",
//    path = "custom_path",
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
