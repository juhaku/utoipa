use proc_macro_error::ResultExt;
use syn::{
    parse::{Parse, ParseStream},
    Attribute,
};

use crate::component::features::{
    impl_into_inner, parse_features, Default, Example, Feature, Format, Inline, Nullable, ReadOnly,
    Rename, RenameAll, Title, ValueType, WriteOnly, XmlAttr,
};

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct NamedFieldStructFeatures(Vec<Feature>);

impl Parse for NamedFieldStructFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(NamedFieldStructFeatures(parse_features!(
            input as Example,
            XmlAttr,
            Title,
            RenameAll
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
            Title,
            RenameAll
        )))
    }
}

impl_into_inner!(EnumFeatures);

pub struct ComplexEnumFeatures(Vec<Feature>);

impl Parse for ComplexEnumFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ComplexEnumFeatures(parse_features!(
            input as Example,
            Default,
            RenameAll
        )))
    }
}

impl_into_inner!(ComplexEnumFeatures);

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
            Nullable,
            Rename
        )))
    }
}

impl_into_inner!(NamedFieldFeatures);

pub struct EnumNamedFieldVariantFeatures(Vec<Feature>);

impl Parse for EnumNamedFieldVariantFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(EnumNamedFieldVariantFeatures(parse_features!(
            input as Example,
            XmlAttr,
            Title,
            Rename,
            RenameAll
        )))
    }
}

impl_into_inner!(EnumNamedFieldVariantFeatures);

pub struct EnumUnnamedFieldVariantFeatures(Vec<Feature>);

impl Parse for EnumUnnamedFieldVariantFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(EnumUnnamedFieldVariantFeatures(parse_features!(
            input as Example,
            Default,
            Title,
            Format,
            ValueType,
            Rename
        )))
    }
}

impl_into_inner!(EnumUnnamedFieldVariantFeatures);

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
