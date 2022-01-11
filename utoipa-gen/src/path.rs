use std::{io::Error, str::FromStr};

use proc_macro2::{Group, Ident, Span, TokenStream as TokenStream2};
use proc_macro_error::{abort_call_site, OptionExt, ResultExt};
use quote::{format_ident, quote, ToTokens};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    token::Bracket,
    LitInt, LitStr, Token,
};

use crate::component_type::{ComponentFormat, ComponentType};

const PATH_STRUCT_PREFIX: &str = "__path_";

// #[utoipa::path(delete,
//    operation_id = "custom_operation_id",
//    path = "/custom/path/{id}/{digest}",
//    tag = "groupping_tag"
//    responses = [
//     (status = 200, description = "delete foo entity successful",
//          body = String, content_type = "text/plain"),
//     (status = 500, description = "internal server error",
//          body = String, content_type = "text/plain")
//     (400, "my bad error", u64),
//     (404, "vault not found"),
//     (status = 500, description = "internal server error", body = String, content_type = "text/plain")
//    ],
//    params = [
//      ("myval" = String, description = "this is description"),
//      ("myval", description = "this is description"),
//      ("myval" = String, path, required, deprecated, description = "this is description"),
//    ]
// )]

// #[utoipa::response(
//      status = 200,
//      description = "success response",
//      body = String,
//      content_type = "text/plain"
// )]
// #[utoipa::response(
//      status = 400,
//      description = "this is bad request",
//      body = String,
//      content_type = "application/json"
// )]
// #[utoipa::response(
//      status = 500,
//      description = "internal server error",
//      body = Error,
//      content_type = "text/plain"
// )]
// #[utoipa::response(
//      status = 404,
//      description = "item not found",
//      body = i32 // because body type is primitive the content_type is not necessary
// )]
// implementation should make assumptions based on response body type. If response body type is primitive type
// content_type is set to text/pain by default

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
    pub params: Option<Vec<Parameter>>,
}

/// Parse implementation for PathAttr will parse arguments
/// exhaustively.
impl Parse for PathAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut path_attr = PathAttr::default();

        loop {
            let ident = input
                .parse::<Ident>()
                .expect_or_abort("failed to parse first ident");
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

            let parse_groups = |input: &ParseStream| {
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>().unwrap();
                }

                let content;
                bracketed!(content in input);

                Punctuated::<Group, Token![,]>::parse_terminated(&content)
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
                    let groups = parse_groups(&input)
                        .expect_or_abort("expected responses to be group separated by comma (,)");

                    path_attr.responses = groups
                        .iter()
                        .map(|group| parse2::<PathResponse>(group.stream()).unwrap_or_abort())
                        .collect::<Vec<_>>();
                }
                "params" => {
                    let groups = parse_groups(&input)
                        .expect_or_abort("expected parameters to be group separated by comma (,)");
                    path_attr.params = Some(
                        groups
                            .iter()
                            .map(|group| parse2::<Parameter>(group.stream()).unwrap_or_abort())
                            .collect::<Vec<Parameter>>(),
                    )
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

/// Parameter of request suchs as in path, header, query or cookie
///
/// For example path `/users/{id}` the path parameter is used to define
/// type, format and other details of the `{id}` parameter within the path
///
/// Parse is executed for following formats:
///
/// * ("id" = String, path, required, deprecated, description = "Users database id"),
/// * ("id", path, required, deprecated, description = "Users database id"),
///
/// The `= String` type statement is optional if automatic resolvation is supported.
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Parameter {
    pub name: String,
    parameter_in: ParameterIn,
    required: bool,
    deprecated: bool,
    pub parameter_type: Option<Ident>,
    is_array: bool,
    description: Option<String>,
}

impl Parameter {
    pub fn new<S: AsRef<str>>(name: S, parameter_type: &Ident, parameter_in: ParameterIn) -> Self {
        let required = parameter_in == ParameterIn::Path;

        Self {
            name: name.as_ref().to_string(),
            parameter_type: Some(parameter_type.clone()),
            parameter_in,
            required,
            ..Default::default()
        }
    }

    pub fn update_parameter_type(&mut self, ident: &Ident) {
        self.parameter_type = Some(ident.clone());
    }
}

impl Parse for Parameter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut parameter = Parameter::default();

        if input.peek(LitStr) {
            // parse name
            let name = input.parse::<LitStr>().unwrap().value();
            parameter.name = name;
            if input.peek(Token![=]) && input.peek2(syn::Ident) {
                // parse type for name if provided
                input.parse::<Token![=]>().unwrap();
                parameter.parameter_type = Some(input.parse::<syn::Ident>().unwrap());
            } else if input.peek(Token![=]) && input.peek2(Bracket) {
                // parse group as array
                input.parse::<Token![=]>().unwrap();
                parameter.is_array = true;
                let group_content;
                bracketed!(group_content in input);
                parameter.parameter_type = Some(group_content.parse::<Ident>().unwrap());
            }
        } else {
            return Err(input.error("expected first element to be LitStr parameter name"));
        }
        if input.peek(Token![,]) {
            input.parse::<Token![,]>().unwrap();
        }

        loop {
            let ident = input.parse::<syn::Ident>().unwrap();
            let name = &*ident.to_string();
            match name {
                "path" | "query" | "header" | "cookie" => {
                    parameter.parameter_in = name.parse::<ParameterIn>().unwrap_or_abort();
                    if parameter.parameter_in == ParameterIn::Path {
                        parameter.required = true; // all path parameters are required by default
                    }
                }
                "required" => parameter.required = true,
                "deprecated" => parameter.deprecated = true,
                "description" => {
                    if input.peek(Token![=]) {
                        input.parse::<Token![=]>().unwrap();
                    }

                    parameter.description = Some(
                        input
                            .parse::<LitStr>()
                            .expect_or_abort("expected description value as LitStr")
                            .value(),
                    )
                }
                _ => return Err(input.error(&format!("unexpected element: {}", name))),
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>().unwrap();
            }
            if input.is_empty() {
                break;
            }
        }
        Ok(parameter)
    }
}
impl ToTokens for Parameter {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &*self.name;
        tokens.extend(quote! { utoipa::openapi::path::Parameter::new(#name) });
        let parameter_in = &self.parameter_in;
        tokens.extend(quote! { .with_in(#parameter_in) });

        let required: Required = self.required.into();
        tokens.extend(quote! { .with_required(#required) });

        let deprecated: Deprecated = self.deprecated.into();
        tokens.extend(quote! { .with_deprecated(#deprecated) });

        if let Some(ref description) = self.description {
            tokens.extend(quote! { .with_description(#description) });
        }

        if let Some(ref parameter_type) = self.parameter_type {
            // TODO unify this property logic with the one in component.rs
            let component_type = ComponentType(parameter_type);
            let mut property = quote! {
                utoipa::openapi::Property::new(
                    #component_type
                )
            };
            let format = ComponentFormat(parameter_type);
            if format.is_known_format() {
                property.extend(quote! {
                    .with_format(#format)
                })
            }
            if self.is_array {
                property.extend(quote! { .to_array() });
            }

            tokens.extend(quote! { .with_schema(#property) });
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq)]
pub enum ParameterIn {
    Query,
    Path,
    Header,
    Cookie,
}

impl Default for ParameterIn {
    fn default() -> Self {
        Self::Path
    }
}

impl FromStr for ParameterIn {
    type Err = syn::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "path" => Ok(Self::Path),
            "query" => Ok(Self::Query),
            "header" => Ok(Self::Header),
            "cookie" => Ok(Self::Cookie),
            _ => Err(syn::Error::new(
                Span::call_site(),
                &format!(
                    "unexpected str: {}, expected one of: path, query, header, cookie",
                    s
                ),
            )),
        }
    }
}

impl ToTokens for ParameterIn {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(match self {
            Self::Path => quote! { utoipa::openapi::path::ParameterIn::Path },
            Self::Query => quote! { utoipa::openapi::path::ParameterIn::Query },
            Self::Header => quote! { utoipa::openapi::path::ParameterIn::Header },
            Self::Cookie => quote! { utoipa::openapi::path::ParameterIn::Cookie },
        })
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
            parameters: self.path_attr.params.as_ref(),
        };

        tokens.extend(quote! {
            #[allow(non_camel_case_types)]
            pub struct #path_struct;

            impl utoipa::Tag for #path_struct {
                fn tag() -> &'static str {
                    #tag
                }
            }

            impl utoipa::Path for #path_struct {
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
    parameters: Option<&'a Vec<Parameter>>,
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
        //         // .with_request_body()
        //         // .with_security()
        let path_struct = format_ident!("{}{}", PATH_STRUCT_PREFIX, self.fn_name);
        let operation_id = self.operation_id;
        tokens.extend(quote! {
            .with_tag(
                [<#path_struct as utoipa::Tag>::tag(),
                    <#path_struct as utoipa::DefaultTag>::tag()
                ]
                .into_iter().find(|s| !s.is_empty()).unwrap_or_else(|| "crate")
            )
            .with_operation_id(
                #operation_id
            )
        });

        let deprecated = self
            .deprecated
            .map(Into::<Deprecated>::into)
            .or(Some(Deprecated::False))
            .unwrap();
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

        if let Some(parameters) = self.parameters {
            parameters
                .iter()
                .for_each(|parameter| tokens.extend(quote! { .with_parameter(#parameter) }));
        }
    }
}

pub enum Deprecated {
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

pub enum Required {
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
