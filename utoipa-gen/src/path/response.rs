use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
use std::borrow::Cow;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Attribute, Error, ExprPath, LitInt, LitStr, Token, TypePath,
};

use crate::{
    component::ComponentSchema, features::attributes::Extensions, parse_utils,
    path::media_type::Schema, AnyValue, Diagnostics, ToTokensDiagnostics,
};

use self::{header::Header, link::LinkTuple};

use super::{
    example::Example,
    media_type::{DefaultSchema, MediaTypeAttr, ParsedType},
    parse,
    status::STATUS_CODES,
};

pub mod derive;
mod header;
pub mod link;

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Response<'r> {
    /// A type that implements `utoipa::IntoResponses`.
    IntoResponses(Cow<'r, TypePath>),
    /// The tuple definition of a response.
    Tuple(ResponseTuple<'r>),
}

impl Parse for Response<'_> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.fork().parse::<ExprPath>().is_ok() {
            Ok(Self::IntoResponses(Cow::Owned(input.parse::<TypePath>()?)))
        } else {
            let response;
            parenthesized!(response in input);
            Ok(Self::Tuple(response.parse()?))
        }
    }
}

impl Response<'_> {
    pub fn get_component_schemas(
        &self,
    ) -> Result<impl Iterator<Item = (bool, ComponentSchema)>, Diagnostics> {
        match self {
            Self::Tuple(tuple) => match &tuple.inner {
                // Only tuple type will have `ComponentSchema`s as of now
                Some(ResponseTupleInner::Value(value)) => {
                    Ok(ResponseComponentSchemaIter::Iter(Box::new(
                        value
                            .content
                            .iter()
                            .map(
                                |media_type| match media_type.schema.get_component_schema() {
                                    Ok(component_schema) => {
                                        Ok(Some(media_type.schema.is_inline())
                                            .zip(component_schema))
                                    }
                                    Err(error) => Err(error),
                                },
                            )
                            .collect::<Result<Vec<_>, Diagnostics>>()?
                            .into_iter()
                            .flatten(),
                    )))
                }
                _ => Ok(ResponseComponentSchemaIter::Empty),
            },
            Self::IntoResponses(_) => Ok(ResponseComponentSchemaIter::Empty),
        }
    }
}

pub enum ResponseComponentSchemaIter<'a, T> {
    Iter(Box<dyn std::iter::Iterator<Item = T> + 'a>),
    Empty,
}

impl<'a, T> Iterator for ResponseComponentSchemaIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Iter(iter) => iter.next(),
            Self::Empty => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Iter(iter) => iter.size_hint(),
            Self::Empty => (0, None),
        }
    }
}

/// Parsed representation of response attributes from `#[utoipa::path]` attribute.
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ResponseTuple<'r> {
    status_code: ResponseStatus,
    inner: Option<ResponseTupleInner<'r>>,
}

const RESPONSE_INCOMPATIBLE_ATTRIBUTES_MSG: &str =
    "The `response` attribute may only be used in conjunction with the `status` attribute";

impl<'r> ResponseTuple<'r> {
    /// Set as `ResponseValue` the content. This will fail if `response` attribute is already
    /// defined.
    fn set_as_value<F: FnOnce(&mut ResponseValue) -> syn::Result<()>>(
        &mut self,
        ident: &Ident,
        attribute: &str,
        op: F,
    ) -> syn::Result<()> {
        match &mut self.inner {
            Some(ResponseTupleInner::Value(value)) => {
                op(value)?;
            }
            Some(ResponseTupleInner::Ref(_)) => {
                return Err(Error::new(ident.span(), format!("Cannot use `{attribute}` in conjunction with `response`. The `response` attribute can only be used in conjunction with `status` attribute.")));
            }
            None => {
                let mut value = ResponseValue {
                    content: vec![MediaTypeAttr::default()],
                    ..Default::default()
                };
                op(&mut value)?;
                self.inner = Some(ResponseTupleInner::Value(value))
            }
        };

        Ok(())
    }

    // Use with the `response` attribute, this will fail if an incompatible attribute has already been set
    fn set_ref_type(&mut self, span: Span, ty: ParsedType<'r>) -> syn::Result<()> {
        match &mut self.inner {
            None => self.inner = Some(ResponseTupleInner::Ref(ty)),
            Some(ResponseTupleInner::Ref(r)) => *r = ty,
            Some(ResponseTupleInner::Value(_)) => {
                return Err(Error::new(span, RESPONSE_INCOMPATIBLE_ATTRIBUTES_MSG))
            }
        }
        Ok(())
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
enum ResponseTupleInner<'r> {
    Value(ResponseValue<'r>),
    Ref(ParsedType<'r>),
}

impl Parse for ResponseTuple<'_> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTES: &str =
            "status, description, body, content_type, headers, example, examples, response";

        let mut response = ResponseTuple::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>().map_err(|error| {
                Error::new(
                    error.span(),
                    format!(
                        "unexpected attribute, expected any of: {EXPECTED_ATTRIBUTES}, {error}"
                    ),
                )
            })?;
            let name = &*ident.to_string();
            match name {
                "status" => {
                    response.status_code =
                        parse_utils::parse_next(input, || input.parse::<ResponseStatus>())?;
                }
                "response" => {
                    response.set_ref_type(
                        input.span(),
                        parse_utils::parse_next(input, || input.parse())?,
                    )?;
                }
                _ => {
                    response.set_as_value(&ident, name, |value| {
                        value.parse_named_attributes(input, &ident)
                    })?;
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(response)
    }
}

impl<'r> From<ResponseValue<'r>> for ResponseTuple<'r> {
    fn from(value: ResponseValue<'r>) -> Self {
        ResponseTuple {
            inner: Some(ResponseTupleInner::Value(value)),
            ..Default::default()
        }
    }
}

impl<'r> From<(ResponseStatus, ResponseValue<'r>)> for ResponseTuple<'r> {
    fn from((status_code, response_value): (ResponseStatus, ResponseValue<'r>)) -> Self {
        ResponseTuple {
            inner: Some(ResponseTupleInner::Value(response_value)),
            status_code,
        }
    }
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ResponseValue<'r> {
    description: parse_utils::LitStrOrExpr,
    headers: Vec<Header>,
    links: Punctuated<LinkTuple, Comma>,
    content: Vec<MediaTypeAttr<'r>>,
    is_content_group: bool,
    extensions: Option<Extensions>,
}

impl Parse for ResponseValue<'_> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut response_value = ResponseValue::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>().map_err(|error| {
                Error::new(
                    error.span(),
                    format!(
                        "unexpected attribute, expected any of: {expected_attributes}, {error}",
                        expected_attributes = ResponseValue::EXPECTED_ATTRIBUTES
                    ),
                )
            })?;
            response_value.parse_named_attributes(input, &ident)?;

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(response_value)
    }
}

impl<'r> ResponseValue<'r> {
    const EXPECTED_ATTRIBUTES: &'static str =
        "description, body, content_type, headers, example, examples";

    fn parse_named_attributes(&mut self, input: ParseStream, attribute: &Ident) -> syn::Result<()> {
        let attribute_name = &*attribute.to_string();

        match attribute_name {
            "description" => {
                self.description = parse::description(input)?;
            }
            "body" => {
                if self.is_content_group {
                    return Err(Error::new(
                        attribute.span(),
                        "cannot set `body` when content(...) is defined in group form",
                    ));
                }

                let schema = parse_utils::parse_next(input, || MediaTypeAttr::parse_schema(input))?;
                if let Some(media_type) = self.content.get_mut(0) {
                    media_type.schema = Schema::Default(schema);
                }
            }
            "content_type" => {
                if self.is_content_group {
                    return Err(Error::new(
                        attribute.span(),
                        "cannot set `content_type` when content(...) is defined in group form",
                    ));
                }
                let content_type = parse_utils::parse_next(input, || {
                    parse_utils::LitStrOrExpr::parse(input)
                }).map_err(|error| Error::new(error.span(),
                        format!(r#"invalid content_type, must be literal string or expression, e.g. "application/json", {error} "#)
                    ))?;

                if let Some(media_type) = self.content.get_mut(0) {
                    media_type.content_type = Some(content_type);
                }
            }
            "headers" => {
                self.headers = header::headers(input)?;
            }
            "content" => {
                self.is_content_group = true;
                fn group_parser<'a>(input: ParseStream) -> syn::Result<MediaTypeAttr<'a>> {
                    let buf;
                    syn::parenthesized!(buf in input);
                    buf.call(MediaTypeAttr::parse)
                }

                let content =
                    parse_utils::parse_comma_separated_within_parethesis_with(input, group_parser)?
                        .into_iter()
                        .collect::<Vec<_>>();

                self.content = content;
            }
            "links" => {
                self.links = parse_utils::parse_comma_separated_within_parenthesis(input)?;
            }
            "extensions" => {
                self.extensions = Some(input.parse::<Extensions>()?);
            }
            _ => {
                self.content
                    .get_mut(0)
                    .expect(
                        "parse named attributes response value must have one media type by default",
                    )
                    .parse_named_attributes(input, attribute)?;
            }
        }
        Ok(())
    }

    fn from_schema<S: Into<Schema<'r>>>(schema: S, description: parse_utils::LitStrOrExpr) -> Self {
        let media_type = MediaTypeAttr {
            schema: schema.into(),
            ..Default::default()
        };

        Self {
            description,
            content: vec![media_type],
            ..Default::default()
        }
    }

    fn from_derive_to_response_value<S: Into<Schema<'r>>>(
        derive_value: DeriveToResponseValue,
        schema: S,
        description: parse_utils::LitStrOrExpr,
    ) -> Self {
        let media_type = MediaTypeAttr {
            content_type: derive_value.content_type,
            schema: schema.into(),
            example: derive_value.example.map(|(example, _)| example),
            examples: derive_value
                .examples
                .map(|(examples, _)| examples)
                .unwrap_or_default(),
            ..MediaTypeAttr::default()
        };

        Self {
            description: if derive_value.description.is_empty_litstr()
                && !description.is_empty_litstr()
            {
                description
            } else {
                derive_value.description
            },
            headers: derive_value.headers,
            content: vec![media_type],
            ..Default::default()
        }
    }

    fn from_derive_into_responses_value<S: Into<Schema<'r>>>(
        response_value: DeriveIntoResponsesValue,
        schema: S,
        description: parse_utils::LitStrOrExpr,
    ) -> Self {
        let media_type = MediaTypeAttr {
            content_type: response_value.content_type,
            schema: schema.into(),
            example: response_value.example.map(|(example, _)| example),
            examples: response_value
                .examples
                .map(|(examples, _)| examples)
                .unwrap_or_default(),
            ..MediaTypeAttr::default()
        };

        ResponseValue {
            description: if response_value.description.is_empty_litstr()
                && !description.is_empty_litstr()
            {
                description
            } else {
                response_value.description
            },
            headers: response_value.headers,
            content: vec![media_type],
            ..Default::default()
        }
    }
}

impl ToTokensDiagnostics for ResponseTuple<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) -> Result<(), Diagnostics> {
        match self.inner.as_ref() {
            Some(ResponseTupleInner::Ref(res)) => {
                let path = &res.ty;
                if res.is_inline {
                    tokens.extend(quote_spanned! {path.span()=>
                        <#path as utoipa::ToResponse>::response().1
                    });
                } else {
                    tokens.extend(quote! {
                        utoipa::openapi::Ref::from_response_name(<#path as utoipa::ToResponse>::response().0)
                    });
                }
            }
            Some(ResponseTupleInner::Value(value)) => {
                let description = &value.description;
                tokens.extend(quote! {
                    utoipa::openapi::ResponseBuilder::new().description(#description)
                });

                for media_type in value.content.iter().filter(|media_type| {
                    !(matches!(media_type.schema, Schema::Default(DefaultSchema::None))
                        && media_type.content_type.is_none())
                }) {
                    let default_content_type = media_type.schema.get_default_content_type()?;

                    let content_type_tokens = media_type
                        .content_type
                        .as_ref()
                        .map(|content_type| content_type.to_token_stream())
                        .unwrap_or_else(|| default_content_type.to_token_stream());
                    let content_tokens = media_type.try_to_token_stream()?;

                    tokens.extend(quote! {
                        .content(#content_type_tokens, #content_tokens)
                    });
                }

                for header in &value.headers {
                    let name = &header.name;
                    let header = crate::as_tokens_or_diagnostics!(header);
                    tokens.extend(quote! {
                        .header(#name, #header)
                    })
                }

                for LinkTuple(name, link) in &value.links {
                    tokens.extend(quote! {
                        .link(#name, #link)
                    })
                }
                if let Some(ref extensions) = value.extensions {
                    tokens.extend(quote! {
                        .extensions(Some(#extensions))
                    });
                }

                tokens.extend(quote! { .build() });
            }
            None => tokens.extend(quote! {
                utoipa::openapi::ResponseBuilder::new().description("")
            }),
        }

        Ok(())
    }
}

trait DeriveResponseValue: Parse {
    fn merge_from(self, other: Self) -> Self;

    fn from_attributes(attributes: &[Attribute]) -> Result<Option<Self>, Diagnostics> {
        Ok(attributes
            .iter()
            .filter(|attribute| attribute.path().get_ident().unwrap() == "response")
            .map(|attribute| attribute.parse_args::<Self>().map_err(Diagnostics::from))
            .collect::<Result<Vec<_>, Diagnostics>>()?
            .into_iter()
            .reduce(|acc, item| acc.merge_from(item)))
    }
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
struct DeriveToResponseValue {
    content_type: Option<parse_utils::LitStrOrExpr>,
    headers: Vec<Header>,
    description: parse_utils::LitStrOrExpr,
    example: Option<(AnyValue, Ident)>,
    examples: Option<(Punctuated<Example, Comma>, Ident)>,
}

impl DeriveResponseValue for DeriveToResponseValue {
    fn merge_from(mut self, other: Self) -> Self {
        if other.content_type.is_some() {
            self.content_type = other.content_type;
        }
        if !other.headers.is_empty() {
            self.headers = other.headers;
        }
        if !other.description.is_empty_litstr() {
            self.description = other.description;
        }
        if other.example.is_some() {
            self.example = other.example;
        }
        if other.examples.is_some() {
            self.examples = other.examples;
        }

        self
    }
}

impl Parse for DeriveToResponseValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut response = DeriveToResponseValue::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "description" => {
                    response.description = parse::description(input)?;
                }
                "content_type" => {
                    response.content_type =
                        Some(parse_utils::parse_next_literal_str_or_expr(input)?);
                }
                "headers" => {
                    response.headers = header::headers(input)?;
                }
                "example" => {
                    response.example = Some((parse::example(input)?, ident));
                }
                "examples" => {
                    response.examples = Some((parse::examples(input)?, ident));
                }
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        format!("unexpected attribute: {attribute_name}, expected any of: inline, description, content_type, headers, example"),
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Comma>()?;
            }
        }

        Ok(response)
    }
}

#[derive(Default)]
struct DeriveIntoResponsesValue {
    status: ResponseStatus,
    content_type: Option<parse_utils::LitStrOrExpr>,
    headers: Vec<Header>,
    description: parse_utils::LitStrOrExpr,
    example: Option<(AnyValue, Ident)>,
    examples: Option<(Punctuated<Example, Comma>, Ident)>,
}

impl DeriveResponseValue for DeriveIntoResponsesValue {
    fn merge_from(mut self, other: Self) -> Self {
        self.status = other.status;

        if other.content_type.is_some() {
            self.content_type = other.content_type;
        }
        if !other.headers.is_empty() {
            self.headers = other.headers;
        }
        if !other.description.is_empty_litstr() {
            self.description = other.description;
        }
        if other.example.is_some() {
            self.example = other.example;
        }
        if other.examples.is_some() {
            self.examples = other.examples;
        }

        self
    }
}

impl Parse for DeriveIntoResponsesValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut response = DeriveIntoResponsesValue::default();
        const MISSING_STATUS_ERROR: &str = "missing expected `status` attribute";
        let first_span = input.span();

        let status_ident = input
            .parse::<Ident>()
            .map_err(|error| Error::new(error.span(), MISSING_STATUS_ERROR))?;

        if status_ident == "status" {
            response.status = parse_utils::parse_next(input, || input.parse::<ResponseStatus>())?;
        } else {
            return Err(Error::new(status_ident.span(), MISSING_STATUS_ERROR));
        }

        if response.status.to_token_stream().is_empty() {
            return Err(Error::new(first_span, MISSING_STATUS_ERROR));
        }

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
        }

        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "description" => {
                    response.description = parse::description(input)?;
                }
                "content_type" => {
                    response.content_type =
                        Some(parse_utils::parse_next_literal_str_or_expr(input)?);
                }
                "headers" => {
                    response.headers = header::headers(input)?;
                }
                "example" => {
                    response.example = Some((parse::example(input)?, ident));
                }
                "examples" => {
                    response.examples = Some((parse::examples(input)?, ident));
                }
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        format!("unexpected attribute: {attribute_name}, expected any of: description, content_type, headers, example, examples"),
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(response)
    }
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
struct ResponseStatus(TokenStream2);

impl Parse for ResponseStatus {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        fn parse_lit_int(input: ParseStream) -> syn::Result<Cow<'_, str>> {
            input.parse::<LitInt>()?.base10_parse().map(Cow::Owned)
        }

        fn parse_lit_str_status_range(input: ParseStream) -> syn::Result<Cow<'_, str>> {
            const VALID_STATUS_RANGES: [&str; 6] = ["default", "1XX", "2XX", "3XX", "4XX", "5XX"];

            input
                .parse::<LitStr>()
                .and_then(|lit_str| {
                    let value = lit_str.value();
                    if !VALID_STATUS_RANGES.contains(&value.as_str()) {
                        Err(Error::new(
                            value.span(),
                            format!(
                                "Invalid status range, expected one of: {}",
                                VALID_STATUS_RANGES.join(", "),
                            ),
                        ))
                    } else {
                        Ok(value)
                    }
                })
                .map(Cow::Owned)
        }

        fn parse_http_status_code(input: ParseStream) -> syn::Result<TokenStream2> {
            let http_status_path = input.parse::<ExprPath>()?;
            let last_segment = http_status_path
                .path
                .segments
                .last()
                .expect("Expected at least one segment in http StatusCode");

            STATUS_CODES
                .iter()
                .find_map(|(code, name)| {
                    if last_segment.ident == name {
                        Some(code.to_string().to_token_stream())
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    Error::new(
                        last_segment.span(),
                        format!(
                            "No associate item `{}` found for struct `http::StatusCode`",
                            last_segment.ident
                        ),
                    )
                })
        }

        let lookahead = input.lookahead1();
        if lookahead.peek(LitInt) {
            parse_lit_int(input).map(|status| Self(status.to_token_stream()))
        } else if lookahead.peek(LitStr) {
            parse_lit_str_status_range(input).map(|status| Self(status.to_token_stream()))
        } else if lookahead.peek(syn::Ident) {
            parse_http_status_code(input).map(Self)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for ResponseStatus {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.0.to_tokens(tokens);
    }
}

pub struct Responses<'a>(pub &'a [Response<'a>]);

impl ToTokensDiagnostics for Responses<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), Diagnostics> {
        tokens.extend(
            self.0
                .iter()
                .map(|response| match response {
                    Response::IntoResponses(path) => {
                        let span = path.span();
                        Ok(quote_spanned! {span =>
                            .responses_from_into_responses::<#path>()
                        })
                    }
                    Response::Tuple(response) => {
                        let code = &response.status_code;
                        let response = crate::as_tokens_or_diagnostics!(response);
                        Ok(quote! { .response(#code, #response) })
                    }
                })
                .collect::<Result<Vec<_>, Diagnostics>>()?
                .into_iter()
                .fold(
                    quote! { utoipa::openapi::ResponsesBuilder::new() },
                    |mut acc, response| {
                        response.to_tokens(&mut acc);

                        acc
                    },
                ),
        );

        tokens.extend(quote! { .build() });

        Ok(())
    }
}
