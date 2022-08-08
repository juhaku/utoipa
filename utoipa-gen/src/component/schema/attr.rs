use std::mem;

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{quote, ToTokens};
use syn::{parenthesized, parse::Parse, Attribute, Error, Token};

use crate::{
    component::{GenericType, TypeTree},
    parse_utils,
    schema_type::SchemaFormat,
    AnyValue,
};

use super::xml::{Xml, XmlAttr};

/// See [`IsInline::is_inline()`].
pub(super) trait IsInline {
    /// Returns `true` if a field's schema/type definition is to be inlined.
    fn is_inline(&self) -> bool;
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct SchemaAttr<T>
where
    T: Sized,
{
    inner: T,
}

impl<T> AsRef<T> for SchemaAttr<T>
where
    T: Sized,
{
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> IsInline for SchemaAttr<T>
where
    T: IsInline,
{
    fn is_inline(&self) -> bool {
        self.inner.is_inline()
    }
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Title(Option<String>);

impl IsInline for Title {
    fn is_inline(&self) -> bool {
        false
    }
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Enum {
    default: Option<AnyValue>,
    example: Option<AnyValue>,
}

impl IsInline for Enum {
    fn is_inline(&self) -> bool {
        false
    }
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Struct {
    example: Option<AnyValue>,
    xml_attr: Option<XmlAttr>,
}

impl IsInline for Struct {
    fn is_inline(&self) -> bool {
        false
    }
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct UnnamedFieldStruct<'c> {
    pub(super) value_type: Option<syn::Type>,
    format: Option<SchemaFormat<'c>>,
    default: Option<AnyValue>,
    example: Option<AnyValue>,
}

impl IsInline for UnnamedFieldStruct<'_> {
    fn is_inline(&self) -> bool {
        false
    }
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct NamedField<'c> {
    example: Option<AnyValue>,
    pub(super) value_type: Option<syn::Type>,
    format: Option<SchemaFormat<'c>>,
    default: Option<AnyValue>,
    write_only: Option<bool>,
    read_only: Option<bool>,
    xml_attr: Option<XmlAttr>,
    pub(super) xml: Option<Xml>,
    inline: bool,
}

impl IsInline for NamedField<'_> {
    fn is_inline(&self) -> bool {
        self.inline
    }
}

impl Parse for SchemaAttr<Title> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE_MESSAGE: &str = "unexpected attribute, expected any of: title";
        let mut title_attr = Title::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>().map_err(|error| {
                Error::new(
                    error.span(),
                    format!("{}, {}", EXPECTED_ATTRIBUTE_MESSAGE, error),
                )
            })?;
            let name = &*ident.to_string();

            if name == "title" {
                title_attr = Title(Some(parse_utils::parse_next_literal_str(input)?))
            }

            if !input.is_empty() {
                parse_utils::skip_past_next_comma(input)?;
            }
        }
        Ok(Self { inner: title_attr })
    }
}

impl Parse for SchemaAttr<Enum> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE_MESSAGE: &str =
            "unexpected attribute, expected any of: default, example";
        let mut enum_attr = Enum::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>().map_err(|error| {
                Error::new(
                    error.span(),
                    format!("{}, {}", EXPECTED_ATTRIBUTE_MESSAGE, error),
                )
            })?;
            let name = &*ident.to_string();

            match name {
                "default" => {
                    enum_attr.default = Some(parse_utils::parse_next(input, || {
                        AnyValue::parse_any(input)
                    })?)
                }
                "example" => {
                    enum_attr.example = Some(parse_utils::parse_next(input, || {
                        AnyValue::parse_any(input)
                    })?)
                }
                _ => return Err(Error::new(ident.span(), EXPECTED_ATTRIBUTE_MESSAGE)),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self { inner: enum_attr })
    }
}

impl SchemaAttr<Struct> {
    pub(super) fn from_attributes_validated(attributes: &[Attribute]) -> Option<Self> {
        parse_schema_attr::<SchemaAttr<Struct>>(attributes).map(|attrs| {
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

impl Parse for SchemaAttr<Struct> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE_MESSAGE: &str =
            "unexpected attribute, expected any of: title, example, xml";
        let mut struct_ = Struct::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>().map_err(|error| {
                Error::new(
                    error.span(),
                    &format!("{}, {}", EXPECTED_ATTRIBUTE_MESSAGE, error),
                )
            })?;
            let name = &*ident.to_string();

            match name {
                "title" => {
                    parse_utils::parse_next_literal_str(input)?; // Handled by SchemaAttr<Title>
                }
                "example" => {
                    struct_.example = Some(parse_utils::parse_next(input, || {
                        AnyValue::parse_lit_str_or_json(input)
                    })?);
                }
                "xml" => {
                    let xml;
                    parenthesized!(xml in input);
                    struct_.xml_attr = Some(xml.parse()?)
                }
                _ => return Err(Error::new(ident.span(), EXPECTED_ATTRIBUTE_MESSAGE)),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { inner: struct_ })
    }
}

impl<'c> Parse for SchemaAttr<UnnamedFieldStruct<'c>> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE_MESSAGE: &str =
            "unexpected attribute, expected any of: title, default, example, format, value_type";
        let mut unnamed_struct = UnnamedFieldStruct::default();

        while !input.is_empty() {
            let attribute = input.parse::<Ident>().map_err(|error| {
                Error::new(
                    error.span(),
                    format!("{}, {}", EXPECTED_ATTRIBUTE_MESSAGE, error),
                )
            })?;
            let name = &*attribute.to_string();

            match name {
                "title" => {
                    parse_utils::parse_next_literal_str(input)?; // Handled by SchemaAttr<Title>
                }
                "default" => {
                    unnamed_struct.default = Some(parse_utils::parse_next(input, || {
                        AnyValue::parse_any(input)
                    })?)
                }
                "example" => {
                    unnamed_struct.example = Some(parse_utils::parse_next(input, || {
                        AnyValue::parse_any(input)
                    })?)
                }
                "format" => {
                    unnamed_struct.format = Some(parse_utils::parse_next(input, || {
                        input.parse::<SchemaFormat<'c>>()
                    })?)
                }
                "value_type" => {
                    unnamed_struct.value_type = Some(parse_utils::parse_next(input, || {
                        input.parse::<syn::Type>()
                    })?)
                }
                _ => return Err(Error::new(attribute.span(), EXPECTED_ATTRIBUTE_MESSAGE)),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            inner: unnamed_struct,
        })
    }
}

impl<'c> SchemaAttr<NamedField<'c>> {
    pub(super) fn from_attributes_validated(
        attributes: &[Attribute],
        component_part: &TypeTree,
    ) -> Option<Self> {
        parse_schema_attr::<SchemaAttr<NamedField>>(attributes)
            .map(|attrs| {
                is_valid_xml_attr(&attrs, component_part);

                attrs
            })
            .map(|mut attrs| {
                if matches!(component_part.generic_type, Some(GenericType::Vec)) {
                    if let Some(ref mut xml) = attrs.inner.xml_attr {
                        let mut value_xml = mem::take(xml);
                        let vec_xml = XmlAttr::with_wrapped(
                            mem::take(&mut value_xml.is_wrapped),
                            mem::take(&mut value_xml.wrap_name),
                        );

                        attrs.inner.xml = Some(Xml::Slice {
                            vec: vec_xml,
                            value: value_xml,
                        });
                    }
                } else if let Some(ref mut xml) = attrs.inner.xml_attr {
                    attrs.inner.xml = Some(Xml::NonSlice(mem::take(xml)));
                }

                attrs
            })
    }
}

#[inline]
fn is_valid_xml_attr(attrs: &SchemaAttr<NamedField>, component_part: &TypeTree) {
    if !matches!(
        component_part.generic_type,
        Some(crate::component::GenericType::Vec)
    ) {
        if let Some(wrapped_ident) = attrs
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
}

impl<'c> Parse for SchemaAttr<NamedField<'c>> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE_MESSAGE: &str = "unexpected attribute, expected any of: example, format, default, write_only, read_only, xml, value_type, inline";
        let mut field = NamedField::default();

        while !input.is_empty() {
            let ident = input.parse::<Ident>().map_err(|error| {
                Error::new(
                    error.span(),
                    format!("{}, {}", EXPECTED_ATTRIBUTE_MESSAGE, error),
                )
            })?;
            let name = &*ident.to_string();

            match name {
                "example" => {
                    field.example = Some(parse_utils::parse_next(input, || {
                        AnyValue::parse_any(input)
                    })?);
                }
                "format" => {
                    field.format = Some(parse_utils::parse_next(input, || {
                        input.parse::<SchemaFormat>()
                    })?)
                }
                "default" => {
                    field.default = Some(parse_utils::parse_next(input, || {
                        AnyValue::parse_any(input)
                    })?)
                }
                "inline" => field.inline = parse_utils::parse_bool_or_true(input)?,
                "write_only" => field.write_only = Some(parse_utils::parse_bool_or_true(input)?),
                "read_only" => field.read_only = Some(parse_utils::parse_bool_or_true(input)?),
                "xml" => {
                    let xml;
                    parenthesized!(xml in input);
                    field.xml_attr = Some(xml.parse()?);
                }
                "value_type" => {
                    field.value_type = Some(parse_utils::parse_next(input, || {
                        input.parse::<syn::Type>()
                    })?);
                }
                _ => return Err(Error::new(ident.span(), EXPECTED_ATTRIBUTE_MESSAGE)),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { inner: field })
    }
}

pub fn parse_schema_attr<T: Sized + Parse>(attributes: &[Attribute]) -> Option<T> {
    attributes
        .iter()
        .find(|attribute| attribute.path.get_ident().unwrap() == "schema")
        .map(|attribute| attribute.parse_args::<T>().unwrap_or_abort())
}

impl<T> ToTokens for SchemaAttr<T>
where
    T: quote::ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.inner.to_tokens(tokens)
    }
}

impl ToTokens for Title {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(ref title) = self.0 {
            tokens.extend(quote! {
                .title(Some(#title))
            })
        }
    }
}

impl ToTokens for Enum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(ref default) = self.default {
            tokens.extend(quote! {
                .default(Some(#default))
            })
        }

        if let Some(ref example) = self.example {
            tokens.extend(quote! {
                .example(Some(#example))
            })
        }
    }
}

impl ToTokens for Struct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(ref example) = self.example {
            tokens.extend(quote! {
                .example(Some(#example))
            })
        }
        if let Some(ref xml) = self.xml_attr {
            tokens.extend(quote!(
                 .xml(Some(#xml))
            ))
        }
    }
}

impl ToTokens for UnnamedFieldStruct<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(ref default) = self.default {
            tokens.extend(quote! {
                .default(Some(#default))
            })
        }

        if let Some(ref example) = self.example {
            tokens.extend(quote! {
                .example(Some(#example))
            })
        }

        if let Some(ref format) = self.format {
            tokens.extend(quote! {
                .format(Some(#format))
            })
        }
    }
}

impl ToTokens for NamedField<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(ref default) = self.default {
            tokens.extend(quote! {
                .default(Some(#default))
            })
        }

        if let Some(ref format) = self.format {
            tokens.extend(quote! {
                .format(Some(#format))
            })
        }

        if let Some(ref example) = self.example {
            tokens.extend(quote! {
                .example(Some(#example))
            })
        }

        if let Some(ref write_only) = self.write_only {
            tokens.extend(quote! {
                .write_only(Some(#write_only))
            })
        }

        if let Some(ref read_only) = self.read_only {
            tokens.extend(quote! {
                .read_only(Some(#read_only))
            })
        }
    }
}
