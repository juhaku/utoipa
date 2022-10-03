use proc_macro_error::ResultExt;
use syn::{
    parse::{Parse, ParseStream},
    Attribute,
};

use crate::component::capabilities::{
    parse_capability_set, CapabilitySet, Default, Example, Format, Inline, ReadOnly, Title,
    ValueType, WriteOnly, XmlAttr,
};

pub fn parse_struct_capabilities(input: ParseStream) -> syn::Result<CapabilitySet> {
    Ok(parse_capability_set!(input as Example, XmlAttr, Title))
}

pub fn parse_unnamed_field_struct_capabilities(input: ParseStream) -> syn::Result<CapabilitySet> {
    Ok(parse_capability_set!(
        input as Example,
        Default,
        Title,
        Format,
        ValueType
    ))
}

pub fn parse_enum_capabilities(input: ParseStream) -> syn::Result<CapabilitySet> {
    Ok(parse_capability_set!(input as Example, Default, Title))
}

pub fn parse_named_field_capabilities(attributes: &[Attribute]) -> Option<CapabilitySet> {
    parse_schema_capabilities_with(attributes, |input| {
        Ok(parse_capability_set!(
            input as Example,
            ValueType,
            Format,
            Default,
            WriteOnly,
            ReadOnly,
            XmlAttr,
            Inline
        ))
    })
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
