use std::borrow::{Borrow, Cow};

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, Data, Field, Fields,
    FieldsNamed, FieldsUnnamed, Generics, Path, PathArguments, Variant,
};

use crate::{
    as_tokens_or_diagnostics,
    component::features::attributes::{Rename, ValueType},
    doc_comment::CommentAttributes,
    Deprecated, Diagnostics, OptionExt, ToTokensDiagnostics,
};

use self::{
    enums::{MixedEnum, PlainEnum},
    features::{
        EnumFeatures, FromAttributes, MixedEnumFeatures, NamedFieldFeatures,
        NamedFieldStructFeatures, UnnamedFieldStructFeatures,
    },
};

use super::{
    features::{
        attributes::{As, Description, RenameAll},
        parse_features, pop_feature, Feature, FeaturesExt, IntoInner, ToTokensExt,
    },
    serde::{self, SerdeContainer, SerdeValue},
    ComponentDescription, ComponentSchema, FieldRename, FlattenedMapSchema, TypeTree,
    VariantRename,
};

mod enums;
mod features;
pub mod xml;

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Root<'p> {
    pub ident: &'p Ident,
    pub generics: &'p Generics,
    pub attributes: &'p [Attribute],
}

pub struct Schema<'a> {
    ident: &'a Ident,
    attributes: &'a [Attribute],
    generics: &'a Generics,
    data: &'a Data,
}

impl<'a> Schema<'a> {
    pub fn new(
        data: &'a Data,
        attributes: &'a [Attribute],
        ident: &'a Ident,
        generics: &'a Generics,
    ) -> Result<Self, Diagnostics> {
        Ok(Self {
            data,
            ident,
            attributes,
            generics,
        })
    }
}

impl ToTokensDiagnostics for Schema<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        let ident = self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let parent = Root {
            ident,
            generics: self.generics,
            attributes: self.attributes,
        };
        let variant = SchemaVariant::new(self.data, &parent)?;

        let name = if let Some(schema_as) = variant.get_schema_as() {
            format_path_ref(&schema_as.0.path)
        } else {
            ident.to_string()
        };

        let mut variant_tokens = TokenStream::new();
        variant.to_tokens(&mut variant_tokens)?;

        tokens.extend(quote! {
            impl #impl_generics utoipa::__dev::ComposeSchema for #ident #ty_generics #where_clause {
                fn compose(
                    mut generics: Vec<utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>>
                ) -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
                    #variant_tokens.into()
                }
            }

            impl #impl_generics utoipa::ToSchema for #ident #ty_generics #where_clause {
                fn name() -> std::borrow::Cow<'static, str> {
                    std::borrow::Cow::Borrowed(#name)
                }
            }
        });
        Ok(())
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
enum SchemaVariant<'a> {
    Named(NamedStructSchema<'a>),
    Unnamed(UnnamedStructSchema<'a>),
    Enum(EnumSchema<'a>),
    Unit(UnitStructVariant),
}

impl<'a> SchemaVariant<'a> {
    pub fn new(data: &'a Data, parent: &'a Root<'a>) -> Result<SchemaVariant<'a>, Diagnostics> {
        match data {
            Data::Struct(content) => match &content.fields {
                Fields::Unnamed(fields) => {
                    let FieldsUnnamed { unnamed, .. } = fields;
                    let mut unnamed_features = parent
                        .attributes
                        .parse_features::<UnnamedFieldStructFeatures>()?
                        .into_inner();

                    let schema_as = pop_feature!(unnamed_features => Feature::As(_) as Option<As>);
                    let description =
                        pop_feature!(unnamed_features => Feature::Description(_)).into_inner();
                    Ok(Self::Unnamed(UnnamedStructSchema {
                        root: parent,
                        description,
                        features: unnamed_features,
                        fields: unnamed,
                        schema_as,
                    }))
                }
                Fields::Named(fields) => {
                    let FieldsNamed { named, .. } = fields;
                    let mut named_features = parent
                        .attributes
                        .parse_features::<NamedFieldStructFeatures>()?
                        .into_inner();
                    let schema_as = pop_feature!(named_features => Feature::As(_) as Option<As>);
                    let description =
                        pop_feature!(named_features => Feature::Description(_)).into_inner();

                    Ok(Self::Named(NamedStructSchema {
                        root: parent,
                        description,
                        rename_all: pop_feature!(named_features => Feature::RenameAll(_) as Option<RenameAll>),
                        features: named_features,
                        fields: named,
                        schema_as,
                    }))
                }
                Fields::Unit => Ok(Self::Unit(UnitStructVariant)),
            },
            Data::Enum(content) => Ok(Self::Enum(EnumSchema::new(parent, &content.variants)?)),
            _ => Err(Diagnostics::with_span(
                parent.ident.span(),
                "unexpected data type, expected syn::Data::Struct or syn::Data::Enum",
            )),
        }
    }

    fn get_schema_as(&self) -> &Option<As> {
        match self {
            Self::Enum(schema) => &schema.schema_as,
            Self::Named(schema) => &schema.schema_as,
            Self::Unnamed(schema) => &schema.schema_as,
            _ => &None,
        }
    }
}

impl ToTokensDiagnostics for SchemaVariant<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        match self {
            Self::Enum(schema) => schema.to_tokens(tokens),
            Self::Named(schema) => schema.to_tokens(tokens),
            Self::Unnamed(schema) => schema.to_tokens(tokens),
            Self::Unit(unit) => {
                unit.to_tokens(tokens);
                Ok(())
            }
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct UnitStructVariant;

impl ToTokens for UnitStructVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! {
            utoipa::openapi::schema::empty()
        });
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct NamedStructSchema<'a> {
    pub root: &'a Root<'a>,
    pub fields: &'a Punctuated<Field, Comma>,
    pub description: Option<Description>,
    pub features: Option<Vec<Feature>>,
    pub rename_all: Option<RenameAll>,
    pub schema_as: Option<As>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct NamedStructFieldOptions<'a> {
    property: Property,
    rename_field_value: Option<Cow<'a, str>>,
    required: Option<super::features::attributes::Required>,
    is_option: bool,
}

impl NamedStructSchema<'_> {
    fn get_named_struct_field_options(
        &self,
        field: &Field,
        field_rules: &SerdeValue,
        container_rules: &SerdeContainer,
    ) -> Result<NamedStructFieldOptions<'_>, Diagnostics> {
        let type_tree = &mut TypeTree::from_type(&field.ty)?;

        let mut field_features = field
            .attrs
            .parse_features::<NamedFieldFeatures>()?
            .into_inner();

        let schema_default = self
            .features
            .as_ref()
            .map(|features| features.iter().any(|f| matches!(f, Feature::Default(_))))
            .unwrap_or(false);
        let serde_default = container_rules.default;

        if schema_default || serde_default {
            let features_inner = field_features.get_or_insert(vec![]);
            if !features_inner
                .iter()
                .any(|f| matches!(f, Feature::Default(_)))
            {
                let field_ident = field.ident.as_ref().unwrap().to_owned();
                let struct_ident = format_ident!("{}", &self.root.ident);
                features_inner.push(Feature::Default(
                    crate::features::attributes::Default::new_default_trait(
                        struct_ident,
                        field_ident.into(),
                    ),
                ));
            }
        }

        // check for Rust's `#[deprecated]` attribute first, then check for `deprecated` feature
        let deprecated = super::get_deprecated(&field.attrs).or_else(|| {
            pop_feature!(field_features => Feature::Deprecated(_)).and_then(|feature| match feature
            {
                Feature::Deprecated(_) => Some(Deprecated::True),
                _ => None,
            })
        });

        let rename_field =
            pop_feature!(field_features => Feature::Rename(_)).and_then(|feature| match feature {
                Feature::Rename(rename) => Some(Cow::Owned(rename.into_value())),
                _ => None,
            });

        let value_type = field_features.as_mut().and_then(
            |features| pop_feature!(features => Feature::ValueType(_) as Option<ValueType>),
        );
        let override_type_tree = value_type
            .as_ref()
            .map_try(|value_type| value_type.as_type_tree())?;
        let comments = CommentAttributes::from_attributes(&field.attrs);
        let description = &ComponentDescription::CommentAttributes(&comments);

        let schema_with = pop_feature!(field_features => Feature::SchemaWith(_));
        let required = pop_feature!(field_features => Feature::Required(_) as Option<crate::component::features::attributes::Required>);
        let type_tree = override_type_tree.as_ref().unwrap_or(type_tree);

        let alias_type = type_tree.get_alias_type()?;
        let alias_type_tree = alias_type.as_ref().map_try(TypeTree::from_type)?;
        let type_tree = alias_type_tree.as_ref().unwrap_or(type_tree);

        let is_option = type_tree.is_option();

        Ok(NamedStructFieldOptions {
            property: if let Some(schema_with) = schema_with {
                Property::SchemaWith(schema_with)
            } else {
                let cs = super::ComponentSchemaProps {
                    type_tree,
                    features: field_features,
                    description: Some(description),
                    deprecated: deprecated.as_ref(),
                    container: &super::Container {
                        generics: self.root.generics,
                    },
                };
                if field_rules.flatten && type_tree.is_map() {
                    Property::FlattenedMap(FlattenedMapSchema::new(cs)?)
                } else {
                    Property::Schema(ComponentSchema::new(cs)?)
                }
            },
            rename_field_value: rename_field,
            required,
            is_option,
        })
    }
}

impl ToTokensDiagnostics for NamedStructSchema<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        let container_rules = serde::parse_container(self.root.attributes)?;

        let fields = self
            .fields
            .iter()
            .map(|field| {
                let mut field_name = Cow::Owned(field.ident.as_ref().unwrap().to_string());

                if Borrow::<str>::borrow(&field_name).starts_with("r#") {
                    field_name = Cow::Owned(field_name[2..].to_string());
                }

                let field_rules = serde::parse_value(&field.attrs);
                let field_rules = match field_rules {
                    Ok(field_rules) => field_rules,
                    Err(diagnostics) => return Err(diagnostics),
                };
                let field_options =
                    self.get_named_struct_field_options(field, &field_rules, &container_rules);

                match field_options {
                    Ok(field_options) => Ok((field_options, field_rules, field_name, field)),
                    Err(options_diagnostics) => Err(options_diagnostics),
                }
            })
            .collect::<Result<Vec<_>, Diagnostics>>()?;

        let mut object_tokens = fields
            .iter()
            .filter(|(_, field_rules, ..)| !field_rules.skip && !field_rules.flatten)
            .map(|(property, field_rules, field_name, field)| {
                Ok((
                    property,
                    field_rules,
                    field_name,
                    field,
                    as_tokens_or_diagnostics!(&property.property),
                ))
            })
            .collect::<Result<Vec<_>, Diagnostics>>()?
            .into_iter()
            .fold(
                quote! { utoipa::openapi::ObjectBuilder::new() },
                |mut object_tokens,
                 (
                    NamedStructFieldOptions {
                        rename_field_value,
                        required,
                        is_option,
                        ..
                    },
                    field_rules,
                    field_name,
                    _field,
                    field_schema,
                )| {
                    let rename_to = field_rules
                        .rename
                        .as_deref()
                        .map(Cow::Borrowed)
                        .or(rename_field_value.as_ref().cloned());
                    let rename_all = container_rules.rename_all.as_ref().or(self
                        .rename_all
                        .as_ref()
                        .map(|rename_all| rename_all.as_rename_rule()));

                    let name =
                        super::rename::<FieldRename>(field_name.borrow(), rename_to, rename_all)
                            .unwrap_or(Cow::Borrowed(field_name.borrow()));

                    object_tokens.extend(quote! {
                        .property(#name, #field_schema)
                    });
                    let component_required =
                        !is_option && super::is_required(field_rules, &container_rules);
                    let required = match (required, component_required) {
                        (Some(required), _) => required.is_true(),
                        (None, component_required) => component_required,
                    };

                    if required {
                        object_tokens.extend(quote! {
                            .required(#name)
                        })
                    }

                    object_tokens
                },
            );

        let flatten_fields = fields
            .iter()
            .filter(|(_, field_rules, ..)| field_rules.flatten)
            .collect::<Vec<_>>();

        let all_of = if !flatten_fields.is_empty() {
            let mut flattened_tokens = TokenStream::new();
            let mut flattened_map_field = None;

            for (options, _, _, field) in flatten_fields {
                let NamedStructFieldOptions { property, .. } = options;
                let property_schema = as_tokens_or_diagnostics!(property);

                match property {
                    Property::Schema(_) | Property::SchemaWith(_) => {
                        flattened_tokens.extend(quote! { .item(#property_schema) })
                    }
                    Property::FlattenedMap(_) => {
                        match flattened_map_field {
                            None => {
                                object_tokens.extend(
                                    quote! { .additional_properties(Some(#property_schema)) },
                                );
                                flattened_map_field = Some(field);
                            }
                            Some(flattened_map_field) => {
                                return Err(Diagnostics::with_span(
                                    self.fields.span(),
                                    format!("The structure `{}` contains multiple flattened map fields.", self.root.ident))
                                    .note(
                                        format!("first flattened map field was declared here as `{}`",
                                        flattened_map_field.ident.as_ref().unwrap()))
                                    .note(format!("second flattened map field was declared here as `{}`", field.ident.as_ref().unwrap()))
                                );
                            }
                        }
                    }
                }
            }

            if flattened_tokens.is_empty() {
                tokens.extend(object_tokens);
                false
            } else {
                tokens.extend(quote! {
                    utoipa::openapi::AllOfBuilder::new()
                        #flattened_tokens
                    .item(#object_tokens)
                });
                true
            }
        } else {
            tokens.extend(object_tokens);
            false
        };

        if !all_of && container_rules.deny_unknown_fields {
            tokens.extend(quote! {
                .additional_properties(Some(utoipa::openapi::schema::AdditionalProperties::FreeForm(false)))
            });
        }

        if let Some(deprecated) = super::get_deprecated(self.root.attributes) {
            tokens.extend(quote! { .deprecated(Some(#deprecated)) });
        }

        if let Some(struct_features) = self.features.as_ref() {
            tokens.extend(struct_features.to_token_stream()?)
        }

        let comments = CommentAttributes::from_attributes(self.root.attributes);
        let description = self
            .description
            .as_ref()
            .map(ComponentDescription::Description)
            .or(Some(ComponentDescription::CommentAttributes(&comments)));

        description.to_tokens(tokens);

        Ok(())
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct UnnamedStructSchema<'a> {
    root: &'a Root<'a>,
    fields: &'a Punctuated<Field, Comma>,
    description: Option<Description>,
    features: Option<Vec<Feature>>,
    schema_as: Option<As>,
}

impl ToTokensDiagnostics for UnnamedStructSchema<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        let fields_len = self.fields.len();
        let first_field = self.fields.first().unwrap();
        let first_part = &TypeTree::from_type(&first_field.ty)?;

        let all_fields_are_same = fields_len == 1
            || self
                .fields
                .iter()
                .skip(1)
                .map(|field| TypeTree::from_type(&field.ty))
                .collect::<Result<Vec<TypeTree>, Diagnostics>>()?
                .iter()
                .all(|schema_part| first_part == schema_part);

        let deprecated = super::get_deprecated(self.root.attributes);
        if all_fields_are_same {
            let mut unnamed_struct_features = self.features.clone();
            let value_type = unnamed_struct_features.as_mut().and_then(
                |features| pop_feature!(features => Feature::ValueType(_) as Option<ValueType>),
            );
            let override_type_tree = value_type
                .as_ref()
                .map_try(|value_type| value_type.as_type_tree())?;

            if fields_len == 1 {
                if let Some(ref mut features) = unnamed_struct_features {
                    let inline =
                        features::parse_schema_features_with(&first_field.attrs, |input| {
                            Ok(parse_features!(
                                input as super::features::attributes::Inline
                            ))
                        })?
                        .unwrap_or_default();

                    features.extend(inline);

                    if pop_feature!(features => Feature::Default(crate::features::attributes::Default(None)))
                        .is_some()
                    {
                        let struct_ident = format_ident!("{}", &self.root.ident);
                        let index: syn::Index = 0.into();
                        features.push(Feature::Default(
                            crate::features::attributes::Default::new_default_trait(struct_ident, index.into()),
                        ));
                    }
                }
            }

            let comments = CommentAttributes::from_attributes(self.root.attributes);
            let description = self
                .description
                .as_ref()
                .map(ComponentDescription::Description)
                .or(Some(ComponentDescription::CommentAttributes(&comments)));
            let type_tree = override_type_tree.as_ref().unwrap_or(first_part);

            let alias_type = type_tree.get_alias_type()?;
            let alias_type_tree = alias_type.as_ref().map_try(TypeTree::from_type)?;
            let type_tree = alias_type_tree.as_ref().unwrap_or(type_tree);

            tokens.extend(
                ComponentSchema::new(super::ComponentSchemaProps {
                    type_tree,
                    features: unnamed_struct_features,
                    description: description.as_ref(),
                    deprecated: deprecated.as_ref(),
                    container: &super::Container {
                        generics: self.root.generics,
                    },
                })?
                .to_token_stream(),
            );
        } else {
            // Struct that has multiple unnamed fields is serialized to array by default with serde.
            // See: https://serde.rs/json.html
            // Typically OpenAPI does not support multi type arrays thus we simply consider the case
            // as generic object array
            tokens.extend(quote! {
                utoipa::openapi::ObjectBuilder::new()
            });

            if let Some(deprecated) = deprecated {
                tokens.extend(quote! { .deprecated(Some(#deprecated)) });
            }

            if let Some(ref attrs) = self.features {
                tokens.extend(attrs.to_token_stream()?)
            }
        }

        if fields_len > 1 {
            let comments = CommentAttributes::from_attributes(self.root.attributes);
            let description = self
                .description
                .as_ref()
                .map(ComponentDescription::Description)
                .or(Some(ComponentDescription::CommentAttributes(&comments)));
            tokens.extend(quote! {
            .to_array_builder()
                .max_items(Some(#fields_len))
                .min_items(Some(#fields_len))
                #description
            })
        }

        Ok(())
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct EnumSchema<'a> {
    schema_type: EnumSchemaType<'a>,
    schema_as: Option<As>,
}

impl<'e> EnumSchema<'e> {
    pub fn new(
        parent: &'e Root<'e>,
        variants: &'e Punctuated<Variant, Comma>,
    ) -> Result<Self, Diagnostics> {
        if variants
            .iter()
            .all(|variant| matches!(variant.fields, Fields::Unit))
        {
            #[cfg(feature = "repr")]
            let mut features = {
                if parent
                    .attributes
                    .iter()
                    .any(|attr| attr.path().is_ident("repr"))
                {
                    features::parse_schema_features_with(parent.attributes, |input| {
                        Ok(parse_features!(
                            input as super::features::attributes::Example,
                            super::features::attributes::Examples,
                            super::features::attributes::Default,
                            super::features::attributes::Title,
                            As
                        ))
                    })?
                    .unwrap_or_default()
                } else {
                    parent
                        .attributes
                        .parse_features::<EnumFeatures>()?
                        .into_inner()
                        .unwrap_or_default()
                }
            };
            #[cfg(not(feature = "repr"))]
            let mut features = {
                parent
                    .attributes
                    .parse_features::<EnumFeatures>()?
                    .into_inner()
                    .unwrap_or_default()
            };

            let schema_as = pop_feature!(features => Feature::As(_) as Option<As>);

            Ok(Self {
                schema_type: EnumSchemaType::Plain(PlainEnum::new(parent, variants, features)?),
                schema_as,
            })
        } else {
            let mut enum_features = parent
                .attributes
                .parse_features::<MixedEnumFeatures>()?
                .into_inner()
                .unwrap_or_default();
            let schema_as = pop_feature!(enum_features => Feature::As(_) as Option<As>);

            Ok(Self {
                schema_type: EnumSchemaType::Mixed(MixedEnum::new(
                    parent,
                    variants,
                    enum_features,
                )?),
                schema_as,
            })
        }
    }
}

impl ToTokensDiagnostics for EnumSchema<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        self.schema_type.to_tokens(tokens)
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
enum EnumSchemaType<'e> {
    Mixed(MixedEnum<'e>),
    Plain(PlainEnum<'e>),
}

impl ToTokensDiagnostics for EnumSchemaType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        let (attributes, description) = match self {
            Self::Mixed(mixed) => {
                mixed.to_tokens(tokens);
                (mixed.root.attributes, &mixed.description)
            }
            Self::Plain(plain) => {
                plain.to_tokens(tokens);
                (plain.root.attributes, &plain.description)
            }
        };

        if let Some(deprecated) = super::get_deprecated(attributes) {
            tokens.extend(quote! { .deprecated(Some(#deprecated)) });
        }
        let comments = CommentAttributes::from_attributes(attributes);
        let description = description
            .as_ref()
            .map(ComponentDescription::Description)
            .or(Some(ComponentDescription::CommentAttributes(&comments)));

        description.to_tokens(tokens);

        Ok(())
    }
}

fn rename_enum_variant<'s>(
    name: &str,
    features: &mut Vec<Feature>,
    variant_rules: &'s SerdeValue,
    container_rules: &'s SerdeContainer,
    rename_all: Option<&RenameAll>,
) -> Option<Cow<'s, str>> {
    let rename = pop_feature!(features => Feature::Rename(_) as Option<Rename>)
        .map(|rename| rename.into_value());
    let rename_to = variant_rules
        .rename
        .as_deref()
        .map(Cow::Borrowed)
        .or(rename.map(Cow::Owned));

    let rename_all = container_rules.rename_all.as_ref().or(rename_all
        .as_ref()
        .map(|rename_all| rename_all.as_rename_rule()));

    super::rename::<VariantRename>(name, rename_to, rename_all)
}

#[cfg_attr(feature = "debug", derive(Debug))]
enum Property {
    Schema(ComponentSchema),
    SchemaWith(Feature),
    FlattenedMap(FlattenedMapSchema),
}

impl ToTokensDiagnostics for Property {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        match self {
            Self::Schema(schema) => schema.to_tokens(tokens)?,
            Self::FlattenedMap(schema) => schema.to_tokens(tokens)?,
            Self::SchemaWith(schema_with) => schema_with.to_tokens(tokens)?,
        }
        Ok(())
    }
}

/// Reformat a path reference string that was generated using [`quote`] to be used as a nice compact schema reference,
/// by removing spaces between colon punctuation and `::` and the path segments.
pub(crate) fn format_path_ref(path: &Path) -> String {
    let mut path = path.clone();

    // Generics and path arguments are unsupported
    if let Some(last_segment) = path.segments.last_mut() {
        last_segment.arguments = PathArguments::None;
    }
    // :: are not officially supported in the spec
    // See: https://github.com/juhaku/utoipa/pull/187#issuecomment-1173101405
    path.to_token_stream().to_string().replace(" :: ", ".")
}
