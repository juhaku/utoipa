use std::borrow::Cow;
use std::ops::Deref;
use std::{io::Error, str::FromStr};

use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{parenthesized, parse::Parse, Token};
use syn::{Expr, ExprLit, Lit, LitStr};

use crate::component::{features::attributes::Extensions, ComponentSchema, GenericType, TypeTree};
use crate::{
    as_tokens_or_diagnostics, parse_utils, Deprecated, Diagnostics, OptionExt, ToTokensDiagnostics,
};
use crate::{schema_type::SchemaType, security_requirement::SecurityRequirementsAttr, Array};

use self::response::Response;
use self::{parameter::Parameter, request_body::RequestBodyAttr, response::Responses};

pub mod example;
pub mod handler;
pub mod media_type;
pub mod parameter;
mod request_body;
pub mod response;
mod status;

const PATH_STRUCT_PREFIX: &str = "__path_";

#[inline]
pub fn format_path_ident(fn_name: Cow<'_, Ident>) -> Cow<'_, Ident> {
    Cow::Owned(quote::format_ident!(
        "{PATH_STRUCT_PREFIX}{}",
        fn_name.as_ref()
    ))
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct PathAttr<'p> {
    methods: Vec<HttpMethod>,
    request_body: Option<RequestBodyAttr<'p>>,
    responses: Vec<Response<'p>>,
    pub(super) path: Option<parse_utils::LitStrOrExpr>,
    operation_id: Option<Expr>,
    tag: Option<parse_utils::LitStrOrExpr>,
    tags: Vec<parse_utils::LitStrOrExpr>,
    params: Vec<Parameter<'p>>,
    security: Option<Array<'p, SecurityRequirementsAttr>>,
    context_path: Option<parse_utils::LitStrOrExpr>,
    impl_for: Option<Ident>,
    description: Option<parse_utils::LitStrOrExpr>,
    summary: Option<parse_utils::LitStrOrExpr>,
    extensions: Option<Extensions>,
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
    pub fn update_request_body(&mut self, schema: Option<crate::ext::ExtSchema<'p>>) {
        use self::media_type::Schema;
        if self.request_body.is_none() {
            if let Some(schema) = schema {
                self.request_body = Some(RequestBodyAttr::from_schema(Schema::Ext(schema)));
            }
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
        const EXPECTED_ATTRIBUTE_MESSAGE: &str = "unexpected identifier, expected any of: method, get, post, put, delete, options, head, patch, trace, operation_id, path, request_body, responses, params, tag, security, context_path, description, summary";
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
                "method" => {
                    path_attr.methods =
                        parse_utils::parse_parethesized_terminated::<HttpMethod, Comma>(input)?
                            .into_iter()
                            .collect()
                }
                "operation_id" => {
                    path_attr.operation_id =
                        Some(parse_utils::parse_next(input, || Expr::parse(input))?);
                }
                "path" => {
                    path_attr.path = Some(parse_utils::parse_next_literal_str_or_expr(input)?);
                }
                "request_body" => {
                    path_attr.request_body = Some(input.parse::<RequestBodyAttr>()?);
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
                        Punctuated::<parse_utils::LitStrOrExpr, Token![,]>::parse_terminated(&tags)
                    })?
                    .into_iter()
                    .collect::<Vec<_>>();
                }
                "security" => {
                    let security;
                    parenthesized!(security in input);
                    path_attr.security = Some(parse_utils::parse_groups_collect(&security)?)
                }
                "context_path" => {
                    path_attr.context_path =
                        Some(parse_utils::parse_next_literal_str_or_expr(input)?)
                }
                "impl_for" => {
                    path_attr.impl_for =
                        Some(parse_utils::parse_next(input, || input.parse::<Ident>())?);
                }
                "description" => {
                    path_attr.description =
                        Some(parse_utils::parse_next_literal_str_or_expr(input)?)
                }
                "summary" => {
                    path_attr.summary = Some(parse_utils::parse_next_literal_str_or_expr(input)?)
                }
                "extensions" => {
                    path_attr.extensions = Some(input.parse::<Extensions>()?);
                }
                _ => {
                    if let Some(path_operation) =
                        attribute_name.parse::<HttpMethod>().into_iter().next()
                    {
                        path_attr.methods = vec![path_operation]
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

/// Path operation HTTP method
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Options,
    Head,
    Patch,
    Trace,
}

impl Parse for HttpMethod {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let method = input
            .parse::<Ident>()
            .map_err(|error| syn::Error::new(error.span(), HttpMethod::ERROR_MESSAGE))?;

        method
            .to_string()
            .parse::<HttpMethod>()
            .map_err(|_| syn::Error::new(method.span(), HttpMethod::ERROR_MESSAGE))
    }
}

impl HttpMethod {
    const ERROR_MESSAGE: &'static str = "unexpected http method, expected one of: get, post, put, delete, options, head, patch, trace";
    /// Create path operation from ident
    ///
    /// Ident must have value of http request type as lower case string such as `get`.
    #[cfg(any(feature = "actix_extras", feature = "rocket_extras"))]
    pub fn from_ident(ident: &Ident) -> Result<Self, Diagnostics> {
        let name = &*ident.to_string();
        name
            .parse::<HttpMethod>()
            .map_err(|error| {
                let mut diagnostics = Diagnostics::with_span(ident.span(), error.to_string());
                if name == "connect" {
                    diagnostics = diagnostics.note("HTTP method `CONNET` is not supported by OpenAPI spec <https://spec.openapis.org/oas/latest.html#path-item-object>");
                }

                diagnostics
            })
    }
}

impl FromStr for HttpMethod {
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
                HttpMethod::ERROR_MESSAGE,
            )),
        }
    }
}

impl ToTokens for HttpMethod {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let path_item_type = match self {
            Self::Get => quote! { utoipa::openapi::HttpMethod::Get },
            Self::Post => quote! { utoipa::openapi::HttpMethod::Post },
            Self::Put => quote! { utoipa::openapi::HttpMethod::Put },
            Self::Delete => quote! { utoipa::openapi::HttpMethod::Delete },
            Self::Options => quote! { utoipa::openapi::HttpMethod::Options },
            Self::Head => quote! { utoipa::openapi::HttpMethod::Head },
            Self::Patch => quote! { utoipa::openapi::HttpMethod::Patch },
            Self::Trace => quote! { utoipa::openapi::HttpMethod::Trace },
        };

        tokens.extend(path_item_type);
    }
}
pub struct Path<'p> {
    path_attr: PathAttr<'p>,
    fn_ident: &'p Ident,
    ext_methods: Vec<HttpMethod>,
    path: Option<String>,
    doc_comments: Option<Vec<String>>,
    deprecated: bool,
}

impl<'p> Path<'p> {
    pub fn new(path_attr: PathAttr<'p>, fn_ident: &'p Ident) -> Self {
        Self {
            path_attr,
            fn_ident,
            ext_methods: Vec::new(),
            path: None,
            doc_comments: None,
            deprecated: false,
        }
    }

    pub fn ext_methods(mut self, methods: Option<Vec<HttpMethod>>) -> Self {
        self.ext_methods = methods.unwrap_or_default();

        self
    }

    pub fn path(mut self, path: Option<String>) -> Self {
        self.path = path;

        self
    }

    pub fn doc_comments(mut self, doc_comments: Vec<String>) -> Self {
        self.doc_comments = Some(doc_comments);

        self
    }

    pub fn deprecated(mut self, deprecated: bool) -> Self {
        self.deprecated = deprecated;

        self
    }
}

impl<'p> ToTokensDiagnostics for Path<'p> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), Diagnostics> {
        let fn_name = &*self.fn_ident.to_string();
        let operation_id = self
            .path_attr
            .operation_id
            .clone()
            .or(Some(
                ExprLit {
                    attrs: vec![],
                    lit: Lit::Str(LitStr::new(fn_name, Span::call_site())),
                }
                .into(),
            ))
            .ok_or_else(|| {
                Diagnostics::new("operation id is not defined for path")
                    .help(format!(
                        "Try to define it in #[utoipa::path(operation_id = {})]",
                        &fn_name
                    ))
                    .help("Did you define the #[utoipa::path(...)] over function?")
            })?;

        let methods = if !self.path_attr.methods.is_empty() {
            &self.path_attr.methods
        } else {
            &self.ext_methods
        };
        if methods.is_empty() {
            let diagnostics = || {
                Diagnostics::new("path operation(s) is not defined for path")
                    .help("Did you forget to define it, e.g. #[utoipa::path(get, ...)]")
                    .help("Or perhaps #[utoipa::path(method(head, get), ...)]")
            };

            #[cfg(any(feature = "actix_extras", feature = "rocket_extras"))]
            {
                return Err(diagnostics().help(
                    "Did you forget to define operation path attribute macro e.g #[get(...)]",
                ));
            }

            #[cfg(not(any(feature = "actix_extras", feature = "rocket_extras")))]
            return Err(diagnostics());
        }

        let method_operations = methods.iter().collect::<Array<_>>();

        let path = self
            .path_attr
            .path
            .as_ref()
            .map(|path| path.to_token_stream())
            .or(self.path.as_ref().map(|path| path.to_token_stream()))
            .ok_or_else(|| {
                let diagnostics = || {
                    Diagnostics::new("path is not defined for #[utoipa::path(...)]").help(
                        r#"Did you forget to define it in #[utoipa::path(..., path = "...")]"#,
                    )
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
                        #context_path,
                        #path
                    )
                };
                context_path_tokens
            })
            .unwrap_or_else(|| {
                quote! {
                    String::from(#path)
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

        let summary = self
            .path_attr
            .summary
            .as_ref()
            .map(Summary::Value)
            .or_else(|| {
                split_comment
                    .as_ref()
                    .map(|(summary, _)| Summary::Str(summary))
            });

        let description = self
            .path_attr
            .description
            .as_ref()
            .map(Description::Value)
            .or_else(|| {
                split_comment
                    .as_ref()
                    .map(|(_, description)| Description::Vec(description))
            });

        let operation: Operation = Operation {
            deprecated: self.deprecated,
            operation_id,
            summary,
            description,
            parameters: self.path_attr.params.as_ref(),
            request_body: self.path_attr.request_body.as_ref(),
            responses: self.path_attr.responses.as_ref(),
            security: self.path_attr.security.as_ref(),
            extensions: self.path_attr.extensions.as_ref(),
        };
        let operation = as_tokens_or_diagnostics!(&operation);

        fn to_schema_references(
            mut schemas: TokenStream2,
            (is_inline, component_schema): (bool, ComponentSchema),
        ) -> TokenStream2 {
            for reference in component_schema.schema_references {
                let name = &reference.name;
                let tokens = &reference.tokens;
                let references = &reference.references;

                #[cfg(feature = "config")]
                let should_collect_schema = (matches!(
                    crate::CONFIG.schema_collect,
                    utoipa_config::SchemaCollect::NonInlined
                ) && !is_inline)
                    || matches!(
                        crate::CONFIG.schema_collect,
                        utoipa_config::SchemaCollect::All
                    );
                #[cfg(not(feature = "config"))]
                let should_collect_schema = !is_inline;
                if should_collect_schema {
                    schemas.extend(quote!( schemas.push((#name, #tokens)); ));
                }
                schemas.extend(quote!( #references; ));
            }

            schemas
        }

        let response_schemas = self
            .path_attr
            .responses
            .iter()
            .map(|response| response.get_component_schemas())
            .collect::<Result<Vec<_>, Diagnostics>>()?
            .into_iter()
            .flatten()
            .fold(TokenStream2::new(), to_schema_references);

        let schemas = self
            .path_attr
            .request_body
            .as_ref()
            .map_try(|request_body| request_body.get_component_schemas())?
            .into_iter()
            .flatten()
            .fold(TokenStream2::new(), to_schema_references);

        let mut tags = self.path_attr.tags.clone();
        if let Some(tag) = self.path_attr.tag.as_ref() {
            // if defined tag is the first before the additional tags
            tags.insert(0, tag.clone());
        }
        let tags_list = tags.into_iter().collect::<Array<_>>();

        let impl_for = if let Some(impl_for) = &self.path_attr.impl_for {
            Cow::Borrowed(impl_for)
        } else {
            let path_struct = format_path_ident(Cow::Borrowed(self.fn_ident));

            tokens.extend(quote! {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                #[derive(Clone)]
                pub struct #path_struct;
            });

            #[cfg(feature = "actix_extras")]
            {
                // Add supporting passthrough implementations only if actix-web service config
                // is implemented and no impl_for has been defined
                if self.path_attr.impl_for.is_none() && !self.ext_methods.is_empty() {
                    let fn_ident = self.fn_ident;
                    tokens.extend(quote! {
                        impl ::actix_web::dev::HttpServiceFactory for #path_struct {
                            fn register(self, __config: &mut actix_web::dev::AppService) {
                                ::actix_web::dev::HttpServiceFactory::register(#fn_ident, __config);
                            }
                        }
                        impl<'t> utoipa::__dev::Tags<'t> for #fn_ident {
                            fn tags() -> Vec<&'t str> {
                                #path_struct::tags()
                            }
                        }
                        impl utoipa::Path for #fn_ident {
                            fn path() -> String {
                                #path_struct::path()
                            }

                            fn methods() -> Vec<utoipa::openapi::path::HttpMethod> {
                                #path_struct::methods()
                            }

                            fn operation() -> utoipa::openapi::path::Operation {
                                #path_struct::operation()
                            }
                        }

                        impl utoipa::__dev::SchemaReferences for #fn_ident {
                            fn schemas(schemas: &mut Vec<(String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>) {
                                <#path_struct as utoipa::__dev::SchemaReferences>::schemas(schemas);
                            }
                        }
                    })
                }
            }

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

                fn methods() -> Vec<utoipa::openapi::path::HttpMethod> {
                    #method_operations.into()
                }

                fn operation() -> utoipa::openapi::path::Operation {
                    use utoipa::openapi::ToArray;
                    use std::iter::FromIterator;
                    #operation.into()
                }
            }

            impl utoipa::__dev::SchemaReferences for #impl_for {
                fn schemas(schemas: &mut Vec<(String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>) {
                    #schemas
                    #response_schemas
                }
            }

        });

        Ok(())
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct Operation<'a> {
    operation_id: Expr,
    summary: Option<Summary<'a>>,
    description: Option<Description<'a>>,
    deprecated: bool,
    parameters: &'a Vec<Parameter<'a>>,
    request_body: Option<&'a RequestBodyAttr<'a>>,
    responses: &'a Vec<Response<'a>>,
    security: Option<&'a Array<'a, SecurityRequirementsAttr>>,
    extensions: Option<&'a Extensions>,
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

        if self.deprecated {
            let deprecated: Deprecated = self.deprecated.into();
            tokens.extend(quote!( .deprecated(Some(#deprecated))))
        }

        if let Some(summary) = &self.summary {
            summary.to_tokens(tokens);
        }

        if let Some(description) = &self.description {
            description.to_tokens(tokens);
        }

        for parameter in self.parameters {
            parameter.to_tokens(tokens)?;
        }

        if let Some(extensions) = self.extensions {
            tokens.extend(quote! { .extensions(Some(#extensions)) })
        }

        Ok(())
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
enum Description<'a> {
    Value(&'a parse_utils::LitStrOrExpr),
    Vec(&'a [String]),
}

impl ToTokens for Description<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Value(value) => tokens.extend(quote! {
                .description(Some(#value))
            }),
            Self::Vec(vec) => {
                let description = vec.join("\n\n");

                if !description.is_empty() {
                    tokens.extend(quote! {
                        .description(Some(#description))
                    })
                }
            }
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
enum Summary<'a> {
    Value(&'a parse_utils::LitStrOrExpr),
    Str(&'a str),
}

impl ToTokens for Summary<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Value(value) => tokens.extend(quote! {
                .summary(Some(#value))
            }),
            Self::Str(str) if !str.is_empty() => tokens.extend(quote! {
                .summary(Some(#str))
            }),
            _ => (),
        }
    }
}

pub trait PathTypeTree {
    /// Resolve default content type based on current [`Type`].
    fn get_default_content_type(&self) -> Cow<'static, str>;

    /// Check whether [`TypeTree`] is a Vec, slice, array or other supported array type
    fn is_array(&self) -> bool;
}

impl<'p> PathTypeTree for TypeTree<'p> {
    /// Resolve default content type based on current [`Type`].
    fn get_default_content_type(&self) -> Cow<'static, str> {
        if self.is_array()
            && self
                .children
                .as_ref()
                .map(|children| {
                    children
                        .iter()
                        .flat_map(|child| child.path.as_ref().zip(Some(child.is_option())))
                        .any(|(path, nullable)| {
                            SchemaType {
                                path: Cow::Borrowed(path),
                                nullable,
                            }
                            .is_byte()
                        })
                })
                .unwrap_or(false)
        {
            Cow::Borrowed("application/octet-stream")
        } else if self
            .path
            .as_ref()
            .map(|path| SchemaType {
                path: Cow::Borrowed(path.deref()),
                nullable: self.is_option(),
            })
            .map(|schema_type| schema_type.is_primitive())
            .unwrap_or(false)
        {
            Cow::Borrowed("text/plain")
        } else {
            Cow::Borrowed("application/json")
        }
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
    use syn::token::Comma;
    use syn::Result;

    use crate::path::example::Example;
    use crate::{parse_utils, AnyValue};

    #[inline]
    pub(super) fn description(input: ParseStream) -> Result<parse_utils::LitStrOrExpr> {
        parse_utils::parse_next_literal_str_or_expr(input)
    }

    #[inline]
    pub(super) fn example(input: ParseStream) -> Result<AnyValue> {
        parse_utils::parse_next(input, || AnyValue::parse_lit_str_or_json(input))
    }

    #[inline]
    pub(super) fn examples(input: ParseStream) -> Result<Punctuated<Example, Comma>> {
        parse_utils::parse_comma_separated_within_parenthesis(input)
    }
}
