use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Generics;
use syn::{parenthesized, parse::Parse, token::Paren, Error, Token};

use crate::component::features::attributes::Inline;
use crate::component::{ComponentSchema, Container};
use crate::{parse_utils, AnyValue, Array, Diagnostics, Required, ToTokensDiagnostics};

use super::example::Example;
use super::{parse, PathType, PathTypeTree};

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum RequestBody<'r> {
    Parsed(RequestBodyAttr<'r>),
    #[cfg(any(
        feature = "actix_extras",
        feature = "rocket_extras",
        feature = "axum_extras"
    ))]
    Ext(crate::ext::RequestBody<'r>),
}

impl ToTokensDiagnostics for RequestBody<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) -> Result<(), Diagnostics> {
        match self {
            Self::Parsed(parsed) => ToTokensDiagnostics::to_tokens(parsed, tokens)?,
            #[cfg(any(
                feature = "actix_extras",
                feature = "rocket_extras",
                feature = "axum_extras"
            ))]
            Self::Ext(ext) => ToTokensDiagnostics::to_tokens(ext, tokens)?,
        };

        Ok(())
    }
}

/// Parsed information related to request body of path.
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
    content: Option<PathType<'r>>,
    content_type: Vec<parse_utils::Value>,
    description: Option<parse_utils::Value>,
    example: Option<AnyValue>,
    examples: Option<Punctuated<Example, Comma>>,
}

impl Parse for RequestBodyAttr<'_> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE_MESSAGE: &str =
            "unexpected attribute, expected any of: content, content_type, description, examples";
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
                                        "unexpected token, expected type such as String, {error}",
                                    ),
                                )
                            })?,
                        );
                    }
                    "content_type" => {
                        request_body_attr.content_type = parse::content_type(&group)?;
                    }
                    "description" => {
                        request_body_attr.description = Some(parse::description(&group)?);
                    }
                    "example" => {
                        request_body_attr.example = Some(parse::example(&group)?);
                    }
                    "examples" => {
                        request_body_attr.examples = Some(parse::examples(&group)?);
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
                        format!("unexpected token, expected type such as String, {error}"),
                    )
                })?),
                ..Default::default()
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokensDiagnostics for RequestBodyAttr<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) -> Result<(), Diagnostics> {
        if let Some(body_type) = &self.content {
            let media_type_schema = match body_type {
                PathType::Ref(ref_type) => quote! {
                    utoipa::openapi::schema::Ref::new(#ref_type)
                },
                PathType::MediaType(body_type) => {
                    let type_tree = body_type.as_type_tree()?;
                    ComponentSchema::new(crate::component::ComponentSchemaProps {
                        type_tree: &type_tree,
                        features: Some(vec![Inline::from(body_type.is_inline).into()]),
                        description: None,
                        deprecated: None,
                        container: &Container {
                            ident: &Ident::new("empty_request_body", Span::call_site()),
                            generics: &Generics::default(),
                        },
                    })?
                    .to_token_stream()
                }
                PathType::InlineSchema(schema, _) => schema.to_token_stream(),
            };
            let mut content = quote! {
                utoipa::openapi::content::ContentBuilder::new()
                    .schema(#media_type_schema)
            };

            if let Some(ref example) = self.example {
                content.extend(quote! {
                    .example(Some(#example))
                })
            }
            if let Some(ref examples) = self.examples {
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

            match body_type {
                PathType::Ref(_) => {
                    tokens.extend(quote! {
                        utoipa::openapi::request_body::RequestBodyBuilder::new()
                            .content("application/json", #content.build())
                    });
                }
                PathType::MediaType(body_type) => {
                    let type_tree = body_type.as_type_tree()?;
                    let required: Required = (!type_tree.is_option()).into();
                    let content_types = if self.content_type.is_empty() {
                        let content_type = type_tree.get_default_content_type();
                        vec![quote!(#content_type)]
                    } else {
                        self.content_type
                            .iter()
                            .map(|content_type| content_type.to_token_stream())
                            .collect()
                    };

                    tokens.extend(quote! {
                        utoipa::openapi::request_body::RequestBodyBuilder::new()
                            .required(Some(#required))
                    });

                    for content_type in content_types {
                        tokens.extend(quote! {
                            .content(#content_type, #content.build())
                        });
                    }
                }
                PathType::InlineSchema(_, _) => {
                    unreachable!("PathType::InlineSchema is not implemented for RequestBodyAttr");
                }
            }
        }

        if let Some(ref description) = self.description {
            tokens.extend(quote! {
                .description(Some(#description))
            })
        }

        tokens.extend(quote! { .build() });

        Ok(())
    }
}
