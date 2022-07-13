use std::mem;

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{quote, ToTokens};
use syn::{parenthesized, parse::Parse, Attribute, Error, Token, TypePath};

use crate::{
    parse_utils,
    schema::{ComponentPart, GenericType},
    AnyValue,
};

use super::xml::{Xml, XmlAttr};

/// See [`IsInline::is_inline()`].
pub(super) trait IsInline {
    /// Returns `true` if a field's schema/type definition is to be inlined.
    fn is_inline(&self) -> bool;
}

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

impl<T> IsInline for ComponentAttr<T>
where
    T: IsInline,
{
    fn is_inline(&self) -> bool {
        self.inner.is_inline()
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
pub struct UnnamedFieldStruct {
    pub(super) value_type: Option<TypePath>,
    format: Option<ComponentFormat>,
    default: Option<AnyValue>,
    example: Option<AnyValue>,
}

impl IsInline for UnnamedFieldStruct {
    fn is_inline(&self) -> bool {
        false
    }
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct NamedField {
    example: Option<AnyValue>,
    pub(super) value_type: Option<TypePath>,
    format: Option<ComponentFormat>,
    default: Option<AnyValue>,
    write_only: Option<bool>,
    read_only: Option<bool>,
    xml_attr: Option<XmlAttr>,
    pub(super) xml: Option<Xml>,
    inline: bool,
}

impl IsInline for NamedField {
    fn is_inline(&self) -> bool {
        self.inline
    }
}

impl Parse for ComponentAttr<Enum> {
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
        const EXPECTED_ATTRIBUTE_MESSAGE: &str =
            "unexpected attribute, expected any of: example, xml";
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

impl Parse for ComponentAttr<UnnamedFieldStruct> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE_MESSAGE: &str =
            "unexpected attribute, expected any of: default, example, format, value_type";
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
                        input.parse::<ComponentFormat>()
                    })?)
                }
                "value_type" => {
                    unnamed_struct.value_type = Some(parse_utils::parse_next(input, || {
                        input.parse::<TypePath>()
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

impl ComponentAttr<NamedField> {
    pub(super) fn from_attributes_validated(
        attributes: &[Attribute],
        component_part: &ComponentPart,
    ) -> Option<Self> {
        parse_component_attr::<ComponentAttr<NamedField>>(attributes)
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
fn is_valid_xml_attr(attrs: &ComponentAttr<NamedField>, component_part: &ComponentPart) {
    if !matches!(
        component_part.generic_type,
        Some(crate::schema::GenericType::Vec)
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

impl Parse for ComponentAttr<NamedField> {
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
                        input.parse::<ComponentFormat>()
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
                        input.parse::<TypePath>()
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

/// [`Parse`] and [`ToTokens`] implementation for [`utoipa::openapi::schema::ComponentFormat`].
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ComponentFormat {
    Int32,
    Int64,
    Float,
    Double,
    Byte,
    Binary,
    Date,
    DateTime,
    Password,
    #[cfg(feature = "uuid")]
    Uuid,
}

impl Parse for ComponentFormat {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const FORMATS: [&str; 10] = [
            "Int32", "Int64", "Float", "Double", "Byte", "Binary", "Date", "DateTime", "Password",
            "Uuid",
        ];
        let allowed_formats = FORMATS
            .into_iter()
            .filter(|_format| {
                #[cfg(feature = "uuid")]
                {
                    true
                }
                #[cfg(not(feature = "uuid"))]
                {
                    _format != &"Uuid"
                }
            })
            .collect::<Vec<_>>();
        let expected_formats = format!(
            "unexpected format, expected one of: {}",
            allowed_formats.join(", ")
        );
        let format = input.parse::<Ident>()?;
        let name = &*format.to_string();

        match name {
            "Int32" => Ok(Self::Int32),
            "Int64" => Ok(Self::Int64),
            "Float" => Ok(Self::Float),
            "Double" => Ok(Self::Double),
            "Byte" => Ok(Self::Byte),
            "Binary" => Ok(Self::Binary),
            "Date" => Ok(Self::Date),
            "DateTime" => Ok(Self::DateTime),
            "Password" => Ok(Self::Password),
            #[cfg(feature = "uuid")]
            "Uuid" => Ok(Self::Uuid),
            _ => Err(Error::new(
                format.span(),
                format!("unexpected format: {name}, expected one of: {expected_formats}"),
            )),
        }
    }
}

impl ToTokens for ComponentFormat {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Int32 => tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Int32)),
            Self::Int64 => tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Int64)),
            Self::Float => tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Float)),
            Self::Double => tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Double)),
            Self::Byte => tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Byte)),
            Self::Binary => tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Binary)),
            Self::Date => tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Date)),
            Self::DateTime => {
                tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::DateTime))
            }
            Self::Password => {
                tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Password))
            }
            #[cfg(feature = "uuid")]
            Self::Uuid => tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Uuid)),
        };
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
        self.inner.to_tokens(tokens)
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

impl ToTokens for UnnamedFieldStruct {
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

impl ToTokens for NamedField {
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
