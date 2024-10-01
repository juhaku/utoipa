use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{Error, Generics, Ident, LitStr, Token};

use crate::component::features::attributes::Inline;
use crate::component::{ComponentSchema, Container, TypeTree};
use crate::path::media_type::ParsedType;
use crate::{parse_utils, Diagnostics, ToTokensDiagnostics};

/// Parsed representation of response header defined in `#[utoipa::path(..)]` attribute.
///
/// Supported configuration format is `("x-my-header-name" = type, description = "optional description of header")`.
/// The `= type` and the `description = ".."` are optional configurations thus so the same configuration
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
/// Example with multiple headers with default values.
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
pub struct Header {
    pub name: String,
    value_type: Option<ParsedType<'static>>,
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
                    format!("unexpected token, expected type such as String, {error}"),
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
                        format!("unexpected attribute, expected: description, {error}"),
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

impl ToTokensDiagnostics for Header {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        if let Some(header_type) = &self.value_type {
            // header property with custom type
            let type_tree = TypeTree::from_type(header_type.ty.as_ref())?;

            let media_type_schema = ComponentSchema::new(crate::component::ComponentSchemaProps {
                type_tree: &type_tree,
                features: vec![Inline::from(header_type.is_inline).into()],
                description: None,
                container: &Container {
                    generics: &Generics::default(),
                },
            })?
            .to_token_stream();

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

        tokens.extend(quote! { .build() });

        Ok(())
    }
}

#[inline]
pub fn headers(input: ParseStream) -> syn::Result<Vec<Header>> {
    let headers;
    syn::parenthesized!(headers in input);

    parse_utils::parse_groups_collect(&headers)
}
