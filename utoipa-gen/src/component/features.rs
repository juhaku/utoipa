use std::{fmt::Display, mem, str::FromStr};

use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    LitFloat, LitInt, LitStr, TypePath,
};

use crate::{
    parse_utils,
    path::parameter::{self, ParameterStyle},
    schema_type::{SchemaFormat, SchemaType},
    AnyValue,
};

use super::{schema, serde::RenameRule, GenericType, TypeTree};

/// Parse `LitInt` from parse stream
fn parse_integer<T: FromStr + Display>(input: ParseStream) -> syn::Result<T>
where
    <T as FromStr>::Err: Display,
{
    parse_utils::parse_next(input, || input.parse::<LitInt>()?.base10_parse())
}

/// Parse any `number`. Tries to parse `LitInt` or `LitFloat` from parse stream.
fn parse_number<T>(input: ParseStream) -> syn::Result<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    parse_utils::parse_next(input, || {
        let lookup = input.lookahead1();
        if lookup.peek(LitInt) {
            input.parse::<LitInt>()?.base10_parse()
        } else if lookup.peek(LitFloat) {
            input.parse::<LitFloat>()?.base10_parse()
        } else {
            Err(lookup.error())
        }
    })
}

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

        impl Display for $ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let name = <Self as Name>::get_name();
                write!(f, "{name}")
            }
        }
    };
}

trait ParseWithIdent {
    fn parse_with_ident(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized;
}

/// Define whether [`Feature`] variant is validatable or not
pub trait Validatable {
    fn is_validatable(&self) -> bool {
        false
    }
}

pub trait Validate: Validatable {
    /// Perform validation check against schema type.
    fn validate(&self, validator: impl Validator);
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
    AllowReserved(AllowReserved),
    Explode(Explode),
    ParameterIn(ParameterIn),
    IntoParamsNames(Names),
    MultipleOf(MultipleOf),
    Maximum(Maximum),
    Minimum(Minimum),
    ExclusiveMaximum(ExclusiveMaximum),
    ExclusiveMinimum(ExclusiveMinimum),
    MaxLength(MaxLength),
    MinLength(MinLength),
    Pattern(Pattern),
    MaxItems(MaxItems),
    MinItems(MinItems),
    MaxProperties(MaxProperties),
    MinProperties(MinProperties),
    SchemaWith(SchemaWith),
}

impl Feature {
    pub fn parse_named<T: Name>(input: syn::parse::ParseStream, ident: Ident) -> syn::Result<Self> {
        let name = T::get_name();

        const ALLOWED_NAMES: [&str; 30] = [
            "default",
            "example",
            "inline",
            "xml",
            "format",
            "value_type",
            "write_only",
            "read_only",
            "title",
            "nullable",
            "rename",
            "rename_all",
            "style",
            "allow_reserved",
            "explode",
            "parameter_in",
            "names",
            "multiple_of",
            "maximum",
            "minimum",
            "exclusive_maximum",
            "exclusive_minimum",
            "max_length",
            "min_length",
            "pattern",
            "max_items",
            "min_items",
            "max_properties",
            "min_properties",
            "schema_with",
        ];

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
            "allow_reserved" => AllowReserved::parse(input).map(Self::AllowReserved),
            "explode" => Explode::parse(input).map(Self::Explode),
            "parameter_in" => ParameterIn::parse(input).map(Self::ParameterIn),
            "names" => Names::parse(input).map(Self::IntoParamsNames),
            "multiple_of" => MultipleOf::parse_with_ident(input, ident).map(Self::MultipleOf),
            "maximum" => Maximum::parse_with_ident(input, ident).map(Self::Maximum),
            "minimum" => Minimum::parse_with_ident(input, ident).map(Self::Minimum),
            "exclusive_maximum" => {
                ExclusiveMaximum::parse_with_ident(input, ident).map(Self::ExclusiveMaximum)
            }
            "exclusive_minimum" => {
                ExclusiveMinimum::parse_with_ident(input, ident).map(Self::ExclusiveMinimum)
            }
            "max_length" => MaxLength::parse_with_ident(input, ident).map(Self::MaxLength),
            "min_length" => MinLength::parse_with_ident(input, ident).map(Self::MinLength),
            "max_items" => MaxItems::parse_with_ident(input, ident).map(Self::MaxItems),
            "min_items" => MinItems::parse_with_ident(input, ident).map(Self::MinItems),
            "pattern" => Pattern::parse_with_ident(input, ident).map(Self::Pattern),
            "max_properties" => {
                MaxProperties::parse_with_ident(input, ident).map(Self::MaxProperties)
            }
            "min_properties" => {
                MinProperties::parse_with_ident(input, ident).map(Self::MinProperties)
            }
            "schema_with" => SchemaWith::parse(input).map(Self::SchemaWith),
            _unexpected => Err(syn::Error::new(
                ident.span(),
                format!(
                    "unexpected name: `{}`, cannot parse named, expected one of: {}",
                    _unexpected,
                    ALLOWED_NAMES.join(", ")
                ),
            )),
        }
    }

    pub fn validate(&self, schema_type: &SchemaType, type_tree: &TypeTree) {
        match self {
            Feature::MultipleOf(multiple_of) => multiple_of.validate(
                ValidatorChain::new(&IsNumber(schema_type)).next(&AboveZeroF64(multiple_of.0)),
            ),
            Feature::Maximum(maximum) => maximum.validate(IsNumber(schema_type)),
            Feature::Minimum(minimum) => minimum.validate(IsNumber(schema_type)),
            Feature::ExclusiveMaximum(exlusive_maximum) => {
                exlusive_maximum.validate(IsNumber(schema_type))
            }
            Feature::ExclusiveMinimum(exclusive_minimum) => {
                exclusive_minimum.validate(IsNumber(schema_type))
            }
            Feature::MaxLength(max_length) => max_length.validate(
                ValidatorChain::new(&IsString(schema_type)).next(&AboveZeroUsize(max_length.0)),
            ),
            Feature::MinLength(min_length) => min_length.validate(
                ValidatorChain::new(&IsString(schema_type)).next(&AboveZeroUsize(min_length.0)),
            ),
            Feature::Pattern(pattern) => pattern.validate(IsString(schema_type)),
            Feature::MaxItems(max_items) => max_items.validate(
                ValidatorChain::new(&AboveZeroUsize(max_items.0)).next(&IsVec(type_tree)),
            ),
            Feature::MinItems(min_items) => min_items.validate(
                ValidatorChain::new(&AboveZeroUsize(min_items.0)).next(&IsVec(type_tree)),
            ),
            _unsupported_variant => {
                const SUPPORTED_VARIANTS: [&str; 10] = [
                    "multiple_of",
                    "maximum",
                    "minimum",
                    "exclusive_maximum",
                    "exclusive_minimum",
                    "max_length",
                    "min_length",
                    "pattern",
                    "max_items",
                    "min_items",
                ];
                panic!(
                    "Unsupported variant: `{variant}` for Validate::validate, expected one of: {variants}",
                    variant = _unsupported_variant,
                    variants = SUPPORTED_VARIANTS.join(", ")
                )
            }
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
            Feature::MultipleOf(multiple_of) => quote! { .multiple_of(Some(#multiple_of)) },
            Feature::AllowReserved(allow_reserved) => {
                quote! { .allow_reserved(Some(#allow_reserved)) }
            }
            Feature::Explode(explode) => quote! { .explode(Some(#explode)) },
            Feature::Maximum(maximum) => quote! { .maximum(Some(#maximum)) },
            Feature::Minimum(minimum) => quote! { .minimum(Some(#minimum)) },
            Feature::ExclusiveMaximum(exclusive_maximum) => {
                quote! { .exclusive_maximum(Some(#exclusive_maximum)) }
            }
            Feature::ExclusiveMinimum(exclusive_minimum) => {
                quote! { .exclusive_minimum(Some(#exclusive_minimum)) }
            }
            Feature::MaxLength(max_length) => quote! { .max_length(Some(#max_length)) },
            Feature::MinLength(min_length) => quote! { .min_length(Some(#min_length)) },
            Feature::Pattern(pattern) => quote! { .pattern(Some(#pattern)) },
            Feature::MaxItems(max_items) => quote! { .max_items(Some(#max_items)) },
            Feature::MinItems(min_items) => quote! { .min_items(Some(#min_items)) },
            Feature::MaxProperties(max_properties) => {
                quote! { .max_properties(Some(#max_properties)) }
            }
            Feature::MinProperties(min_properties) => {
                quote! { .max_properties(Some(#min_properties)) }
            }
            Feature::SchemaWith(with_schema) => with_schema.to_token_stream(),
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

impl Display for Feature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Feature::Default(default) => default.fmt(f),
            Feature::Example(example) => example.fmt(f),
            Feature::XmlAttr(xml) => xml.fmt(f),
            Feature::Format(format) => format.fmt(f),
            Feature::WriteOnly(write_only) => write_only.fmt(f),
            Feature::ReadOnly(read_only) => read_only.fmt(f),
            Feature::Title(title) => title.fmt(f),
            Feature::Nullable(nullable) => nullable.fmt(f),
            Feature::Rename(rename) => rename.fmt(f),
            Feature::Style(style) => style.fmt(f),
            Feature::ParameterIn(parameter_in) => parameter_in.fmt(f),
            Feature::AllowReserved(allow_reserved) => allow_reserved.fmt(f),
            Feature::Explode(explode) => explode.fmt(f),
            Feature::RenameAll(rename_all) => rename_all.fmt(f),
            Feature::ValueType(value_type) => value_type.fmt(f),
            Feature::Inline(inline) => inline.fmt(f),
            Feature::IntoParamsNames(names) => names.fmt(f),
            Feature::MultipleOf(multiple_of) => multiple_of.fmt(f),
            Feature::Maximum(maximum) => maximum.fmt(f),
            Feature::Minimum(minimum) => minimum.fmt(f),
            Feature::ExclusiveMaximum(exclusive_maximum) => exclusive_maximum.fmt(f),
            Feature::ExclusiveMinimum(exclusive_minimum) => exclusive_minimum.fmt(f),
            Feature::MaxLength(max_length) => max_length.fmt(f),
            Feature::MinLength(min_length) => min_length.fmt(f),
            Feature::Pattern(pattern) => pattern.fmt(f),
            Feature::MaxItems(max_items) => max_items.fmt(f),
            Feature::MinItems(min_items) => min_items.fmt(f),
            Feature::MaxProperties(max_properties) => max_properties.fmt(f),
            Feature::MinProperties(min_properties) => min_properties.fmt(f),
            Feature::SchemaWith(with_schema) => with_schema.fmt(f),
        }
    }
}

impl Validatable for Feature {
    fn is_validatable(&self) -> bool {
        match &self {
            Feature::Default(default) => default.is_validatable(),
            Feature::Example(example) => example.is_validatable(),
            Feature::XmlAttr(xml) => xml.is_validatable(),
            Feature::Format(format) => format.is_validatable(),
            Feature::WriteOnly(write_only) => write_only.is_validatable(),
            Feature::ReadOnly(read_only) => read_only.is_validatable(),
            Feature::Title(title) => title.is_validatable(),
            Feature::Nullable(nullable) => nullable.is_validatable(),
            Feature::Rename(rename) => rename.is_validatable(),
            Feature::Style(style) => style.is_validatable(),
            Feature::ParameterIn(parameter_in) => parameter_in.is_validatable(),
            Feature::AllowReserved(allow_reserved) => allow_reserved.is_validatable(),
            Feature::Explode(explode) => explode.is_validatable(),
            Feature::RenameAll(rename_all) => rename_all.is_validatable(),
            Feature::ValueType(value_type) => value_type.is_validatable(),
            Feature::Inline(inline) => inline.is_validatable(),
            Feature::IntoParamsNames(names) => names.is_validatable(),
            Feature::MultipleOf(multiple_of) => multiple_of.is_validatable(),
            Feature::Maximum(maximum) => maximum.is_validatable(),
            Feature::Minimum(minimum) => minimum.is_validatable(),
            Feature::ExclusiveMaximum(exclusive_maximum) => exclusive_maximum.is_validatable(),
            Feature::ExclusiveMinimum(exclusive_minimum) => exclusive_minimum.is_validatable(),
            Feature::MaxLength(max_length) => max_length.is_validatable(),
            Feature::MinLength(min_length) => min_length.is_validatable(),
            Feature::Pattern(pattern) => pattern.is_validatable(),
            Feature::MaxItems(max_items) => max_items.is_validatable(),
            Feature::MinItems(min_items) => min_items.is_validatable(),
            Feature::MaxProperties(max_properties) => max_properties.is_validatable(),
            Feature::MinProperties(min_properties) => min_properties.is_validatable(),
            Feature::SchemaWith(with_schema) => with_schema.is_validatable(),
        }
    }
}

macro_rules! is_validatable {
    ( $( $ident:ident => $validatable:literal ),* ) => {
        $(
            impl Validatable for $ident {
                fn is_validatable(&self) -> bool {
                    $validatable
                }
            }
        )*
    };
}

is_validatable! {
    Default => false,
    Example => false,
    XmlAttr => false,
    Format => false,
    WriteOnly => false,
    ReadOnly => false,
    Title => false,
    Nullable => false,
    Rename => false,
    Style => false,
    ParameterIn => false,
    AllowReserved => false,
    Explode => false,
    RenameAll => false,
    ValueType => false,
    Inline => false,
    Names => false,
    MultipleOf => true,
    Maximum => true,
    Minimum => true,
    ExclusiveMaximum => true,
    ExclusiveMinimum => true,
    MaxLength => true,
    MinLength => true,
    Pattern => true,
    MaxItems => true,
    MinItems => true,
    MaxProperties => false,
    MinProperties => false,
    SchemaWith => false
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

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct MultipleOf(f64, Ident);

impl Validate for MultipleOf {
    fn validate(&self, validator: impl Validator) {
        if let Err(error) = validator.is_valid() {
            abort! {self.1, "`multiple_of` error: {}", error;
                help = "See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-multipleof`"
            }
        };
    }
}

impl ParseWithIdent for MultipleOf {
    fn parse_with_ident(input: ParseStream, ident: Ident) -> syn::Result<Self> {
        parse_number(input).map(|multiple_of| Self(multiple_of, ident))
    }
}

impl ToTokens for MultipleOf {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

name!(MultipleOf = "multiple_of");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Maximum(f64, Ident);

impl Validate for Maximum {
    fn validate(&self, validator: impl Validator) {
        if let Err(error) = validator.is_valid() {
            abort! {self.1, "`maximum` error: {}", error;
                help = "See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-maximum`"
            }
        }
    }
}

impl ParseWithIdent for Maximum {
    fn parse_with_ident(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_number(input).map(|maximum| Self(maximum, ident))
    }
}

impl ToTokens for Maximum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

name!(Maximum = "maximum");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Minimum(f64, Ident);

impl Validate for Minimum {
    fn validate(&self, validator: impl Validator) {
        if let Err(error) = validator.is_valid() {
            abort! {self.1, "`minimum` error: {}", error;
                help = "See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-minimum`"
            }
        }
    }
}

impl ParseWithIdent for Minimum {
    fn parse_with_ident(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_number(input).map(|maximum| Self(maximum, ident))
    }
}

impl ToTokens for Minimum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

name!(Minimum = "minimum");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct ExclusiveMaximum(f64, Ident);

impl Validate for ExclusiveMaximum {
    fn validate(&self, validator: impl Validator) {
        if let Err(error) = validator.is_valid() {
            abort! {self.1, "`exclusive_maximum` error: {}", error;
                help = "See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-exclusivemaximum`"
            }
        }
    }
}

impl ParseWithIdent for ExclusiveMaximum {
    fn parse_with_ident(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_number(input).map(|max| Self(max, ident))
    }
}

impl ToTokens for ExclusiveMaximum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

name!(ExclusiveMaximum = "exclusive_maximum");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct ExclusiveMinimum(f64, Ident);

impl Validate for ExclusiveMinimum {
    fn validate(&self, validator: impl Validator) {
        if let Err(error) = validator.is_valid() {
            abort! {self.1, "`exclusive_minimum` error: {}", error;
                help = "See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-exclusiveminimum`"
            }
        }
    }
}

impl ParseWithIdent for ExclusiveMinimum {
    fn parse_with_ident(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_number(input).map(|min| Self(min, ident))
    }
}

impl ToTokens for ExclusiveMinimum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

name!(ExclusiveMinimum = "exclusive_minimum");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct MaxLength(usize, Ident);

impl Validate for MaxLength {
    fn validate(&self, validator: impl Validator) {
        if let Err(error) = validator.is_valid() {
            abort! {self.1, "`max_length` error: {}", error;
                help = "See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-maxlength`"
            }
        }
    }
}

impl ParseWithIdent for MaxLength {
    fn parse_with_ident(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_integer(input).map(|max_length| Self(max_length, ident))
    }
}

impl ToTokens for MaxLength {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

name!(MaxLength = "max_length");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct MinLength(usize, Ident);

impl Validate for MinLength {
    fn validate(&self, validator: impl Validator) {
        if let Err(error) = validator.is_valid() {
            abort! {self.1, "`min_length` error: {}", error;
                help = "See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-minlength`"
            }
        }
    }
}

impl ParseWithIdent for MinLength {
    fn parse_with_ident(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_integer(input).map(|max_length| Self(max_length, ident))
    }
}

impl ToTokens for MinLength {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

name!(MinLength = "min_length");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Pattern(String, Ident);

impl Validate for Pattern {
    fn validate(&self, validator: impl Validator) {
        if let Err(error) = validator.is_valid() {
            abort! {self.1, "`pattern` error: {}", error;
                help = "See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-pattern`"
            }
        }
    }
}

impl ParseWithIdent for Pattern {
    fn parse_with_ident(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_utils::parse_next(input, || input.parse::<LitStr>())
            .map(|pattern| Self(pattern.value(), ident))
    }
}

impl ToTokens for Pattern {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

name!(Pattern = "pattern");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct MaxItems(usize, Ident);

impl Validate for MaxItems {
    fn validate(&self, validator: impl Validator) {
        if let Err(error) = validator.is_valid() {
            abort! {self.1, "`max_items` error: {}", error;
                help = "See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-maxitems"
            }
        }
    }
}

impl ParseWithIdent for MaxItems {
    fn parse_with_ident(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_number(input).map(|max_items| Self(max_items, ident))
    }
}

impl ToTokens for MaxItems {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

name!(MaxItems = "max_items");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct MinItems(usize, Ident);

impl Validate for MinItems {
    fn validate(&self, validator: impl Validator) {
        if let Err(error) = validator.is_valid() {
            abort! {self.1, "`min_items` error: {}", error;
                help = "See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-minitems"
            }
        }
    }
}

impl ParseWithIdent for MinItems {
    fn parse_with_ident(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_number(input).map(|max_items| Self(max_items, ident))
    }
}

impl ToTokens for MinItems {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

name!(MinItems = "min_items");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct MaxProperties(usize, Ident);

impl ParseWithIdent for MaxProperties {
    fn parse_with_ident(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_integer(input).map(|max_properties| Self(max_properties, ident))
    }
}

impl ToTokens for MaxProperties {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

name!(MaxProperties = "max_properties");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct MinProperties(usize, Ident);

impl ParseWithIdent for MinProperties {
    fn parse_with_ident(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_integer(input).map(|min_properties| Self(min_properties, ident))
    }
}

impl ToTokens for MinProperties {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

name!(MinProperties = "min_properties");
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct SchemaWith(TypePath);

impl Parse for SchemaWith {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        parse_utils::parse_next(input, || input.parse::<TypePath>().map(Self))
    }
}

impl ToTokens for SchemaWith {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let path = &self.0;
        tokens.extend(quote! {
            #path()
        })
    }
}

name!(SchemaWith = "schema_with");

pub trait Validator {
    fn is_valid(&self) -> Result<(), &'static str>;
}

pub struct IsNumber<'a>(pub &'a SchemaType<'a>);

impl Validator for IsNumber<'_> {
    fn is_valid(&self) -> Result<(), &'static str> {
        if self.0.is_number() {
            Ok(())
        } else {
            Err("can only be used with `number` type")
        }
    }
}

pub struct IsString<'a>(&'a SchemaType<'a>);

impl Validator for IsString<'_> {
    fn is_valid(&self) -> Result<(), &'static str> {
        if self.0.is_string() {
            Ok(())
        } else {
            Err("can only be used with `string` type")
        }
    }
}

pub struct IsInteger<'a>(&'a SchemaType<'a>);

impl Validator for IsInteger<'_> {
    fn is_valid(&self) -> Result<(), &'static str> {
        if self.0.is_integer() {
            Ok(())
        } else {
            Err("can only be used with `integer` type")
        }
    }
}

pub struct IsVec<'a>(&'a TypeTree<'a>);

impl Validator for IsVec<'_> {
    fn is_valid(&self) -> Result<(), &'static str> {
        if self.0.generic_type == Some(GenericType::Vec) {
            Ok(())
        } else {
            Err("can only be used with `Vec`, `array` or `slice` types")
        }
    }
}

pub struct AboveZeroUsize(usize);

impl Validator for AboveZeroUsize {
    fn is_valid(&self) -> Result<(), &'static str> {
        if self.0 != 0 {
            Ok(())
        } else {
            Err("can only be above zero value")
        }
    }
}

pub struct AboveZeroF64(f64);

impl Validator for AboveZeroF64 {
    fn is_valid(&self) -> Result<(), &'static str> {
        if self.0 > 0.0 {
            Ok(())
        } else {
            Err("can only be above zero value")
        }
    }
}

pub struct ValidatorChain<'c> {
    inner: &'c dyn Validator,
    next: Option<&'c dyn Validator>,
}

impl Validator for ValidatorChain<'_> {
    fn is_valid(&self) -> Result<(), &'static str> {
        self.inner.is_valid().and_then(|_| {
            if let Some(validator) = self.next.as_ref() {
                validator.is_valid()
            } else {
                // if there is no next validator consider it valid
                Ok(())
            }
        })
    }
}

impl<'c> ValidatorChain<'c> {
    pub fn new(validator: &'c dyn Validator) -> Self {
        Self {
            inner: validator,
            next: None,
        }
    }

    pub fn next(mut self, validator: &'c dyn Validator) -> Self {
        self.next = Some(validator);

        self
    }
}

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

    fn pop_value_type_feature(&mut self) -> Option<super::features::ValueType>;

    /// Pop [`Rename`] feature if exists in [`Vec<Feature>`] list.
    fn pop_rename_feature(&mut self) -> Option<Rename>;

    /// Pop [`RenameAll`] feature if exists in [`Vec<Feature>`] list.
    fn pop_rename_all_feature(&mut self) -> Option<RenameAll>;

    /// Extract [`XmlAttr`] feature for given `type_tree` if it has generic type [`GenericType::Vec`]
    fn extract_vec_xml_feature(&mut self, type_tree: &TypeTree) -> Option<Feature>;
}

impl FeaturesExt for Vec<Feature> {
    fn pop_by(&mut self, op: impl FnMut(&Feature) -> bool) -> Option<Feature> {
        self.iter()
            .position(op)
            .map(|index| self.swap_remove(index))
    }

    fn pop_value_type_feature(&mut self) -> Option<super::features::ValueType> {
        self.pop_by(|feature| matches!(feature, Feature::ValueType(_)))
            .and_then(|feature| match feature {
                Feature::ValueType(value_type) => Some(value_type),
                _ => None,
            })
    }

    fn pop_rename_feature(&mut self) -> Option<Rename> {
        self.pop_by(|feature| matches!(feature, Feature::Rename(_)))
            .and_then(|feature| match feature {
                Feature::Rename(rename) => Some(rename),
                _ => None,
            })
    }

    fn pop_rename_all_feature(&mut self) -> Option<RenameAll> {
        self.pop_by(|feature| matches!(feature, Feature::RenameAll(_)))
            .and_then(|feature| match feature {
                Feature::RenameAll(rename_all) => Some(rename_all),
                _ => None,
            })
    }

    fn extract_vec_xml_feature(&mut self, type_tree: &TypeTree) -> Option<Feature> {
        self.iter_mut().find_map(|feature| match feature {
            Feature::XmlAttr(xml_feature) => {
                let (vec_xml, value_xml) = xml_feature.split_for_vec(type_tree);

                // replace the original xml attribute with splitted value xml
                if let Some(mut xml) = value_xml {
                    mem::swap(xml_feature, &mut xml)
                }

                vec_xml.map(Feature::XmlAttr)
            }
            _ => None,
        })
    }
}

impl FeaturesExt for Option<Vec<Feature>> {
    fn pop_by(&mut self, op: impl FnMut(&Feature) -> bool) -> Option<Feature> {
        self.as_mut().and_then(|features| features.pop_by(op))
    }

    fn pop_value_type_feature(&mut self) -> Option<super::features::ValueType> {
        self.as_mut()
            .and_then(|features| features.pop_value_type_feature())
    }

    fn pop_rename_feature(&mut self) -> Option<Rename> {
        self.as_mut()
            .and_then(|features| features.pop_rename_feature())
    }

    fn pop_rename_all_feature(&mut self) -> Option<RenameAll> {
        self.as_mut()
            .and_then(|features| features.pop_rename_all_feature())
    }

    fn extract_vec_xml_feature(&mut self, type_tree: &TypeTree) -> Option<Feature> {
        self.as_mut()
            .and_then(|features| features.extract_vec_xml_feature(type_tree))
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
