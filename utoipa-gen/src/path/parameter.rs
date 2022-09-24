use std::{borrow::Cow, fmt::Display};

use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseBuffer, ParseStream},
    Error, ExprPath, LitStr, Token,
};

#[cfg(any(
    feature = "actix_extras",
    feature = "rocket_extras",
    feature = "axum_extras"
))]
use crate::ext::{ArgumentIn, ValueArgument};
use crate::{
    component::into_params::FieldParamContainerAttributes, parse_utils, AnyValue, Deprecated,
    Required, Type,
};

use super::property::Property;

/// Parameter of request suchs as in path, header, query or cookie
///
/// For example path `/users/{id}` the path parameter is used to define
/// type, format and other details of the `{id}` parameter within the path
///
/// Parse is executed for following formats:
///
/// * ("id" = String, path, deprecated, description = "Users database id"),
/// * ("id", path, deprecated, description = "Users database id"),
///
/// The `= String` type statement is optional if automatic resolvation is supported.
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Parameter<'a> {
    Value(ValueParameter<'a>),
    /// Identifier for a struct that implements `IntoParams` trait.
    Struct(StructParameter),
}

impl Parse for Parameter<'_> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.fork().parse::<ExprPath>().is_ok() {
            Ok(Self::Struct(StructParameter {
                path: input.parse()?,
                parameter_in_fn: None,
            }))
        } else {
            Ok(Self::Value(input.parse()?))
        }
    }
}

impl ToTokens for Parameter<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Parameter::Value(parameter) => tokens.extend(quote! { .parameter(#parameter) }),
            Parameter::Struct(StructParameter {
                path,
                parameter_in_fn,
            }) => {
                let last_ident = &path.path.segments.last().unwrap().ident;

                let default_parameter_in_provider = &quote! { || None };
                let parameter_in_provider = parameter_in_fn
                    .as_ref()
                    .unwrap_or(default_parameter_in_provider);
                tokens.extend(quote_spanned! {last_ident.span()=>
                    .parameters(
                        Some(<#path as utoipa::IntoParams>::into_params(#parameter_in_provider))
                    )
                })
            }
        }
    }
}

#[cfg(any(
    feature = "actix_extras",
    feature = "rocket_extras",
    feature = "axum_extras"
))]
impl<'a> From<ValueArgument<'a>> for Parameter<'a> {
    fn from(argument: ValueArgument<'a>) -> Self {
        Self::Value(ValueParameter {
            name: argument.name.unwrap_or_else(|| Cow::Owned(String::new())),
            parameter_in: if argument.argument_in == ArgumentIn::Path {
                ParameterIn::Path
            } else {
                ParameterIn::Query
            },
            parameter_type: argument
                .type_path
                .map(|ty| Type::new(ty, argument.is_array, argument.is_option)),
            ..Default::default()
        })
    }
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ValueParameter<'a> {
    pub name: Cow<'a, str>,
    parameter_in: ParameterIn,
    deprecated: bool,
    description: Option<String>,
    parameter_type: Option<Type<'a>>,
    parameter_ext: Option<ParameterExt>,
}

impl<'p> ValueParameter<'p> {
    #[cfg(any(
        feature = "actix_extras",
        feature = "rocket_extras",
        feature = "axum_extras"
    ))]
    pub fn update_parameter_type(
        &mut self,
        type_path: Option<Cow<'p, syn::Path>>,
        is_array: bool,
        is_option: bool,
    ) {
        self.parameter_type = type_path.map(|ty| Type::new(ty, is_array, is_option));
    }
}

impl Parse for ValueParameter<'_> {
    fn parse(input_with_parens: ParseStream) -> syn::Result<Self> {
        let input: ParseBuffer;
        parenthesized!(input in input_with_parens);

        let mut parameter = ValueParameter::default();

        if input.peek(LitStr) {
            // parse name
            let name = input.parse::<LitStr>()?.value();
            parameter.name = Cow::Owned(name);

            if input.peek(Token![=]) {
                parameter.parameter_type = Some(parse_utils::parse_next(&input, || {
                    input.parse().map_err(|error| {
                        Error::new(
                            error.span(),
                            format!("unexpected token, expected type such as String, {}", error),
                        )
                    })
                })?);
            }
        } else {
            return Err(input.error("unparseable parameter name, expected literal string"));
        }

        input.parse::<Token![,]>()?;

        fn expected_attribute_message() -> String {
            let parameter_in_variants = ParameterIn::VARIANTS
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");

            format!(
                "unexpected attribute, expected any of: {}, deprecated, description, style, explode, allow_reserved, example",
                parameter_in_variants
            )
        }

        while !input.is_empty() {
            if ParameterExt::is_parameter_ext(&input) {
                let ext = parameter
                    .parameter_ext
                    .get_or_insert(ParameterExt::default());
                let parameter_ext = input.call(ParameterExt::parse_once)?;

                ext.merge(parameter_ext);
            } else {
                if input.fork().parse::<ParameterIn>().is_ok() {
                    parameter.parameter_in = input.parse()?;
                } else {
                    let ident = input.parse::<Ident>().map_err(|error| {
                        Error::new(
                            error.span(),
                            format!("{}, {}", expected_attribute_message(), error),
                        )
                    })?;
                    let name = &*ident.to_string();

                    match name {
                        "deprecated" => {
                            parameter.deprecated = parse_utils::parse_bool_or_true(&input)?
                        }
                        "description" => {
                            parameter.description = Some(
                                parse_utils::parse_next(&input, || input.parse::<LitStr>())?
                                    .value(),
                            )
                        }
                        _ => return Err(Error::new(ident.span(), expected_attribute_message())),
                    }
                }

                if !input.is_empty() {
                    input.parse::<Token![,]>()?;
                }
            }
        }

        Ok(parameter)
    }
}

impl ToTokens for ValueParameter<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &*self.name;
        tokens.extend(quote! {
            utoipa::openapi::path::ParameterBuilder::from(utoipa::openapi::path::Parameter::new(#name))
        });
        let parameter_in = &self.parameter_in;
        tokens.extend(quote! { .parameter_in(#parameter_in) });

        let deprecated: Deprecated = self.deprecated.into();
        tokens.extend(quote! { .deprecated(Some(#deprecated)) });

        if let Some(ref description) = self.description {
            tokens.extend(quote! { .description(Some(#description)) });
        }

        if let Some(ref ext) = self.parameter_ext {
            if let Some(ref style) = ext.style {
                tokens.extend(quote! { .style(Some(#style)) });
            }
            if let Some(ref explode) = ext.explode {
                tokens.extend(quote! { .explode(Some(#explode)) });
            }

            if let Some(ref allow_reserved) = ext.allow_reserved {
                tokens.extend(quote! { .allow_reserved(Some(#allow_reserved)) });
            }

            if let Some(ref example) = ext.example {
                tokens.extend(quote! { .example(Some(#example)) });
            }
        }

        if let Some(parameter_type) = &self.parameter_type {
            let property = Property::new(parameter_type);
            let required: Required = (!parameter_type.is_option).into();

            tokens.extend(quote! { .schema(Some(#property)).required(#required) });
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct StructParameter {
    pub path: ExprPath,
    /// quote!{ ... } of function which should implement `parameter_in_provider` for [`utoipa::IntoParams::into_param`]
    parameter_in_fn: Option<TokenStream>,
}

impl StructParameter {
    #[cfg(any(
        feature = "actix_extras",
        feature = "rocket_extras",
        feature = "axum_extras"
    ))]
    pub fn update_parameter_in(&mut self, parameter_in_provider: &mut TokenStream) {
        use std::mem;
        self.parameter_in_fn = Some(mem::take(parameter_in_provider));
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ParameterIn {
    Query,
    Path,
    Header,
    Cookie,
}

impl ParameterIn {
    pub const VARIANTS: &'static [Self] = &[Self::Query, Self::Path, Self::Header, Self::Cookie];
}

impl Display for ParameterIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterIn::Query => write!(f, "Query"),
            ParameterIn::Path => write!(f, "Path"),
            ParameterIn::Header => write!(f, "Header"),
            ParameterIn::Cookie => write!(f, "Cookie"),
        }
    }
}

impl Default for ParameterIn {
    fn default() -> Self {
        Self::Path
    }
}

impl Parse for ParameterIn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        fn expected_style() -> String {
            let variants: String = ParameterIn::VARIANTS
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("unexpected in, expected one of: {}", variants)
        }
        let style = input.parse::<Ident>()?;

        match &*style.to_string() {
            "Path" => Ok(Self::Path),
            "Query" => Ok(Self::Query),
            "Header" => Ok(Self::Header),
            "Cookie" => Ok(Self::Cookie),
            _ => Err(Error::new(style.span(), expected_style())),
        }
    }
}

impl ToTokens for ParameterIn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Self::Path => quote! { utoipa::openapi::path::ParameterIn::Path },
            Self::Query => quote! { utoipa::openapi::path::ParameterIn::Query },
            Self::Header => quote! { utoipa::openapi::path::ParameterIn::Header },
            Self::Cookie => quote! { utoipa::openapi::path::ParameterIn::Cookie },
        })
    }
}

/// Provides extended parsed attributes for [`Parameter`]. This type is also used
/// via into params derive.
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ParameterExt {
    pub style: Option<ParameterStyle>,
    pub explode: Option<bool>,
    pub allow_reserved: Option<bool>,
    pub(crate) example: Option<AnyValue>,
}

impl From<&'_ FieldParamContainerAttributes<'_>> for ParameterExt {
    fn from(attributes: &FieldParamContainerAttributes) -> Self {
        Self {
            style: attributes.style,
            ..ParameterExt::default()
        }
    }
}

impl ParameterExt {
    pub fn merge(&mut self, from: ParameterExt) {
        if from.style.is_some() {
            self.style = from.style
        }
        if from.explode.is_some() {
            self.explode = from.explode
        }
        if from.allow_reserved.is_some() {
            self.allow_reserved = from.allow_reserved
        }
        if from.example.is_some() {
            self.example = from.example
        }
    }

    pub fn parse_once(input: ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE_MESSAGE: &str =
            "unexpected attribute, expected any of: style, explode, allow_reserved, example";

        let ident = input.parse::<Ident>().map_err(|error| {
            Error::new(
                error.span(),
                format!("{}, {}", EXPECTED_ATTRIBUTE_MESSAGE, error),
            )
        })?;
        let name = &*ident.to_string();

        let ext = match name {
            "style" => ParameterExt {
                style: Some(parse_utils::parse_next(input, || {
                    input.parse::<ParameterStyle>()
                })?),
                ..Default::default()
            },
            "explode" => ParameterExt {
                explode: Some(parse_utils::parse_bool_or_true(input)?),
                ..Default::default()
            },
            "allow_reserved" => ParameterExt {
                allow_reserved: Some(parse_utils::parse_bool_or_true(input)?),
                ..Default::default()
            },
            "example" => ParameterExt {
                example: Some(parse_utils::parse_next(input, || {
                    AnyValue::parse_any(input)
                })?),
                ..Default::default()
            },
            _ => return Err(Error::new(ident.span(), EXPECTED_ATTRIBUTE_MESSAGE)),
        };

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
        }

        Ok(ext)
    }

    pub fn is_parameter_ext(input: ParseStream) -> bool {
        let fork = input.fork();
        if fork.peek(syn::Ident) {
            let ident = fork.parse::<Ident>().unwrap();
            let name = &*ident.to_string();

            matches!(name, "style" | "explode" | "allow_reserved" | "example")
        } else {
            false
        }
    }
}

impl Parse for ParameterExt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut parameter_ext = ParameterExt::default();

        while !input.is_empty() {
            let ext = input.call(Self::parse_once)?;
            parameter_ext.merge(ext);
        }

        Ok(parameter_ext)
    }
}

/// See definitions from `utoipa` crate path.rs
#[derive(Copy, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ParameterStyle {
    Matrix,
    Label,
    Form,
    Simple,
    SpaceDelimited,
    PipeDelimited,
    DeepObject,
}

impl Parse for ParameterStyle {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        const EXPECTED_STYLE: &str =  "unexpected style, expected one of: Matrix, Label, Form, Simple, SpaceDelimited, PipeDelimited, DeepObject";
        let style = input.parse::<Ident>()?;

        match &*style.to_string() {
            "Matrix" => Ok(ParameterStyle::Matrix),
            "Label" => Ok(ParameterStyle::Label),
            "Form" => Ok(ParameterStyle::Form),
            "Simple" => Ok(ParameterStyle::Simple),
            "SpaceDelimited" => Ok(ParameterStyle::SpaceDelimited),
            "PipeDelimited" => Ok(ParameterStyle::PipeDelimited),
            "DeepObject" => Ok(ParameterStyle::DeepObject),
            _ => Err(Error::new(style.span(), EXPECTED_STYLE)),
        }
    }
}

impl ToTokens for ParameterStyle {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ParameterStyle::Matrix => {
                tokens.extend(quote! { utoipa::openapi::path::ParameterStyle::Matrix })
            }
            ParameterStyle::Label => {
                tokens.extend(quote! { utoipa::openapi::path::ParameterStyle::Label })
            }
            ParameterStyle::Form => {
                tokens.extend(quote! { utoipa::openapi::path::ParameterStyle::Form })
            }
            ParameterStyle::Simple => {
                tokens.extend(quote! { utoipa::openapi::path::ParameterStyle::Simple })
            }
            ParameterStyle::SpaceDelimited => {
                tokens.extend(quote! { utoipa::openapi::path::ParameterStyle::SpaceDelimited })
            }
            ParameterStyle::PipeDelimited => {
                tokens.extend(quote! { utoipa::openapi::path::ParameterStyle::PipeDelimited })
            }
            ParameterStyle::DeepObject => {
                tokens.extend(quote! { utoipa::openapi::path::ParameterStyle::DeepObject })
            }
        }
    }
}
