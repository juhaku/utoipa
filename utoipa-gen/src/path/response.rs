use proc_macro2::{Group, Ident, TokenStream as TokenStream2};
use proc_macro_error::ResultExt;
use quote::{quote, ToTokens};
use syn::{
    bracketed, parse::Parse, punctuated::Punctuated, token::Comma, Error, LitInt, LitStr, Token,
};

use crate::{parse_utils, Example, Type};

use super::{property::Property, ContentTypeResolver};

/// Parsed representation of response attributes from `#[utoipa::path]` attribute.
///
/// Configuration options:
///   * **status** Http status code of the response e.g. `200`
///   * **description** Description of the response
///   * **body** Optional response body type. Can be primitive, struct or enum type and slice types are supported
///     by wrapping the type with brackets e.g. `[Foo]`
///   * **content_type** Optional content type of the response e.g. `"text/plain"`
///   * **headers** Optional response headers. See [`Header`] for detailed description and usage
///
/// Only status and description are mandatory for describing response. Responses which does not
/// define `body = type` are treated as they would not return any response back. Content type of
/// responses will be if not provided determined automatically suggesting that any primitive type such as
/// integer, string or boolean are treated as `"text/plain"` and struct types are treated as `"application/json"`.
///
/// # Examples
///
/// Minimal example example providing responses.
/// ```text
/// #[utoipa::path(
///     ...
///     responses = [
///         (status = 200, description = "success response"),
///     ]
/// )]
/// ```
///
/// Example with all supported configuration.
/// ```text
/// #[utoipa::path(
///     ...
///     responses = [
///         (status = 200, description = "success response", body = [Foo], content_type = "text/xml",
///             headers = [
///                 ("xrfs-token" = String, description = "New csrf token sent back in response header")
///             ]
///         ),
///     ]
/// )]
/// ```
///
/// Example with multiple responses.
/// ```text
/// #[utoipa::path(
///     ...
///     responses = [
///         (status = 200, description = "success response", body = [Foo]),
///         (status = 401, description = "unauthorized to access", body = UnautorizedError),
///         (status = 404, description = "foo not found", body = NotFoundError),
///     ]
/// )]
/// ```
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Response {
    status_code: i32,
    description: String,
    response_type: Option<Type>,
    content_type: Option<String>,
    headers: Vec<Header>,
    example: Option<Example>,
}

impl Parse for Response {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut response = Response::default();

        loop {
            let ident = input
                .parse::<Ident>()
                .expect_or_abort("unparseable Response expected identifier");
            let name = &*ident.to_string();

            match name {
                "status" => {
                    response.status_code = parse_utils::parse_next(input, || {
                        input
                            .parse::<LitInt>()
                            .unwrap()
                            .base10_parse()
                            .expect_or_abort("unparseable status, expected literal integer")
                    });
                }
                "description" => {
                    response.description = parse_utils::parse_next_lit_str(
                        input,
                        "unparseable description, expected literal string",
                    );
                }
                "body" => {
                    response.response_type = Some(parse_utils::parse_next(input, || {
                        input.parse::<Type>().unwrap_or_abort()
                    }));
                }
                "content_type" => {
                    response.content_type = Some(parse_utils::parse_next_lit_str(
                        input,
                        "unparseable content_type, expected literal string",
                    ));
                }
                "headers" => {
                    let groups = parse_utils::parse_next(input, || {
                        let content;
                        bracketed!(content in input);
                        Punctuated::<Group, Comma>::parse_terminated(&content)
                    })
                    .expect_or_abort("unparseable headers, expected group separated by comma (,)");

                    response.headers = groups
                        .into_iter()
                        .map(|group| syn::parse2::<Header>(group.stream()).unwrap_or_abort())
                        .collect::<Vec<_>>();
                }
                "example" => {
                    response.example = Some(parse_utils::parse_next_lit_str_or_json_example(
                        input, &ident,
                    ));
                }
                _ => {
                    let error_msg = format!(
                        "unexpected identifer: {}, expected any of: status, description, body, content_type, headers",
                        &name
                    );

                    return Err(Error::new(ident.span(), error_msg));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>().unwrap_or_abort();
            }
            if input.is_empty() {
                break;
            }
        }

        Ok(response)
    }
}

pub struct Responses<'a>(pub &'a [Response]);

impl ContentTypeResolver for Responses<'_> {}

impl ToTokens for Responses<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote! { utoipa::openapi::Responses::new() });

        self.0.iter().for_each(|response| {
            let status = response.status_code.to_string();
            let description = &response.description;

            let mut response_tokens = quote! {
                utoipa::openapi::Response::new(#description)
            };

            if let Some(ref response_body_type) = response.response_type {
                let body_type = &response_body_type.ty;

                let component = Property::new(response_body_type.is_array, body_type);

                let content_type = self.resolve_content_type(
                    response.content_type.as_ref(),
                    &component.component_type,
                );

                let mut content = quote! {
                    utoipa::openapi::Content::new(#component)
                };
                if let Some(ref example) = response.example {
                    content.extend(quote! {
                        .with_example(#example)
                    })
                }

                response_tokens.extend(quote! {
                    .with_content(#content_type, #content)
                })
            }

            response.headers.iter().for_each(|header| {
                let name = &header.name;
                let header_tokens = new_header_tokens(header);

                response_tokens.extend(quote! {
                    .with_header(#name, #header_tokens)
                })
            });

            tokens.extend(quote! {
                .with_response(#status, #response_tokens)
            });
        })
    }
}

#[inline]
fn new_header_tokens(header: &Header) -> TokenStream2 {
    let mut header_tokens = if let Some(ref header_type) = header.value_type {
        // header property with custom type
        let header_type = Property::new(header_type.is_array, &header_type.ty);

        quote! {
            utoipa::openapi::Header::new(#header_type)
        }
    } else {
        // default header (string type)
        quote! {
            utoipa::openapi::Header::default()
        }
    };

    if let Some(ref description) = header.description {
        header_tokens.extend(quote! {
            .with_description(#description)
        })
    }

    header_tokens
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
    value_type: Option<Type>,
    description: Option<String>,
}

impl Parse for Header {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut header = Header {
            name: input
                .parse::<LitStr>()
                .expect_or_abort("unexpected attribute for Header name, expected literal string")
                .value(),
            ..Default::default()
        };

        if input.peek(Token![=]) {
            input.parse::<Token![=]>().unwrap_or_abort();

            header.value_type = Some(
                input
                    .parse::<Type>()
                    .expect_or_abort("unparseable Header type, expected identifer"),
            );
        }

        if input.peek(Token![,]) {
            input.parse::<Token![,]>().unwrap_or_abort();
        }

        if input.peek(syn::Ident) {
            let description = input
                .parse::<Ident>()
                .expect_or_abort("unparseable Header description, expected identifier");

            if description == "description" {
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>().unwrap_or_abort();
                }

                let description = input
                    .parse::<LitStr>()
                    .expect_or_abort("unparseable description, expected literal string")
                    .value();
                header.description = Some(description);
            } else {
                return Err(Error::new(
                    description.span(),
                    format!(
                        "unexpected identifer: {}, expected: description",
                        description
                    ),
                ));
            }
        }

        Ok(header)
    }
}
