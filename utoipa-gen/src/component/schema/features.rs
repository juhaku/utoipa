use proc_macro_error::ResultExt;
use syn::{
    parse::{Parse, ParseBuffer, ParseStream},
    Attribute,
};

use crate::component::features::{
    impl_into_inner, parse_features, Default, Example, ExclusiveMaximum, ExclusiveMinimum, Feature,
    Format, Inline, IntoInner, MaxItems, MaxLength, MaxProperties, Maximum, MinItems, MinLength,
    MinProperties, Minimum, MultipleOf, Nullable, Pattern, ReadOnly, Rename, RenameAll, SchemaWith,
    Title, ValueType, WriteOnly, XmlAttr,
};

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct NamedFieldStructFeatures(Vec<Feature>);

impl Parse for NamedFieldStructFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(NamedFieldStructFeatures(parse_features!(
            input as Example,
            XmlAttr,
            Title,
            RenameAll,
            MaxProperties,
            MinProperties
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
            Rename,
            MultipleOf,
            Maximum,
            Minimum,
            ExclusiveMaximum,
            ExclusiveMinimum,
            MaxLength,
            MinLength,
            Pattern,
            MaxItems,
            MinItems,
            SchemaWith
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
        T: Parse + Merge<T>;
}

impl FromAttributes for &'_ [Attribute] {
    fn parse_features<T>(&self) -> Option<T>
    where
        T: Parse + Merge<T>,
    {
        parse_schema_features::<T>(self)
    }
}

impl FromAttributes for Vec<Attribute> {
    fn parse_features<T>(&self) -> Option<T>
    where
        T: Parse + Merge<T>,
    {
        parse_schema_features::<T>(self)
    }
}

pub trait Merge<T>: IntoInner<Vec<Feature>> {
    fn merge(self, from: T) -> Self;
}

macro_rules! impl_merge {
    ( $($ident:ident),* ) => {
        $(
            impl AsMut<Vec<Feature>> for $ident {
                fn as_mut(&mut self) -> &mut Vec<Feature> {
                    &mut self.0
                }
            }

            impl Merge<$ident> for $ident {
                fn merge(mut self, from: $ident) -> Self {
                    let a = self.as_mut();
                    let mut b = from.into_inner();

                    a.append(&mut b);

                    self
                }
            }
        )*
    };
}

impl_merge!(
    NamedFieldStructFeatures,
    UnnamedFieldStructFeatures,
    EnumFeatures,
    ComplexEnumFeatures,
    NamedFieldFeatures,
    EnumNamedFieldVariantFeatures,
    EnumUnnamedFieldVariantFeatures
);

pub fn parse_schema_features<T: Sized + Parse + Merge<T>>(attributes: &[Attribute]) -> Option<T> {
    attributes
        .iter()
        .filter(|attribute| attribute.path.get_ident().unwrap() == "schema")
        .map(|attribute| attribute.parse_args::<T>().unwrap_or_abort())
        .reduce(|acc, item| acc.merge(item))
}

impl IntoInner<Vec<Feature>> for Vec<Feature> {
    fn into_inner(self) -> Vec<Feature> {
        self
    }
}

impl Merge<Vec<Feature>> for Vec<Feature> {
    fn merge(mut self, mut from: Vec<Feature>) -> Self {
        self.append(&mut from);
        self
    }
}

pub fn parse_schema_features_with<
    T: Merge<T>,
    P: for<'r> FnOnce(&'r ParseBuffer<'r>) -> syn::Result<T> + Copy,
>(
    attributes: &[Attribute],
    parser: P,
) -> Option<T> {
    attributes
        .iter()
        .filter(|attribute| attribute.path.get_ident().unwrap() == "schema")
        .map(|attributes| attributes.parse_args_with(parser).unwrap_or_abort())
        .reduce(|acc, item| acc.merge(item))
}
