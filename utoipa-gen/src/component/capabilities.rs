use proc_macro2::{Ident, Span};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{parenthesized, parse::Parse};

use crate::{parse_utils, schema_type::SchemaFormat, AnyValue};

use super::schema;

trait Name {
    fn get_name() -> &'static str;
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
}

impl Capability {
    fn parse_named<T: Name>(input: syn::parse::ParseStream) -> syn::Result<Self> {
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
            _unexpected => Err(syn::Error::new(Span::call_site(), format!("unexpected name: {}, expected one of: default, example, inline, xml, format, value_type, write_only, read_only, title", _unexpected))),
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
            _unexpected => {
                abort! {
                    Span::call_site(),
                    "unexpected capability: {}, expected one of: Default, Example, Inline, XmlAttr, Format, WriteOnly, ReadOnly, Title", stringify!(_unexpected),
                }
            }
        };

        tokens.extend(capability)
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

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct XmlAttr(schema::xml::XmlAttr);

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

macro_rules! parse_capability_set {
    ($ident:ident as $( $capability:ident ),*) => {
        {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Vec<Capability>> {
                let names = [$( $capability::get_name(), )* ];
                let mut capabilities = Vec::<Capability>::new();
                let attributes = names.join(", ");
                let error_message = format!("unexpected attribute, expected any of: {attributes}");

                while !input.is_empty() {
                    let ident = input.parse::<Ident>().map_err(|error| {
                        Error::new(
                            error.span(),
                            format!("unexpected attribute, expected any of: {attributes}, {error}"),
                        )
                    })?;
                    let name = &*ident.to_string();

                    $(
                        if name == $capability::get_name() {
                            capabilities.push(Capability::parse_named::<$capability>(input)?);
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
}

pub(crate) use parse_capability_set;

pub struct CapabilitySet(Vec<Capability>);

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

impl FromIterator<Capability> for CapabilitySet {
    fn from_iter<T: IntoIterator<Item = Capability>>(iter: T) -> Self {
        Self(iter.into_iter().collect::<Vec<_>>())
    }
}
