use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{parenthesized, parse::Parse, token::Paren, Error, Token};

use crate::{parse_utils, Required, Type};

use super::{property::Property, ContentTypeResolver};

/// Parsed information related to requst body of path.
///
/// Supported configuration options:
///   * **content** Request body content object type. Can also be array e.g. `content = [String]`.
///   * **content_type** Defines the actual content mime type of a request body such as `application/json`.
///     If not provided really rough guess logic is used. Basically all primitive types are treated as `text/plain`
///     and Object types are expected to be `application/json` by default.
///   * **description** Additional description for request body content type.
/// # Examples
///
/// Request body in path with all supported info. Where content type is treated as a String and expected
/// to be xml.
/// ```text
/// #[utoipa::path(
///    request_body = (content = String, description = "foobar", content_type = "text/xml"),
/// )]
///
/// It is also possible to provide the request body type simply by providing only the content object type.
/// ```text
/// #[utoipa::path(
///    request_body = Foo,
/// )]
/// ```
///
/// Or the request body content can also be an array as well by surrounding it with brackets `[..]`.
/// ```text
/// #[utoipa::path(
///    request_body = [Foo],
/// )]
/// ```
///
/// To define optional request body just wrap the type in `Option<type>`.
/// ```text
/// #[utoipa::path(
///    request_body = Option<[Foo]>,
/// )]
/// ```
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RequestBodyAttr<'r> {
    content: Option<Type<'r>>,
    content_type: Option<String>,
    description: Option<String>,
}

impl Parse for RequestBodyAttr<'_> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE_MESSAGE: &str =
            "unexpected attribute, expected any of: content, content_type, description";
        let lookahead = input.lookahead1();

        if lookahead.peek(Paren) {
            let group;
            parenthesized!(group in input);

            let mut request_body_attr = RequestBodyAttr::default();
            while !group.is_empty() {
                let ident = group
                    .parse::<Ident>()
                    .map_err(|error| Error::new(error.span(), EXPECTED_ATTRIBUTE_MESSAGE))?;
                let attribute_name = &*ident.to_string();

                match attribute_name {
                    "content" => {
                        request_body_attr.content = Some(
                            parse_utils::parse_next(&group, || group.parse()).map_err(|error| {
                                Error::new(
                                    error.span(),
                                    format!(
                                        "unexpected token, expected type such as String, {}",
                                        error
                                    ),
                                )
                            })?,
                        );
                    }
                    "content_type" => {
                        request_body_attr.content_type =
                            Some(parse_utils::parse_next_literal_str(&group)?)
                    }
                    "description" => {
                        request_body_attr.description =
                            Some(parse_utils::parse_next_literal_str(&group)?)
                    }
                    _ => return Err(Error::new(ident.span(), EXPECTED_ATTRIBUTE_MESSAGE)),
                }

                if !group.is_empty() {
                    group.parse::<Token![,]>()?;
                }
            }

            Ok(request_body_attr)
        } else if lookahead.peek(Token![=]) {
            input.parse::<Token![=]>()?;

            Ok(RequestBodyAttr {
                content: Some(input.parse().map_err(|error| {
                    Error::new(
                        error.span(),
                        format!("unexpected token, expected type such as String, {}", error),
                    )
                })?),
                content_type: None,
                description: None,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl ContentTypeResolver for RequestBodyAttr<'_> {}

impl ToTokens for RequestBodyAttr<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if let Some(body_type) = &self.content {
            let property = Property::new(body_type);

            let content_type =
                self.resolve_content_type(self.content_type.as_ref(), &property.schema_type());
            let required: Required = (!body_type.is_option).into();

            tokens.extend(quote! {
                utoipa::openapi::request_body::RequestBodyBuilder::new()
                    .content(#content_type, utoipa::openapi::Content::new(#property))
                    .required(Some(#required))
            });
        }

        if let Some(ref description) = self.description {
            tokens.extend(quote! {
                .description(Some(#description))
            })
        }

        tokens.extend(quote! { .build() })
    }
}
