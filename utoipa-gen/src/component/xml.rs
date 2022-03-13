use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{parenthesized, parse::Parse, token::Paren, LitStr, Token};

use crate::parse_utils;

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub(super) struct XmlAttr {
    pub(super) name: Option<String>,
    pub(super) namespace: Option<String>,
    pub(super) prefix: Option<String>,
    pub(super) is_attribute: bool,
    pub(super) is_wrapped: Option<Ident>,
    pub(super) wrap_name: Option<String>,
}

impl XmlAttr {
    pub(super) fn with_wrapped(is_wrapped: Option<Ident>, wrap_name: Option<String>) -> Self {
        Self {
            is_wrapped,
            wrap_name,
            ..Default::default()
        }
    }
}

impl Parse for XmlAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut xml = XmlAttr::default();

        while !input.is_empty() {
            let attribute = input.parse::<Ident>()?;
            let attribute_name = &*attribute.to_string();

            match attribute_name {
                "name" => xml.name = Some(parse_utils::parse_next(input, || input.parse::<LitStr>())?.value()),
                "namespace" => xml.namespace = Some(parse_utils::parse_next(input, || input.parse::<LitStr>())?.value()),
                "prefix" => xml.prefix = Some(parse_utils::parse_next(input, || input.parse::<LitStr>())?.value()),
                "attribute" => xml.is_attribute = parse_utils::parse_bool_or_true(input)?,
                // wrapped or wrapped(name = "wrap_name")
                "wrapped" => {
                    if input.peek(Paren) {
                        let group;
                        parenthesized!(group in input);

                        let wrapped_attribute =group.parse::<Ident>()?;
                        if wrapped_attribute != "name" {
                            return Err(syn::Error::new(wrapped_attribute.span(), "unexpected wrapped attribute, expected: name"));
                        }
                        group.parse::<Token![=]>()?;
                        xml.wrap_name = Some(group.parse::<LitStr>()?.value());
                    }
                    xml.is_wrapped = Some(attribute);
                },
                _ => {
                    return Err(syn::Error::new(attribute.span(), &format!("unexpected attribute: {attribute_name}, expected one of: name, namespace, prefix, attribute, wrapped")))
                },
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>().unwrap();
            }
        }

        Ok(xml)
    }
}

impl ToTokens for XmlAttr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote! {
            utoipa::openapi::xml::Xml::new()
        });

        if let Some(ref name) = self.name {
            tokens.extend(quote! {
                .name(Some(#name))
            })
        }

        if let Some(ref namespace) = self.namespace {
            tokens.extend(quote! {
                .namespace(Some(#namespace))
            })
        }

        if let Some(ref prefix) = self.prefix {
            tokens.extend(quote! {
                .prefix(Some(#prefix))
            })
        }

        if self.is_attribute {
            tokens.extend(quote! {
                .attribute(Some(true))
            })
        }

        if self.is_wrapped.is_some() {
            tokens.extend(quote! {
                .wrapped(Some(true))
            });

            // if is wrapped and wrap name is defined use wrap name instead
            if let Some(ref wrap_name) = self.wrap_name {
                tokens.extend(quote! {
                    .name(Some(#wrap_name))
                })
            }
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub(super) enum Xml {
    NonSlice(XmlAttr),
    Slice { vec: XmlAttr, value: XmlAttr },
}
