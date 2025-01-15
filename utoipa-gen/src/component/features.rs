use std::{fmt::Display, mem};

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::ParseStream;

use crate::{
    as_tokens_or_diagnostics, schema_type::SchemaType, Diagnostics, OptionExt, ToTokensDiagnostics,
};

use self::validators::{AboveZeroF64, AboveZeroUsize, IsNumber, IsString, IsVec, ValidatorChain};

use super::TypeTree;

pub mod attributes;
pub mod validation;
pub mod validators;

pub trait FeatureLike: Parse {
    fn get_name() -> std::borrow::Cow<'static, str>
    where
        Self: Sized;
}

macro_rules! impl_feature {
    ( $( $name:literal => )? $( #[$meta:meta] )* $vis:vis $key:ident $ty:ident $( $tt:tt )* ) => {
        $( #[$meta] )*
        $vis $key $ty $( $tt )*

        impl $crate::features::FeatureLike for $ty {
            fn get_name() -> std::borrow::Cow<'static, str> {
                impl_feature!( @name $ty name: $( $name )* )
            }
        }

        impl std::fmt::Display for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let name = <Self as $crate::features::FeatureLike>::get_name();
                write!(f, "{name}", name = name.as_ref())
            }
        }
    };
    ( @name $ty:ident name: $name:literal ) => {
        std::borrow::Cow::Borrowed($name)
    };
    ( @name $ty:ident name: ) => {
        {
            let snake = $crate::component::serde::RenameRule::Snake;
            let renamed = snake.rename_variant(stringify!($ty));
            std::borrow::Cow::Owned(renamed)
        }
    };
}
use impl_feature;

/// Define whether [`Feature`] variant is validatable or not
pub trait Validatable {
    fn is_validatable(&self) -> bool {
        false
    }
}

pub trait Validate: Validatable {
    /// Perform validation check against schema type.
    fn validate(&self, validator: impl validators::Validator) -> Option<Diagnostics>;
}

pub trait Parse {
    fn parse(input: ParseStream, attribute: Ident) -> syn::Result<Self>
    where
        Self: std::marker::Sized;
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub enum Feature {
    Example(attributes::Example),
    Examples(attributes::Examples),
    Default(attributes::Default),
    Inline(attributes::Inline),
    XmlAttr(attributes::XmlAttr),
    Format(attributes::Format),
    ValueType(attributes::ValueType),
    WriteOnly(attributes::WriteOnly),
    ReadOnly(attributes::ReadOnly),
    Title(attributes::Title),
    Nullable(attributes::Nullable),
    Rename(attributes::Rename),
    RenameAll(attributes::RenameAll),
    Style(attributes::Style),
    AllowReserved(attributes::AllowReserved),
    Explode(attributes::Explode),
    ParameterIn(attributes::ParameterIn),
    IntoParamsNames(attributes::IntoParamsNames),
    SchemaWith(attributes::SchemaWith),
    Description(attributes::Description),
    Deprecated(attributes::Deprecated),
    As(attributes::As),
    AdditionalProperties(attributes::AdditionalProperties),
    Required(attributes::Required),
    ContentEncoding(attributes::ContentEncoding),
    ContentMediaType(attributes::ContentMediaType),
    Discriminator(attributes::Discriminator),
    Bound(attributes::Bound),
    Ignore(attributes::Ignore),
    NoRecursion(attributes::NoRecursion),
    MultipleOf(validation::MultipleOf),
    Maximum(validation::Maximum),
    Minimum(validation::Minimum),
    ExclusiveMaximum(validation::ExclusiveMaximum),
    ExclusiveMinimum(validation::ExclusiveMinimum),
    MaxLength(validation::MaxLength),
    MinLength(validation::MinLength),
    Pattern(validation::Pattern),
    MaxItems(validation::MaxItems),
    MinItems(validation::MinItems),
    MaxProperties(validation::MaxProperties),
    MinProperties(validation::MinProperties),
    Extensions(attributes::Extensions),
}

impl Feature {
    pub fn validate(&self, schema_type: &SchemaType, type_tree: &TypeTree) -> Option<Diagnostics> {
        match self {
            Feature::MultipleOf(multiple_of) => multiple_of.validate(
                ValidatorChain::new(&IsNumber(schema_type)).next(&AboveZeroF64(&multiple_of.0)),
            ),
            Feature::Maximum(maximum) => maximum.validate(IsNumber(schema_type)),
            Feature::Minimum(minimum) => minimum.validate(IsNumber(schema_type)),
            Feature::ExclusiveMaximum(exclusive_maximum) => {
                exclusive_maximum.validate(IsNumber(schema_type))
            }
            Feature::ExclusiveMinimum(exclusive_minimum) => {
                exclusive_minimum.validate(IsNumber(schema_type))
            }
            Feature::MaxLength(max_length) => max_length.validate(
                ValidatorChain::new(&IsString(schema_type)).next(&AboveZeroUsize(&max_length.0)),
            ),
            Feature::MinLength(min_length) => min_length.validate(
                ValidatorChain::new(&IsString(schema_type)).next(&AboveZeroUsize(&min_length.0)),
            ),
            Feature::Pattern(pattern) => pattern.validate(IsString(schema_type)),
            Feature::MaxItems(max_items) => max_items.validate(
                ValidatorChain::new(&AboveZeroUsize(&max_items.0)).next(&IsVec(type_tree)),
            ),
            Feature::MinItems(min_items) => min_items.validate(
                ValidatorChain::new(&AboveZeroUsize(&min_items.0)).next(&IsVec(type_tree)),
            ),
            unsupported => {
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
                    "Unsupported variant: `{unsupported}` for Validate::validate, expected one of: {variants}",
                    variants = SUPPORTED_VARIANTS.join(", ")
                )
            }
        }
    }
}

impl ToTokensDiagnostics for Feature {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), Diagnostics> {
        let feature = match &self {
            Feature::Default(default) => quote! { .default(#default) },
            Feature::Example(example) => quote! { .example(Some(#example)) },
            Feature::Examples(examples) => quote! { .examples(#examples) },
            Feature::XmlAttr(xml) => quote! { .xml(Some(#xml)) },
            Feature::Format(format) => quote! { .format(Some(#format)) },
            Feature::WriteOnly(write_only) => quote! { .write_only(Some(#write_only)) },
            Feature::ReadOnly(read_only) => quote! { .read_only(Some(#read_only)) },
            Feature::Title(title) => quote! { .title(Some(#title)) },
            Feature::Nullable(_nullable) => return Err(Diagnostics::new("Nullable does not support `ToTokens`")),
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
            Feature::SchemaWith(schema_with) => schema_with.to_token_stream(),
            Feature::Description(description) => quote! { .description(Some(#description)) },
            Feature::Deprecated(deprecated) => quote! { .deprecated(Some(#deprecated)) },
            Feature::AdditionalProperties(additional_properties) => {
                quote! { .additional_properties(Some(#additional_properties)) }
            }
            Feature::ContentEncoding(content_encoding) => quote! { .content_encoding(#content_encoding) },
            Feature::ContentMediaType(content_media_type) => quote! { .content_media_type(#content_media_type) },
            Feature::Discriminator(discriminator) => quote! { .discriminator(Some(#discriminator)) },
            Feature::Bound(_) => {
                // specially handled on generating impl blocks.
                TokenStream::new()
            }
            Feature::RenameAll(_) => {
                return Err(Diagnostics::new("RenameAll feature does not support `ToTokens`"))
            }
            Feature::ValueType(_) => {
                return Err(Diagnostics::new("ValueType feature does not support `ToTokens`")
                    .help("ValueType is supposed to be used with `TypeTree` in same manner as a resolved struct/field type."))
            }
            Feature::Inline(_) => {
                // inline feature is ignored by `ToTokens`
                TokenStream::new()
            }
            Feature::NoRecursion(_) => return Err(Diagnostics::new("NoRecursion does not support `ToTokens`")),
            Feature::IntoParamsNames(_) => {
                return Err(Diagnostics::new("Names feature does not support `ToTokens`")
                    .help("Names is only used with IntoParams to artificially give names for unnamed struct type `IntoParams`."))
            }
            Feature::As(_) => {
                return Err(Diagnostics::new("As does not support `ToTokens`"))
            }
            Feature::Required(required) => {
                let name = <attributes::Required as FeatureLike>::get_name();
                quote! { .#name(#required) }
            }
            Feature::Ignore(_) => return Err(Diagnostics::new("Ignore does not support `ToTokens`")),
            Feature::Extensions(extensions) => quote! { .extensions(Some(#extensions)) },
        };

        tokens.extend(feature);

        Ok(())
    }
}

impl ToTokensDiagnostics for Option<Feature> {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        if let Some(this) = self {
            this.to_tokens(tokens)
        } else {
            Ok(())
        }
    }
}

impl Display for Feature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Feature::Default(default) => default.fmt(f),
            Feature::Example(example) => example.fmt(f),
            Feature::Examples(examples) => examples.fmt(f),
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
            Feature::SchemaWith(schema_with) => schema_with.fmt(f),
            Feature::Description(description) => description.fmt(f),
            Feature::Deprecated(deprecated) => deprecated.fmt(f),
            Feature::As(as_feature) => as_feature.fmt(f),
            Feature::AdditionalProperties(additional_properties) => additional_properties.fmt(f),
            Feature::Required(required) => required.fmt(f),
            Feature::ContentEncoding(content_encoding) => content_encoding.fmt(f),
            Feature::ContentMediaType(content_media_type) => content_media_type.fmt(f),
            Feature::Discriminator(discriminator) => discriminator.fmt(f),
            Feature::Bound(bound) => bound.fmt(f),
            Feature::Ignore(ignore) => ignore.fmt(f),
            Feature::NoRecursion(no_recursion) => no_recursion.fmt(f),
            Feature::Extensions(extensions) => extensions.fmt(f),
        }
    }
}

impl Validatable for Feature {
    fn is_validatable(&self) -> bool {
        match &self {
            Feature::Default(default) => default.is_validatable(),
            Feature::Example(example) => example.is_validatable(),
            Feature::Examples(examples) => examples.is_validatable(),
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
            Feature::SchemaWith(schema_with) => schema_with.is_validatable(),
            Feature::Description(description) => description.is_validatable(),
            Feature::Deprecated(deprecated) => deprecated.is_validatable(),
            Feature::As(as_feature) => as_feature.is_validatable(),
            Feature::AdditionalProperties(additional_properties) => {
                additional_properties.is_validatable()
            }
            Feature::Required(required) => required.is_validatable(),
            Feature::ContentEncoding(content_encoding) => content_encoding.is_validatable(),
            Feature::ContentMediaType(content_media_type) => content_media_type.is_validatable(),
            Feature::Discriminator(discriminator) => discriminator.is_validatable(),
            Feature::Bound(bound) => bound.is_validatable(),
            Feature::Ignore(ignore) => ignore.is_validatable(),
            Feature::NoRecursion(no_recursion) => no_recursion.is_validatable(),
            Feature::Extensions(extensions) => extensions.is_validatable(),
        }
    }
}

macro_rules! is_validatable {
    ( $( $ty:path $( = $validatable:literal )? ),* ) => {
        $(
            impl Validatable for $ty {
            $(
                fn is_validatable(&self) -> bool {
                    $validatable
                }
            )?
            }
        )*
    };
}

is_validatable! {
    attributes::Default,
    attributes::Example,
    attributes::Examples,
    attributes::XmlAttr,
    attributes::Format,
    attributes::WriteOnly,
    attributes::ReadOnly,
    attributes::Title,
    attributes::Nullable,
    attributes::Rename,
    attributes::RenameAll,
    attributes::Style,
    attributes::ParameterIn,
    attributes::AllowReserved,
    attributes::Explode,
    attributes::ValueType,
    attributes::Inline,
    attributes::IntoParamsNames,
    attributes::SchemaWith,
    attributes::Description,
    attributes::Deprecated,
    attributes::As,
    attributes::AdditionalProperties,
    attributes::Required,
    attributes::ContentEncoding,
    attributes::ContentMediaType,
    attributes::Discriminator,
    attributes::Bound,
    attributes::Ignore,
    attributes::NoRecursion,
    validation::MultipleOf = true,
    validation::Maximum = true,
    validation::Minimum = true,
    validation::ExclusiveMaximum = true,
    validation::ExclusiveMinimum = true,
    validation::MaxLength = true,
    validation::MinLength = true,
    validation::Pattern = true,
    validation::MaxItems = true,
    validation::MinItems = true,
    validation::MaxProperties,
    validation::MinProperties,
    attributes::Extensions
}

macro_rules! parse_features {
    ($ident:ident as $( $feature:path ),*) => {
        {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Vec<crate::component::features::Feature>> {
                let names = [$( <crate::component::features::parse_features!(@as_ident $feature) as crate::component::features::FeatureLike>::get_name(), )* ];
                let mut features = Vec::<crate::component::features::Feature>::new();
                let attributes = names.join(", ");

                while !input.is_empty() {
                    let ident = input.parse::<syn::Ident>().or_else(|_| {
                        input.parse::<syn::Token![as]>().map(|as_| syn::Ident::new("as", as_.span))
                    }).map_err(|error| {
                        syn::Error::new(
                            error.span(),
                            format!("unexpected attribute, expected any of: {attributes}, {error}"),
                        )
                    })?;
                    let name = &*ident.to_string();

                    $(
                        if name == <crate::component::features::parse_features!(@as_ident $feature) as crate::component::features::FeatureLike>::get_name() {
                            features.push(<$feature as crate::component::features::Parse>::parse(input, ident)?.into());
                            if !input.is_empty() {
                                input.parse::<syn::Token![,]>()?;
                            }
                            continue;
                        }
                    )*

                    if !names.contains(&std::borrow::Cow::Borrowed(name)) {
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
                Feature::Inline(inline) if inline.0 => Some(inline),
                _ => None,
            })
            .is_some()
    }
}

pub trait ToTokensExt {
    fn to_token_stream(&self) -> Result<TokenStream, Diagnostics>;
}

impl ToTokensExt for Vec<Feature> {
    fn to_token_stream(&self) -> Result<TokenStream, Diagnostics> {
        Ok(self
            .iter()
            .map(|feature| Ok(as_tokens_or_diagnostics!(feature)))
            .collect::<Result<Vec<TokenStream>, Diagnostics>>()?
            .into_iter()
            .fold(TokenStream::new(), |mut tokens, item| {
                item.to_tokens(&mut tokens);
                tokens
            }))
    }
}

pub trait FeaturesExt {
    fn pop_by(&mut self, op: impl FnMut(&Feature) -> bool) -> Option<Feature>;

    /// Extract [`XmlAttr`] feature for given `type_tree` if it has generic type [`GenericType::Vec`]
    fn extract_vec_xml_feature(
        &mut self,
        type_tree: &TypeTree,
    ) -> Result<Option<Feature>, Diagnostics>;
}

impl FeaturesExt for Vec<Feature> {
    fn pop_by(&mut self, op: impl FnMut(&Feature) -> bool) -> Option<Feature> {
        self.iter()
            .position(op)
            .map(|index| self.swap_remove(index))
    }

    fn extract_vec_xml_feature(
        &mut self,
        type_tree: &TypeTree,
    ) -> Result<Option<Feature>, Diagnostics> {
        self.iter_mut()
            .find_map(|feature| match feature {
                Feature::XmlAttr(xml_feature) => {
                    match xml_feature.split_for_vec(type_tree) {
                        Ok((vec_xml, value_xml)) => {
                            // replace the original xml attribute with split value xml
                            if let Some(mut xml) = value_xml {
                                mem::swap(xml_feature, &mut xml)
                            }

                            Some(Ok(vec_xml.map(Feature::XmlAttr)))
                        }
                        Err(diagnostics) => Some(Err(diagnostics)),
                    }
                }
                _ => None,
            })
            .and_then_try(|value| value)
    }
}

impl FeaturesExt for Option<Vec<Feature>> {
    fn pop_by(&mut self, op: impl FnMut(&Feature) -> bool) -> Option<Feature> {
        self.as_mut().and_then(|features| features.pop_by(op))
    }

    fn extract_vec_xml_feature(
        &mut self,
        type_tree: &TypeTree,
    ) -> Result<Option<Feature>, Diagnostics> {
        self.as_mut()
            .and_then_try(|features| features.extract_vec_xml_feature(type_tree))
    }
}

/// Pull out a `Feature` from `Vec` of features by given match predicate.
/// This macro can be called in two forms demonstrated below.
/// ```text
///  let _: Option<Feature> = pop_feature!(features => Feature::Inline(_));
///  let _: Option<Inline> = pop_feature!(feature => Feature::Inline(_) as Option<Inline>);
/// ```
///
/// The `as ...` syntax can be used to directly convert the `Feature` instance to it's inner form.
macro_rules! pop_feature {
    ($features:ident => $( $ty:tt )* ) => {{
        pop_feature!( @inner $features $( $ty )* )
    }};
    ( @inner $features:ident $ty:tt :: $tv:tt ( $t:pat ) $( $tt:tt)* ) => {
        {
        let f = $features.pop_by(|feature| matches!(feature, $ty :: $tv ($t) ) );
        pop_feature!( @rest f $( $tt )* )
        }
    };
    ( @rest $feature:ident as $ty:ty ) => {
        {
        let inner: $ty = $feature.into_inner();
        inner
        }

    };
    ( @rest $($tt:tt)* ) => {
        $($tt)*
    };
}

pub(crate) use pop_feature;

pub trait IntoInner<T> {
    fn into_inner(self) -> T;
}

macro_rules! impl_feature_into_inner {
    ( $( $feat:ident :: $impl:ident , )* ) => {
        $(
            impl IntoInner<Option<$feat::$impl>> for Option<Feature> {
                fn into_inner(self) -> Option<$feat::$impl> {
                    self.and_then(|feature| match feature {
                        Feature::$impl(value) => Some(value),
                        _ => None,
                    })
                }
            }
        )*
    };
}

impl_feature_into_inner! {
    attributes::Example,
    attributes::Examples,
    attributes::Default,
    attributes::Inline,
    attributes::XmlAttr,
    attributes::Format,
    attributes::ValueType,
    attributes::WriteOnly,
    attributes::ReadOnly,
    attributes::Title,
    attributes::Nullable,
    attributes::Rename,
    attributes::RenameAll,
    attributes::Style,
    attributes::AllowReserved,
    attributes::Explode,
    attributes::ParameterIn,
    attributes::IntoParamsNames,
    attributes::SchemaWith,
    attributes::Description,
    attributes::Deprecated,
    attributes::As,
    attributes::Required,
    attributes::AdditionalProperties,
    attributes::Discriminator,
    attributes::Bound,
    attributes::Ignore,
    attributes::NoRecursion,
    validation::MultipleOf,
    validation::Maximum,
    validation::Minimum,
    validation::ExclusiveMaximum,
    validation::ExclusiveMinimum,
    validation::MaxLength,
    validation::MinLength,
    validation::Pattern,
    validation::MaxItems,
    validation::MinItems,
    validation::MaxProperties,
    validation::MinProperties,
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

pub trait Merge<T>: IntoInner<Vec<Feature>> {
    fn merge(self, from: T) -> Self;
}

macro_rules! impl_merge {
    ( $($ident:ident),* ) => {
        $(
            impl AsMut<Vec<Feature>> for $ident {
                fn as_mut(&mut self) -> &mut Vec<Feature> {
                    &mut self.0
                }
            }

            impl crate::component::features::Merge<$ident> for $ident {
                fn merge(mut self, from: $ident) -> Self {
                    use $crate::component::features::IntoInner;
                    let a = self.as_mut();
                    let mut b = from.into_inner();

                    a.append(&mut b);

                    self
                }
            }
        )*
    };
}

pub(crate) use impl_merge;

impl IntoInner<Vec<Feature>> for Vec<Feature> {
    fn into_inner(self) -> Vec<Feature> {
        self
    }
}

impl Merge<Vec<Feature>> for Vec<Feature> {
    fn merge(mut self, mut from: Vec<Feature>) -> Self {
        self.append(&mut from);
        self
    }
}
