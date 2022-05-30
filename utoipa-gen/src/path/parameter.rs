use std::{borrow::Cow, str::FromStr};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Error, LitStr, Token,
};

#[cfg(any(feature = "actix_extras", feature = "rocket_extras"))]
use crate::ext::{Argument, ArgumentIn};
use crate::{parse_utils, AnyValue, Deprecated, Required, Type};

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
    Value(ParameterValue<'a>),
    #[cfg(any(feature = "actix_extras", feature = "rocket_extras"))]
    TokenStream(TokenStream),
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ParameterValue<'a> {
    pub name: Cow<'a, str>,
    parameter_in: ParameterIn,
    deprecated: bool,
    description: Option<String>,
    parameter_type: Option<Type<'a>>,
    parameter_ext: Option<ParameterExt>,
}

impl<'p> ParameterValue<'p> {
    #[cfg(any(feature = "actix_extras", feature = "rocket_extras"))]
    pub fn update_parameter_type(
        &mut self,
        ident: Option<&'p Ident>,
        is_array: bool,
        is_option: bool,
    ) {
        self.parameter_type = ident.map(|ty| Type::new(Cow::Borrowed(ty), is_array, is_option));
    }
}

#[cfg(any(feature = "actix_extras", feature = "rocket_extras"))]
impl<'a> From<Argument<'a>> for Parameter<'a> {
    fn from(argument: Argument<'a>) -> Self {
        match argument {
            Argument::Value(value) => Self::Value(ParameterValue {
                name: value.name.unwrap_or_else(|| Cow::Owned(String::new())),
                parameter_in: if value.argument_in == ArgumentIn::Path {
                    ParameterIn::Path
                } else {
                    ParameterIn::Query
                },
                parameter_type: value
                    .ident
                    .map(|ty| Type::new(Cow::Borrowed(ty), value.is_array, value.is_option)),
                ..Default::default()
            }),
            Argument::TokenStream(stream) => Self::TokenStream(stream),
        }
    }
}

impl Parse for Parameter<'_> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut parameter = ParameterValue::default();

        if input.peek(LitStr) {
            // parse name
            let name = input.parse::<LitStr>()?.value();
            parameter.name = Cow::Owned(name);

            if input.peek(Token![=]) {
                parameter.parameter_type = Some(parse_utils::parse_next(input, || {
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
        const EXPECTED_ATTRIBUTE_MESSAGE: &str = "unexpected attribute, expected any of: path, query, header, cookie, deprecated, description, style, explode, allow_reserved, example";

        while !input.is_empty() {
            let fork = input.fork();

            let use_parameter_ext = if fork.peek(syn::Ident) {
                let ident = fork.parse::<Ident>().unwrap();
                let name = &*ident.to_string();

                matches!(name, "style" | "explode" | "allow_reserved" | "example")
            } else {
                false
            };

            if use_parameter_ext {
                let ext = parameter
                    .parameter_ext
                    .get_or_insert(ParameterExt::default());
                let parameter_ext = input.call(ParameterExt::parse_once)?;

                ext.merge(parameter_ext);
            } else {
                let ident = input.parse::<Ident>().map_err(|error| {
                    Error::new(
                        error.span(),
                        format!("{}, {}", EXPECTED_ATTRIBUTE_MESSAGE, error),
                    )
                })?;
                let name = &*ident.to_string();

                match name {
                    "path" | "query" | "header" | "cookie" => {
                        parameter.parameter_in = name
                            .parse::<ParameterIn>()
                            .map_err(|error| Error::new(ident.span(), error))?;
                    }
                    "deprecated" => parameter.deprecated = parse_utils::parse_bool_or_true(input)?,
                    "description" => {
                        parameter.description = Some(
                            parse_utils::parse_next(input, || input.parse::<LitStr>())?.value(),
                        )
                    }
                    _ => return Err(Error::new(ident.span(), EXPECTED_ATTRIBUTE_MESSAGE)),
                }
                if !input.is_empty() {
                    input.parse::<Token![,]>()?;
                }
            }
        }

        Ok(Parameter::Value(parameter))
    }
}

impl ToTokens for Parameter<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        fn handle_single_parameter(tokens: &mut TokenStream, parameter: &ParameterValue) {
            let name = &*parameter.name;
            tokens.extend(quote! {
                utoipa::openapi::path::ParameterBuilder::from(utoipa::openapi::path::Parameter::new(#name))
            });
            let parameter_in = &parameter.parameter_in;
            tokens.extend(quote! { .parameter_in(#parameter_in) });

            let deprecated: Deprecated = parameter.deprecated.into();
            tokens.extend(quote! { .deprecated(Some(#deprecated)) });

            if let Some(ref description) = parameter.description {
                tokens.extend(quote! { .description(Some(#description)) });
            }

            if let Some(ref ext) = parameter.parameter_ext {
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

            if let Some(ref parameter_type) = parameter.parameter_type {
                let property = Property::new(parameter_type.is_array, &parameter_type.ty);
                let required: Required = (!parameter_type.is_option).into();

                tokens.extend(quote! { .schema(Some(#property)).required(#required) });
            }
        }

        match self {
            Parameter::Value(parameter) => handle_single_parameter(tokens, parameter),
            #[cfg(any(feature = "actix_extras", feature = "rocket_extras"))]
            Parameter::TokenStream(stream) => {
                tokens.extend(quote! { #stream });
            }
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq)]
pub enum ParameterIn {
    Query,
    Path,
    Header,
    Cookie,
}

impl Default for ParameterIn {
    fn default() -> Self {
        Self::Path
    }
}

impl FromStr for ParameterIn {
    type Err = syn::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "path" => Ok(Self::Path),
            "query" => Ok(Self::Query),
            "header" => Ok(Self::Header),
            "cookie" => Ok(Self::Cookie),
            _ => Err(syn::Error::new(
                Span::call_site(),
                &format!(
                    "unexpected str: {}, expected one of: path, query, header, cookie",
                    s
                ),
            )),
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

impl ParameterExt {
    fn merge(&mut self, from: ParameterExt) {
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

    fn parse_once(input: ParseStream) -> syn::Result<Self> {
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
