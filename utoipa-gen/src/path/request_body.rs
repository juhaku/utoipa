use proc_macro2::{Ident, TokenStream as TokenStream2};
use proc_macro_error::ResultExt;
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::Parse,
    token::{Bracket, Paren},
    Error, Token,
};

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
pub struct RequestBodyAttr {
    content: Option<Type>,
    content_type: Option<String>,
    description: Option<String>,
}

impl Parse for RequestBodyAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Paren) {
            let group;
            parenthesized!(group in input);

            let mut request_body_attr = RequestBodyAttr::default();
            loop {
                let ident = group
                    .parse::<Ident>()
                    .expect_or_abort("unparseable RequestBodyAttr, expected identifer");
                let name = &*ident.to_string();

                match name {
                    "content" => {
                        request_body_attr.content = Some(parse_utils::parse_next(&group, || {
                            group.parse::<Type>().unwrap()
                        }));
                    }
                    "content_type" => {
                        request_body_attr.content_type = Some(parse_utils::parse_next_lit_str(
                            &group,
                            "unparseable content_type, expected literal string",
                        ))
                    }
                    "description" => {
                        request_body_attr.description = Some(parse_utils::parse_next_lit_str(
                            &group,
                            "unparseable description, expected literal string",
                        ))
                    }
                    _ => {
                        return Err(Error::new(
                            ident.span(),
                            format!(
                                "unexpected identifer: {}, expected any of: content, content_type, description",
                                &name
                            ),
                        ))
                    }
                }

                if group.peek(Token![,]) {
                    group.parse::<Token![,]>().unwrap();
                }
                if group.is_empty() {
                    break;
                }
            }

            Ok(request_body_attr)
        } else if lookahead.peek(Bracket) || lookahead.peek(syn::Ident) {
            Ok(RequestBodyAttr {
                content: Some(input.parse().unwrap()),
                content_type: None,
                description: None,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl ContentTypeResolver for RequestBodyAttr {}

impl ToTokens for RequestBodyAttr {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if let Some(ref body_type) = self.content {
            let property = Property::new(body_type.is_array, &body_type.ty);

            let content_type =
                self.resolve_content_type(self.content_type.as_ref(), &property.component_type);
            let required: Required = (!body_type.is_option).into();

            tokens.extend(quote! {
                utoipa::openapi::request_body::RequestBody::new()
                    .with_content(#content_type, #property)
                    .with_required(#required)
            });
        }

        if let Some(ref description) = self.description {
            tokens.extend(quote! {
                .with_description(#description)
            })
        }
    }
}
