use std::str::FromStr;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Error, LitStr, Token,
};

use crate::{parse_utils, Deprecated, Required, Type};

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
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Parameter {
    pub name: String,
    parameter_in: ParameterIn,
    deprecated: bool,
    description: Option<String>,
    parameter_type: Option<Type>,
}

impl Parameter {

    #[cfg(feature = "actix_extras")]
    pub fn new<S: Into<String>>(name: S, parameter_type: &Ident, parameter_in: ParameterIn) -> Self {
        Self {
            name: name.into(),
            parameter_type: Some(Type::new(parameter_type.clone())),
            parameter_in,
            ..Default::default()
        }
    }

    #[cfg(feature = "actix_extras")]
    pub fn update_parameter_type(&mut self, ident: &Ident) {
        self.parameter_type = Some(Type::new(ident.clone()));
    }
}

impl Parse for Parameter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut parameter = Parameter::default();

        if input.peek(LitStr) {
            // parse name
            let name = input.parse::<LitStr>()?.value();
            parameter.name = name;

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

        Ok(parameter)
    }
}

impl ToTokens for Parameter {
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

        if let Some(ref parameter_type) = self.parameter_type {
            let property = Property::new(parameter_type.is_array, &parameter_type.ty);
            let required: Required = (!parameter_type.is_option).into();

            tokens.extend(quote! { .schema(Some(#property)).required(#required) });
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
