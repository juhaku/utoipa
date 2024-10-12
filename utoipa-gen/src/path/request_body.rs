use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::{parse::Parse, Error, Token};

use crate::component::ComponentSchema;
use crate::{parse_utils, Diagnostics, Required, ToTokensDiagnostics};

use super::media_type::{MediaTypeAttr, Schema};
use super::parse;

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
///    request_body(content = String, description = "foobar", content_type = "text/xml"),
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
///
/// request_body(
///     description = "This is request body",
///     content_type = "content/type",
///     content = Schema,
///     example = ...,
///     examples(..., ...),
///     encoding(...)
/// )
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RequestBodyAttr<'r> {
    description: Option<parse_utils::LitStrOrExpr>,
    content: Vec<MediaTypeAttr<'r>>,
}

impl<'r> RequestBodyAttr<'r> {
    fn new() -> Self {
        Self {
            description: Default::default(),
            content: vec![MediaTypeAttr::default()],
        }
    }

    #[cfg(any(
        feature = "actix_extras",
        feature = "rocket_extras",
        feature = "axum_extras"
    ))]
    pub fn from_schema(schema: Schema<'r>) -> RequestBodyAttr<'r> {
        Self {
            content: vec![MediaTypeAttr {
                schema,
                ..Default::default()
            }],
            ..Self::new()
        }
    }

    pub fn get_component_schemas(
        &self,
    ) -> Result<impl Iterator<Item = (bool, ComponentSchema)>, Diagnostics> {
        Ok(self
            .content
            .iter()
            .map(
                |media_type| match media_type.schema.get_component_schema() {
                    Ok(component_schema) => {
                        Ok(Some(media_type.schema.is_inline()).zip(component_schema))
                    }
                    Err(error) => Err(error),
                },
            )
            .collect::<Result<Vec<_>, Diagnostics>>()?
            .into_iter()
            .flatten())
    }
}

impl Parse for RequestBodyAttr<'_> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE_MESSAGE: &str =
            "unexpected attribute, expected any of: content, content_type, description, examples, example";
        let lookahead = input.lookahead1();

        if lookahead.peek(Paren) {
            let group;
            syn::parenthesized!(group in input);

            let mut is_content_group = false;
            let mut request_body_attr = RequestBodyAttr::new();
            while !group.is_empty() {
                let ident = group
                    .parse::<Ident>()
                    .map_err(|error| Error::new(error.span(), EXPECTED_ATTRIBUTE_MESSAGE))?;
                let attribute_name = &*ident.to_string();

                match attribute_name {
                    "content" => {
                        if group.peek(Token![=]) {
                            group.parse::<Token![=]>()?;
                            let schema = MediaTypeAttr::parse_schema(&group)?;
                            if let Some(media_type) = request_body_attr.content.get_mut(0) {
                                media_type.schema = Schema::Default(schema);
                            }
                        } else if group.peek(Paren) {
                            is_content_group = true;
                            fn group_parser<'a>(
                                input: ParseStream,
                            ) -> syn::Result<MediaTypeAttr<'a>> {
                                let buf;
                                syn::parenthesized!(buf in input);
                                buf.call(MediaTypeAttr::parse)
                            }

                            let media_type =
                                parse_utils::parse_comma_separated_within_parethesis_with(
                                    &group,
                                    group_parser,
                                )?
                                .into_iter()
                                .collect::<Vec<_>>();

                            request_body_attr.content = media_type;
                        } else {
                            return Err(Error::new(ident.span(), "unexpected content format, expected either `content = schema` or `content(...)`"));
                        }
                    }
                    "content_type" => {
                        if is_content_group {
                            return Err(Error::new(ident.span(), "cannot set `content_type` when content(...) is defined in group form"));
                        }
                        let content_type = parse_utils::parse_next(&group, || {
                            parse_utils::LitStrOrExpr::parse(&group)
                        }).map_err(|error| Error::new(error.span(),
                                format!(r#"invalid content_type, must be literal string or expression, e.g. "application/json", {error} "#)
                            ))?;

                        if let Some(media_type) = request_body_attr.content.get_mut(0) {
                            media_type.content_type = Some(content_type);
                        }
                    }
                    "description" => {
                        request_body_attr.description = Some(parse::description(&group)?);
                    }
                    _ => {
                        MediaTypeAttr::parse_named_attributes(
                            request_body_attr
                                .content
                                .get_mut(0)
                                .expect("parse request body named attributes must have media type"),
                            &group,
                            &ident,
                        )?;
                    }
                }

                if !group.is_empty() {
                    group.parse::<Token![,]>()?;
                }
            }

            Ok(request_body_attr)
        } else if lookahead.peek(Token![=]) {
            input.parse::<Token![=]>()?;

            let media_type = MediaTypeAttr {
                schema: Schema::Default(MediaTypeAttr::parse_schema(input)?),
                content_type: None,
                example: None,
                examples: Punctuated::default(),
            };

            Ok(RequestBodyAttr {
                content: vec![media_type],
                description: None,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokensDiagnostics for RequestBodyAttr<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        let media_types = self
            .content
            .iter()
            .map(|media_type| {
                let default_content_type_result = media_type.schema.get_default_content_type();
                let type_tree = media_type.schema.get_type_tree();

                match (default_content_type_result, type_tree) {
                    (Ok(content_type), Ok(type_tree)) => Ok((content_type, media_type, type_tree)),
                    (Err(diagnostics), _) => Err(diagnostics),
                    (_, Err(diagnostics)) => Err(diagnostics),
                }
            })
            .collect::<Result<Vec<_>, Diagnostics>>()?;

        let any_required = media_types.iter().any(|(_, _, type_tree)| {
            type_tree
                .as_ref()
                .map(|type_tree| !type_tree.is_option())
                .unwrap_or(false)
        });

        tokens.extend(quote! {
            utoipa::openapi::request_body::RequestBodyBuilder::new()
        });
        for (content_type, media_type, _) in media_types {
            let content_type_tokens = media_type
                .content_type
                .as_ref()
                .map(|content_type| content_type.to_token_stream())
                .unwrap_or_else(|| content_type.to_token_stream());
            let content_tokens = media_type.try_to_token_stream()?;

            tokens.extend(quote! {
                .content(#content_type_tokens, #content_tokens)
            });
        }

        if any_required {
            let required: Required = any_required.into();
            tokens.extend(quote! {
                .required(Some(#required))
            })
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
