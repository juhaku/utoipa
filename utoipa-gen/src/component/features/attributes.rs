use std::mem;

use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::{Error, LitStr, Token, TypePath, WherePredicate};

use crate::component::serde::RenameRule;
use crate::component::{schema, GenericType, TypeTree};
use crate::parse_utils::{LitBoolOrExprPath, LitStrOrExpr};
use crate::path::parameter::{self, ParameterStyle};
use crate::schema_type::KnownFormat;
use crate::{parse_utils, AnyValue, Array, Diagnostics};

use super::{impl_feature, Feature, Parse};
use quote::quote;

mod extensions;
pub use extensions::Extensions;

impl_feature! {
    #[derive(Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Default(pub(crate) Option<AnyValue>);
}

impl Default {
    pub fn new_default_trait(struct_ident: Ident, field_ident: syn::Member) -> Self {
        Self(Some(AnyValue::new_default_trait(struct_ident, field_ident)))
    }
}

impl Parse for Default {
    fn parse(input: syn::parse::ParseStream, _: proc_macro2::Ident) -> syn::Result<Self> {
        if input.peek(syn::Token![=]) {
            parse_utils::parse_next(input, || AnyValue::parse_any(input)).map(|any| Self(Some(any)))
        } else {
            Ok(Self(None))
        }
    }
}

impl ToTokens for Default {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match &self.0 {
            Some(inner) => tokens.extend(quote! {Some(#inner)}),
            None => tokens.extend(quote! {None}),
        }
    }
}

impl From<self::Default> for Feature {
    fn from(value: self::Default) -> Self {
        Feature::Default(value)
    }
}

impl_feature! {
    #[derive(Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Example(AnyValue);
}

impl Parse for Example {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_next(input, || AnyValue::parse_any(input)).map(Self)
    }
}

impl ToTokens for Example {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<Example> for Feature {
    fn from(value: Example) -> Self {
        Feature::Example(value)
    }
}

impl_feature! {
    #[derive(Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Examples(Vec<AnyValue>);
}

impl Parse for Examples {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        let examples;
        syn::parenthesized!(examples in input);

        Ok(Self(
            Punctuated::<AnyValue, Token![,]>::parse_terminated_with(
                &examples,
                AnyValue::parse_any,
            )?
            .into_iter()
            .collect(),
        ))
    }
}

impl ToTokens for Examples {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if !self.0.is_empty() {
            let examples = Array::Borrowed(&self.0).to_token_stream();
            examples.to_tokens(tokens);
        }
    }
}

impl From<Examples> for Feature {
    fn from(value: Examples) -> Self {
        Feature::Examples(value)
    }
}

impl_feature! {"xml" =>
    #[derive(Default, Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct XmlAttr(schema::xml::XmlAttr);
}

impl XmlAttr {
    /// Split [`XmlAttr`] for [`GenericType::Vec`] returning tuple of [`XmlAttr`]s where first
    /// one is for a vec and second one is for object field.
    pub fn split_for_vec(
        &mut self,
        type_tree: &TypeTree,
    ) -> Result<(Option<XmlAttr>, Option<XmlAttr>), Diagnostics> {
        if matches!(type_tree.generic_type, Some(GenericType::Vec)) {
            let mut value_xml = mem::take(self);
            let vec_xml = schema::xml::XmlAttr::with_wrapped(
                mem::take(&mut value_xml.0.is_wrapped),
                mem::take(&mut value_xml.0.wrap_name),
            );

            Ok((Some(XmlAttr(vec_xml)), Some(value_xml)))
        } else {
            self.validate_xml(&self.0)?;

            Ok((None, Some(mem::take(self))))
        }
    }

    #[inline]
    fn validate_xml(&self, xml: &schema::xml::XmlAttr) -> Result<(), Diagnostics> {
        if let Some(wrapped_ident) = xml.is_wrapped.as_ref() {
            Err(Diagnostics::with_span(
                wrapped_ident.span(),
                "cannot use `wrapped` attribute in non slice field type",
            )
            .help("Try removing `wrapped` attribute or make your field `Vec`"))
        } else {
            Ok(())
        }
    }
}

impl Parse for XmlAttr {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        let xml;
        syn::parenthesized!(xml in input);
        xml.parse::<schema::xml::XmlAttr>().map(Self)
    }
}

impl ToTokens for XmlAttr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<XmlAttr> for Feature {
    fn from(value: XmlAttr) -> Self {
        Feature::XmlAttr(value)
    }
}

impl_feature! {
    #[derive(Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Format(KnownFormat);
}

impl Parse for Format {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_next(input, || input.parse::<KnownFormat>()).map(Self)
    }
}

impl ToTokens for Format {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<Format> for Feature {
    fn from(value: Format) -> Self {
        Feature::Format(value)
    }
}

impl_feature! {
    #[derive(Clone, Copy)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct WriteOnly(bool);
}

impl Parse for WriteOnly {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for WriteOnly {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<WriteOnly> for Feature {
    fn from(value: WriteOnly) -> Self {
        Feature::WriteOnly(value)
    }
}

impl_feature! {
    #[derive(Clone, Copy)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct ReadOnly(bool);
}

impl Parse for ReadOnly {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for ReadOnly {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<ReadOnly> for Feature {
    fn from(value: ReadOnly) -> Self {
        Feature::ReadOnly(value)
    }
}

impl_feature! {
    #[derive(Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Title(String);
}

impl Parse for Title {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_next_literal_str(input).map(Self)
    }
}

impl ToTokens for Title {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<Title> for Feature {
    fn from(value: Title) -> Self {
        Feature::Title(value)
    }
}

impl_feature! {
    #[derive(Clone, Copy)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Nullable(bool);
}

impl Nullable {
    pub fn new() -> Self {
        Self(true)
    }

    pub fn value(&self) -> bool {
        self.0
    }

    pub fn into_schema_type_token_stream(self) -> proc_macro2::TokenStream {
        if self.0 {
            quote! {utoipa::openapi::schema::Type::Null}
        } else {
            proc_macro2::TokenStream::new()
        }
    }
}

impl Parse for Nullable {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for Nullable {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<Nullable> for Feature {
    fn from(value: Nullable) -> Self {
        Feature::Nullable(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct Rename(String);
}

impl Rename {
    pub fn into_value(self) -> String {
        self.0
    }
}

impl Parse for Rename {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_next_literal_str(input).map(Self)
    }
}

impl ToTokens for Rename {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<Rename> for Feature {
    fn from(value: Rename) -> Self {
        Feature::Rename(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct RenameAll(RenameRule);

}
impl RenameAll {
    pub fn as_rename_rule(&self) -> &RenameRule {
        &self.0
    }
}

impl Parse for RenameAll {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        let litstr = parse_utils::parse_next(input, || input.parse::<LitStr>())?;

        litstr
            .value()
            .parse::<RenameRule>()
            .map_err(|error| syn::Error::new(litstr.span(), error.to_string()))
            .map(Self)
    }
}

impl From<RenameAll> for Feature {
    fn from(value: RenameAll) -> Self {
        Feature::RenameAll(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct Style(ParameterStyle);
}

impl From<ParameterStyle> for Style {
    fn from(style: ParameterStyle) -> Self {
        Self(style)
    }
}

impl Parse for Style {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_next(input, || input.parse::<ParameterStyle>().map(Self))
    }
}

impl ToTokens for Style {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl From<Style> for Feature {
    fn from(value: Style) -> Self {
        Feature::Style(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct ParameterIn(parameter::ParameterIn);
}

impl ParameterIn {
    pub fn is_query(&self) -> bool {
        matches!(self.0, parameter::ParameterIn::Query)
    }
}

impl Parse for ParameterIn {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_next(input, || input.parse::<parameter::ParameterIn>().map(Self))
    }
}

impl ToTokens for ParameterIn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<ParameterIn> for Feature {
    fn from(value: ParameterIn) -> Self {
        Feature::ParameterIn(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct AllowReserved(bool);
}

impl Parse for AllowReserved {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for AllowReserved {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl From<AllowReserved> for Feature {
    fn from(value: AllowReserved) -> Self {
        Feature::AllowReserved(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct Explode(bool);
}

impl Parse for Explode {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for Explode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl From<Explode> for Feature {
    fn from(value: Explode) -> Self {
        Feature::Explode(value)
    }
}

impl_feature! {
    #[derive(Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct ValueType(syn::Type);
}

impl ValueType {
    /// Create [`TypeTree`] from current [`syn::Type`].
    pub fn as_type_tree(&self) -> Result<TypeTree, Diagnostics> {
        TypeTree::from_type(&self.0)
    }
}

impl Parse for ValueType {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_next(input, || input.parse::<syn::Type>()).map(Self)
    }
}

impl From<ValueType> for Feature {
    fn from(value: ValueType) -> Self {
        Feature::ValueType(value)
    }
}

impl_feature! {
    #[derive(Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Inline(pub(super) bool);
}

impl Parse for Inline {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl From<bool> for Inline {
    fn from(value: bool) -> Self {
        Inline(value)
    }
}

impl From<Inline> for Feature {
    fn from(value: Inline) -> Self {
        Feature::Inline(value)
    }
}

impl_feature! {"names" =>
    /// Specify names of unnamed fields with `names(...) attribute for `IntoParams` derive.
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct IntoParamsNames(Vec<String>);
}

impl IntoParamsNames {
    pub fn into_values(self) -> Vec<String> {
        self.0
    }
}

impl Parse for IntoParamsNames {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        Ok(Self(
            parse_utils::parse_comma_separated_within_parenthesis::<LitStr>(input)?
                .iter()
                .map(LitStr::value)
                .collect(),
        ))
    }
}

impl From<IntoParamsNames> for Feature {
    fn from(value: IntoParamsNames) -> Self {
        Feature::IntoParamsNames(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct SchemaWith(TypePath);
}

impl Parse for SchemaWith {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self> {
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

impl From<SchemaWith> for Feature {
    fn from(value: SchemaWith) -> Self {
        Feature::SchemaWith(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct Description(parse_utils::LitStrOrExpr);
}

impl Parse for Description {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        parse_utils::parse_next_literal_str_or_expr(input).map(Self)
    }
}

impl ToTokens for Description {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<String> for Description {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl From<Description> for Feature {
    fn from(value: Description) -> Self {
        Self::Description(value)
    }
}

impl_feature! {
    /// Deprecated feature parsed from macro attributes.
    ///
    /// This feature supports only syntax parsed from utoipa specific macro attributes, it does not
    /// support Rust `#[deprecated]` attribute.
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct Deprecated(bool);
}

impl Parse for Deprecated {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for Deprecated {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let deprecated: crate::Deprecated = self.0.into();
        deprecated.to_tokens(tokens);
    }
}

impl From<Deprecated> for Feature {
    fn from(value: Deprecated) -> Self {
        Self::Deprecated(value)
    }
}

impl From<bool> for Deprecated {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct As(pub TypePath);
}

impl As {
    /// Returns this `As` attribute type path formatted as string supported by OpenAPI spec whereas
    /// double colons (::) are replaced with dot (.).
    pub fn to_schema_formatted_string(&self) -> String {
        // See: https://github.com/juhaku/utoipa/pull/187#issuecomment-1173101405
        // :: are not officially supported in the spec
        self.0
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }
}

impl Parse for As {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        parse_utils::parse_next(input, || input.parse()).map(Self)
    }
}

impl From<As> for Feature {
    fn from(value: As) -> Self {
        Self::As(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct AdditionalProperties(bool);
}

impl Parse for AdditionalProperties {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for AdditionalProperties {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let additional_properties = &self.0;
        tokens.extend(quote!(
            utoipa::openapi::schema::AdditionalProperties::FreeForm(
                #additional_properties
            )
        ))
    }
}

impl From<AdditionalProperties> for Feature {
    fn from(value: AdditionalProperties) -> Self {
        Self::AdditionalProperties(value)
    }
}

impl_feature! {
    #[derive(Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Required(pub bool);
}

impl Required {
    pub fn is_true(&self) -> bool {
        self.0
    }
}

impl Parse for Required {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        parse_utils::parse_bool_or_true(input).map(Self)
    }
}

impl ToTokens for Required {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl From<crate::Required> for Required {
    fn from(value: crate::Required) -> Self {
        if value == crate::Required::True {
            Self(true)
        } else {
            Self(false)
        }
    }
}

impl From<bool> for Required {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl From<Required> for Feature {
    fn from(value: Required) -> Self {
        Self::Required(value)
    }
}

impl_feature! {
    #[derive(Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct ContentEncoding(String);
}

impl Parse for ContentEncoding {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        parse_utils::parse_next_literal_str(input).map(Self)
    }
}

impl ToTokens for ContentEncoding {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<ContentEncoding> for Feature {
    fn from(value: ContentEncoding) -> Self {
        Self::ContentEncoding(value)
    }
}

impl_feature! {
    #[derive(Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct ContentMediaType(String);
}

impl Parse for ContentMediaType {
    fn parse(input: ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        parse_utils::parse_next_literal_str(input).map(Self)
    }
}

impl ToTokens for ContentMediaType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<ContentMediaType> for Feature {
    fn from(value: ContentMediaType) -> Self {
        Self::ContentMediaType(value)
    }
}

// discriminator = ...
// discriminator(property_name = ..., mapping(
//      (value = ...),
//      (value2 = ...)
// ))
impl_feature! {
    #[derive(Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Discriminator(LitStrOrExpr, Punctuated<(LitStrOrExpr, LitStrOrExpr), Token![,]>, Ident);
}

impl Discriminator {
    fn new(attribute: Ident) -> Self {
        Self(LitStrOrExpr::default(), Punctuated::default(), attribute)
    }

    pub fn get_attribute(&self) -> &Ident {
        &self.2
    }
}

impl Parse for Discriminator {
    fn parse(input: ParseStream, attribute: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![=]) {
            parse_utils::parse_next_literal_str_or_expr(input)
                .map(|property_name| Self(property_name, Punctuated::new(), attribute))
        } else if lookahead.peek(Paren) {
            let discriminator_stream;
            syn::parenthesized!(discriminator_stream in input);

            let mut discriminator = Discriminator::new(attribute);

            while !discriminator_stream.is_empty() {
                let property = discriminator_stream.parse::<Ident>()?;
                let name = &*property.to_string();

                match name {
                    "property_name" => {
                        discriminator.0 =
                            parse_utils::parse_next_literal_str_or_expr(&discriminator_stream)?
                    }
                    "mapping" => {
                        let mapping_stream;
                        syn::parenthesized!(mapping_stream in &discriminator_stream);
                        let mappings: Punctuated<(LitStrOrExpr, LitStrOrExpr), Token![,]> =
                            Punctuated::parse_terminated_with(&mapping_stream, |input| {
                                let inner;
                                syn::parenthesized!(inner in input);

                                let key = inner.parse::<LitStrOrExpr>()?;
                                inner.parse::<Token![=]>()?;
                                let value = inner.parse::<LitStrOrExpr>()?;

                                Ok((key, value))
                            })?;
                        discriminator.1 = mappings;
                    }
                    unexpected => {
                        return Err(Error::new(
                            property.span(),
                            format!(
                                "unexpected identifier {}, expected any of: property_name, mapping",
                                unexpected
                            ),
                        ))
                    }
                }

                if !discriminator_stream.is_empty() {
                    discriminator_stream.parse::<Token![,]>()?;
                }
            }

            Ok(discriminator)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for Discriminator {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Discriminator(property_name, mapping, _) = self;

        struct Mapping<'m>(&'m LitStrOrExpr, &'m LitStrOrExpr);

        impl ToTokens for Mapping<'_> {
            fn to_tokens(&self, tokens: &mut TokenStream) {
                let Mapping(property_name, value) = *self;

                tokens.extend(quote! {
                    (#property_name, #value)
                })
            }
        }

        let discriminator = if !mapping.is_empty() {
            let mapping = mapping
                .iter()
                .map(|(key, value)| Mapping(key, value))
                .collect::<Array<Mapping>>();

            quote! {
                utoipa::openapi::schema::Discriminator::with_mapping(#property_name, #mapping)
            }
        } else {
            quote! {
                utoipa::openapi::schema::Discriminator::new(#property_name)
            }
        };

        discriminator.to_tokens(tokens);
    }
}

impl From<Discriminator> for Feature {
    fn from(value: Discriminator) -> Self {
        Self::Discriminator(value)
    }
}

// bound = "GenericTy: Trait"
impl_feature! {
    #[derive(Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Bound(pub(crate) Punctuated<WherePredicate, Token![,]>);
}

impl Parse for Bound {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self> {
        let litstr = parse_utils::parse_next(input, || input.parse::<LitStr>())?;
        let bounds =
            syn::parse::Parser::parse_str(<Punctuated<_, _>>::parse_terminated, &litstr.value())
                .map_err(|err| syn::Error::new(litstr.span(), err.to_string()))?;
        Ok(Self(bounds))
    }
}

impl ToTokens for Bound {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<Bound> for Feature {
    fn from(value: Bound) -> Self {
        Feature::Bound(value)
    }
}

impl_feature! {
    /// Ignore feature parsed from macro attributes.
    #[derive(Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Ignore(pub LitBoolOrExprPath);
}

impl Parse for Ignore {
    fn parse(input: syn::parse::ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        parse_utils::parse_next_literal_bool_or_call(input).map(Self)
    }
}

impl ToTokens for Ignore {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.0.to_token_stream())
    }
}

impl From<Ignore> for Feature {
    fn from(value: Ignore) -> Self {
        Self::Ignore(value)
    }
}

impl From<bool> for Ignore {
    fn from(value: bool) -> Self {
        Self(value.into())
    }
}

// Nothing to parse, it is considered to be set when attribute itself is parsed via
// `parse_features!`.
impl_feature! {
    #[derive(Clone)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct NoRecursion;
}

impl Parse for NoRecursion {
    fn parse(_: ParseStream, _: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized,
    {
        Ok(Self)
    }
}

impl From<NoRecursion> for Feature {
    fn from(value: NoRecursion) -> Self {
        Self::NoRecursion(value)
    }
}
