use std::borrow::Cow;
use std::ops::Deref;
use std::{io::Error, str::FromStr};

use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Paren;
use syn::{parenthesized, parse::Parse, Token};
use syn::{Expr, ExprLit, Lit, LitStr, Type};

use crate::component::{GenericType, TypeTree};
use crate::path::request_body::RequestBody;
use crate::{as_tokens_or_diagnostics, parse_utils, Deprecated, Diagnostics, ToTokensDiagnostics};
use crate::{schema_type::SchemaType, security_requirement::SecurityRequirementsAttr, Array};

use self::response::Response;
use self::{parameter::Parameter, request_body::RequestBodyAttr, response::Responses};

pub mod example;
pub mod parameter;
mod request_body;
pub mod response;
mod status;

pub(crate) const PATH_STRUCT_PREFIX: &str = "__path_";

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct PathAttr<'p> {
    path_operation: Option<PathOperation>,
    request_body: Option<RequestBody<'p>>,
    responses: Vec<Response<'p>>,
    pub(super) path: Option<parse_utils::Value>,
    operation_id: Option<Expr>,
    tag: Option<parse_utils::Value>,
    tags: Vec<parse_utils::Value>,
    params: Vec<Parameter<'p>>,
    security: Option<Array<'p, SecurityRequirementsAttr>>,
    context_path: Option<parse_utils::Value>,
    impl_for: Option<Ident>,
}

impl<'p> PathAttr<'p> {
    #[cfg(feature = "auto_into_responses")]
    pub fn responses_from_into_responses(&mut self, ty: &'p syn::TypePath) {
        self.responses
            .push(Response::IntoResponses(Cow::Borrowed(ty)))
    }

    #[cfg(any(
        feature = "actix_extras",
        feature = "rocket_extras",
        feature = "axum_extras"
    ))]
    pub fn update_request_body(&mut self, request_body: Option<crate::ext::RequestBody<'p>>) {
        use std::mem;

        if self.request_body.is_none() {
            self.request_body = request_body
                .map(RequestBody::Ext)
                .or(mem::take(&mut self.request_body));
        }
    }

    /// Update path with external parameters from extensions.
    #[cfg(any(
        feature = "actix_extras",
        feature = "rocket_extras",
        feature = "axum_extras"
    ))]
    pub fn update_parameters_ext<I: IntoIterator<Item = Parameter<'p>>>(
        &mut self,
        ext_parameters: I,
    ) {
        let ext_params = ext_parameters.into_iter();

        let (existing_incoming_params, new_params): (Vec<Parameter>, Vec<Parameter>) =
            ext_params.partition(|param| self.params.iter().any(|p| p == param));

        for existing_incoming in existing_incoming_params {
            if let Some(param) = self.params.iter_mut().find(|p| **p == existing_incoming) {
                param.merge(existing_incoming);
            }
        }

        self.params.extend(
            new_params
                .into_iter()
                .filter(|param| !matches!(param, Parameter::IntoParamsIdent(_))),
        );
    }
}

impl Parse for PathAttr<'_> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE_MESSAGE: &str = "unexpected identifier, expected any of: operation_id, path, get, post, put, delete, options, head, patch, trace, connect, request_body, responses, params, tag, security, context_path";
        let mut path_attr = PathAttr::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>().map_err(|error| {
                syn::Error::new(
                    error.span(),
                    format!("{EXPECTED_ATTRIBUTE_MESSAGE}, {error}"),
                )
            })?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "operation_id" => {
                    path_attr.operation_id =
                        Some(parse_utils::parse_next(input, || Expr::parse(input))?);
                }
                "path" => {
                    path_attr.path = Some(parse_utils::parse_next_literal_str_or_expr(input)?);
                }
                "request_body" => {
                    path_attr.request_body =
                        Some(RequestBody::Parsed(input.parse::<RequestBodyAttr>()?));
                }
                "responses" => {
                    let responses;
                    parenthesized!(responses in input);
                    path_attr.responses =
                        Punctuated::<Response, Token![,]>::parse_terminated(&responses)
                            .map(|punctuated| punctuated.into_iter().collect::<Vec<Response>>())?;
                }
                "params" => {
                    let params;
                    parenthesized!(params in input);
                    path_attr.params =
                        Punctuated::<Parameter, Token![,]>::parse_terminated(&params)
                            .map(|punctuated| punctuated.into_iter().collect::<Vec<Parameter>>())?;
                }
                "tag" => {
                    path_attr.tag = Some(parse_utils::parse_next_literal_str_or_expr(input)?);
                }
                "tags" => {
                    path_attr.tags = parse_utils::parse_next(input, || {
                        let tags;
                        syn::bracketed!(tags in input);
                        Punctuated::<parse_utils::Value, Token![,]>::parse_terminated(&tags)
                    })?
                    .into_iter()
                    .collect::<Vec<_>>();
                }
                "security" => {
                    let security;
                    parenthesized!(security in input);
                    path_attr.security = Some(parse_utils::parse_groups(&security)?)
                }
                "context_path" => {
                    path_attr.context_path =
                        Some(parse_utils::parse_next_literal_str_or_expr(input)?)
                }
                "impl_for" => {
                    path_attr.impl_for =
                        Some(parse_utils::parse_next(input, || input.parse::<Ident>())?);
                }
                _ => {
                    // any other case it is expected to be path operation
                    if let Some(path_operation) =
                        attribute_name.parse::<PathOperation>().into_iter().next()
                    {
                        path_attr.path_operation = Some(path_operation)
                    } else {
                        return Err(syn::Error::new(ident.span(), EXPECTED_ATTRIBUTE_MESSAGE));
                    }
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
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
    #[cfg(any(feature = "actix_extras", feature = "rocket_extras"))]
    pub fn from_ident(ident: &Ident) -> Result<Self, Diagnostics> {
        ident
            .to_string()
            .as_str()
            .parse::<PathOperation>()
            .map_err(|error| Diagnostics::with_span(ident.span(), error.to_string()))
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
pub struct Path<'p> {
    path_attr: PathAttr<'p>,
    fn_name: String,
    path_operation: Option<PathOperation>,
    path: Option<String>,
    doc_comments: Option<Vec<String>>,
    deprecated: Option<bool>,
}

impl<'p> Path<'p> {
    pub fn new(path_attr: PathAttr<'p>, fn_name: &str) -> Self {
        Self {
            path_attr,
            fn_name: fn_name.to_string(),
            path_operation: None,
            path: None,
            doc_comments: None,
            deprecated: None,
        }
    }

    pub fn path_operation(mut self, path_operation: Option<PathOperation>) -> Self {
        self.path_operation = path_operation;

        self
    }

    pub fn path(mut self, path_provider: impl FnOnce() -> Option<String>) -> Self {
        self.path = path_provider();

        self
    }

    pub fn doc_comments(mut self, doc_comments: Vec<String>) -> Self {
        self.doc_comments = Some(doc_comments);

        self
    }

    pub fn deprecated(mut self, deprecated: Option<bool>) -> Self {
        self.deprecated = deprecated;

        self
    }
}

impl<'p> ToTokensDiagnostics for Path<'p> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), Diagnostics> {
        let operation_id = self
            .path_attr
            .operation_id
            .clone()
            .or(Some(
                ExprLit {
                    attrs: vec![],
                    lit: Lit::Str(LitStr::new(&self.fn_name, Span::call_site())),
                }
                .into(),
            ))
            .ok_or_else(|| {
                Diagnostics::new("operation id is not defined for path")
                    .help(format!(
                        "Try to define it in #[utoipa::path(operation_id = {})]",
                        &self.fn_name
                    ))
                    .help("Did you define the #[utoipa::path(...)] over function?")
            })?;

        let path_operation = self
            .path_attr
            .path_operation
            .as_ref()
            .or(self.path_operation.as_ref())
            .ok_or_else(|| {
                let diagnostics = || {
                    Diagnostics::new("path operation is not defined for path")
                        .help("Did you forget to define it, e.g. #[utoipa::path(get, ...)]")
                };

                #[cfg(any(feature = "actix_extras", feature = "rocket_extras"))]
                {
                    diagnostics().help(
                        "Did you forget to define operation path attribute macro e.g #[get(...)]",
                    )
                }

                #[cfg(not(any(feature = "actix_extras", feature = "rocket_extras")))]
                diagnostics()
            })?;

        let path = self
            .path_attr
            .path
            .as_ref()
            .map(|path| path.to_token_stream())
            .or(Some(self.path.to_token_stream()))
            .ok_or_else(|| {
                let diagnostics = || {
                    Diagnostics::new("path is not defined for path")
                        .help(r#"Did you forget to define it in #[utoipa::path(path = "...")]"#)
                };

                #[cfg(any(feature = "actix_extras", feature = "rocket_extras"))]
                {
                    diagnostics().help(
                        "Did you forget to define operation path attribute macro e.g #[get(...)]",
                    )
                }

                #[cfg(not(any(feature = "actix_extras", feature = "rocket_extras")))]
                diagnostics()
            })?;

        let path_with_context_path = self
            .path_attr
            .context_path
            .as_ref()
            .map(|context_path| {
                let context_path = context_path.to_token_stream();
                let context_path_tokens = quote! {
                    format!("{}{}",
                        #context_path.to_string().replace('"', ""),
                        #path.to_string().replace('"', "")
                    )
                };
                context_path_tokens
            })
            .unwrap_or_else(|| {
                quote! {
                    #path.to_string().replace('"', "")
                }
            });

        let split_comment = self.doc_comments.as_ref().map(|comments| {
            let mut split = comments.split(|comment| comment.trim().is_empty());
            let summary = split
                .by_ref()
                .next()
                .map(|summary| summary.join("\n"))
                .unwrap_or_default();
            let description = split.map(|lines| lines.join("\n")).collect::<Vec<_>>();

            (summary, description)
        });

        let operation: Operation = Operation {
            deprecated: &self.deprecated,
            operation_id,
            summary: split_comment.as_ref().and_then(|(summary, _)| {
                if summary.is_empty() {
                    None
                } else {
                    Some(summary)
                }
            }),
            description: split_comment
                .as_ref()
                .map(|(_, description)| description.as_ref()),
            parameters: self.path_attr.params.as_ref(),
            request_body: self.path_attr.request_body.as_ref(),
            responses: self.path_attr.responses.as_ref(),
            security: self.path_attr.security.as_ref(),
        };
        let operation = as_tokens_or_diagnostics!(&operation);

        let mut tags = self.path_attr.tags.clone();
        match self.path_attr.tag.as_ref() {
            Some(tag) if tags.is_empty() => {
                tags.push(tag.clone());
            }
            _ => (),
        }
        let tags_list = tags.into_iter().collect::<Array<_>>();

        let impl_for = if let Some(impl_for) = &self.path_attr.impl_for {
            impl_for.clone()
        } else {
            let path_struct = format_ident!("{}{}", PATH_STRUCT_PREFIX, self.fn_name);
            tokens.extend(quote! {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                pub struct #path_struct;
            });
            path_struct
        };

        tokens.extend(quote! {
            impl<'t> utoipa::__dev::Tags<'t> for #impl_for {
                fn tags() -> Vec<&'t str> {
                    #tags_list.into()
                }
            }
            impl utoipa::Path for #impl_for {
                fn path() -> String {
                    #path_with_context_path
                }

                fn path_item() -> utoipa::openapi::path::PathItem {
                    use utoipa::openapi::ToArray;
                    use std::iter::FromIterator;
                    utoipa::openapi::PathItem::new(
                        #path_operation,
                        #operation
                    )
                }
            }
        });

        Ok(())
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct Operation<'a> {
    operation_id: Expr,
    summary: Option<&'a String>,
    description: Option<&'a [String]>,
    deprecated: &'a Option<bool>,
    parameters: &'a Vec<Parameter<'a>>,
    request_body: Option<&'a RequestBody<'a>>,
    responses: &'a Vec<Response<'a>>,
    security: Option<&'a Array<'a, SecurityRequirementsAttr>>,
}

impl ToTokensDiagnostics for Operation<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) -> Result<(), Diagnostics> {
        tokens.extend(quote! { utoipa::openapi::path::OperationBuilder::new() });

        if let Some(request_body) = self.request_body {
            let request_body = as_tokens_or_diagnostics!(request_body);
            tokens.extend(quote! {
                .request_body(Some(#request_body))
            })
        }

        let responses = Responses(self.responses);
        let responses = as_tokens_or_diagnostics!(&responses);
        tokens.extend(quote! {
            .responses(#responses)
        });
        if let Some(security_requirements) = self.security {
            tokens.extend(quote! {
                .securities(Some(#security_requirements))
            })
        }
        let operation_id = &self.operation_id;
        tokens.extend(quote_spanned! { operation_id.span() =>
            .operation_id(Some(#operation_id))
        });

        if let Some(deprecated) = self.deprecated.map(Into::<Deprecated>::into) {
            tokens.extend(quote!( .deprecated(Some(#deprecated))))
        }

        if let Some(summary) = self.summary {
            tokens.extend(quote! {
                .summary(Some(#summary))
            })
        }

        if let Some(description) = self.description {
            let description = description.join("\n\n");

            if !description.is_empty() {
                tokens.extend(quote! {
                    .description(Some(#description))
                })
            }
        }

        for parameter in self.parameters {
            parameter.to_tokens(tokens)?;
        }

        Ok(())
    }
}

/// Represents either `ref("...")` or `Type` that can be optionally inlined with `inline(Type)`.
#[cfg_attr(feature = "debug", derive(Debug))]
enum PathType<'p> {
    Ref(String),
    MediaType(InlineType<'p>),
    InlineSchema(TokenStream2, Type),
}

impl Parse for PathType<'_> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        let is_ref = if (fork.parse::<Option<Token![ref]>>()?).is_some() {
            fork.peek(Paren)
        } else {
            false
        };

        if is_ref {
            input.parse::<Token![ref]>()?;
            let ref_stream;
            parenthesized!(ref_stream in input);
            Ok(Self::Ref(ref_stream.parse::<LitStr>()?.value()))
        } else {
            Ok(Self::MediaType(input.parse()?))
        }
    }
}

// inline(syn::Type) | syn::Type
#[cfg_attr(feature = "debug", derive(Debug))]
struct InlineType<'i> {
    ty: Cow<'i, Type>,
    is_inline: bool,
}

impl InlineType<'_> {
    /// Get's the underlying [`syn::Type`] as [`TypeTree`].
    fn as_type_tree(&self) -> Result<TypeTree, Diagnostics> {
        TypeTree::from_type(&self.ty)
    }
}

impl Parse for InlineType<'_> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        let is_inline = if let Some(ident) = fork.parse::<Option<Ident>>()? {
            ident == "inline" && fork.peek(Paren)
        } else {
            false
        };

        let ty = if is_inline {
            input.parse::<Ident>()?;
            let inlined;
            parenthesized!(inlined in input);

            inlined.parse::<Type>()?
        } else {
            input.parse::<Type>()?
        };

        Ok(InlineType {
            ty: Cow::Owned(ty),
            is_inline,
        })
    }
}

pub trait PathTypeTree {
    /// Resolve default content type based on current [`Type`].
    fn get_default_content_type(&self) -> &str;

    #[allow(unused)]
    /// Check whether [`TypeTree`] an option
    fn is_option(&self) -> bool;

    /// Check whether [`TypeTree`] is a Vec, slice, array or other supported array type
    fn is_array(&self) -> bool;
}

impl PathTypeTree for TypeTree<'_> {
    /// Resolve default content type based on current [`Type`].
    fn get_default_content_type(&self) -> &'static str {
        if self.is_array()
            && self
                .children
                .as_ref()
                .map(|children| {
                    children
                        .iter()
                        .flat_map(|child| &child.path)
                        .any(|path| SchemaType(path).is_byte())
                })
                .unwrap_or(false)
        {
            "application/octet-stream"
        } else if self
            .path
            .as_ref()
            .map(|path| SchemaType(path.deref()))
            .map(|schema_type| schema_type.is_primitive())
            .unwrap_or(false)
        {
            "text/plain"
        } else {
            "application/json"
        }
    }

    /// Check whether [`TypeTree`] an option
    fn is_option(&self) -> bool {
        matches!(self.generic_type, Some(GenericType::Option))
    }

    /// Check whether [`TypeTree`] is a Vec, slice, array or other supported array type
    fn is_array(&self) -> bool {
        match self.generic_type {
            Some(GenericType::Vec | GenericType::Set) => true,
            Some(_) => self
                .children
                .as_ref()
                .unwrap()
                .iter()
                .any(|child| child.is_array()),
            None => false,
        }
    }
}

mod parse {
    use syn::parse::ParseStream;
    use syn::punctuated::Punctuated;
    use syn::token::{Bracket, Comma};
    use syn::{bracketed, Result};

    use crate::path::example::Example;
    use crate::{parse_utils, AnyValue};

    #[inline]
    pub(super) fn description(input: ParseStream) -> Result<parse_utils::Value> {
        parse_utils::parse_next_literal_str_or_expr(input)
    }

    #[inline]
    pub(super) fn content_type(input: ParseStream) -> Result<Vec<parse_utils::Value>> {
        parse_utils::parse_next(input, || {
            let look_content_type = input.lookahead1();
            if look_content_type.peek(Bracket) {
                let content_types;
                bracketed!(content_types in input);
                Ok(
                    Punctuated::<parse_utils::Value, Comma>::parse_terminated(&content_types)?
                        .into_iter()
                        .collect(),
                )
            } else {
                Ok(vec![input.parse::<parse_utils::Value>()?])
            }
        })
    }

    #[inline]
    pub(super) fn example(input: ParseStream) -> Result<AnyValue> {
        parse_utils::parse_next(input, || AnyValue::parse_lit_str_or_json(input))
    }

    #[inline]
    pub(super) fn examples(input: ParseStream) -> Result<Punctuated<Example, Comma>> {
        parse_utils::parse_punctuated_within_parenthesis(input)
    }
}
