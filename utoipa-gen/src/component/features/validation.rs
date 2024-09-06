use std::str::FromStr;

use proc_macro2::{Ident, Literal, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::parse::ParseStream;
use syn::LitStr;

use crate::{parse_utils, Diagnostics};

use super::validators::Validator;
use super::{impl_feature, Feature, Parse, Validate};

#[inline]
fn from_str<T: FromStr>(number: &str, span: Span) -> syn::Result<T>
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    T::from_str(number).map_err(|error| syn::Error::new(span, error))
}

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct NumberValue {
    minus: bool,
    pub lit: Literal,
}

impl NumberValue {
    pub fn try_from_str<T>(&self) -> syn::Result<T>
    where
        T: FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        let number = if self.minus {
            format!("-{}", &self.lit)
        } else {
            self.lit.to_string()
        };

        let parsed = from_str::<T>(&number, self.lit.span())?;
        Ok(parsed)
    }
}

impl syn::parse::Parse for NumberValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut minus = false;
        let result = input.step(|cursor| {
            let mut rest = *cursor;

            while let Some((tt, next)) = rest.token_tree() {
                match &tt {
                    TokenTree::Punct(punct) if punct.as_char() == '-' => {
                        minus = true;
                    }
                    TokenTree::Literal(lit) => return Ok((lit.clone(), next)),
                    _ => (),
                }
                rest = next;
            }
            Err(cursor.error("no `literal` value found after this point"))
        })?;

        Ok(Self { minus, lit: result })
    }
}

impl ToTokens for NumberValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let punct = if self.minus { Some(quote! {-}) } else { None };
        let lit = &self.lit;

        tokens.extend(quote! {
            #punct #lit
        })
    }
}

#[inline]
fn parse_next_number_value(input: ParseStream) -> syn::Result<NumberValue> {
    use syn::parse::Parse;
    parse_utils::parse_next(input, || NumberValue::parse(input))
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct MultipleOf(pub(super) NumberValue, Ident);
}

impl Validate for MultipleOf {
    fn validate(&self, validator: impl Validator) -> Option<Diagnostics> {
        match validator.is_valid() {
            Err(error) => Some(Diagnostics::with_span(self.1.span(), format!( "`multiple_of` error: {}", error))
                .help("See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-multipleof`")),
            _ => None
        }
    }
}

impl Parse for MultipleOf {
    fn parse(input: ParseStream, ident: Ident) -> syn::Result<Self> {
        parse_next_number_value(input).map(|number| Self(number, ident))
    }
}

impl ToTokens for MultipleOf {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<MultipleOf> for Feature {
    fn from(value: MultipleOf) -> Self {
        Feature::MultipleOf(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct Maximum(pub(super) NumberValue, Ident);
}

impl Validate for Maximum {
    fn validate(&self, validator: impl Validator) -> Option<Diagnostics> {
        match validator.is_valid() {
            Err(error) => Some(Diagnostics::with_span(self.1.span(), format!("`maximum` error: {}", error))
                .help("See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-maximum`")),
            _ => None,
        }
    }
}

impl Parse for Maximum {
    fn parse(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_next_number_value(input).map(|number| Self(number, ident))
    }
}

impl ToTokens for Maximum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<Maximum> for Feature {
    fn from(value: Maximum) -> Self {
        Feature::Maximum(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct Minimum(NumberValue, Ident);
}

impl Minimum {
    pub fn new(value: f64, span: Span) -> Self {
        Self(
            NumberValue {
                minus: value < 0.0,
                lit: Literal::f64_suffixed(value),
            },
            Ident::new("empty", span),
        )
    }
}

impl Validate for Minimum {
    fn validate(&self, validator: impl Validator) -> Option<Diagnostics> {
        match validator.is_valid() {
            Err(error) => Some(
                Diagnostics::with_span(self.1.span(), format!("`minimum` error: {}", error))
                .help("See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-minimum`")
            ),
            _ => None,
        }
    }
}

impl Parse for Minimum {
    fn parse(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_next_number_value(input).map(|number| Self(number, ident))
    }
}

impl ToTokens for Minimum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<Minimum> for Feature {
    fn from(value: Minimum) -> Self {
        Feature::Minimum(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct ExclusiveMaximum(NumberValue, Ident);
}

impl Validate for ExclusiveMaximum {
    fn validate(&self, validator: impl Validator) -> Option<Diagnostics> {
        match validator.is_valid() {
            Err(error) => Some(Diagnostics::with_span(self.1.span(), format!("`exclusive_maximum` error: {}", error))
                .help("See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-exclusivemaximum`")),
            _ => None,
        }
    }
}

impl Parse for ExclusiveMaximum {
    fn parse(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_next_number_value(input).map(|number| Self(number, ident))
    }
}

impl ToTokens for ExclusiveMaximum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<ExclusiveMaximum> for Feature {
    fn from(value: ExclusiveMaximum) -> Self {
        Feature::ExclusiveMaximum(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct ExclusiveMinimum(NumberValue, Ident);
}

impl Validate for ExclusiveMinimum {
    fn validate(&self, validator: impl Validator) -> Option<Diagnostics> {
        match validator.is_valid() {
            Err(error) => Some(Diagnostics::with_span(self.1.span(), format!("`exclusive_minimum` error: {}", error))
                .help("See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-exclusiveminimum`")),
            _ => None,
        }
    }
}

impl Parse for ExclusiveMinimum {
    fn parse(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_next_number_value(input).map(|number| Self(number, ident))
    }
}

impl ToTokens for ExclusiveMinimum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<ExclusiveMinimum> for Feature {
    fn from(value: ExclusiveMinimum) -> Self {
        Feature::ExclusiveMinimum(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct MaxLength(pub(super) NumberValue, Ident);
}

impl Validate for MaxLength {
    fn validate(&self, validator: impl Validator) -> Option<Diagnostics> {
        match validator.is_valid() {
            Err(error) => Some(Diagnostics::with_span(self.1.span(), format!("`max_length` error: {}", error))
                .help("See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-maxlength`")),
            _ => None,
        }
    }
}

impl Parse for MaxLength {
    fn parse(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_next_number_value(input).map(|number| Self(number, ident))
    }
}

impl ToTokens for MaxLength {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<MaxLength> for Feature {
    fn from(value: MaxLength) -> Self {
        Feature::MaxLength(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct MinLength(pub(super) NumberValue, Ident);
}

impl Validate for MinLength {
    fn validate(&self, validator: impl Validator) -> Option<Diagnostics> {
        match validator.is_valid() {
            Err(error) => Some(Diagnostics::with_span(self.1.span(), format!("`min_length` error: {}", error))
                .help("See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-minlength`")),
            _ => None,
        }
    }
}

impl Parse for MinLength {
    fn parse(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_next_number_value(input).map(|number| Self(number, ident))
    }
}

impl ToTokens for MinLength {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<MinLength> for Feature {
    fn from(value: MinLength) -> Self {
        Feature::MinLength(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct Pattern(String, Ident);
}

impl Validate for Pattern {
    fn validate(&self, validator: impl Validator) -> Option<Diagnostics> {
        match validator.is_valid() {
            Err(error) => Some(Diagnostics::with_span(self.1.span(), format!("`pattern` error: {}", error))
                .help("See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-pattern`")
            ),
            _ => None,
        }
    }
}

impl Parse for Pattern {
    fn parse(input: ParseStream, ident: Ident) -> syn::Result<Self>
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

impl From<Pattern> for Feature {
    fn from(value: Pattern) -> Self {
        Feature::Pattern(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct MaxItems(pub(super) NumberValue, Ident);
}

impl Validate for MaxItems {
    fn validate(&self, validator: impl Validator) -> Option<Diagnostics> {
        match validator.is_valid() {
            Err(error) => Some(Diagnostics::with_span(self.1.span(), format!("`max_items` error: {}", error))
                .help("See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-maxitems")),
            _ => None,
        }
    }
}

impl Parse for MaxItems {
    fn parse(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_next_number_value(input).map(|number| Self(number, ident))
    }
}

impl ToTokens for MaxItems {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<MaxItems> for Feature {
    fn from(value: MaxItems) -> Self {
        Feature::MaxItems(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct MinItems(pub(super) NumberValue, Ident);
}

impl Validate for MinItems {
    fn validate(&self, validator: impl Validator) -> Option<Diagnostics> {
        match validator.is_valid() {
            Err(error) => Some(Diagnostics::with_span(self.1.span(), format!("`min_items` error: {}", error))
                .help("See more details: `http://json-schema.org/draft/2020-12/json-schema-validation.html#name-minitems")),
            _ => None,
        }
    }
}

impl Parse for MinItems {
    fn parse(input: ParseStream, ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_next_number_value(input).map(|number| Self(number, ident))
    }
}

impl ToTokens for MinItems {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<MinItems> for Feature {
    fn from(value: MinItems) -> Self {
        Feature::MinItems(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct MaxProperties(NumberValue, ());
}

impl Parse for MaxProperties {
    fn parse(input: ParseStream, _ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_next_number_value(input).map(|number| Self(number, ()))
    }
}

impl ToTokens for MaxProperties {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<MaxProperties> for Feature {
    fn from(value: MaxProperties) -> Self {
        Feature::MaxProperties(value)
    }
}

impl_feature! {
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(Clone)]
    pub struct MinProperties(NumberValue, ());
}

impl Parse for MinProperties {
    fn parse(input: ParseStream, _ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_next_number_value(input).map(|number| Self(number, ()))
    }
}

impl ToTokens for MinProperties {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl From<MinProperties> for Feature {
    fn from(value: MinProperties) -> Self {
        Feature::MinProperties(value)
    }
}
