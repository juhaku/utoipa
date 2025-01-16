use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::ParseStream;
use syn::token::Paren;
use syn::{parse::Parse, Error, Token};

use crate::component::{features::attributes::Extensions, ComponentSchema};
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
    extensions: Option<Extensions>,
}

impl<'r> RequestBodyAttr<'r> {
    fn new() -> Self {
        Self {
            description: Default::default(),
            content: vec![MediaTypeAttr::default()],
            extensions: Default::default(),
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
            "unexpected attribute, expected any of: content, content_type, description, examples, example, encoding, extensions";
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
                    "extensions" => {
                        request_body_attr.extensions = Some(group.parse::<Extensions>()?);
                    }
                    _ => {
                        request_body_attr
                            .content
                            .get_mut(0)
                            .expect("parse request body named attributes must have media type")
                            .parse_named_attributes(&group, &ident)?;
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
                ..MediaTypeAttr::default()
            };

            Ok(RequestBodyAttr {
                content: vec![media_type],
                description: None,
                extensions: None,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokensDiagnostics for RequestBodyAttr<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        tokens.extend(quote! {
            utoipa::openapi::request_body::RequestBodyBuilder::new()
        });

        let mut any_required = false;

        for media_type in self.content.iter() {
            let content_type_tokens = match media_type.content_type.as_ref() {
                Some(ct) => ct.to_token_stream(),
                None => media_type
                    .schema
                    .get_default_content_type()?
                    .to_token_stream(),
            };

            let content_tokens = media_type.try_to_token_stream()?;

            tokens.extend(quote! {
                .content(#content_type_tokens, #content_tokens)
            });

            any_required = any_required
                || media_type
                    .schema
                    .get_type_tree()?
                    .as_ref()
                    .map(|t| !t.is_option())
                    .unwrap_or(false);
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
        if let Some(ref extensions) = self.extensions {
            tokens.extend(quote! {
                .extensions(Some(#extensions))
            });
        }

        tokens.extend(quote! { .build() });

        Ok(())
    }
}
