use std::{borrow::Cow, fmt::Display};

use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseBuffer, ParseStream},
    Error, Generics, LitStr, Token, TypePath,
};

use crate::{
    as_tokens_or_diagnostics,
    component::{
        self,
        features::{
            attributes::{
                AllowReserved, Description, Example, Explode, Extensions, Format, Nullable,
                ReadOnly, Style, WriteOnly, XmlAttr,
            },
            impl_into_inner, parse_features,
            validation::{
                ExclusiveMaximum, ExclusiveMinimum, MaxItems, MaxLength, Maximum, MinItems,
                MinLength, Minimum, MultipleOf, Pattern,
            },
            Feature, ToTokensExt,
        },
        ComponentSchema, Container, TypeTree,
    },
    parse_utils, Diagnostics, Required, ToTokensDiagnostics,
};

use super::media_type::ParsedType;

/// Parameter of request such as in path, header, query or cookie
///
/// For example path `/users/{id}` the path parameter is used to define
/// type, format and other details of the `{id}` parameter within the path
///
/// Parse is executed for following formats:
///
/// * ("id" = String, path, deprecated, description = "Users database id"),
/// * ("id", path, deprecated, description = "Users database id"),
///
/// The `= String` type statement is optional if automatic resolution is supported.
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq, Eq)]
pub enum Parameter<'a> {
    Value(ValueParameter<'a>),
    /// Identifier for a struct that implements `IntoParams` trait.
    IntoParamsIdent(IntoParamsIdentParameter<'a>),
}

#[cfg(any(
    feature = "actix_extras",
    feature = "rocket_extras",
    feature = "axum_extras"
))]
impl<'p> Parameter<'p> {
    pub fn merge(&mut self, other: Parameter<'p>) {
        match (self, other) {
            (Self::Value(value), Parameter::Value(other)) => {
                let (schema_features, _) = &value.features;
                // if value parameter schema has not been defined use the external one
                if value.parameter_schema.is_none() {
                    value.parameter_schema = other.parameter_schema;
                }

                if let Some(parameter_schema) = &mut value.parameter_schema {
                    parameter_schema.features.clone_from(schema_features);
                }
            }
            (Self::IntoParamsIdent(into_params), Parameter::IntoParamsIdent(other)) => {
                *into_params = other;
            }
            _ => (),
        }
    }
}

impl Parse for Parameter<'_> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.fork().parse::<TypePath>().is_ok() {
            Ok(Self::IntoParamsIdent(IntoParamsIdentParameter {
                path: Cow::Owned(input.parse::<TypePath>()?.path),
                parameter_in_fn: None,
            }))
        } else {
            Ok(Self::Value(input.parse()?))
        }
    }
}

impl ToTokensDiagnostics for Parameter<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        match self {
            Parameter::Value(parameter) => {
                let parameter = as_tokens_or_diagnostics!(parameter);
                tokens.extend(quote! { .parameter(#parameter) });
            }
            Parameter::IntoParamsIdent(IntoParamsIdentParameter {
                path,
                parameter_in_fn,
            }) => {
                let last_ident = &path.segments.last().unwrap().ident;

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

        Ok(())
    }
}

#[cfg(any(
    feature = "actix_extras",
    feature = "rocket_extras",
    feature = "axum_extras"
))]
impl<'a> From<crate::ext::ValueArgument<'a>> for Parameter<'a> {
    fn from(argument: crate::ext::ValueArgument<'a>) -> Self {
        let parameter_in = if argument.argument_in == crate::ext::ArgumentIn::Path {
            ParameterIn::Path
        } else {
            ParameterIn::Query
        };

        let option_is_nullable = parameter_in != ParameterIn::Query;

        Self::Value(ValueParameter {
            name: argument.name.unwrap_or_else(|| Cow::Owned(String::new())),
            parameter_in,
            parameter_schema: argument.type_tree.map(|type_tree| ParameterSchema {
                parameter_type: ParameterType::External(type_tree),
                features: Vec::new(),
                option_is_nullable,
            }),
            ..Default::default()
        })
    }
}

#[cfg(any(
    feature = "actix_extras",
    feature = "rocket_extras",
    feature = "axum_extras"
))]
impl<'a> From<crate::ext::IntoParamsType<'a>> for Parameter<'a> {
    fn from(value: crate::ext::IntoParamsType<'a>) -> Self {
        Self::IntoParamsIdent(IntoParamsIdentParameter {
            path: value.type_path.expect("IntoParams type must have a path"),
            parameter_in_fn: Some(value.parameter_in_provider),
        })
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct ParameterSchema<'p> {
    parameter_type: ParameterType<'p>,
    features: Vec<Feature>,
    option_is_nullable: bool,
}

impl ToTokensDiagnostics for ParameterSchema<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        let mut to_tokens = |param_schema, required| {
            tokens.extend(quote! { .schema(Some(#param_schema)).required(#required) });
        };

        match &self.parameter_type {
            #[cfg(any(
                feature = "actix_extras",
                feature = "rocket_extras",
                feature = "axum_extras"
            ))]
            ParameterType::External(type_tree) => {
                let required: Required = (!type_tree.is_option()).into();

                to_tokens(
                    ComponentSchema::for_params(
                        component::ComponentSchemaProps {
                            type_tree,
                            features: self.features.clone(),
                            description: None,
                            container: &Container {
                                generics: &Generics::default(),
                            },
                        },
                        self.option_is_nullable,
                    )?
                    .to_token_stream(),
                    required,
                );
                Ok(())
            }
            ParameterType::Parsed(inline_type) => {
                let type_tree = TypeTree::from_type(inline_type.ty.as_ref())?;
                let required: Required = (!type_tree.is_option()).into();
                let mut schema_features = Vec::<Feature>::new();
                schema_features.clone_from(&self.features);
                schema_features.push(Feature::Inline(inline_type.is_inline.into()));

                to_tokens(
                    ComponentSchema::for_params(
                        component::ComponentSchemaProps {
                            type_tree: &type_tree,
                            features: schema_features,
                            description: None,
                            container: &Container {
                                generics: &Generics::default(),
                            },
                        },
                        self.option_is_nullable,
                    )?
                    .to_token_stream(),
                    required,
                );
                Ok(())
            }
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
enum ParameterType<'p> {
    #[cfg(any(
        feature = "actix_extras",
        feature = "rocket_extras",
        feature = "axum_extras"
    ))]
    External(crate::component::TypeTree<'p>),
    Parsed(ParsedType<'p>),
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ValueParameter<'a> {
    pub name: Cow<'a, str>,
    parameter_in: ParameterIn,
    parameter_schema: Option<ParameterSchema<'a>>,
    features: (Vec<Feature>, Vec<Feature>),
}

impl PartialEq for ValueParameter<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.parameter_in == other.parameter_in
    }
}

impl Eq for ValueParameter<'_> {}

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
                parameter.parameter_schema = Some(ParameterSchema {
                    parameter_type: ParameterType::Parsed(parse_utils::parse_next(&input, || {
                        input.parse().map_err(|error| {
                            Error::new(
                                error.span(),
                                format!("unexpected token, expected type such as String, {error}"),
                            )
                        })
                    })?),
                    features: Vec::new(),
                    option_is_nullable: true,
                });
            }
        } else {
            return Err(input.error("unparsable parameter name, expected literal string"));
        }

        input.parse::<Token![,]>()?;

        if input.fork().parse::<ParameterIn>().is_ok() {
            parameter.parameter_in = input.parse()?;
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        let (schema_features, parameter_features) = input
            .parse::<ParameterFeatures>()?
            .split_for_parameter_type();

        parameter.features = (schema_features.clone(), parameter_features);
        if let Some(parameter_schema) = &mut parameter.parameter_schema {
            parameter_schema.features = schema_features;

            if parameter.parameter_in == ParameterIn::Query {
                parameter_schema.option_is_nullable = false;
            }
        }

        Ok(parameter)
    }
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
struct ParameterFeatures(Vec<Feature>);

impl Parse for ParameterFeatures {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(parse_features!(
            // param features
            input as Style,
            Explode,
            AllowReserved,
            Example,
            crate::component::features::attributes::Deprecated,
            Description,
            // param schema features
            Format,
            WriteOnly,
            ReadOnly,
            Nullable,
            XmlAttr,
            MultipleOf,
            Maximum,
            Minimum,
            ExclusiveMaximum,
            ExclusiveMinimum,
            MaxLength,
            MinLength,
            Pattern,
            MaxItems,
            MinItems,
            Extensions
        )))
    }
}

impl ParameterFeatures {
    /// Split parsed features to two `Vec`s of [`Feature`]s.
    ///
    /// * First vec contains parameter type schema features.
    /// * Second vec contains generic parameter features.
    fn split_for_parameter_type(self) -> (Vec<Feature>, Vec<Feature>) {
        self.0.into_iter().fold(
            (Vec::new(), Vec::new()),
            |(mut schema_features, mut param_features), feature| {
                match feature {
                    Feature::Format(_)
                    | Feature::WriteOnly(_)
                    | Feature::ReadOnly(_)
                    | Feature::Nullable(_)
                    | Feature::XmlAttr(_)
                    | Feature::MultipleOf(_)
                    | Feature::Maximum(_)
                    | Feature::Minimum(_)
                    | Feature::ExclusiveMaximum(_)
                    | Feature::ExclusiveMinimum(_)
                    | Feature::MaxLength(_)
                    | Feature::MinLength(_)
                    | Feature::Pattern(_)
                    | Feature::MaxItems(_)
                    | Feature::MinItems(_) => {
                        schema_features.push(feature);
                    }
                    _ => {
                        param_features.push(feature);
                    }
                };

                (schema_features, param_features)
            },
        )
    }
}

impl_into_inner!(ParameterFeatures);

impl ToTokensDiagnostics for ValueParameter<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        let name = &*self.name;
        tokens.extend(quote! {
            utoipa::openapi::path::ParameterBuilder::from(utoipa::openapi::path::Parameter::new(#name))
        });
        let parameter_in = &self.parameter_in;
        tokens.extend(quote! { .parameter_in(#parameter_in) });

        let (schema_features, param_features) = &self.features;

        tokens.extend(param_features.to_token_stream()?);

        if !schema_features.is_empty() && self.parameter_schema.is_none() {
            return Err(
                Diagnostics::new("Missing `parameter_type` attribute, cannot define schema features without it.")
                .help("See docs for more details <https://docs.rs/utoipa/latest/utoipa/attr.path.html#parameter-type-attributes>")
            );
        }

        if let Some(parameter_schema) = &self.parameter_schema {
            parameter_schema.to_tokens(tokens)?;
        }

        Ok(())
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct IntoParamsIdentParameter<'i> {
    pub path: Cow<'i, syn::Path>,
    /// quote!{ ... } of function which should implement `parameter_in_provider` for [`utoipa::IntoParams::into_param`]
    parameter_in_fn: Option<TokenStream>,
}

// Compare paths loosely only by segment idents ignoring possible generics
impl PartialEq for IntoParamsIdentParameter<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.path
            .segments
            .iter()
            .map(|segment| &segment.ident)
            .collect::<Vec<_>>()
            == other
                .path
                .segments
                .iter()
                .map(|segment| &segment.ident)
                .collect::<Vec<_>>()
    }
}

impl Eq for IntoParamsIdentParameter<'_> {}

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
            format!("unexpected in, expected one of: {variants}")
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
