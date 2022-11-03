use proc_macro_error::ResultExt;
use syn::{
    parse::{Parse, ParseStream},
    Attribute,
};

use crate::component::features::{
    parse_features, Default, Example, Feature, Format, Inline, Nullable, ReadOnly, Title,
    ValueType, WriteOnly, XmlAttr,
};

pub trait IntoInner<T> {
    fn into_inner(self) -> T;
}

macro_rules! impl_into_inner {
    ($ident:ident) => {
        impl IntoInner<Vec<Feature>> for $ident {
            fn into_inner(self) -> Vec<Feature> {
                self.0
            }
        }

        impl IntoInner<Option<Vec<Feature>>> for Option<$ident> {
            fn into_inner(self) -> Option<Vec<Feature>> {
                self.map(IntoInner::into_inner)
            }
        }
    };
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct NamedFieldStructFeatures(Vec<Feature>);

impl Parse for NamedFieldStructFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(NamedFieldStructFeatures(parse_features!(
            input as Example,
            XmlAttr,
            Title
        )))
    }
}

impl_into_inner!(NamedFieldStructFeatures);

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct UnnamedFieldStructFeatures(Vec<Feature>);

impl Parse for UnnamedFieldStructFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(UnnamedFieldStructFeatures(parse_features!(
            input as Example,
            Default,
            Title,
            Format,
            ValueType
        )))
    }
}

impl_into_inner!(UnnamedFieldStructFeatures);

pub struct EnumFeatures(Vec<Feature>);

impl Parse for EnumFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(EnumFeatures(parse_features!(
            input as Example,
            Default,
            Title
        )))
    }
}

impl_into_inner!(EnumFeatures);

pub struct NamedFieldFeatures(Vec<Feature>);

impl Parse for NamedFieldFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(NamedFieldFeatures(parse_features!(
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

impl_into_inner!(NamedFieldFeatures);

pub trait FromAttributes {
    fn parse_features<T>(&self) -> Option<T>
    where
        T: Parse;
}

impl FromAttributes for &'_ [Attribute] {
    fn parse_features<T>(&self) -> Option<T>
    where
        T: Parse,
    {
        parse_schema_features::<T>(self)
    }
}

impl FromAttributes for Vec<Attribute> {
    fn parse_features<T>(&self) -> Option<T>
    where
        T: Parse,
    {
        parse_schema_features::<T>(self)
    }
}

pub fn parse_schema_features<T: Sized + Parse>(attributes: &[Attribute]) -> Option<T> {
    parse_schema_features_with(attributes, T::parse)
}

pub fn parse_schema_features_with<T>(
    attributes: &[Attribute],
    parser: impl FnOnce(ParseStream) -> syn::Result<T>,
) -> Option<T> {
    attributes
        .iter()
        .find(|attribute| attribute.path.get_ident().unwrap() == "schema")
        .map(|attribute| attribute.parse_args_with(parser).unwrap_or_abort())
}
