use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    bracketed, parenthesized,
    parse::Parse,
    punctuated::Punctuated,
    token::{Bracket, Comma},
    Error, LitInt, LitStr, Token,
};

use crate::{parse_utils, Example, Type};

use super::{property::Property, ContentTypeResolver};

/// Parsed representation of response attributes from `#[utoipa::path]` attribute.
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Response<'r> {
    status_code: i32,
    description: String,
    response_type: Option<Type<'r>>,
    content_type: Option<Vec<String>>,
    headers: Vec<Header<'r>>,
    example: Option<Example>,
}

impl Parse for Response<'_> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE_MESSAGE: &str = "unexpected attribute, expected any of: status, description, body, content_type, headers";
        let mut response = Response::default();

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
                        parse_utils::parse_next(input, || input.parse::<LitInt>())?
                            .base10_parse()?;
                }
                "description" => {
                    response.description = parse_utils::parse_next_literal_str(input)?;
                }
                "body" => {
                    response.response_type = Some(
                        parse_utils::parse_next(input, || input.parse::<Type>()).map_err(
                            |error| {
                                Error::new(
                                    ident.span(),
                                    format!(
                                        "unexpected token, expected type such as String, {}",
                                        error
                                    ),
                                )
                            },
                        )?,
                    );
                }
                "content_type" => {
                    response.content_type = Some(parse_utils::parse_next(input, || {
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

                    response.headers = parse_utils::parse_groups(&headers)?;
                }
                "example" => {
                    response.example = Some(parse_utils::parse_next_lit_str_or_json_example(
                        input, &ident,
                    ));
                }
                _ => return Err(Error::new(ident.span(), EXPECTED_ATTRIBUTE_MESSAGE)),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(response)
    }
}

impl ToTokens for Response<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let description = &self.description;
        tokens.extend(quote! {
            utoipa::openapi::ResponseBuilder::new().description(#description)
        });

        if let Some(ref body_type) = self.response_type {
            let body_ty = &body_type.ty;

            let component = Property::new(body_type.is_array, body_ty);
            let mut content = quote! {
                utoipa::openapi::ContentBuilder::new().schema(#component)
            };

            if let Some(ref example) = self.example {
                content.extend(quote! {
                    .example(Some(#example))
                })
            }

            if let Some(content_types) = self.content_type.as_ref() {
                content_types.iter().for_each(|content_type| {
                    tokens.extend(quote! {
                        .content(#content_type, #content.build())
                    })
                })
            } else {
                let default_type = self.resolve_content_type(None, &component.component_type);
                tokens.extend(quote! {
                    .content(#default_type, #content.build())
                });
            }
        }

        self.headers.iter().for_each(|header| {
            let name = &header.name;
            tokens.extend(quote! {
                .header(#name, #header)
            })
        });

        tokens.extend(quote! { .build() })
    }
}

impl ContentTypeResolver for Response<'_> {}

pub struct Responses<'a>(pub &'a [Response<'a>]);

impl ToTokens for Responses<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.0.is_empty() {
            tokens.extend(quote! { utoipa::openapi::Responses::new() })
        } else {
            let responses = self.0.iter().fold(quote! {}, |mut acc, response| {
                let code = &response.status_code.to_string();
                acc.extend(quote! { (#code, #response), });

                acc
            });

            tokens.extend(quote! {
                utoipa::openapi::Responses::from_iter([#responses])
            });
        }
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
        if let Some(ref header_type) = self.value_type {
            // header property with custom type
            let header_type = Property::new(header_type.is_array, &header_type.ty);

            tokens.extend(quote! {
                utoipa::openapi::HeaderBuilder::new().schema(#header_type)
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
