use std::{slice::Iter, vec::IntoIter};

use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, emit_warning, OptionExt, ResultExt};
use quote::{quote_spanned, ToTokens};
use syn::{parse::Parse, Attribute, ExprPath, Lit, Token};

use crate::doc_comment::CommentAttributes;

const COMPONENT_ATTRIBUTE_TYPE: &str = "component";

#[cfg_attr(feature = "debug", derive(Debug))]
/// AttributeType is parsed representation of `#[component(...)]` attribute values of Component derive.
pub(crate) enum AttributeType {
    Default(String, Span),
    Format(String, Span),
    Example(String, Span),
}

impl ToTokens for AttributeType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Default(value, span) => {
                if value.contains("::") {
                    let method = syn::parse_str::<ExprPath>(value)
                        .map(|method| {
                            quote_spanned! {*span=>
                                #method().to_string()
                            }
                        })
                        .unwrap_or_else(|_| {
                            quote_spanned! {*span=>
                                #value
                            }
                        });

                    tokens.extend(method);
                } else {
                    tokens.extend(quote_spanned! {*span=>
                        #value
                    })
                }
            }
            Self::Example(value, span) => tokens.extend(quote_spanned! {*span=>
                #value
            }),
            Self::Format(value, span) => {
                let path = syn::parse_str::<ExprPath>(&format!("utoipa::openapi::{}", value))
                    .expect_or_abort(&format!("parse path failed: {}", value));

                tokens.extend(quote_spanned! {*span=>
                    #path
                })
            }
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
/// Wrapper struct for containing parsed [`enum@AttributeType`]s. It implements custom parser
/// to parse `#[component(...)]` attribute content of Component derive macro.
pub(crate) struct ComponentAttribute(pub(crate) Vec<AttributeType>);

impl Parse for ComponentAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attribute = ComponentAttribute(vec![]);
        loop {
            let ident = input
                .parse::<Ident>()
                .expect_or_abort("Expected Ident as first item in the TokenStream");
            let name = &*ident.to_string();
            input.parse::<Token![=]>().unwrap_or_abort();

            let lookahead = input.lookahead1();
            if lookahead.peek(Lit) {
                let lit = &input.parse::<Lit>().unwrap_or_abort();
                let lit_value = match lit {
                    Lit::Bool(bool) => bool.value().to_string(),
                    Lit::Byte(byte) => byte.value().to_string(),
                    Lit::ByteStr(byte_str) => {
                        String::from_utf8(byte_str.value()).unwrap_or_else(|_| {
                            abort!(
                                input.span(),
                                format!("Unparseable utf8 content in: {}", name)
                            )
                        })
                    }
                    Lit::Char(char) => char.value().to_string(),
                    Lit::Float(float) => float.base10_digits().to_string(),
                    Lit::Int(int) => int.base10_digits().to_string(),
                    Lit::Str(str) => str.value(),
                    Lit::Verbatim(_) => {
                        abort!(
                            input.span(),
                            format!("Unparseable literal in field: {}", name)
                        )
                    }
                };

                match name {
                    "default" => attribute
                        .0
                        .push(AttributeType::Default(lit_value, lit.span())),
                    "format" => attribute
                        .0
                        .push(AttributeType::Format(lit_value, lit.span())),
                    "example" => attribute
                        .0
                        .push(AttributeType::Example(lit_value, lit.span())),
                    _ => emit_warning!(
                        input.span(),
                        format!("Unsupported attribute field: {}", name)
                    ),
                };
            } else {
                return Err(lookahead.error());
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>().unwrap_or_abort();
            }
            if input.is_empty() {
                break;
            }
        }

        Ok(attribute)
    }
}

impl IntoIterator for ComponentAttribute {
    type Item = AttributeType;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

fn is_component_attribute(attribute: &&Attribute) -> bool {
    *attribute
        .path
        .get_ident()
        .expect_or_abort("Expected component attribute with one path segment")
        == COMPONENT_ATTRIBUTE_TYPE
}

/// Parses [`struct@ComponentAttribute`] from given syn::Attributes using only first matching attribute.
pub(crate) fn parse_component_attribute(attributes: &[Attribute]) -> Option<ComponentAttribute> {
    attributes
        .iter()
        .find(is_component_attribute)
        .map(|component_attribute| {
            component_attribute
                .parse_args::<ComponentAttribute>()
                .unwrap_or_abort()
        })
}
