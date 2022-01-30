use proc_macro2::{Group, Ident, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseBuffer},
    Attribute, Error, ExprPath, Lit, Token,
};

use crate::parse_utils;

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
    default: Option<TokenStream>,
    example: Option<String>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Struct {
    example: Option<TokenStream>,
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct UnnamedFieldStruct {
    default: Option<TokenStream>,
    example: Option<String>,
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct NamedField {
    example: Option<String>,
    format: Option<ExprPath>,
    default: Option<TokenStream>,
    write_only: Option<bool>,
    read_only: Option<bool>,
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
                "default" => {
                    enum_attr.default = Some(parse_utils::parse_next(input, || {
                        parse_default_as_token_stream(input, name)
                    }))
                }
                "example" => {
                    enum_attr.example = Some(parse_utils::parse_next(input, || {
                        parse_lit_as_string(input, name, "unparseable example, expected literal")
                    }))
                }
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        format!(
                            "unexpected identifer: {}, expected any of: default, example",
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
            "example" => {
                let tokens = parse_utils::parse_next(input, || {
                    if input.peek(syn::Ident) && input.peek2(Token![!]) {
                        input.parse::<Ident>().unwrap();
                        input.parse::<Token![!]>().unwrap();

                        Ok(input
                            .parse::<Group>()
                            .expect_or_abort("unparseable example, expected parenthesis"))
                    } else {
                        Err(Error::new(
                            ident.span(),
                            "unexpected example, expected json!(...)",
                        ))
                    }
                })?;

                Ok(Self {
                    inner: Struct {
                        example: Some(tokens.stream()),
                    },
                })
            }
            _ => Err(Error::new(
                ident.span(),
                format!("unexpected identifer: {}, expected: example", name),
            )),
        }
    }
}

impl Parse for ComponentAttr<UnnamedFieldStruct> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut unnamed_struct = UnnamedFieldStruct::default();

        loop {
            let attribute = input.parse::<Ident>().expect_or_abort(
                "Unparseable ComponentAttr<UnnamedFieldStruct>, expected identifier",
            );
            let name = &*attribute.to_string();

            match name {
                "default" => {
                    unnamed_struct.default = Some(parse_utils::parse_next(input, || {
                        parse_default_as_token_stream(input, name)
                    }))
                }
                "example" => {
                    unnamed_struct.example = Some(parse_utils::parse_next(input, || {
                        parse_lit_as_string(
                            input,
                            name,
                            "unparseable example, expected literal string",
                        )
                    }))
                }
                _ => {
                    return Err(Error::new(
                        attribute.span(),
                        format!(
                            "unexpected identifier: {}, expected any of: default, example",
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

        Ok(Self {
            inner: unnamed_struct,
        })
    }
}

impl Parse for ComponentAttr<NamedField> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut field = NamedField::default();

        loop {
            let ident = input
                .parse::<Ident>()
                .expect_or_abort("Unparseable ComponentAttr<NamedField>, expected identifier");
            let name = &*ident.to_string();

            match name {
                "example" => {
                    field.example = Some(parse_utils::parse_next(input, || {
                        parse_lit_as_string(input, name, "unparseable example, expected literal")
                    }));
                }
                "format" => {
                    let format = parse_utils::parse_next(input, || {
                        input.parse::<ExprPath>().expect_or_abort(
                            "unparseable format expected expression path e.g. ComponentFormat::String",
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
                "write_only" => field.write_only = Some(parse_utils::parse_bool_or_true(input)),
                "read_only" => field.read_only = Some(parse_utils::parse_bool_or_true(input)),
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        format!(
                            "unexpected identifier: {}, expected any of: example, format, default, write_only, read_only",
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
                format!("unparseable utf8 content in: {}", &field)
            )
        }),
        Lit::Char(char) => char.value().to_string(),
        Lit::Float(float) => float.base10_digits().to_string(),
        Lit::Int(int) => int.base10_digits().to_string(),
        Lit::Str(str) => str.value(),
        Lit::Verbatim(_) => {
            abort!(
                input.span(),
                format!("unparseable literal in field: {}", &field)
            )
        }
    }
}

fn parse_default_as_token_stream(input: &ParseBuffer, name: &str) -> TokenStream {
    if input.peek(Lit) {
        let literal = parse_lit_as_string(
            input,
            name,
            &format!("unparseable {}, expected literal", name),
        );
        quote_spanned! {input.span()=>
            #literal
        }
    } else {
        let method = input.parse::<ExprPath>().expect_or_abort(&format!(
            "unparseable {}, expected literal or expresssion path",
            name
        ));
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
        if let Some(ref default) = self.default {
            tokens.extend(quote! {
                .with_default(serde_json::json!(#default))
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
        if let Some(ref example) = self.example {
            tokens.extend(quote! {
                .with_example(serde_json::json!(#example))
            })
        }
    }
}

impl ToTokens for UnnamedFieldStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(ref default) = self.default {
            tokens.extend(quote! {
                .with_default(serde_json::json!(#default))
            })
        }

        if let Some(ref example) = self.example {
            tokens.extend(quote! {
                .with_example(#example)
            })
        }
    }
}

impl ToTokens for NamedField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(ref default) = self.default {
            tokens.extend(quote! {
                .with_default(serde_json::json!(#default))
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

        if let Some(ref write_only) = self.write_only {
            tokens.extend(quote! {
                .with_write_only(#write_only)
            })
        }

        if let Some(ref read_only) = self.read_only {
            tokens.extend(quote! {
                .with_read_only(#read_only)
            })
        }
    }
}
