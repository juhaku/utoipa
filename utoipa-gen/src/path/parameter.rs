use std::{str::FromStr, borrow::Cow};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Error, LitStr, Token,
};

use crate::{parse_utils, Deprecated, Required, Type, ext::{Argument, ArgumentIn}};

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
}


impl<'p> ParameterValue<'p> {
    #[cfg(any(feature = "actix_extras", feature = "rocket_extras"))]
    pub fn update_parameter_type(&mut self, ident: Option<&'p Ident>, is_array: bool, is_option: bool) {
        self.parameter_type = ident.map(|ty| Type::new(Cow::Borrowed(ty), is_array, is_option));
    }
}

#[cfg(any(feature = "actix_extras", feature = "rocket_extras"))]
impl<'a> From<Argument<'a>> for Parameter<'a> {
    fn from(argument: Argument<'a>) -> Self {
        match argument {
            Argument::Value(value) => {
                Self::Value(ParameterValue {
            name: value.name.unwrap_or_else(|| Cow::Owned(String::new())),
            parameter_in: if value.argument_in == ArgumentIn::Path {
                ParameterIn::Path
            } else {
                ParameterIn::Query
            },
            parameter_type: value.ident.map(|ty| Type::new(Cow::Borrowed(ty), value.is_array, value.is_option)),
            ..Default::default()
        })
            },
            Argument::TokenStream(stream) => {
                Self::TokenStream(stream)
            }
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
        const EXPECTED_ATTRIBUTE_MESSAGE: &str = "unexpected attribute, expected any of: path, query, header, cookie, deprecated, description";

        while !input.is_empty() {
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
                    parameter.description =
                        Some(parse_utils::parse_next(input, || input.parse::<LitStr>())?.value())
                }
                _ => return Err(Error::new(ident.span(), EXPECTED_ATTRIBUTE_MESSAGE)),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Parameter::Value(parameter))
    }
}

impl ToTokens for Parameter<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Parameter::Value(parameter) = self {
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

            if let Some(ref parameter_type) = parameter.parameter_type {
                let property = Property::new(parameter_type.is_array, &parameter_type.ty);
                let required: Required = (!parameter_type.is_option).into();

                tokens.extend(quote! { .schema(Some(#property)).required(#required) });
            }
        } else if let Parameter::TokenStream(stream) = self {
            tokens.extend(quote! { #stream });
        };
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
