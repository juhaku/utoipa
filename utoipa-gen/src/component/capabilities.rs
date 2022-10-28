use std::{mem, ops::DerefMut};

use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    Attribute,
};

use crate::{parse_utils, schema_type::SchemaFormat, AnyValue};

use super::{schema, GenericType, TypeTree};

pub trait Name {
    fn get_name() -> &'static str;
}

pub trait ToCapabilities {
    fn to_capablities(self) -> Vec<Capability>;
}

macro_rules! name {
    ( $ident:ident = $name:literal ) => {
        impl Name for $ident {
            fn get_name() -> &'static str {
                $name
            }
        }
    };
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Capability {
    Example(Example),
    Default(Default),
    Inline(Inline),
    XmlAttr(XmlAttr),
    Format(Format),
    ValueType(ValueType),
    WriteOnly(WriteOnly),
    ReadOnly(ReadOnly),
    Title(Title),
    Nullable(Nullable),
}

impl Capability {
    pub fn parse_named<T: Name>(input: syn::parse::ParseStream, ident: Ident) -> syn::Result<Self> {
        let name = T::get_name();
        match name {
            "default" => Default::parse(input).map(Self::Default),
            "example" => Example::parse(input).map(Self::Example),
            "inline" => Inline::parse(input).map(Self::Inline),
            "xml" => XmlAttr::parse(input).map(Self::XmlAttr),
            "format" => Format::parse(input).map(Self::Format),
            "value_type" => ValueType::parse(input).map(Self::ValueType),
            "write_only" => WriteOnly::parse(input).map(Self::WriteOnly),
            "read_only" => ReadOnly::parse(input).map(Self::ReadOnly),
            "title" => Title::parse(input).map(Self::Title),
            "nullable" => Nullable::parse(input).map(Self::Nullable),
            _unexpected => Err(syn::Error::new(ident.span(), format!("unexpected name: {}, expected one of: default, example, inline, xml, format, value_type, write_only, read_only, title", _unexpected))),
        }
    }
}

impl ToTokens for Capability {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let capability = match &self {
            Capability::Default(default) => quote! { .default(Some(#default)) },
            Capability::Example(example) => quote! { .example(Some(#example)) },
            Capability::Inline(inline) => quote! { .inline(Some(#inline)) },
            Capability::XmlAttr(xml) => quote! { .xml(Some(#xml)) },
            Capability::Format(format) => quote! { .format(Some(#format)) },
            Capability::WriteOnly(write_only) => quote! { .write_only(Some(#write_only)) },
            Capability::ReadOnly(read_only) => quote! { .read_only(Some(#read_only)) },
            Capability::Title(title) => quote! { .title(Some(#title)) },
            Capability::Nullable(nullable) => quote! { .nullable(#nullable) },
            Capability::ValueType(_) => {
                abort! {
                    Span::call_site(),
                    "unexpected capability: {}, expected one of: Default, Example, Inline, XmlAttr, Format, WriteOnly, ReadOnly, Title", "ValueType",
                }
            }
        };

        tokens.extend(capability)
    }
}

pub struct CapabilityRef<'c, T: Name + ToTokens>(pub &'c T);

impl<T> ToTokens for CapabilityRef<'_, T>
where
    T: Name + ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let capability = self.0;
        let name_ident = format_ident!("{}", T::get_name());

        tokens.extend(quote! {
            .#name_ident(Some(#capability))
        })
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Example(AnyValue);

impl Parse for Example {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        parse_utils::parse_next(input, || AnyValue::parse_any(input)).map(Self)
    }
}

impl ToTokens for Example {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

name!(Example = "example");

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Default(AnyValue);

impl Parse for Default {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        parse_utils::parse_next(input, || AnyValue::parse_any(input)).map(Self)
    }
}

impl ToTokens for Default {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

name!(Default = "default");

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Inline(bool);

impl Parse for Inline {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for Inline {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

name!(Inline = "inline");

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct XmlAttr(schema::xml::XmlAttr);

impl XmlAttr {
    /// Split [`XmlAttr`] for [`GenericType::Vec`] returning tuple of [`XmlAttr`]s where first
    /// one is for a vec and second one is for object field.
    pub fn split_for_vec(&mut self, type_tree: &TypeTree) -> (Option<XmlAttr>, Option<XmlAttr>) {
        if matches!(type_tree.generic_type, Some(GenericType::Vec)) {
            let mut value_xml = mem::take(self);
            let vec_xml = schema::xml::XmlAttr::with_wrapped(
                mem::take(&mut value_xml.0.is_wrapped),
                mem::take(&mut value_xml.0.wrap_name),
            );

            (Some(XmlAttr(vec_xml)), Some(XmlAttr(value_xml.0)))
        } else {
            self.validate_xml(&self.0);

            (None, Some(mem::take(self)))
        }
    }

    #[inline]
    fn validate_xml(&self, xml: &schema::xml::XmlAttr) {
        if let Some(wrapped_ident) = xml.is_wrapped.as_ref() {
            abort! {wrapped_ident, "cannot use `wrapped` attribute in non slice field type";
                help = "Try removing `wrapped` attribute or make your field `Vec`"
            }
        }
    }
}

impl Parse for XmlAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let xml;
        parenthesized!(xml in input);
        xml.parse::<schema::xml::XmlAttr>().map(Self)
    }
}

impl ToTokens for XmlAttr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

name!(XmlAttr = "xml");

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Format(SchemaFormat<'static>);

impl Parse for Format {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        parse_utils::parse_next(input, || input.parse::<SchemaFormat>()).map(Self)
    }
}

impl ToTokens for Format {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

name!(Format = "format");

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ValueType(syn::Type);

impl ValueType {
    /// Create [`TypeTree`] from current [`syn::Type`].
    pub fn as_type_tree(&self) -> TypeTree {
        TypeTree::from_type(&self.0)
    }
}

impl Parse for ValueType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        parse_utils::parse_next(input, || input.parse::<syn::Type>()).map(Self)
    }
}

// impl ToTokens for ValueType {
//     fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
//         tokens.extend(self.0.to_token_stream())
//     }
// }

name!(ValueType = "value_type");

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct WriteOnly(bool);

impl Parse for WriteOnly {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for WriteOnly {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

name!(WriteOnly = "write_only");

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ReadOnly(bool);

impl Parse for ReadOnly {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for ReadOnly {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

name!(ReadOnly = "read_only");

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Title(String);

impl Parse for Title {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        parse_utils::parse_next_literal_str(input).map(Self)
    }
}

impl ToTokens for Title {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

name!(Title = "title");

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Nullable(bool);

impl Parse for Nullable {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for Nullable {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

name!(Nullable = "nullable");

macro_rules! parse_capability_set {
    ($ident:ident as $( $capability:path ),*) => {
        {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Vec<crate::component::capabilities::Capability>> {
                let names = [$( <crate::component::capabilities::parse_capability_set!(@as_ident $capability) as crate::component::capabilities::Name>::get_name(), )* ];
                let mut capabilities = Vec::<crate::component::capabilities::Capability>::new();
                let attributes = names.join(", ");

                while !input.is_empty() {
                    let ident = input.parse::<syn::Ident>().map_err(|error| {
                        syn::Error::new(
                            error.span(),
                            format!("unexpected attribute, expected any of: {attributes}, {error}"),
                        )
                    })?;
                    let name = &*ident.to_string();

                    $(
                        if name == <crate::component::capabilities::parse_capability_set!(@as_ident $capability) as crate::component::capabilities::Name>::get_name() {
                            capabilities.push(crate::component::capabilities::Capability::parse_named::<$capability>(input, ident)?);
                            if !input.is_empty() {
                                input.parse::<syn::Token![,]>()?;
                            }
                            continue;
                        }
                    )*

                    if !names.contains(&name) {
                        return Err(syn::Error::new(ident.span(), format!("unexpected attribute: {name}, expected any of: {attributes}")))
                    }
                }

                Ok(capabilities)
            }

            CapabilitySet(parse($ident)?)
        }
    };
    (@as_ident $( $tt:tt )* ) => {
        $( $tt )*
    }
}

pub(crate) use parse_capability_set;

pub fn parse_capablities(
    attributes: &[Attribute],
    parser: impl FnOnce(ParseStream) -> syn::Result<CapabilitySet>,
) -> Option<CapabilitySet> {
    attributes
        .iter()
        .find(|attribute| attribute.path.get_ident().unwrap() == "schema")
        .map(|attribute| attribute.parse_args_with(parser).unwrap_or_abort())
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct CapabilitySet(pub Vec<Capability>);

impl CapabilitySet {
    /// Removes the found [`Capability`] from [`CapabilitySet`] and returns it.
    ///
    /// If capablity is not found by given operation `None` will be returned.
    pub fn pop_by(&mut self, op: impl FnMut(&Capability) -> bool) -> Option<Capability> {
        self.0
            .iter()
            .position(op)
            .map(|index| self.0.swap_remove(index))
    }
}

impl ToTokens for CapabilitySet {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for capability in &self.0 {
            capability.to_tokens(tokens)
        }
        // tokens.extend(self.0.iter().map(|capability| capability.to_token_stream()))
    }
}

impl std::ops::Deref for CapabilitySet {
    type Target = [Capability];

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl DerefMut for CapabilitySet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut()
    }
}

impl FromIterator<Capability> for CapabilitySet {
    fn from_iter<T: IntoIterator<Item = Capability>>(iter: T) -> Self {
        Self(iter.into_iter().collect::<Vec<_>>())
    }
}
