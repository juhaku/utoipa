use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::parse::ParseStream;
use syn::LitStr;

use crate::{parse_utils, Diagnostics};

use super::validators::Validator;
use super::{name, parse_integer, parse_number, Feature, Parse, Validate};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct MultipleOf(pub(super) f64, Ident);

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
        parse_number(input).map(|multiple_of| Self(multiple_of, ident))
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

name!(MultipleOf = "multiple_of");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Maximum(pub(super) f64, Ident);

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
        parse_number(input).map(|maximum| Self(maximum, ident))
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

name!(Maximum = "maximum");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Minimum(f64, Ident);

impl Minimum {
    pub fn new(value: f64, span: Span) -> Self {
        Self(value, Ident::new("empty", span))
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
        parse_number(input).map(|maximum| Self(maximum, ident))
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

name!(Minimum = "minimum");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct ExclusiveMaximum(f64, Ident);

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
        parse_number(input).map(|max| Self(max, ident))
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

name!(ExclusiveMaximum = "exclusive_maximum");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct ExclusiveMinimum(f64, Ident);

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
        parse_number(input).map(|min| Self(min, ident))
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

name!(ExclusiveMinimum = "exclusive_minimum");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct MaxLength(pub(super) usize, Ident);

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
        parse_integer(input).map(|max_length| Self(max_length, ident))
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

name!(MaxLength = "max_length");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct MinLength(pub(super) usize, Ident);

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
        parse_integer(input).map(|max_length| Self(max_length, ident))
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

name!(MinLength = "min_length");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Pattern(String, Ident);

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

name!(Pattern = "pattern");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct MaxItems(pub(super) usize, Ident);

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
        parse_number(input).map(|max_items| Self(max_items, ident))
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

name!(MaxItems = "max_items");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct MinItems(pub(super) usize, Ident);

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
        parse_number(input).map(|max_items| Self(max_items, ident))
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

name!(MinItems = "min_items");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct MaxProperties(usize, ());

impl Parse for MaxProperties {
    fn parse(input: ParseStream, _ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_integer(input).map(|max_properties| Self(max_properties, ()))
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

name!(MaxProperties = "max_properties");

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct MinProperties(usize, ());

impl Parse for MinProperties {
    fn parse(input: ParseStream, _ident: Ident) -> syn::Result<Self>
    where
        Self: Sized,
    {
        parse_integer(input).map(|min_properties| Self(min_properties, ()))
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

name!(MinProperties = "min_properties");
