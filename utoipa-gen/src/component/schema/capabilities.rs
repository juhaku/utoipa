use proc_macro_error::ResultExt;
use syn::{parse::Parse, Attribute};

use crate::component::capabilities::{
    parse_capability_set, CapabilitySet, Example, Title, XmlAttr,
};

pub struct StructCapabilities(pub CapabilitySet);

impl Parse for StructCapabilities {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(parse_capability_set!(
            input as Example,
            XmlAttr,
            Title
        )))
    }
}

pub fn parse_scheme_capabilities<T: Sized + Parse>(attributes: &[Attribute]) -> Option<T> {
    attributes
        .iter()
        .find(|attribute| attribute.path.get_ident().unwrap() == "schema")
        .map(|attribute| attribute.parse_args::<T>().unwrap_or_abort())
}
