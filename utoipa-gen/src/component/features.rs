use std::mem;

use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{parenthesized, parse::Parse, LitStr};

use crate::{
    parse_utils,
    path::parameter::{self, ParameterStyle},
    schema_type::SchemaFormat,
    AnyValue,
};

use super::{schema, serde::RenameRule, GenericType, TypeTree};

pub trait Name {
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
#[derive(Clone)]
pub enum Feature {
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
    Rename(Rename),
    RenameAll(RenameAll),
    Style(Style),
    AllowReserve(AllowReserved),
    Explode(Explode),
    ParameterIn(ParameterIn),
    IntoParamsNames(Names),
}

impl Feature {
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
            "rename" => Rename::parse(input).map(Self::Rename),
            "rename_all" => RenameAll::parse(input).map(Self::RenameAll),
            "style" => Style::parse(input).map(Self::Style),
            "allow_reserved" => AllowReserved::parse(input).map(Self::AllowReserve),
            "explode" => Explode::parse(input).map(Self::Explode),
            "parameter_in" => ParameterIn::parse(input).map(Self::ParameterIn),
            "names" => Names::parse(input).map(Self::IntoParamsNames),
            _unexpected => Err(syn::Error::new(ident.span(), format!("unexpected name: {}, expected one of: default, example, inline, xml, format, value_type, write_only, read_only, title", _unexpected))),
        }
    }
}

impl ToTokens for Feature {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let feature = match &self {
            Feature::Default(default) => quote! { .default(Some(#default)) },
            Feature::Example(example) => quote! { .example(Some(#example)) },
            Feature::XmlAttr(xml) => quote! { .xml(Some(#xml)) },
            Feature::Format(format) => quote! { .format(Some(#format)) },
            Feature::WriteOnly(write_only) => quote! { .write_only(Some(#write_only)) },
            Feature::ReadOnly(read_only) => quote! { .read_only(Some(#read_only)) },
            Feature::Title(title) => quote! { .title(Some(#title)) },
            Feature::Nullable(nullable) => quote! { .nullable(#nullable) },
            Feature::Rename(rename) => rename.to_token_stream(),
            Feature::Style(style) => quote! { .style(Some(#style)) },
            Feature::ParameterIn(parameter_in) => quote! { .parameter_in(#parameter_in) },
            Feature::AllowReserve(allow_reserved) => {
                quote! { .allow_reserved(Some(#allow_reserved)) }
            }
            Feature::Explode(explode) => quote! { .explode(Some(#explode)) },
            Feature::RenameAll(_) => {
                abort! {
                    Span::call_site(),
                    "RenameAll feature does not support `ToTokens`"
                }
            }
            Feature::ValueType(_) => {
                abort! {
                    Span::call_site(),
                    "ValueType feature does not support `ToTokens`";
                    help = "ValueType is supposed to be used with `TypeTree` in same manner as a resolved struct/field type.";
                }
            }
            Feature::Inline(_) => {
                abort! {
                    Span::call_site(),
                    "Inline feature does not support `ToTokens`"
                }
            }
            Feature::IntoParamsNames(_) => {
                abort! {
                    Span::call_site(),
                    "Names feature does not support `ToTokens`";
                    help = "Names is only used with IntoParams to artificially give names for unnamed struct type `IntoParams`."
                }
            }
        };

        tokens.extend(feature)
    }
}

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Inline(bool);

impl Parse for Inline {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

name!(Inline = "inline");

#[derive(Default, Clone)]
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

            (Some(XmlAttr(vec_xml)), Some(value_xml))
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

#[derive(Clone)]
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

#[derive(Clone)]
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

name!(ValueType = "value_type");

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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

#[derive(Clone)]
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

#[derive(Clone, Copy)]
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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Rename(String);

impl Rename {
    pub fn into_value(self) -> String {
        self.0
    }
}

impl Parse for Rename {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        parse_utils::parse_next_literal_str(input).map(Self)
    }
}

impl ToTokens for Rename {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

name!(Rename = "rename");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct RenameAll(RenameRule);

impl RenameAll {
    pub fn as_rename_rule(&self) -> &RenameRule {
        &self.0
    }
}

impl Parse for RenameAll {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let litstr = parse_utils::parse_next(input, || input.parse::<LitStr>())?;

        litstr
            .value()
            .parse::<RenameRule>()
            .map_err(|error| syn::Error::new(litstr.span(), error.to_string()))
            .map(Self)
    }
}

name!(RenameAll = "rename_all");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Style(ParameterStyle);

impl From<ParameterStyle> for Style {
    fn from(style: ParameterStyle) -> Self {
        Self(style)
    }
}

impl Parse for Style {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        parse_utils::parse_next(input, || input.parse::<ParameterStyle>().map(Self))
    }
}

impl ToTokens for Style {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

name!(Style = "style");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct AllowReserved(bool);

impl Parse for AllowReserved {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for AllowReserved {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

name!(AllowReserved = "allow_reserved");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Explode(bool);

impl Parse for Explode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for Explode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

name!(Explode = "explode");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct ParameterIn(parameter::ParameterIn);

impl Parse for ParameterIn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        parse_utils::parse_next(input, || input.parse::<parameter::ParameterIn>().map(Self))
    }
}

impl ToTokens for ParameterIn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

name!(ParameterIn = "parameter_in");

/// Specify names of unnamed fields with `names(...) attribute for `IntoParams` derive.
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Names(Vec<String>);

impl Names {
    pub fn into_values(self) -> Vec<String> {
        self.0
    }
}

impl Parse for Names {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(
            parse_utils::parse_punctuated_within_parenthesis::<LitStr>(input)?
                .iter()
                .map(LitStr::value)
                .collect(),
        ))
    }
}

name!(Names = "names");

macro_rules! parse_features {
    ($ident:ident as $( $feature:path ),*) => {
        {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Vec<crate::component::features::Feature>> {
                let names = [$( <crate::component::features::parse_features!(@as_ident $feature) as crate::component::features::Name>::get_name(), )* ];
                let mut features = Vec::<crate::component::features::Feature>::new();
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
                        if name == <crate::component::features::parse_features!(@as_ident $feature) as crate::component::features::Name>::get_name() {
                            features.push(crate::component::features::Feature::parse_named::<$feature>(input, ident)?);
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

                Ok(features)
            }

            parse($ident)?
        }
    };
    (@as_ident $( $tt:tt )* ) => {
        $( $tt )*
    }
}

pub(crate) use parse_features;

pub trait IsInline {
    fn is_inline(&self) -> bool;
}

impl IsInline for Vec<Feature> {
    fn is_inline(&self) -> bool {
        self.iter()
            .find_map(|feature| match feature {
                Feature::Inline(inline) => Some(inline),
                _ => None,
            })
            .is_some()
    }
}

pub trait ToTokensExt {
    fn to_token_stream(&self) -> TokenStream;
}

impl ToTokensExt for Vec<Feature> {
    fn to_token_stream(&self) -> TokenStream {
        self.iter().fold(TokenStream::new(), |mut tokens, item| {
            item.to_tokens(&mut tokens);
            tokens
        })
    }
}

pub trait FeaturesExt {
    fn pop_by(&mut self, op: impl FnMut(&Feature) -> bool) -> Option<Feature>;

    fn find_value_type_feature_as_value_type(&mut self) -> Option<super::features::ValueType>;

    fn find_rename_feature_as_rename(&mut self) -> Option<Rename>;
}

impl FeaturesExt for Vec<Feature> {
    fn pop_by(&mut self, op: impl FnMut(&Feature) -> bool) -> Option<Feature> {
        self.iter()
            .position(op)
            .map(|index| self.swap_remove(index))
    }

    fn find_value_type_feature_as_value_type(&mut self) -> Option<super::features::ValueType> {
        self.pop_by(|feature| matches!(feature, Feature::ValueType(_)))
            .and_then(|feature| match feature {
                Feature::ValueType(value_type) => Some(value_type),
                _ => None,
            })
    }

    fn find_rename_feature_as_rename(&mut self) -> Option<Rename> {
        self.pop_by(|feature| matches!(feature, Feature::Rename(_)))
            .and_then(|feature| match feature {
                Feature::Rename(rename) => Some(rename),
                _ => None,
            })
    }
}

impl FeaturesExt for Option<Vec<Feature>> {
    fn pop_by(&mut self, op: impl FnMut(&Feature) -> bool) -> Option<Feature> {
        self.as_mut().and_then(|features| features.pop_by(op))
    }

    fn find_value_type_feature_as_value_type(&mut self) -> Option<super::features::ValueType> {
        self.as_mut()
            .and_then(|features| features.find_value_type_feature_as_value_type())
    }

    fn find_rename_feature_as_rename(&mut self) -> Option<Rename> {
        self.as_mut()
            .and_then(|features| features.find_rename_feature_as_rename())
    }
}

macro_rules! pop_feature {
    ($features:ident => $value:pat_param) => {{
        $features.pop_by(|feature| matches!(feature, $value))
    }};
}

pub(crate) use pop_feature;

pub trait IntoInner<T> {
    fn into_inner(self) -> T;
}

macro_rules! impl_into_inner {
    ($ident:ident) => {
        impl crate::component::features::IntoInner<Vec<Feature>> for $ident {
            fn into_inner(self) -> Vec<Feature> {
                self.0
            }
        }

        impl crate::component::features::IntoInner<Option<Vec<Feature>>> for Option<$ident> {
            fn into_inner(self) -> Option<Vec<Feature>> {
                self.map(crate::component::features::IntoInner::into_inner)
            }
        }
    };
}

pub(crate) use impl_into_inner;
