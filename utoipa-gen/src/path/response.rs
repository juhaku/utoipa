use std::borrow::Cow;

use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    bracketed, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Bracket, Comma},
    Error, ExprPath, LitInt, LitStr, Token,
};

use crate::{component::TypeTree, parse_utils, AnyValue, Array};

use super::{
    example::Example, property::MediaTypeSchema, status::STATUS_CODES, InlineableType, PathTypeTree,
};

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Response {
    /// A type that implements `utoipa::IntoResponses`.
    IntoResponses(ExprPath),
    /// The tuple definition of a response.
    Tuple(ResponseTuple),
}

impl Parse for Response {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.fork().parse::<ExprPath>().is_ok() {
            Ok(Self::IntoResponses(input.parse()?))
        } else {
            let response;
            parenthesized!(response in input);
            Ok(Self::Tuple(response.parse()?))
        }
    }
}

/// Parsed representation of response attributes from `#[utoipa::path]` attribute.
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ResponseTuple {
    status_code: ResponseStatus,
    inner: Option<ResponseTupleInner>,
}

const RESPONSE_INCOMPATIBLE_ATTRIBUTES_MSG: &str =
    "The `response` attribute may only be used in conjunction with the `status` attribute";

impl ResponseTuple {
    // This will error if the `response` attribute has already been set
    fn as_value(&mut self, span: Span) -> syn::Result<&mut ResponseValue> {
        if self.inner.is_none() {
            self.inner = Some(ResponseTupleInner::Value(ResponseValue::default()));
        }
        if let ResponseTupleInner::Value(val) = self.inner.as_mut().unwrap() {
            Ok(val)
        } else {
            Err(Error::new(span, RESPONSE_INCOMPATIBLE_ATTRIBUTES_MSG))
        }
    }

    // Use with the `response` attribute, this will fail if an incompatible attribute has already been set
    fn set_ref_type(&mut self, span: Span, ty: InlineableType) -> syn::Result<()> {
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
enum ResponseTupleInner {
    Value(ResponseValue),
    Ref(InlineableType),
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ResponseValue {
    description: String,
    response_type: Option<InlineableType>,
    content_type: Option<Vec<String>>,
    headers: Vec<Header>,
    example: Option<AnyValue>,
    examples: Option<Punctuated<Example, Comma>>,
    content: Punctuated<Content, Comma>,
}

impl Parse for ResponseTuple {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE_MESSAGE: &str = "unexpected attribute, expected any of: status, description, body, content_type, headers, response";

        let mut response = ResponseTuple::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>().map_err(|error| {
                Error::new(
                    error.span(),
                    format!("{}, {}", EXPECTED_ATTRIBUTE_MESSAGE, error),
                )
            })?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "status" => {
                    response.status_code =
                        parse_utils::parse_next(input, || input.parse::<ResponseStatus>())?;
                }
                "description" => {
                    response.as_value(input.span())?.description =
                        parse_utils::parse_next_literal_str(input)?;
                }
                "body" => {
                    response.as_value(input.span())?.response_type =
                        Some(parse_utils::parse_next(input, || input.parse())?);
                }
                "content_type" => {
                    response.as_value(input.span())?.content_type =
                        Some(parse_utils::parse_next(input, || {
                            let look_content_type = input.lookahead1();
                            if look_content_type.peek(LitStr) {
                                Ok(vec![input.parse::<LitStr>()?.value()])
                            } else if look_content_type.peek(Bracket) {
                                let content_types;
                                bracketed!(content_types in input);
                                Ok(
                                    Punctuated::<LitStr, Comma>::parse_terminated(&content_types)?
                                        .into_iter()
                                        .map(|lit| lit.value())
                                        .collect(),
                                )
                            } else {
                                Err(look_content_type.error())
                            }
                        })?);
                }
                "headers" => {
                    let headers;
                    parenthesized!(headers in input);

                    response.as_value(input.span())?.headers = parse_utils::parse_groups(&headers)?;
                }
                "example" => {
                    response.as_value(input.span())?.example =
                        Some(parse_utils::parse_next(input, || {
                            AnyValue::parse_lit_str_or_json(input)
                        })?);
                }
                "examples" => {
                    response.as_value(input.span())?.examples =
                        Some(parse_utils::parse_punctuated_within_parenthesis(input)?);
                }
                "content" => {
                    response.as_value(input.span())?.content =
                        parse_utils::parse_punctuated_within_parenthesis(input)?;
                }
                "response" => {
                    response.set_ref_type(
                        input.span(),
                        parse_utils::parse_next(input, || input.parse())?,
                    )?;
                }
                _ => return Err(Error::new(ident.span(), EXPECTED_ATTRIBUTE_MESSAGE)),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        if response.inner.is_none() {
            response.inner = Some(ResponseTupleInner::Value(ResponseValue::default()))
        }

        Ok(response)
    }
}

impl ToTokens for ResponseTuple {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self.inner.as_ref().unwrap() {
            ResponseTupleInner::Ref(res) => {
                let path = &res.ty;
                tokens.extend(quote! {
                    utoipa::openapi::Ref::from_response_name(<#path as utoipa::ToResponse>::response().0)
                });
            }
            ResponseTupleInner::Value(val) => {
                let description = &val.description;
                tokens.extend(quote! {
                    utoipa::openapi::ResponseBuilder::new().description(#description)
                });

                let create_content = |type_tree: &TypeTree,
                                      is_inline: bool,
                                      example: &Option<AnyValue>,
                                      examples: &Option<Punctuated<Example, Comma>>|
                 -> TokenStream2 {
                    let media_schema_type = MediaTypeSchema {
                        type_tree,
                        is_inline,
                    };
                    let mut content = quote! { utoipa::openapi::ContentBuilder::new().schema(#media_schema_type) };

                    if let Some(ref example) = example {
                        content.extend(quote! {
                            .example(Some(#example))
                        })
                    }
                    if let Some(ref examples) = examples {
                        let examples = examples
                            .iter()
                            .map(|example| {
                                let name = &example.name;
                                quote!((#name, #example))
                            })
                            .collect::<Array<TokenStream2>>();
                        content.extend(quote!(
                            .examples_from_iter(#examples)
                        ))
                    }

                    content
                };

                if let Some(response_type) = &val.response_type {
                    let type_tree = response_type.as_type_tree();
                    let content = create_content(
                        &type_tree,
                        response_type.is_inline,
                        &val.example,
                        &val.examples,
                    );

                    if let Some(content_types) = val.content_type.as_ref() {
                        content_types.iter().for_each(|content_type| {
                            tokens.extend(quote! {
                                .content(#content_type, #content.build())
                            })
                        })
                    } else {
                        let default_type = type_tree.get_default_content_type();
                        tokens.extend(quote! {
                            .content(#default_type, #content.build())
                        });
                    }
                }

                val.content
                    .iter()
                    .map(|Content(content_type, body, example, examples)| {
                        let type_tree = body.as_type_tree();
                        let content = create_content(&type_tree, body.is_inline, example, examples);
                        (Cow::Borrowed(&**content_type), content)
                    })
                    .for_each(|(content_type, content)| {
                        tokens.extend(quote! { .content(#content_type, #content.build()) })
                    });

                val.headers.iter().for_each(|header| {
                    let name = &header.name;
                    tokens.extend(quote! {
                        .header(#name, #header)
                    })
                });

                tokens.extend(quote! { .build() });
            }
        }
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
                        &format!(
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

// content(
//   ("application/json" = Response, example = "...", examples(..., ...)),
//   ("application/json2" = Response2, example = "...", examples("...", "..."))
// )
#[cfg_attr(feature = "debug", derive(Debug))]
struct Content(
    String,
    InlineableType,
    Option<AnyValue>,
    Option<Punctuated<Example, Comma>>,
);

impl Parse for Content {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);

        let content_type = content.parse::<LitStr>()?;
        content.parse::<Token![=]>()?;
        let body = content.parse()?;
        content.parse::<Option<Comma>>()?;
        let mut example = None::<AnyValue>;
        let mut examples = None::<Punctuated<Example, Comma>>;

        while !content.is_empty() {
            let ident = content.parse::<Ident>()?;
            let attribute_name = &*ident.to_string();
            match attribute_name {
                "example" => {
                    example = Some(parse_utils::parse_next(&content, || {
                        AnyValue::parse_json(&content)
                    })?)
                }
                "examples" => {
                    examples = Some(parse_utils::parse_punctuated_within_parenthesis(&content)?)
                }
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        &format!(
                            "unexpected attribute: {ident}, expected one of: example, examples"
                        ),
                    ));
                }
            }

            if !content.is_empty() {
                content.parse::<Comma>()?;
            }
        }

        Ok(Content(content_type.value(), body, example, examples))
    }
}

pub struct Responses<'a>(pub &'a [Response]);

impl ToTokens for Responses<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.iter().fold(
            quote! { utoipa::openapi::ResponsesBuilder::new() },
            |mut acc, response| {
                match response {
                    Response::IntoResponses(path) => {
                        let span = path.span();
                        acc.extend(quote_spanned! {span =>
                            .responses_from_into_responses::<#path>()
                        })
                    }
                    Response::Tuple(response) => {
                        let code = &response.status_code;
                        acc.extend(quote! { .response(#code, #response) });
                    }
                }

                acc
            },
        ));

        tokens.extend(quote! { .build() });
    }
}

/// Parsed representation of response header defined in `#[utoipa::path(..)]` attribute.
///
/// Supported configuration format is `("x-my-header-name" = type, description = "optional description of header")`.
/// The `= type` and the `descripiton = ".."` are optional configurations thus so the same configuration
/// could be written as follows: `("x-my-header-name")`.
///
/// The `type` can be any typical type supported as a header argument such as `String, i32, u64, bool` etc.
/// and if not provided it will default to `String`.
///
/// # Examples
///
/// Example of 200 success response which does return nothing back in response body, but returns a
/// new csrf token in response headers.
/// ```text
/// #[utoipa::path(
///     ...
///     responses = [
///         (status = 200, description = "success response",
///             headers = [
///                 ("xrfs-token" = String, description = "New csrf token sent back in response header")
///             ]
///         ),
///     ]
/// )]
/// ```
///
/// Example with default values.
/// ```text
/// #[utoipa::path(
///     ...
///     responses = [
///         (status = 200, description = "success response",
///             headers = [
///                 ("xrfs-token")
///             ]
///         ),
///     ]
/// )]
/// ```
///
/// Example with multiplea headers with default values.
/// ```text
/// #[utoipa::path(
///     ...
///     responses = [
///         (status = 200, description = "success response",
///             headers = [
///                 ("xrfs-token"),
///                 ("another-header"),
///             ]
///         ),
///     ]
/// )]
/// ```
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
struct Header {
    name: String,
    value_type: Option<InlineableType>,
    description: Option<String>,
}

impl Parse for Header {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut header = Header {
            name: input.parse::<LitStr>()?.value(),
            ..Default::default()
        };

        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;

            header.value_type = Some(input.parse().map_err(|error| {
                Error::new(
                    error.span(),
                    format!("unexpected token, expected type such as String, {}", error),
                )
            })?);
        }

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
        }

        if input.peek(syn::Ident) {
            input
                .parse::<Ident>()
                .map_err(|error| {
                    Error::new(
                        error.span(),
                        format!("unexpected attribute, expected: description, {}", error),
                    )
                })
                .and_then(|ident| {
                    if ident != "description" {
                        return Err(Error::new(
                            ident.span(),
                            "unexpected attribute, expected: description",
                        ));
                    }
                    Ok(ident)
                })?;
            input.parse::<Token![=]>()?;
            header.description = Some(input.parse::<LitStr>()?.value());
        }

        Ok(header)
    }
}

impl ToTokens for Header {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if let Some(header_type) = &self.value_type {
            // header property with custom type
            let type_tree = header_type.as_type_tree();
            let media_type_schema = MediaTypeSchema {
                type_tree: &type_tree,
                is_inline: header_type.is_inline,
            };

            tokens.extend(quote! {
                utoipa::openapi::HeaderBuilder::new().schema(#media_type_schema)
            })
        } else {
            // default header (string type)
            tokens.extend(quote! {
                Into::<utoipa::openapi::HeaderBuilder>::into(utoipa::openapi::Header::default())
            })
        };

        if let Some(ref description) = self.description {
            tokens.extend(quote! {
                .description(Some(#description))
            })
        }

        tokens.extend(quote! { .build() })
    }
}
