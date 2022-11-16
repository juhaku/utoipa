use std::borrow::Cow;

use proc_macro2::TokenStream;
use proc_macro_error::{abort, ResultExt};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, Data, Field,
    Generics, Ident,
};

use crate::{
    component::{
        self,
        features::{
            self, AllowReserved, Example, Explode, Inline, Names, Rename, RenameAll, Style,
        },
        FieldRename,
    },
    doc_comment::CommentAttributes,
    schema_type::{SchemaFormat, SchemaType},
    Array, Required,
};

use super::{
    features::{
        impl_into_inner, parse_features, pop_feature, Feature, FeaturesExt, IntoInner, ToTokensExt,
    },
    serde::{self, SerdeContainer},
    GenericType, TypeTree, ValueType,
};

/// Container attribute `#[into_params(...)]`.
pub struct IntoParamsFeatures(Vec<Feature>);

impl Parse for IntoParamsFeatures {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(parse_features!(
            input as Style,
            features::ParameterIn,
            Names,
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

impl ToTokens for IntoParams {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let mut into_params_features = self
            .attrs
            .iter()
            .find(|attr| attr.path.is_ident("into_params"))
            .map(|attribute| {
                attribute
                    .parse_args::<IntoParamsFeatures>()
                    .unwrap_or_abort()
                    .into_inner()
            });
        let serde_container = serde::parse_container(&self.attrs);

        // #[param] is only supported over fields
        if self.attrs.iter().any(|attr| attr.path.is_ident("param")) {
            abort! {
                ident,
                "found `param` attribute in unsupported context";
                help = "Did you mean `into_params`?",
            }
        }

        let names = into_params_features.as_mut().and_then(|features| {
            features
                .pop_by(|feature| matches!(feature, Feature::IntoParamsNames(_)))
                .and_then(|feature| match feature {
                    Feature::IntoParamsNames(names) => Some(names.into_values()),
                    _ => None,
                })
        });

        let style = pop_feature!(into_params_features => Feature::Style(_));
        let parameter_in = pop_feature!(into_params_features => Feature::ParameterIn(_));
        let rename_all = pop_feature!(into_params_features => Feature::RenameAll(_));

        let params = self
            .get_struct_fields(&names.as_ref())
            .enumerate()
            .map(|(index, field)| {
                Param {
                    field,
                    container_attributes: FieldParamContainerAttributes {
                        rename_all: rename_all.as_ref().and_then(|feature| {
                            match feature {
                                Feature::RenameAll(rename_all) => Some(rename_all),
                                _ => None
                            }
                        }),
                        style: &style,
                        parameter_in: &parameter_in,
                        name: names.as_ref()
                            .map(|names| names.get(index).unwrap_or_else(|| abort!(
                                ident,
                                "There is no name specified in the names(...) container attribute for tuple struct field {}",
                                index
                            ))),
                    },
                    serde_container: serde_container.as_ref(),
                }
            })
            .collect::<Array<Param>>();

        tokens.extend(quote! {
            impl #impl_generics utoipa::IntoParams for #ident #ty_generics #where_clause {
                fn into_params(parameter_in_provider: impl Fn() -> Option<utoipa::openapi::path::ParameterIn>) -> Vec<utoipa::openapi::path::Parameter> {
                    #params.to_vec()
                }
            }
        });
    }
}

impl IntoParams {
    fn get_struct_fields(
        &self,
        field_names: &Option<&Vec<String>>,
    ) -> impl Iterator<Item = &Field> {
        let ident = &self.ident;
        let abort = |note: &str| {
            abort! {
                ident,
                "unsupported data type, expected struct with named fields `struct {} {{...}}` or unnamed fields `struct {}(...)`",
                ident.to_string(),
                ident.to_string();
                note = note
            }
        };

        match &self.data {
            Data::Struct(data_struct) => match &data_struct.fields {
                syn::Fields::Named(named_fields) => {
                    if field_names.is_some() {
                        abort! {ident, "`#[into_params(names(...))]` is not supported attribute on a struct with named fields"}
                    }
                    named_fields.named.iter()
                }
                syn::Fields::Unnamed(unnamed_fields) => {
                    self.validate_unnamed_field_names(&unnamed_fields.unnamed, field_names);
                    unnamed_fields.unnamed.iter()
                }
                _ => abort("Unit type struct is not supported"),
            },
            _ => abort("Only struct type is supported"),
        }
    }

    fn validate_unnamed_field_names(
        &self,
        unnamed_fields: &Punctuated<Field, Comma>,
        field_names: &Option<&Vec<String>>,
    ) {
        let ident = &self.ident;
        match field_names {
            Some(names) => {
                if names.len() != unnamed_fields.len() {
                    abort! {
                        ident,
                        "declared names amount '{}' does not match to the unnamed fields amount '{}' in type: {}",
                            names.len(), unnamed_fields.len(), ident;
                        help = r#"Did you forget to add a field name to `#[into_params(names(... , "field_name"))]`"#;
                        help = "Or have you added extra name but haven't defined a type?"
                    }
                }
            }
            None => {
                abort! {
                    ident,
                    "struct with unnamed fields must have explicit name declarations.";
                    help = "Try defining `#[into_params(names(...))]` over your type: {}", ident,
                }
            }
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

#[cfg_attr(feature = "debug", derive(Debug))]
struct Param<'a> {
    /// Field in the container used to create a single parameter.
    field: &'a Field,
    /// Attributes on the container which are relevant for this macro.
    container_attributes: FieldParamContainerAttributes<'a>,
    /// Either serde rename all rule or into_params rename all rule if provided.
    serde_container: Option<&'a SerdeContainer>,
}

impl ToTokens for Param<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field = self.field;
        let ident = &field.ident;
        let mut name = &*ident
            .as_ref()
            .map(|ident| ident.to_string())
            .or_else(|| self.container_attributes.name.cloned())
            .unwrap_or_else(|| abort!(
                field, "No name specified for unnamed field.";
                help = "Try adding #[into_params(names(...))] container attribute to specify the name for this field"
            ));

        if name.starts_with("r#") {
            name = &name[2..];
        }

        let field_param_serde = serde::parse_value(&field.attrs);

        let mut field_features = field
            .attrs
            .iter()
            .find(|attribute| attribute.path.is_ident("param"))
            .map(|attribute| {
                attribute
                    .parse_args::<FieldFeatures>()
                    .unwrap_or_abort()
                    .into_inner()
            })
            .unwrap_or_default();

        if let Some(ref style) = self.container_attributes.style {
            if !field_features
                .iter()
                .any(|feature| matches!(&feature, Feature::Style(_)))
            {
                field_features.push(style.clone()); // could try to use cow to avoid cloning
            };
        }
        let value_type = field_features.pop_value_type_feature();
        let is_inline = field_features
            .pop_by(|feature| matches!(feature, Feature::Inline(_)))
            .is_some();
        let rename = field_features
            .pop_rename_feature()
            .map(|rename| rename.into_value());
        let rename = field_param_serde
            .as_ref()
            .and_then(|field_param_serde| field_param_serde.rename.as_deref().map(Cow::Borrowed))
            .or_else(|| rename.map(Cow::Owned));
        let rename_all = self
            .serde_container
            .as_ref()
            .and_then(|serde_container| serde_container.rename_all.as_ref())
            .or_else(|| {
                self.container_attributes
                    .rename_all
                    .map(|rename_all| rename_all.as_rename_rule())
            });
        let name = super::rename::<FieldRename>(name, rename.as_deref(), rename_all)
            .unwrap_or(Cow::Borrowed(name));
        let type_tree = TypeTree::from_type(&field.ty);

        tokens.extend(quote! { utoipa::openapi::path::ParameterBuilder::new()
            .name(#name)
        });
        tokens.extend(
            if let Some(ref parameter_in) = self.container_attributes.parameter_in {
                parameter_in.into_token_stream()
            } else {
                quote! {
                    .parameter_in(parameter_in_provider().unwrap_or_default())
                }
            },
        );

        if let Some(deprecated) = super::get_deprecated(&field.attrs) {
            tokens.extend(quote! { .deprecated(Some(#deprecated)) });
        }
        if let Some(comment) = CommentAttributes::from_attributes(&field.attrs).first() {
            tokens.extend(quote! {
                .description(Some(#comment))
            })
        }

        let component = value_type
            .as_ref()
            .map(|value_type| value_type.as_type_tree())
            .unwrap_or(type_tree);

        let is_default = super::is_default(&self.serde_container, &field_param_serde.as_ref());
        let required: Required =
            (!(matches!(&component.generic_type, Some(GenericType::Option)) || is_default)).into();
        tokens.extend(quote! {
            .required(#required)
        });
        tokens.extend(field_features.to_token_stream());

        let schema = ParamType {
            component: &component,
            field_features: &field_features,
            is_inline,
        };
        tokens.extend(quote! { .schema(Some(#schema)).build() });
    }
}

struct FieldFeatures(Vec<Feature>);

impl_into_inner!(FieldFeatures);

impl Parse for FieldFeatures {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(parse_features!(
            input as component::features::ValueType,
            Inline,
            Rename,
            Style,
            AllowReserved,
            Example,
            Explode
        )))
    }
}

struct ParamType<'a> {
    component: &'a TypeTree<'a>,
    field_features: &'a Vec<Feature>,
    is_inline: bool,
}

impl ToTokens for ParamType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let component = self.component;

        match &component.generic_type {
            Some(GenericType::Vec) => {
                let param_type = ParamType {
                    component: component
                        .children
                        .as_ref()
                        .expect("Vec ParamType should have children")
                        .iter()
                        .next()
                        .expect("Vec ParamType should have 1 child"),
                    field_features: self.field_features,
                    is_inline: self.is_inline,
                };

                tokens.extend(quote! {
                    utoipa::openapi::Schema::Array(
                        utoipa::openapi::ArrayBuilder::new().items(#param_type).build()
                    )
                });
            }
            Some(GenericType::Option)
            | Some(GenericType::Cow)
            | Some(GenericType::Box)
            | Some(GenericType::RefCell) => {
                let param_type = ParamType {
                    component: component
                        .children
                        .as_ref()
                        .expect("Generic container ParamType should have children")
                        .iter()
                        .next()
                        .expect("Generic container ParamType should have 1 child"),
                    field_features: self.field_features,
                    is_inline: self.is_inline,
                };

                tokens.extend(param_type.into_token_stream())
            }
            Some(GenericType::Map) => {
                // Maps are treated as generic objects with no named properties and
                // additionalProperties denoting the type

                let component_property = ParamType {
                    component: component
                        .children
                        .as_ref()
                        .expect("Map ParamType should have children")
                        .iter()
                        .nth(1)
                        .expect("Map Param type should have 2 child"),
                    field_features: self.field_features,
                    is_inline: self.is_inline,
                };

                tokens.extend(quote! {
                    utoipa::openapi::ObjectBuilder::new().additional_properties(Some(#component_property))
                });
            }
            None => {
                match component.value_type {
                    ValueType::Primitive => {
                        let type_path = &**component.path.as_ref().unwrap();
                        let schema_type = SchemaType(type_path);

                        tokens.extend(quote! {
                            utoipa::openapi::ObjectBuilder::new().schema_type(#schema_type)
                        });

                        let format: SchemaFormat = (type_path).into();
                        if format.is_known_format() {
                            tokens.extend(quote! {
                                .format(Some(#format))
                            })
                        }
                    }
                    ValueType::Object => {
                        let component_path = &**component
                            .path
                            .as_ref()
                            .expect("component should have a path");
                        if self.is_inline {
                            tokens.extend(quote_spanned! {component_path.span()=>
                                <#component_path as utoipa::ToSchema>::schema()
                            })
                        } else if component.is_object() {
                            tokens.extend(quote! {
                                utoipa::openapi::ObjectBuilder::new()
                            });
                        } else {
                            let name: String = component_path
                                .segments
                                .last()
                                .expect("Expected there to be at least one element in the path")
                                .ident
                                .to_string();
                            tokens.extend(quote! {
                                utoipa::openapi::Ref::from_schema_name(#name)
                            });
                        }
                    }
                    // TODO support for tuple types
                    ValueType::Tuple => (),
                }
            }
        };
    }
}
