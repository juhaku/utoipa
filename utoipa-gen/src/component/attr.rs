use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseBuffer},
    Attribute, Error, ExprPath, Lit, Token,
};

use crate::{parse_utils, Deprecated};

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ComponentAttr<T>
where
    T: Sized,
{
    inner: T,
}

impl<T> AsRef<T> for ComponentAttr<T>
where
    T: Sized,
{
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Enum {
    deprecated: bool,
    default: Option<TokenStream>,
    example: Option<String>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Struct {
    deprecated: bool,
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct NamedField {
    deprecated: bool,
    example: Option<String>,
    format: Option<ExprPath>,
    default: Option<TokenStream>,
}

impl Parse for ComponentAttr<Enum> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut enum_attr = Enum::default();
        loop {
            let ident = input
                .parse::<Ident>()
                .expect_or_abort("Unparseable ComponentAttr<Enum>, expected Ident");
            let name = &*ident.to_string();

            match name {
                "deprecated" => {
                    enum_attr.deprecated = parse_utils::parse_bool_or_true(input);
                }
                "default" => {
                    enum_attr.default = Some(parse_utils::parse_next(input, || {
                        parse_default_as_token_stream(input, name)
                    }))
                }
                "example" => {
                    enum_attr.example = Some(parse_utils::parse_next(input, || {
                        parse_lit_as_string(input, name, "unparseable example, expected Literal")
                    }))
                }
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        format!(
                            "unexpected attribute: {}, expected: deprecated, default, example",
                            name
                        ),
                    ))
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>().unwrap();
            }

            if input.is_empty() {
                break;
            }
        }
        Ok(Self { inner: enum_attr })
    }
}

impl Parse for ComponentAttr<Struct> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input
            .parse::<Ident>()
            .expect_or_abort("Unparseable ComponentAttr<Struct>, expected Ident");
        let name = &*ident.to_string();

        match name {
            "deprecated" => Ok(Self {
                inner: Struct {
                    deprecated: parse_utils::parse_bool_or_true(input),
                },
            }),
            _ => Err(Error::new(
                ident.span(),
                format!("unexpected attribute: {}, expected: deprecated", name),
            )),
        }
    }
}

impl Parse for ComponentAttr<NamedField> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut field = NamedField::default();
        loop {
            let ident = input
                .parse::<Ident>()
                .expect_or_abort("Unparseable ComponentAttr<NamedField>, expected Ident");
            let name = &*ident.to_string();

            match name {
                "deprecated" => {
                    field.deprecated = parse_utils::parse_bool_or_true(input);
                }
                "example" => {
                    field.example = Some(parse_utils::parse_next(input, || {
                        parse_lit_as_string(input, name, "unparseable example, expected Literal")
                    }));
                }
                "format" => {
                    let format = parse_utils::parse_next(input, || {
                        input.parse::<ExprPath>().expect_or_abort(
                            "unparseable format expected ExprPath e.g. ComponentFormat::String",
                        )
                    });

                    if format.path.segments.first().unwrap().ident != "utoipa" {
                        let appended_path: ExprPath = syn::parse_quote!(utoipa::openapi::#format);
                        field.format = Some(appended_path);
                    } else {
                        field.format = Some(format);
                    }
                }
                "default" => {
                    field.default = Some(parse_utils::parse_next(input, || {
                        parse_default_as_token_stream(input, name)
                    }))
                }
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        format!(
                        "unexpected attribute: {}, expected: deprecated, example, format, default",
                        name
                    ),
                    ))
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>().unwrap();
            }
            if input.is_empty() {
                break;
            }
        }

        Ok(Self { inner: field })
    }
}

fn parse_lit_as_string(input: &ParseBuffer, field: &str, error_msg: &str) -> String {
    let lit = &input.parse::<Lit>().expect_or_abort(error_msg);
    match lit {
        Lit::Bool(bool) => bool.value().to_string(),
        Lit::Byte(byte) => byte.value().to_string(),
        Lit::ByteStr(byte_str) => String::from_utf8(byte_str.value()).unwrap_or_else(|_| {
            abort!(
                input.span(),
                format!("Unparseable utf8 content in: {}", &field)
            )
        }),
        Lit::Char(char) => char.value().to_string(),
        Lit::Float(float) => float.base10_digits().to_string(),
        Lit::Int(int) => int.base10_digits().to_string(),
        Lit::Str(str) => str.value(),
        Lit::Verbatim(_) => {
            abort!(
                input.span(),
                format!("Unparseable literal in field: {}", &field)
            )
        }
    }
}

fn parse_default_as_token_stream(input: &ParseBuffer, name: &str) -> TokenStream {
    if input.peek(Lit) {
        let literal = parse_lit_as_string(input, name, "unparseable default, expected Literal");
        quote_spanned! {input.span()=>
            #literal
        }
    } else {
        let method = input
            .parse::<ExprPath>()
            .expect_or_abort("unparseable default, expected Literal, or ExprPath");
        quote_spanned! {input.span()=>
            #method()
        }
    }
}

pub fn parse_component_attr<T: Sized + Parse>(attributes: &[Attribute]) -> Option<T> {
    attributes
        .iter()
        .find(|attribute| attribute.path.get_ident().unwrap() == "component")
        .map(|attribute| attribute.parse_args::<T>().unwrap_or_abort())
}

impl<T> ToTokens for ComponentAttr<T>
where
    T: quote::ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.inner.to_token_stream())
    }
}

impl ToTokens for Enum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let deprecated: Deprecated = self.deprecated.into();
        tokens.extend(quote! {
            .with_deprecated(#deprecated)
        });

        if let Some(ref default) = self.default {
            tokens.extend(quote! {
                .with_default(#default)
            })
        }

        if let Some(ref example) = self.example {
            tokens.extend(quote! {
                .with_example(#example)
            })
        }
    }
}

impl ToTokens for Struct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let deprecated: Deprecated = self.deprecated.into();
        tokens.extend(quote! {
            .with_deprecated(#deprecated)
        })
    }
}

impl ToTokens for NamedField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let deprecated: Deprecated = self.deprecated.into();
        tokens.extend(quote! {
            .with_deprecated(#deprecated)
        });

        if let Some(ref default) = self.default {
            tokens.extend(quote! {
                .with_default(#default)
            })
        }

        if let Some(ref format) = self.format {
            tokens.extend(quote! {
                .with_format(#format)
            })
        }

        if let Some(ref example) = self.example {
            tokens.extend(quote! {
                .with_example(#example)
            })
        }
    }
}
