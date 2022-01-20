use std::{io::Error, str::FromStr};

use proc_macro2::{Group, Ident, TokenStream as TokenStream2};
use proc_macro_error::{abort, OptionExt, ResultExt};
use quote::{format_ident, quote, ToTokens};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    Token,
};

use crate::{
    component_type::ComponentType,
    ext::{Argument, ArgumentIn},
};
use crate::{parse_utils, Deprecated};

use self::{
    parameter::{Parameter, ParameterIn},
    request_body::RequestBodyAttr,
    response::{Response, Responses},
};

pub mod parameter;
mod property;
mod request_body;
mod response;

const PATH_STRUCT_PREFIX: &str = "__path_";

/// PathAttr is parsed `#[utoipa::path(...)]` proc macro and its attributes.
/// Parsed attributes can be used to override or append OpenAPI Path
/// options.
///
/// # Example
/// ```text
/// #[utoipa::path(delete,
///    operation_id = "custom_operation_id",
///    path = "/custom/path/{id}/{digest}",
///    tag = "groupping_tag"
///    request_body = [Foo]
///    responses = [
///         (status = 200, description = "success update Foos", body = [Foo], content_type = "application/json",
///             headers = [
///                 ("fooo-bar" = String, description = "custom header value")
///             ]
///         ),
///         (status = 500, description = "internal server error", body = String, content_type = "text/plain",
///             headers = [
///                 ("fooo-bar" = String, description = "custom header value")
///             ]
///         ),
///    ],
///    params = [
///      ("id" = u64, description = "Id of Foo"),
///      ("digest", description = "Foos message digest of last updated"),
///      ("x-csrf-token", header, required, deprecated),
///    ]
/// )]
/// ```
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct PathAttr {
    path_operation: Option<PathOperation>,
    request_body: Option<RequestBodyAttr>,
    responses: Vec<Response>,
    path: Option<String>,
    operation_id: Option<String>,
    tag: Option<String>,
    params: Option<Vec<Parameter>>,
}

impl PathAttr {
    #[cfg(feature = "actix_extras")]
    pub fn update_parameters(&mut self, arguments: Option<Vec<Argument>>) {
        if let Some(arguments) = arguments {
            let new_parameter = |argument: &Argument| {
                Parameter::new(
                    &argument.name.as_ref().unwrap(),
                    argument.ident,
                    if argument.argument_in == ArgumentIn::Path {
                        ParameterIn::Path
                    } else {
                        ParameterIn::Query
                    },
                )
            };

            if let Some(ref mut parameters) = self.params {
                parameters.iter_mut().for_each(|parameter| {
                    if let Some(argument) = arguments
                        .iter()
                        .find(|argument| argument.name.as_ref() == Some(&parameter.name))
                    {
                        parameter.update_parameter_type(argument.ident)
                    }
                });

                // add argument to the parameters if argument has a name and it does not exists in parameters
                arguments
                    .iter()
                    .filter(|argument| argument.has_name())
                    .for_each(|argument| {
                        // cannot use filter() for mutli borrow situation. :(
                        if !parameters
                            .iter()
                            .any(|parameter| Some(&parameter.name) == argument.name.as_ref())
                        {
                            // if parameters does not contain argument
                            parameters.push(new_parameter(argument))
                        }
                    });
            } else {
                // no parameters at all, add arguments to the parameters if argument has a name
                let mut params = Vec::with_capacity(arguments.len());
                arguments
                    .iter()
                    .filter(|argument| argument.has_name())
                    .map(new_parameter)
                    .for_each(|parameter| params.push(parameter));
                self.params = Some(params);
            }
        }
    }
}

impl Parse for PathAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut path_attr = PathAttr::default();

        loop {
            let ident = input
                .parse::<Ident>()
                .expect_or_abort("unparseable PatAttr, expected identifer");
            let attribute = &*ident.to_string();

            let parse_groups = |input: &ParseStream| {
                parse_utils::parse_next(input, || {
                    let content;
                    bracketed!(content in input);

                    Punctuated::<Group, Token![,]>::parse_terminated(&content)
                })
            };

            match attribute {
                "operation_id" => {
                    path_attr.operation_id = Some(parse_utils::parse_next_lit_str(
                        input,
                        "unparseable operation_id, expected literal string",
                    ));
                }
                "path" => {
                    path_attr.path = Some(parse_utils::parse_next_lit_str(
                        input,
                        "unparseable path, expected literal string",
                    ));
                }
                "request_body" => {
                    if input.peek(Token![=]) {
                        input.parse::<Token![=]>().unwrap();
                    }
                    path_attr.request_body =
                        Some(input.parse::<RequestBodyAttr>().unwrap_or_abort());
                }
                "responses" => {
                    let groups = parse_groups(&input).expect_or_abort(
                        "unparseable responses, expected group separated by comma (,)",
                    );

                    path_attr.responses = groups
                        .into_iter()
                        .map(|group| syn::parse2::<Response>(group.stream()).unwrap_or_abort())
                        .collect::<Vec<_>>();
                }
                "params" => {
                    let groups = parse_groups(&input).expect_or_abort(
                        "unparseable params, expected group separated by comma (,)",
                    );
                    path_attr.params = Some(
                        groups
                            .iter()
                            .map(|group| parse2::<Parameter>(group.stream()).unwrap_or_abort())
                            .collect::<Vec<Parameter>>(),
                    )
                }
                "tag" => {
                    path_attr.tag = Some(parse_utils::parse_next_lit_str(
                        input,
                        "unparseable tag, expected literal string",
                    ));
                }
                _ => {
                    // any other case it is expected to be path operation
                    if let Some(path_operation) =
                        attribute.parse::<PathOperation>().into_iter().next()
                    {
                        path_attr.path_operation = Some(path_operation)
                    } else {
                        let erro_msg = format!("unexpected identifier: {}, expected any of: operation_id, path, get, post, put, delete, options, head, patch, trace, connect, request_body, responses, params, tag", attribute);
                        return Err(syn::Error::new(ident.span(), erro_msg));
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
    Connect,
}

impl PathOperation {
    /// Create path operation from ident
    ///
    /// Ident must have value of http request type as lower case string such as `get`.
    pub fn from_ident(ident: &Ident) -> Self {
        match ident.to_string().as_str().parse::<PathOperation>() {
            Ok(operation) => operation,
            Err(error) => abort!(ident.span(), format!("{}", error)),
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
            "connect" => Ok(Self::Connect),
            _ => Err(Error::new(
                std::io::ErrorKind::Other,
                "invalid PathOperation expected one of: get, post, put, delete, options, head, patch, trace, connect",
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
            Self::Connect => quote! { utoipa::openapi::PathItemType::Connect },
        };

        tokens.extend(path_item_type);
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
            path_struct: &path_struct,
            deprecated: &self.deprecated,
            operation_id,
            summary: self
                .doc_comments
                .as_ref()
                .and_then(|comments| comments.iter().next()),
            description: self.doc_comments.as_ref(),
            parameters: self.path_attr.params.as_ref(),
            request_body: self.path_attr.request_body.as_ref(),
            responses: self.path_attr.responses.as_ref(),
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
                    use utoipa::openapi::ToArray;
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
    path_struct: &'a Ident,
    operation_id: &'a String,
    summary: Option<&'a String>,
    description: Option<&'a Vec<String>>,
    deprecated: &'a Option<bool>,
    parameters: Option<&'a Vec<Parameter>>,
    request_body: Option<&'a RequestBodyAttr>,
    responses: &'a Vec<Response>,
}

impl ToTokens for Operation<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(quote! { utoipa::openapi::path::Operation::new() });

        if let Some(request_body) = self.request_body {
            tokens.extend(quote! {
                .with_request_body(#request_body)
            })
        }

        let responses = Responses(self.responses);
        tokens.extend(quote! {
            .with_responses(#responses)
        });
        //         // .with_security()
        let path_struct = self.path_struct;
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

trait ContentTypeResolver {
    fn resolve_content_type<'a>(
        &self,
        content_type: Option<&'a String>,
        component_type: &ComponentType<'a>,
    ) -> &'a str {
        if let Some(content_type) = content_type {
            content_type
        } else if component_type.is_primitive() {
            "text/plain"
        } else {
            "application/json"
        }
    }
}
