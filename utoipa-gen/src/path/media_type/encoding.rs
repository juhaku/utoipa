use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, Error, Token};

use crate::parse_utils;

// (content_type = "...", explode = true, allow_reserved = false,)
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Encoding {
    pub(super) content_type: Option<parse_utils::LitStrOrExpr>,
    // pub(super) headers: BTreeMap<String, Header>,
    // pub(super) style: Option<ParameterStyle>,
    pub(super) explode: Option<bool>,
    pub(super) allow_reserved: Option<bool>,
    // pub(super) extensions: Option<Extensions>,
}

impl Parse for Encoding {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);

        let mut encoding = Encoding::default();

        while !content.is_empty() {
            let ident = content.parse::<Ident>()?;
            let attribute_name = &*ident.to_string();
            match attribute_name {
                "content_type" => {
                    encoding.content_type = Some(
                        parse_utils::parse_next_literal_str_or_expr(&content)?
                    )
                }
                // "headers" => {}
                // "style" => {}
                "explode" => {
                    encoding.explode = Some(
                        parse_utils::parse_bool_or_true(&content)?
                    )
                }
                "allow_reserved" => {
                    encoding.allow_reserved = Some(
                        parse_utils::parse_bool_or_true(&content)?
                    )
                }
                // "extensions"  => {}
                _ => {
                    return Err(
                        Error::new(
                            ident.span(),
                            format!("unexpected attribute: {attribute_name}, expected one of: content_type, explode, allow_reserved")
                        )
                    )
                }
            }

            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(encoding)
    }
}

impl ToTokens for Encoding {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let content_type = self
            .content_type
            .as_ref()
            .map(|content_type| quote!(.content_type(Some(#content_type))));
        let explode = self
            .explode
            .as_ref()
            .map(|value| quote!(.explode(Some(#value))));
        let allow_reserved = self
            .allow_reserved
            .as_ref()
            .map(|allow_reserved| quote!(.allow_reserved(Some(#allow_reserved))));

        tokens.extend(quote! {
            utoipa::openapi::encoding::EncodingBuilder::new()
                #content_type
                #explode
                #allow_reserved
        })
    }
}
