use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, Data, Field,
    Generics, Ident,
};

use crate::{
    component::{
        self,
        features::{
            self,
            attributes::{
                AdditionalProperties, AllowReserved, Example, Explode, Format, Inline,
                IntoParamsNames, Nullable, ReadOnly, Rename, RenameAll, SchemaWith, Style,
                WriteOnly, XmlAttr,
            },
            validation::{
                ExclusiveMaximum, ExclusiveMinimum, MaxItems, MaxLength, Maximum, MinItems,
                MinLength, Minimum, MultipleOf, Pattern,
            },
        },
        FieldRename,
    },
    doc_comment::CommentAttributes,
    Array, Diagnostics, GenericsExt, OptionExt, Required, ToTokensDiagnostics,
};

use super::{
    features::{
        impl_into_inner, impl_merge, parse_features, pop_feature, Feature, FeaturesExt, IntoInner,
        Merge, ToTokensExt,
    },
    serde::{self, SerdeContainer, SerdeValue},
    ComponentSchema, Container, TypeTree,
};

impl_merge!(IntoParamsFeatures, FieldFeatures);

/// Container attribute `#[into_params(...)]`.
pub struct IntoParamsFeatures(Vec<Feature>);

impl Parse for IntoParamsFeatures {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(parse_features!(
            input as Style,
            features::attributes::ParameterIn,
            IntoParamsNames,
            RenameAll
        )))
    }
}

impl_into_inner!(IntoParamsFeatures);

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct IntoParams {
    /// Attributes tagged on the whole struct or enum.
    pub attrs: Vec<Attribute>,
    /// Generics required to complete the definition.
    pub generics: Generics,
    /// Data within the struct or enum.
    pub data: Data,
    /// Name of the struct or enum.
    pub ident: Ident,
}

impl ToTokensDiagnostics for IntoParams {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), Diagnostics> {
        let ident = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let mut into_params_features = self
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("into_params"))
            .map(|attribute| {
                attribute
                    .parse_args::<IntoParamsFeatures>()
                    .map(IntoParamsFeatures::into_inner)
                    .map_err(Diagnostics::from)
            })
            .collect::<Result<Vec<_>, Diagnostics>>()?
            .into_iter()
            .reduce(|acc, item| acc.merge(item));
        let serde_container = serde::parse_container(&self.attrs)?;

        // #[param] is only supported over fields
        if self.attrs.iter().any(|attr| attr.path().is_ident("param")) {
            return Err(Diagnostics::with_span(
                ident.span(),
                "found `param` attribute in unsupported context",
            )
            .help("Did you mean `into_params`?"));
        }

        let names = into_params_features.as_mut().and_then(|features| {
            let into_params_names = pop_feature!(features => Feature::IntoParamsNames(_));
            IntoInner::<Option<IntoParamsNames>>::into_inner(into_params_names)
                .map(|names| names.into_values())
        });

        let style = pop_feature!(into_params_features => Feature::Style(_));
        let parameter_in = pop_feature!(into_params_features => Feature::ParameterIn(_));
        let rename_all = pop_feature!(into_params_features => Feature::RenameAll(_));

        let params = self
            .get_struct_fields(&names.as_ref())?
            .enumerate()
            .map(|(index, field)| match serde::parse_value(&field.attrs) {
                Ok(serde_value) => Ok((index, field, serde_value)),
                Err(diagnostics) => Err(diagnostics)
            })
            .collect::<Result<Vec<_>, Diagnostics>>()?
            .into_iter()
            .filter_map(|(index, field, field_serde_params)| {
                if !field_serde_params.skip {
                    Some((index, field, field_serde_params))
                } else {
                    None
                }
            })
            .map(|(index, field, field_serde_params)| {
                let name = names.as_ref()
                    .map_try(|names| names.get(index).ok_or_else(|| Diagnostics::with_span(
                        ident.span(),
                        format!("There is no name specified in the names(...) container attribute for tuple struct field {}", index),
                    )));
                let name = match name {
                    Ok(name) => name,
                    Err(diagnostics) => return Err(diagnostics)
                };
                let param = Param {
                    field,
                    field_serde_params,
                    container_attributes: FieldParamContainerAttributes {
                        rename_all: rename_all.as_ref().and_then(|feature| {
                            match feature {
                                Feature::RenameAll(rename_all) => Some(rename_all),
                                _ => None
                            }
                        }),
                        style: &style,
                        parameter_in: &parameter_in,
                        name,
                    },
                    serde_container: &serde_container,
                    generics: &self.generics
                };

                let mut param_tokens = TokenStream::new();
                match ToTokensDiagnostics::to_tokens(&param, &mut param_tokens) {
                    Ok(_) => Ok(param_tokens),
                    Err(diagnostics) => Err(diagnostics)
                }
            })
            .collect::<Result<Array<TokenStream>, Diagnostics>>()?;

        tokens.extend(quote! {
            impl #impl_generics utoipa::IntoParams for #ident #ty_generics #where_clause {
                fn into_params(parameter_in_provider: impl Fn() -> Option<utoipa::openapi::path::ParameterIn>) -> Vec<utoipa::openapi::path::Parameter> {
                    #params.to_vec()
                }
            }
        });

        Ok(())
    }
}

impl IntoParams {
    fn get_struct_fields(
        &self,
        field_names: &Option<&Vec<String>>,
    ) -> Result<impl Iterator<Item = &Field>, Diagnostics> {
        let ident = &self.ident;
        match &self.data {
            Data::Struct(data_struct) => match &data_struct.fields {
                syn::Fields::Named(named_fields) => {
                    if field_names.is_some() {
                        return Err(Diagnostics::with_span(
                            ident.span(),
                            "`#[into_params(names(...))]` is not supported attribute on a struct with named fields")
                        );
                    }
                    Ok(named_fields.named.iter())
                }
                syn::Fields::Unnamed(unnamed_fields) => {
                    match self.validate_unnamed_field_names(&unnamed_fields.unnamed, field_names) {
                        None => Ok(unnamed_fields.unnamed.iter()),
                        Some(diagnostics) => Err(diagnostics),
                    }
                }
                _ => Err(Diagnostics::with_span(
                    ident.span(),
                    "Unit type struct is not supported",
                )),
            },
            _ => Err(Diagnostics::with_span(
                ident.span(),
                "Only struct type is supported",
            )),
        }
    }

    fn validate_unnamed_field_names(
        &self,
        unnamed_fields: &Punctuated<Field, Comma>,
        field_names: &Option<&Vec<String>>,
    ) -> Option<Diagnostics> {
        let ident = &self.ident;
        match field_names {
            Some(names) => {
                if names.len() != unnamed_fields.len() {
                    Some(Diagnostics::with_span(
                        ident.span(),
                        format!("declared names amount '{}' does not match to the unnamed fields amount '{}' in type: {}", 
                            names.len(), unnamed_fields.len(), ident)
                    )
                        .help(r#"Did you forget to add a field name to `#[into_params(names(... , "field_name"))]`"#)
                        .help("Or have you added extra name but haven't defined a type?")
                    )
                } else {
                    None
                }
            }
            None => Some(
                Diagnostics::with_span(
                    ident.span(),
                    "struct with unnamed fields must have explicit name declarations.",
                )
                .help(format!(
                    "Try defining `#[into_params(names(...))]` over your type: {}",
                    ident
                )),
            ),
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct FieldParamContainerAttributes<'a> {
    /// See [`IntoParamsAttr::style`].
    style: &'a Option<Feature>,
    /// See [`IntoParamsAttr::names`]. The name that applies to this field.
    name: Option<&'a String>,
    /// See [`IntoParamsAttr::parameter_in`].
    parameter_in: &'a Option<Feature>,
    /// Custom rename all if serde attribute is not present.
    rename_all: Option<&'a RenameAll>,
}

struct FieldFeatures(Vec<Feature>);

impl_into_inner!(FieldFeatures);

impl Parse for FieldFeatures {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(parse_features!(
            // param features
            input as component::features::attributes::ValueType,
            Rename,
            Style,
            AllowReserved,
            Example,
            Explode,
            SchemaWith,
            component::features::attributes::Required,
            // param schema features
            Inline,
            Format,
            component::features::attributes::Default,
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
            AdditionalProperties
        )))
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct Param<'a> {
    /// Field in the container used to create a single parameter.
    field: &'a Field,
    //// Field serde params parsed from field attributes.
    field_serde_params: SerdeValue,
    /// Attributes on the container which are relevant for this macro.
    container_attributes: FieldParamContainerAttributes<'a>,
    /// Either serde rename all rule or into_params rename all rule if provided.
    serde_container: &'a SerdeContainer,
    /// Container gnerics
    generics: &'a Generics,
}

impl Param<'_> {
    /// Resolve [`Param`] features and split features into two [`Vec`]s. Features are split by
    /// whether they should be rendered in [`Param`] itself or in [`Param`]s schema.
    ///
    /// Method returns a tuple containing two [`Vec`]s of [`Feature`].
    fn resolve_field_features(&self) -> Result<(Vec<Feature>, Vec<Feature>), syn::Error> {
        let mut field_features = self
            .field
            .attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident("param"))
            .map(|attribute| {
                attribute
                    .parse_args::<FieldFeatures>()
                    .map(FieldFeatures::into_inner)
            })
            .collect::<Result<Vec<_>, syn::Error>>()?
            .into_iter()
            .reduce(|acc, item| acc.merge(item))
            .unwrap_or_default();

        if let Some(ref style) = self.container_attributes.style {
            if !field_features
                .iter()
                .any(|feature| matches!(&feature, Feature::Style(_)))
            {
                field_features.push(style.clone()); // could try to use cow to avoid cloning
            };
        }

        Ok(field_features.into_iter().fold(
            (Vec::<Feature>::new(), Vec::<Feature>::new()),
            |(mut schema_features, mut param_features), feature| {
                match feature {
                    Feature::Inline(_)
                    | Feature::Format(_)
                    | Feature::Default(_)
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
                    | Feature::MinItems(_)
                    | Feature::AdditionalProperties(_) => {
                        schema_features.push(feature);
                    }
                    _ => {
                        param_features.push(feature);
                    }
                };

                (schema_features, param_features)
            },
        ))
    }
}

impl ToTokensDiagnostics for Param<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        let field = self.field;
        let field_serde_params = &self.field_serde_params;
        let ident = &field.ident;
        let mut name = &*ident
            .as_ref()
            .map(|ident| ident.to_string())
            .or_else(|| self.container_attributes.name.cloned())
            .ok_or_else(||
                Diagnostics::with_span(field.span(), "No name specified for unnamed field.")
                    .help("Try adding #[into_params(names(...))] container attribute to specify the name for this field")
            )?;

        if name.starts_with("r#") {
            name = &name[2..];
        }

        let (schema_features, mut param_features) =
            self.resolve_field_features().map_err(Diagnostics::from)?;

        let rename = pop_feature!(param_features => Feature::Rename(_) as Option<Rename>)
            .map(|rename| rename.into_value());
        let rename_to = field_serde_params
            .rename
            .as_deref()
            .map(Cow::Borrowed)
            .or(rename.map(Cow::Owned));
        let rename_all = self.serde_container.rename_all.as_ref().or(self
            .container_attributes
            .rename_all
            .map(|rename_all| rename_all.as_rename_rule()));
        let name = super::rename::<FieldRename>(name, rename_to, rename_all)
            .unwrap_or(Cow::Borrowed(name));
        let type_tree = TypeTree::from_type(&field.ty)?;

        tokens.extend(quote! { utoipa::openapi::path::ParameterBuilder::new()
            .name(#name)
        });
        tokens.extend(
            if let Some(ref parameter_in) = self.container_attributes.parameter_in {
                parameter_in.to_token_stream()
            } else {
                quote! {
                    .parameter_in(parameter_in_provider().unwrap_or_default())
                }
            },
        );

        if let Some(deprecated) = super::get_deprecated(&field.attrs) {
            tokens.extend(quote! { .deprecated(Some(#deprecated)) });
        }

        let schema_with = pop_feature!(param_features => Feature::SchemaWith(_));
        if let Some(schema_with) = schema_with {
            let schema_with = crate::as_tokens_or_diagnostics!(&schema_with);
            tokens.extend(quote! { .schema(Some(#schema_with)).build() });
        } else {
            let description =
                CommentAttributes::from_attributes(&field.attrs).as_formatted_string();
            if !description.is_empty() {
                tokens.extend(quote! { .description(Some(#description))})
            }

            let value_type = pop_feature!(param_features => Feature::ValueType(_) as Option<features::attributes::ValueType>);
            let component = value_type
                .as_ref()
                .map_try(|value_type| value_type.as_type_tree())?
                .unwrap_or(type_tree);

            let required: Option<features::attributes::Required> =
                pop_feature!(param_features => Feature::Required(_)).into_inner();
            let component_required = !component.is_option()
                && super::is_required(field_serde_params, self.serde_container);

            let required = match (required, component_required) {
                (Some(required_feature), _) => Into::<Required>::into(required_feature.is_true()),
                (None, component_required) => Into::<Required>::into(component_required),
            };

            tokens.extend(quote! {
                .required(#required)
            });
            tokens.extend(param_features.to_token_stream()?);

            let schema = ComponentSchema::new(component::ComponentSchemaProps {
                type_tree: &component,
                features: Some(schema_features),
                description: None,
                deprecated: None,
                container: &Container {
                    ident: &Ident::new("empty_param", Span::call_site()),
                    generics: &self.generics,
                },
            })?;
            let schema_tokens = crate::as_tokens_or_diagnostics!(&schema);

            tokens.extend(quote! { .schema(Some(#schema_tokens)).build() });
        }

        Ok(())
    }
}
