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

use crate::{parse_utils, AnyValue, Type};

use super::{property::Property, ContentTypeResolver};

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Response<'r> {
    /// A type that implements `utoipa::IntoResponses`.
    IntoResponses(ExprPath),
    /// The tuple definition of a response.
    Tuple(ResponseTuple<'r>),
}

impl Parse for Response<'_> {
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
pub struct ResponseTuple<'r> {
    status_code: String,
    inner: Option<ResponseTupleInner<'r>>,
}

const RESPONSE_INCOMPATIBLE_ATTRIBUTES_MSG: &str =
    "The `response` attribute may only be used in conjunction with the `status` attribute";

impl<'r> ResponseTuple<'r> {
    // This will error if the `response` attribute has already been set
    fn as_value(&mut self, span: Span) -> syn::Result<&mut ResponseValue<'r>> {
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
    fn set_ref_type(&mut self, span: Span, ty: Type<'r>) -> syn::Result<()> {
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
    Ref(Type<'r>),
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ResponseValue<'r> {
    description: String,
    response_type: Option<Type<'r>>,
    content_type: Option<Vec<String>>,
    headers: Vec<Header<'r>>,
    example: Option<AnyValue>,
}

impl Parse for ResponseTuple<'_> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE_MESSAGE: &str = "unexpected attribute, expected any of: status, description, body, content_type, headers, response";
        const VALID_STATUS_RANGES: &[&str] = &["default", "1XX", "2XX", "3XX", "4XX", "5XX"];

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
                    response.status_code = parse_utils::parse_next(input, || {
                        let lookahead = input.lookahead1();
                        if lookahead.peek(LitInt) {
                            input.parse::<LitInt>()?.base10_parse()
                        } else if lookahead.peek(LitStr) {
                            let value = input.parse::<LitStr>()?.value();
                            if !VALID_STATUS_RANGES.contains(&value.as_str()) {
                                return Err(Error::new(
                                    input.span(),
                                    format!(
                                        "Invalid status range, expected one of: {}",
                                        VALID_STATUS_RANGES.join(", ")
                                    ),
                                ));
                            }
                            Ok(value)
                        } else {
                            Err(lookahead.error())
                        }
                    })?
                }
                "description" => {
                    response.as_value(input.span())?.description =
                        parse_utils::parse_next_literal_str(input)?;
                }
                "body" => {
                    response.as_value(input.span())?.response_type =
                        Some(parse_utils::parse_next(input, || input.parse::<Type>())?);
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
                "response" => {
                    response.set_ref_type(
                        input.span(),
                        parse_utils::parse_next(input, || input.parse::<Type>())?,
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

impl ToTokens for ResponseTuple<'_> {
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

                if let Some(response_type) = &val.response_type {
                    let property = Property::new(response_type);

                    let mut content = quote! {
                        utoipa::openapi::ContentBuilder::new().schema(#property)
                    };

                    if let Some(ref example) = val.example {
                        content.extend(quote! {
                            .example(Some(#example))
                        })
                    }

                    if let Some(content_types) = val.content_type.as_ref() {
                        content_types.iter().for_each(|content_type| {
                            tokens.extend(quote! {
                                .content(#content_type, #content.build())
                            })
                        })
                    } else {
                        let default_type = self.resolve_content_type(None, &property.schema_type());
                        tokens.extend(quote! {
                            .content(#default_type, #content.build())
                        });
                    }
                }

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

impl ContentTypeResolver for ResponseTuple<'_> {}

pub struct Responses<'a>(pub &'a [Response<'a>]);

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
struct Header<'h> {
    name: String,
    value_type: Option<Type<'h>>,
    description: Option<String>,
}

impl Parse for Header<'_> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut header = Header {
            name: input.parse::<LitStr>()?.value(),
            ..Default::default()
        };

        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;

            header.value_type = Some(input.parse::<Type>().map_err(|error| {
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

impl ToTokens for Header<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if let Some(header_type) = &self.value_type {
            // header property with custom type
            let header_type_property = Property::new(header_type);

            tokens.extend(quote! {
                utoipa::openapi::HeaderBuilder::new().schema(#header_type_property)
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
