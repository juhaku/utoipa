use syn::{
    parse::{Parse, ParseBuffer, ParseStream},
    Attribute,
};

use crate::{
    component::features::{
        attributes::{
            AdditionalProperties, As, Bound, ContentEncoding, ContentMediaType, Deprecated,
            Description, Discriminator, Example, Examples, Format, Ignore, Inline, NoRecursion,
            Nullable, ReadOnly, Rename, RenameAll, Required, SchemaWith, Title, ValueType,
            WriteOnly, XmlAttr,
        },
        impl_into_inner, impl_merge, parse_features,
        validation::{
            ExclusiveMaximum, ExclusiveMinimum, MaxItems, MaxLength, MaxProperties, Maximum,
            MinItems, MinLength, MinProperties, Minimum, MultipleOf, Pattern,
        },
        Feature, Merge,
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
            crate::component::features::attributes::Default,
            Deprecated,
            Description,
            Bound,
            NoRecursion
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
            crate::component::features::attributes::Default,
            Title,
            Format,
            ValueType,
            As,
            Deprecated,
            Description,
            ContentEncoding,
            ContentMediaType,
            Bound,
            NoRecursion,
            Pattern
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
            crate::component::features::attributes::Default,
            Title,
            RenameAll,
            As,
            Deprecated,
            Description,
            Bound
        )))
    }
}

impl_into_inner!(EnumFeatures);

pub struct MixedEnumFeatures(Vec<Feature>);

impl Parse for MixedEnumFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(MixedEnumFeatures(parse_features!(
            input as Example,
            Examples,
            crate::component::features::attributes::Default,
            Title,
            RenameAll,
            As,
            Deprecated,
            Description,
            Discriminator,
            NoRecursion
        )))
    }
}

impl_into_inner!(MixedEnumFeatures);

pub struct NamedFieldFeatures(Vec<Feature>);

impl Parse for NamedFieldFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(NamedFieldFeatures(parse_features!(
            input as Example,
            Examples,
            ValueType,
            Format,
            crate::component::features::attributes::Default,
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
            Deprecated,
            ContentEncoding,
            ContentMediaType,
            Ignore,
            NoRecursion
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
            crate::component::features::attributes::Default,
            XmlAttr,
            Title,
            Rename,
            RenameAll,
            Deprecated,
            MaxProperties,
            MinProperties,
            NoRecursion
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
            crate::component::features::attributes::Default,
            Title,
            Format,
            ValueType,
            Rename,
            Deprecated,
            NoRecursion
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
    MixedEnumFeatures,
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
