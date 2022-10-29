use proc_macro_error::ResultExt;
use syn::{
    parse::{Parse, ParseStream},
    Attribute,
};

use crate::component::capabilities::{
    parse_capability_set, Capability, Default, Example, Format, Inline, Nullable, ReadOnly, Title,
    ValueType, WriteOnly, XmlAttr,
};

pub trait IntoInner<T> {
    fn into_inner(self) -> T;
}

macro_rules! impl_into_inner {
    ($ident:ident) => {
        impl IntoInner<Vec<Capability>> for $ident {
            fn into_inner(self) -> Vec<Capability> {
                self.0
            }
        }

        impl IntoInner<Option<Vec<Capability>>> for Option<$ident> {
            fn into_inner(self) -> Option<Vec<Capability>> {
                self.map(IntoInner::into_inner)
            }
        }
    };
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct NamedFieldStructCapabilities(Vec<Capability>);

impl Parse for NamedFieldStructCapabilities {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(NamedFieldStructCapabilities(parse_capability_set!(
            input as Example,
            XmlAttr,
            Title
        )))
    }
}

impl_into_inner!(NamedFieldStructCapabilities);

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct UnnamedFieldStructCapabilities(Vec<Capability>);

impl Parse for UnnamedFieldStructCapabilities {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(UnnamedFieldStructCapabilities(parse_capability_set!(
            input as Example,
            Default,
            Title,
            Format,
            ValueType
        )))
    }
}

impl_into_inner!(UnnamedFieldStructCapabilities);

pub struct EnumCapabilities(Vec<Capability>);

impl Parse for EnumCapabilities {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(EnumCapabilities(parse_capability_set!(
            input as Example,
            Default,
            Title
        )))
    }
}

impl_into_inner!(EnumCapabilities);

pub struct NamedFieldCapablities(Vec<Capability>);

impl Parse for NamedFieldCapablities {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(NamedFieldCapablities(parse_capability_set!(
            input as Example,
            ValueType,
            Format,
            Default,
            WriteOnly,
            ReadOnly,
            XmlAttr,
            Inline,
            Nullable
        )))
    }
}

impl_into_inner!(NamedFieldCapablities);

pub trait FromAttributes {
    fn parse_capabilities<T>(&self) -> Option<T>
    where
        T: Parse;
}

impl FromAttributes for &'_ [Attribute] {
    fn parse_capabilities<T>(&self) -> Option<T>
    where
        T: Parse,
    {
        parse_schema_capabilities::<T>(self)
    }
}

impl FromAttributes for Vec<Attribute> {
    fn parse_capabilities<T>(&self) -> Option<T>
    where
        T: Parse,
    {
        parse_schema_capabilities::<T>(self)
    }
}

pub fn parse_schema_capabilities<T: Sized + Parse>(attributes: &[Attribute]) -> Option<T> {
    parse_schema_capabilities_with(attributes, T::parse)
}

pub fn parse_schema_capabilities_with<T>(
    attributes: &[Attribute],
    parser: impl FnOnce(ParseStream) -> syn::Result<T>,
) -> Option<T> {
    attributes
        .iter()
        .find(|attribute| attribute.path.get_ident().unwrap() == "schema")
        .map(|attribute| attribute.parse_args_with(parser).unwrap_or_abort())
}
