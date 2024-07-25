use syn::{
    parse::{Parse, ParseBuffer, ParseStream},
    Attribute,
};

use crate::{
    component::features::{
        impl_into_inner, impl_merge, parse_features, AdditionalProperties, As, Default, Deprecated,
        Description, Example, Examples, ExclusiveMaximum, ExclusiveMinimum, Feature, Format,
        Inline, IntoInner, MaxItems, MaxLength, MaxProperties, Maximum, Merge, MinItems, MinLength,
        MinProperties, Minimum, MultipleOf, Nullable, Pattern, ReadOnly, Rename, RenameAll,
        Required, SchemaWith, Title, ValueType, WriteOnly, XmlAttr,
    },
    Diagnostics,
};

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct NamedFieldStructFeatures(Vec<Feature>);

impl Parse for NamedFieldStructFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(NamedFieldStructFeatures(parse_features!(
            input as Example,
            Examples,
            XmlAttr,
            Title,
            RenameAll,
            MaxProperties,
            MinProperties,
            As,
            Default,
            Deprecated,
            Description
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
            Examples,
            Default,
            Title,
            Format,
            ValueType,
            As,
            Deprecated,
            Description
        )))
    }
}

impl_into_inner!(UnnamedFieldStructFeatures);

pub struct EnumFeatures(Vec<Feature>);

impl Parse for EnumFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(EnumFeatures(parse_features!(
            input as Example,
            Examples,
            Default,
            Title,
            RenameAll,
            As,
            Deprecated,
            Description
        )))
    }
}

impl_into_inner!(EnumFeatures);

pub struct ComplexEnumFeatures(Vec<Feature>);

impl Parse for ComplexEnumFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ComplexEnumFeatures(parse_features!(
            input as Example,
            Examples,
            Default,
            RenameAll,
            As,
            Deprecated,
            Description
        )))
    }
}

impl_into_inner!(ComplexEnumFeatures);

pub struct NamedFieldFeatures(Vec<Feature>);

impl Parse for NamedFieldFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(NamedFieldFeatures(parse_features!(
            input as Example,
            Examples,
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
            SchemaWith,
            AdditionalProperties,
            Required,
            Deprecated
        )))
    }
}

impl_into_inner!(NamedFieldFeatures);

pub struct EnumNamedFieldVariantFeatures(Vec<Feature>);

impl Parse for EnumNamedFieldVariantFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(EnumNamedFieldVariantFeatures(parse_features!(
            input as Example,
            Examples,
            XmlAttr,
            Title,
            Rename,
            RenameAll,
            Deprecated
        )))
    }
}

impl_into_inner!(EnumNamedFieldVariantFeatures);

pub struct EnumUnnamedFieldVariantFeatures(Vec<Feature>);

impl Parse for EnumUnnamedFieldVariantFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(EnumUnnamedFieldVariantFeatures(parse_features!(
            input as Example,
            Examples,
            Default,
            Title,
            Format,
            ValueType,
            Rename,
            Deprecated
        )))
    }
}

impl_into_inner!(EnumUnnamedFieldVariantFeatures);

pub trait FromAttributes {
    fn parse_features<T>(&self) -> Result<Option<T>, Diagnostics>
    where
        T: Parse + Merge<T>;
}

impl FromAttributes for &'_ [Attribute] {
    fn parse_features<T>(&self) -> Result<Option<T>, Diagnostics>
    where
        T: Parse + Merge<T>,
    {
        parse_schema_features::<T>(self)
    }
}

impl FromAttributes for Vec<Attribute> {
    fn parse_features<T>(&self) -> Result<Option<T>, Diagnostics>
    where
        T: Parse + Merge<T>,
    {
        parse_schema_features::<T>(self)
    }
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

pub fn parse_schema_features<T: Sized + Parse + Merge<T>>(
    attributes: &[Attribute],
) -> Result<Option<T>, Diagnostics> {
    Ok(attributes
        .iter()
        .filter(|attribute| {
            attribute
                .path()
                .get_ident()
                .map(|ident| *ident == "schema")
                .unwrap_or(false)
        })
        .map(|attribute| attribute.parse_args::<T>().map_err(Diagnostics::from))
        .collect::<Result<Vec<T>, Diagnostics>>()?
        .into_iter()
        .reduce(|acc, item| acc.merge(item)))
}

pub fn parse_schema_features_with<
    T: Merge<T>,
    P: for<'r> FnOnce(&'r ParseBuffer<'r>) -> syn::Result<T> + Copy,
>(
    attributes: &[Attribute],
    parser: P,
) -> Result<Option<T>, Diagnostics> {
    Ok(attributes
        .iter()
        .filter(|attribute| {
            attribute
                .path()
                .get_ident()
                .map(|ident| *ident == "schema")
                .unwrap_or(false)
        })
        .map(|attributes| {
            attributes
                .parse_args_with(parser)
                .map_err(Diagnostics::from)
        })
        .collect::<Result<Vec<T>, Diagnostics>>()?
        .into_iter()
        .reduce(|acc, item| acc.merge(item)))
}
