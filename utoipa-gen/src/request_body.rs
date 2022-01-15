use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseBuffer},
    token::{Bracket, Paren},
    LitBool, LitStr, Token,
};

use crate::{
    component_type::{ComponentFormat, ComponentType},
    MediaType, Required,
};

/// Parsed information related to requst body of path.
///
/// Supported configuration options:
///   * **content** Request body content object type. Can also be array e.g. `content = [String]`.
///   * **required** Defines is request body mandatory. Supports also short form e.g. `required`
///     without the `= bool` suffix.
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
///    request_body = (content = String, required = true, description = "foobar", content_type = "text/xml"),
/// )]
///
///  ```
///  The `required` attribute could be rewritten like so without the `= bool` suffix.
///```text
/// #[utoipa::path(
///    request_body = (content = String, required, description = "foobar", content_type = "text/xml"),
/// )]
/// ```
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
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RequestBodyAttr {
    content: MediaType,
    content_type: Option<String>,
    required: Option<bool>,
    description: Option<String>,
}

impl Parse for RequestBodyAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let parse_lit_str = |group: &ParseBuffer| -> String {
            if group.peek(Token![=]) {
                group.parse::<Token![=]>().unwrap();
            }
            group.parse::<LitStr>().unwrap().value()
        };

        let lookahead = input.lookahead1();
        if lookahead.peek(Paren) {
            let group;
            parenthesized!(group in input);

            let mut request_body_attr = RequestBodyAttr::default();
            loop {
                let ident = group.parse::<Ident>().unwrap();
                let name = &*ident.to_string();

                match name {
                    "content" => {
                        if group.peek(Token![=]) {
                            group.parse::<Token![=]>().unwrap();
                        }

                        request_body_attr.content = group.parse::<MediaType>().unwrap();
                    }
                    "content_type" => request_body_attr.content_type = Some(parse_lit_str(&group)),
                    "required" => {
                        // support assign form as: required = bool
                        if group.peek(Token![=]) && group.peek2(LitBool) {
                            group.parse::<Token![=]>().unwrap();

                            request_body_attr.required = Some(group.parse::<LitBool>().unwrap().value());
                        } else {
                            // quick form as: required
                            request_body_attr.required = Some(true);
                        }
                    }
                    "description" => request_body_attr.description = Some(parse_lit_str(&group)),
                    _ => return Err(group.error(format!("unexpedted attribute: {}, expected values: content, content_type, required, description", &name)))
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
                content: input.parse().unwrap(),
                content_type: None,
                description: None,
                required: None,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for RequestBodyAttr {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // TODO refactor component type & format to its own type
        let body_type = self.content.ty.as_ref().unwrap();
        let component_type = ComponentType(body_type);

        let mut component = if component_type.is_primitive() {
            let mut component = quote! {
                utoipa::openapi::Property::new(
                    #component_type
                )
            };

            let format = ComponentFormat(body_type);
            if format.is_known_format() {
                component.extend(quote! {
                    .with_format(#format)
                })
            }

            component
        } else {
            let name = &*body_type.to_string();

            quote! {
                utoipa::openapi::Ref::from_component_name(#name)
            }
        };

        if self.content.is_array {
            component.extend(quote! {
                .to_array()
            });
        }

        let content_type = if let Some(ref content_type) = self.content_type {
            content_type
        } else if component_type.is_primitive() {
            "text/plain"
        } else {
            "application/json"
        };

        tokens.extend(quote! {
            utoipa::openapi::request_body::RequestBody::new()
                .with_content(#content_type, #component)
        });

        if let Some(required) = self.required {
            let required: Required = required.into();
            tokens.extend(quote! {
                .with_required(#required)
            })
        }

        if let Some(ref description) = self.description {
            tokens.extend(quote! {
                .with_description(#description)
            })
        }
    }
}
