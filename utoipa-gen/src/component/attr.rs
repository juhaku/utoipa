use std::mem;

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseBuffer},
    Attribute, Error, ExprPath, Lit, Token,
};

use crate::{parse_utils, Example};

use super::{
    xml::{Xml, XmlAttr},
    ComponentPart,
};

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
    example: Option<TokenStream>,
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Struct {
    example: Option<Example>,
    xml_attr: Option<XmlAttr>,
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct UnnamedFieldStruct {
    default: Option<TokenStream>,
    example: Option<TokenStream>,
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct NamedField {
    example: Option<TokenStream>,
    format: Option<ExprPath>,
    default: Option<TokenStream>,
    write_only: Option<bool>,
    read_only: Option<bool>,
    xml_attr: Option<XmlAttr>,
    pub(super) xml: Option<Xml>,
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
                        parse_lit_or_fn_ref_as_token_stream(input, name)
                    }))
                }
                "example" => {
                    enum_attr.example = Some(parse_utils::parse_next(input, || {
                        parse_lit_or_fn_ref_as_token_stream(input, name)
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

impl ComponentAttr<Struct> {
    pub(super) fn from_attributes_validated(attributes: &[Attribute]) -> Option<Self> {
        parse_component_attr::<ComponentAttr<Struct>>(attributes).map(|attrs| {
            if let Some(ref wrapped_ident) = attrs
                .as_ref()
                .xml_attr
                .as_ref()
                .and_then(|xml| xml.is_wrapped.as_ref())
            {
                abort! {wrapped_ident, "cannot use `wrapped` attribute in non slice type";
                    help = "Try removing `wrapped` attribute"
                }
            }

            attrs
        })
    }
}

impl Parse for ComponentAttr<Struct> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut struct_ = Struct::default();

        while !input.is_empty() {
            let ident = input
                .parse::<Ident>()
                .expect_or_abort("Unparseable ComponentAttr<Struct>, expected Ident");
            let name = &*ident.to_string();

            match name {
                "example" => {
                    struct_.example = Some(parse_utils::parse_next_lit_str_or_json_example(
                        input, &ident,
                    ));
                }
                "xml" => {
                    let xml;
                    parenthesized!(xml in input);
                    struct_.xml_attr = Some(xml.parse()?)
                }
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        format!("unexpected identifer: {name}, expected one of: example, xml"),
                    ))
                }
            }
        }

        Ok(Self { inner: struct_ })
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
                        parse_lit_or_fn_ref_as_token_stream(input, name)
                    }))
                }
                "example" => {
                    unnamed_struct.example = Some(parse_utils::parse_next(input, || {
                        parse_lit_or_fn_ref_as_token_stream(input, name)
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

impl ComponentAttr<NamedField> {
    pub(super) fn from_attributes_validated(
        attributes: &[Attribute],
        component_part: &ComponentPart,
    ) -> Option<Self> {
        parse_component_attr::<ComponentAttr<NamedField>>(attributes).map(|attrs| {
            if !matches!(
                component_part.generic_type,
                Some(crate::component::GenericType::Vec)
            ) {
                if let Some(ref wrapped_ident) = attrs
                    .as_ref()
                    .xml_attr
                    .as_ref()
                    .and_then(|xml| xml.is_wrapped.as_ref())
                {
                    abort! {wrapped_ident, "cannot use `wrapped` attribute in non slice field type";
                        help = "Try removing `wrapped` attribute or make your field `Vec`"
                    }
                }
            }

            attrs
        })
        .map(|mut attrs| {
            if matches!(component_part.generic_type, Some(super::GenericType::Vec)) {
                if let Some(ref mut xml) = attrs.inner.xml_attr {
                    let mut value_xml = mem::take(xml);
                    let vec_xml = XmlAttr::with_wrapped(
                        mem::take(&mut value_xml.is_wrapped),
                        mem::take(&mut value_xml.wrap_name));

                    attrs.inner.xml = Some(Xml::Slice{vec: vec_xml, value: value_xml});
                }
            } else if let Some(ref mut xml) = attrs.inner.xml_attr {
                attrs.inner.xml = Some(Xml::NonSlice(mem::take(xml)));
            }

            attrs
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
                        parse_lit_or_fn_ref_as_token_stream(input, name)
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
                        parse_lit_or_fn_ref_as_token_stream(input, name)
                    }))
                }
                "write_only" => field.write_only = Some(parse_utils::parse_bool_or_true(input)?),
                "read_only" => field.read_only = Some(parse_utils::parse_bool_or_true(input)?),
                "xml" => {
                    let xml;
                    parenthesized!(xml in input);
                    field.xml_attr = Some(xml.parse()?)
                }
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        format!(
                            "unexpected identifier: {}, expected any of: example, format, default, write_only, read_only, xml",
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

fn parse_lit_or_fn_ref_as_token_stream(input: &ParseBuffer, name: &str) -> TokenStream {
    if input.peek(Lit) {
        let literal = input.parse::<Lit>().unwrap();

        #[cfg(feature = "json")]
        {
            quote! {
                serde_json::json!(#literal)
            }
        }

        #[cfg(not(feature = "json"))]
        {
            quote! {
                format!("{}", #literal)
            }
        }
    } else {
        let method = input.parse::<ExprPath>().unwrap_or_else(|error| {
            let message = &format!("unparseable {}, expected literal or expresssion path", name);
            abort! {
                error.span(), message;
                help = "Try to define {} = value", name;
                help = r#"You should define either literal value e.g. {} = 1 or {} = "value""#, name, name;
                help = r#"You can also use function reference e.g {} = String::default"#, name
            }
        });

        #[cfg(feature = "json")]
        {
            quote! {
                serde_json::json!(#method())
            }
        }
        #[cfg(not(feature = "json"))]
        {
            quote! {
                format!("{}", #method())
            }
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
        if let Some(ref example) = self.example {
            tokens.extend(quote! {
                .with_example(#example)
            })
        }
        if let Some(ref xml) = self.xml_attr {
            tokens.extend(quote!(
                 .with_xml(#xml)
            ))
        }
    }
}

impl ToTokens for UnnamedFieldStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
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

impl ToTokens for NamedField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
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
