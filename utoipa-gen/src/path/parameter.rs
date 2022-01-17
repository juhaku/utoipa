use std::str::FromStr;

use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::ResultExt;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    LitStr, Token,
};

use crate::{parse_utils, Deprecated, MediaType, Required};

use super::property::Property;

/// Parameter of request suchs as in path, header, query or cookie
///
/// For example path `/users/{id}` the path parameter is used to define
/// type, format and other details of the `{id}` parameter within the path
///
/// Parse is executed for following formats:
///
/// * ("id" = String, path, required, deprecated, description = "Users database id"),
/// * ("id", path, required, deprecated, description = "Users database id"),
///
/// The `= String` type statement is optional if automatic resolvation is supported.
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Parameter {
    pub name: String,
    parameter_in: ParameterIn,
    required: bool,
    deprecated: bool,
    description: Option<String>,
    parameter_type: Option<MediaType>,
}

impl Parameter {
    pub fn new<S: AsRef<str>>(name: S, parameter_type: &Ident, parameter_in: ParameterIn) -> Self {
        let required = parameter_in == ParameterIn::Path;

        Self {
            name: name.as_ref().to_string(),
            parameter_type: Some(MediaType::new(parameter_type.clone())),
            parameter_in,
            required,
            ..Default::default()
        }
    }

    pub fn update_parameter_type(&mut self, ident: &Ident) {
        self.parameter_type = Some(MediaType::new(ident.clone()));
    }
}

impl Parse for Parameter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut parameter = Parameter::default();

        if input.peek(LitStr) {
            // parse name
            let name = input.parse::<LitStr>().unwrap().value();
            parameter.name = name;

            if input.peek(Token![=]) {
                parameter.parameter_type =
                    parse_utils::parse_next(input, || {
                        Some(input.parse().expect_or_abort(
                            "unparseable parameter type expected: Ident or [Ident]",
                        ))
                    });
            }
        } else {
            return Err(input.error("expected first element to be LitStr parameter name"));
        }
        if input.peek(Token![,]) {
            input.parse::<Token![,]>().unwrap();
        }

        loop {
            let ident = input.parse::<syn::Ident>().unwrap();
            let name = &*ident.to_string();
            match name {
                "path" | "query" | "header" | "cookie" => {
                    parameter.parameter_in = name.parse::<ParameterIn>().unwrap_or_abort();
                    if parameter.parameter_in == ParameterIn::Path {
                        parameter.required = true; // all path parameters are required by default
                    }
                }
                "required" => parameter.required = parse_utils::parse_bool_or_true(input),
                "deprecated" => parameter.deprecated = parse_utils::parse_bool_or_true(input),
                "description" => {
                    if input.peek(Token![=]) {
                        input.parse::<Token![=]>().unwrap();
                    }

                    parameter.description = Some(
                        ResultExt::expect_or_abort(
                            input.parse::<LitStr>(),
                            "expected description value as LitStr",
                        )
                        .value(),
                    )
                }
                _ => return Err(input.error(&format!("unexpected element: {}", name))),
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>().unwrap();
            }
            if input.is_empty() {
                break;
            }
        }
        Ok(parameter)
    }
}

impl ToTokens for Parameter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &*self.name;
        tokens.extend(quote! { utoipa::openapi::path::Parameter::new(#name) });
        let parameter_in = &self.parameter_in;
        tokens.extend(quote! { .with_in(#parameter_in) });

        let required: Required = self.required.into();
        tokens.extend(quote! { .with_required(#required) });

        let deprecated: Deprecated = self.deprecated.into();
        tokens.extend(quote! { .with_deprecated(#deprecated) });

        if let Some(ref description) = self.description {
            tokens.extend(quote! { .with_description(#description) });
        }

        if let Some(ref parameter_type) = self.parameter_type {
            let property = Property::new(parameter_type.is_array, &parameter_type.ty);

            tokens.extend(quote! { .with_schema(#property) });
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
