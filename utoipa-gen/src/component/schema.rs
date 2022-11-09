use std::{borrow::Cow, mem};

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::Parse, parse_quote, punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute,
    Data, Field, Fields, FieldsNamed, FieldsUnnamed, Generics, Path, PathArguments, Token,
    TypePath, Variant, Visibility,
};

use crate::{
    doc_comment::CommentAttributes,
    schema_type::{SchemaFormat, SchemaType},
    Array, Deprecated,
};

use self::features::{
    EnumFeatures, FromAttributes, NamedFieldFeatures, NamedFieldStructFeatures,
    UnnamedFieldStructFeatures,
};

use super::{
    features::{parse_features, Feature, FeaturesExt, IntoInner, IsInline, ToTokensExt},
    serde::{self, SerdeContainer, SerdeValue},
    FieldRename, GenericType, TypeTree, ValueType, VariantRename,
};

mod features;
pub mod xml;

pub struct Schema<'a> {
    ident: &'a Ident,
    attributes: &'a [Attribute],
    generics: &'a Generics,
    aliases: Option<Punctuated<AliasSchema, Comma>>,
    data: &'a Data,
    vis: &'a Visibility,
}

impl<'a> Schema<'a> {
    pub fn new(
        data: &'a Data,
        attributes: &'a [Attribute],
        ident: &'a Ident,
        generics: &'a Generics,
        vis: &'a Visibility,
    ) -> Self {
        let aliases = if generics.type_params().count() > 0 {
            parse_aliases(attributes)
        } else {
            None
        };

        Self {
            data,
            ident,
            attributes,
            generics,
            aliases,
            vis,
        }
    }
}

impl ToTokens for Schema<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = self.ident;
        let variant = SchemaVariant::new(self.data, self.attributes, ident, self.generics, None);
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let aliases = self.aliases.as_ref().map(|aliases| {
            let alias_schemas = aliases
                .iter()
                .map(|alias| {
                    let name = &*alias.name;

                    let variant = SchemaVariant::new(
                        self.data,
                        self.attributes,
                        ident,
                        self.generics,
                        Some(alias),
                    );
                    quote! { (#name, #variant.into()) }
                })
                .collect::<Array<TokenStream>>();

            quote! {
                fn aliases() -> Vec<(&'static str, utoipa::openapi::schema::Schema)> {
                    #alias_schemas.to_vec()
                }
            }
        });

        let type_aliases = self.aliases.as_ref().map(|aliases| {
            aliases
                .iter()
                .map(|alias| {
                    let name = quote::format_ident!("{}", alias.name);
                    let ty = &alias.ty;
                    let (_, alias_type_generics, _) = &alias.generics.split_for_impl();
                    let vis = self.vis;

                    quote! {
                        #vis type #name = #ty #alias_type_generics;
                    }
                })
                .fold(quote! {}, |mut tokens, alias| {
                    tokens.extend(alias);

                    tokens
                })
        });

        tokens.extend(quote! {
            impl #impl_generics utoipa::ToSchema for #ident #ty_generics #where_clause {
                fn schema() -> utoipa::openapi::schema::Schema {
                    #variant.into()
                }

                #aliases
            }

            #type_aliases
        })
    }
}

enum SchemaVariant<'a> {
    Named(NamedStructSchema<'a>),
    Unnamed(UnnamedStructSchema<'a>),
    Enum(EnumSchema<'a>),
}

impl<'a> SchemaVariant<'a> {
    pub fn new(
        data: &'a Data,
        attributes: &'a [Attribute],
        ident: &'a Ident,
        generics: &'a Generics,
        alias: Option<&'a AliasSchema>,
    ) -> SchemaVariant<'a> {
        match data {
            Data::Struct(content) => match &content.fields {
                Fields::Unnamed(fields) => {
                    let FieldsUnnamed { unnamed, .. } = fields;
                    Self::Unnamed(UnnamedStructSchema {
                        attributes,
                        features: attributes
                            .parse_features::<UnnamedFieldStructFeatures>()
                            .into_inner(),
                        fields: unnamed,
                    })
                }
                Fields::Named(fields) => {
                    let FieldsNamed { named, .. } = fields;
                    Self::Named(NamedStructSchema {
                        attributes,
                        features: attributes
                            .parse_features::<NamedFieldStructFeatures>()
                            .into_inner(),
                        fields: named,
                        generics: Some(generics),
                        alias,
                    })
                }
                Fields::Unit => abort!(
                    ident.span(),
                    "unexpected Field::Unit expected struct with Field::Named or Field::Unnamed"
                ),
            },
            Data::Enum(content) => Self::Enum(EnumSchema {
                attributes,
                variants: &content.variants,
            }),
            _ => abort!(
                ident.span(),
                "unexpected data type, expected syn::Data::Struct or syn::Data::Enum"
            ),
        }
    }
}

impl ToTokens for SchemaVariant<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Enum(schema) => schema.to_tokens(tokens),
            Self::Named(schema) => schema.to_tokens(tokens),
            Self::Unnamed(schema) => schema.to_tokens(tokens),
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct NamedStructSchema<'a> {
    fields: &'a Punctuated<Field, Comma>,
    attributes: &'a [Attribute],
    features: Option<Vec<Feature>>,
    generics: Option<&'a Generics>,
    alias: Option<&'a AliasSchema>,
}

impl ToTokens for NamedStructSchema<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let container_rules = serde::parse_container(self.attributes);
        let mut object_tokens = quote! { utoipa::openapi::ObjectBuilder::new() };

        let flatten_fields: Vec<&Field> = self
            .fields
            .iter()
            .filter(|field| {
                let field_rule = serde::parse_value(&field.attrs);
                is_flatten(&field_rule)
            })
            .collect();

        self.fields
            .iter()
            .filter_map(|field| {
                let field_rule = serde::parse_value(&field.attrs);

                if is_flatten(&field_rule) {
                    return None;
                };

                if is_not_skipped(&field_rule) {
                    Some((field, field_rule))
                } else {
                    None
                }
            })
            .for_each(|(field, field_rule)| {
                let mut field_name = &*field.ident.as_ref().unwrap().to_string();

                if field_name.starts_with("r#") {
                    field_name = &field_name[2..];
                }

                let name = super::rename::<FieldRename>(
                    field_name,
                    field_rule
                        .as_ref()
                        .map(|field_rule| field_rule.rename.as_str()),
                    container_rules
                        .as_ref()
                        .and_then(|container_rule| container_rule.rename_all.as_ref()),
                )
                .unwrap_or(Cow::Borrowed(field_name));

                with_field_as_schema_property(self, field, |schema_property| {
                    object_tokens.extend(quote! {
                        .property(#name, #schema_property)
                    });

                    if !schema_property.is_option()
                        && !super::is_default(&container_rules.as_ref(), &field_rule.as_ref())
                    {
                        object_tokens.extend(quote! {
                            .required(#name)
                        })
                    }
                })
            });

        if !flatten_fields.is_empty() {
            tokens.extend(quote! {
                utoipa::openapi::AllOfBuilder::new()
            });

            for field in flatten_fields {
                with_field_as_schema_property(self, field, |schema_property| {
                    tokens.extend(quote! { .item(#schema_property) });
                })
            }

            tokens.extend(quote! {
                .item(#object_tokens)
            })
        } else {
            tokens.extend(object_tokens)
        }

        if let Some(deprecated) = super::get_deprecated(self.attributes) {
            tokens.extend(quote! { .deprecated(Some(#deprecated)) });
        }

        if let Some(struct_features) = self.features.as_ref() {
            tokens.extend(struct_features.to_token_stream())
        }

        if let Some(comment) = CommentAttributes::from_attributes(self.attributes).first() {
            tokens.extend(quote! {
                .description(Some(#comment))
            })
        }
    }
}

fn with_field_as_schema_property<R>(
    schema: &NamedStructSchema,
    field: &Field,
    yield_: impl FnOnce(SchemaProperty<'_>) -> R,
) -> R {
    let type_tree = &mut TypeTree::from_type(&field.ty);

    let mut field_features = field
        .attrs
        .parse_features::<NamedFieldFeatures>()
        .into_inner();

    if let Some((generic_types, alias)) = schema.generics.zip(schema.alias) {
        generic_types
            .type_params()
            .enumerate()
            .for_each(|(index, generic)| {
                if let Some(generic_type) = type_tree.find_mut_by_ident(&generic.ident) {
                    generic_type
                        .update_path(&alias.generics.type_params().nth(index).unwrap().ident);
                };
            })
    }

    let deprecated = super::get_deprecated(&field.attrs);
    let value_type = field_features
        .as_mut()
        .and_then(|features| features.find_value_type_feature_as_value_type());
    let override_type_tree = value_type
        .as_ref()
        .map(|value_type| value_type.as_type_tree());
    let comments = CommentAttributes::from_attributes(&field.attrs);

    yield_(SchemaProperty::new(
        override_type_tree.as_ref().unwrap_or(type_tree),
        Some(&comments),
        field_features.as_ref(),
        deprecated.as_ref(),
    ))
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct UnnamedStructSchema<'a> {
    fields: &'a Punctuated<Field, Comma>,
    attributes: &'a [Attribute],
    features: Option<Vec<Feature>>,
}

impl ToTokens for UnnamedStructSchema<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fields_len = self.fields.len();
        let first_field = self.fields.first().unwrap();
        let first_part = &TypeTree::from_type(&first_field.ty);

        let mut is_object = matches!(first_part.value_type, ValueType::Object);

        let all_fields_are_same = fields_len == 1
            || self.fields.iter().skip(1).all(|field| {
                let schema_part = &TypeTree::from_type(&field.ty);

                first_part == schema_part
            });

        let deprecated = super::get_deprecated(self.attributes);
        if all_fields_are_same {
            let mut unnamed_struct_features = self.features.clone();
            let value_type = unnamed_struct_features
                .as_mut()
                .and_then(|features| features.find_value_type_feature_as_value_type());
            let override_type_tree = value_type
                .as_ref()
                .map(|value_type| value_type.as_type_tree());

            if override_type_tree.is_some() {
                is_object = override_type_tree
                    .as_ref()
                    .map(|override_type| matches!(override_type.value_type, ValueType::Object))
                    .unwrap_or_default();
            }

            tokens.extend(
                SchemaProperty::new(
                    override_type_tree.as_ref().unwrap_or(first_part),
                    None,
                    unnamed_struct_features.as_ref(),
                    deprecated.as_ref(),
                )
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
                tokens.extend(attrs.to_token_stream())
            }
        };

        if let Some(comment) = CommentAttributes::from_attributes(self.attributes).first() {
            if !is_object {
                tokens.extend(quote! {
                    .description(Some(#comment))
                })
            }
        }

        if fields_len > 1 {
            tokens.extend(
                quote! { .to_array_builder().max_items(Some(#fields_len)).min_items(Some(#fields_len)) },
            )
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct EnumSchema<'a> {
    variants: &'a Punctuated<Variant, Comma>,
    attributes: &'a [Attribute],
}

impl ToTokens for EnumSchema<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self
            .variants
            .iter()
            .all(|variant| matches!(variant.fields, Fields::Unit))
        {
            #[cfg(feature = "repr")]
            {
                tokens.extend(
                    self.attributes
                        .iter()
                        .find_map(|attribute| {
                            if attribute.path.is_ident("repr") {
                                attribute.parse_args::<TypePath>().ok()
                            } else {
                                None
                            }
                        })
                        .map(|enum_type| {
                            ReprEnum {
                                variants: self.variants,
                                attributes: self.attributes,
                                enum_type,
                            }
                            .to_token_stream()
                        })
                        .unwrap_or_else(|| {
                            SimpleEnum {
                                attributes: self.attributes,
                                variants: self.variants,
                            }
                            .to_token_stream()
                        }),
                )
            }

            #[cfg(not(feature = "repr"))]
            {
                tokens.extend(
                    SimpleEnum {
                        attributes: self.attributes,
                        variants: self.variants,
                    }
                    .to_token_stream(),
                )
            }
        } else {
            tokens.extend(
                ComplexEnum {
                    attributes: self.attributes,
                    variants: self.variants,
                }
                .to_token_stream(),
            )
        };
    }
}

#[cfg(feature = "repr")]
#[cfg_attr(feature = "debug", derive(Debug))]
struct ReprEnum<'a> {
    variants: &'a Punctuated<Variant, Comma>,
    attributes: &'a [Attribute],
    enum_type: TypePath,
}

#[cfg(feature = "repr")]
impl<'e> EnumTokens<'e> for ReprEnum<'e> {
    fn variant_to_tokens_stream(
        &self,
        variant: &Variant,
        _: &Option<SerdeContainer>,
    ) -> Option<TokenStream> {
        let variant_rules = serde::parse_value(&variant.attrs);
        if is_not_skipped(&variant_rules) {
            let ty = &variant.ident;
            let repr = &self.enum_type;

            Some(quote! {
                Self::#ty as #repr
            })
        } else {
            None
        }
    }
}

#[cfg(feature = "repr")]
impl ToTokens for ReprEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        Self::to_enum_tokens(
            self,
            (self.variants, self.attributes, Some(&self.enum_type)),
            tokens,
            |tokens, container_rules| {
                self.unit_enum_tokens(
                    container_rules,
                    (self.variants, self.attributes, Some(&self.enum_type)),
                    tokens,
                )
            },
        )
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct SimpleEnum<'a> {
    variants: &'a Punctuated<Variant, Comma>,
    attributes: &'a [Attribute],
}

impl<'e> EnumTokens<'e> for SimpleEnum<'e> {
    fn variant_to_tokens_stream(
        &self,
        variant: &Variant,
        container_rules: &Option<SerdeContainer>,
    ) -> Option<TokenStream> {
        let variant_rules = serde::parse_value(&variant.attrs);

        if is_not_skipped(&variant_rules) {
            let name = &*variant.ident.to_string();
            let variant_name = super::rename::<VariantRename>(
                name,
                variant_rules
                    .as_ref()
                    .map(|field_rule| field_rule.rename.as_str()),
                container_rules
                    .as_ref()
                    .and_then(|container_rule| container_rule.rename_all.as_ref()),
            );

            if let Some(renamed_name) = variant_name {
                Some(quote! { #renamed_name })
            } else {
                Some(quote! { #name })
            }
        } else {
            None
        }
    }
}

impl ToTokens for SimpleEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        Self::to_enum_tokens(
            self,
            (self.variants, self.attributes, None),
            tokens,
            |tokens, container_rules| {
                self.unit_enum_tokens(
                    container_rules,
                    (self.variants, self.attributes, None),
                    tokens,
                )
            },
        )
    }
}

type EnumContent<'e> = (
    &'e Punctuated<Variant, Comma>,
    &'e [Attribute],
    Option<&'e TypePath>,
);

trait EnumTokens<'e>: ToTokens {
    fn variant_to_tokens_stream(
        &self,
        _: &Variant,
        _: &Option<SerdeContainer>,
    ) -> Option<TokenStream> {
        None
    }

    fn to_enum_tokens(
        &self,
        content: EnumContent,
        tokens: &mut TokenStream,
        enum_value_to_tokens: impl FnOnce(&mut TokenStream, &Option<SerdeContainer>),
    ) where
        Self: Sized,
    {
        let (_, attributes, _) = content;

        let container_rules = serde::parse_container(attributes);
        enum_value_to_tokens(tokens, &container_rules);

        let attrs = attributes.parse_features::<EnumFeatures>();

        if let Some(attributes) = attrs {
            tokens.extend(attributes.into_inner().to_token_stream());
        }

        if let Some(deprecated) = super::get_deprecated(attributes) {
            tokens.extend(quote! { .deprecated(Some(#deprecated)) });
        }

        if let Some(comment) = CommentAttributes::from_attributes(attributes).first() {
            tokens.extend(quote! {
                .description(Some(#comment))
            })
        }
    }

    fn unit_enum_tokens(
        &self,
        container_rules: &Option<SerdeContainer>,
        content: EnumContent,
        tokens: &mut TokenStream,
    ) {
        let (variants, _, enum_type) = content;

        let enum_values = variants
            .iter()
            .filter_map(|variant| self.variant_to_tokens_stream(variant, container_rules))
            .collect::<Array<TokenStream>>();

        tokens.extend(match container_rules {
            Some(serde_container) if !serde_container.tag.is_empty() => EnumRepresentation {
                enum_type,
                values: enum_values,
                title: &None,
            }
            .to_tagged_tokens(serde_container.tag.as_str()),
            _ => EnumRepresentation {
                enum_type,
                values: enum_values,
                title: &None,
            }
            .to_tokens(),
        });
    }
}

struct ComplexEnum<'a> {
    variants: &'a Punctuated<Variant, Comma>,
    attributes: &'a [Attribute],
}

impl ComplexEnum<'_> {
    /// Produce tokens that represent a variant of a [`ComplexEnum`].
    fn variant_tokens(variant_name: Cow<'_, str>, variant: &Variant) -> TokenStream {
        // TODO need to be able to split variant.attrs for variant and the struct representation!
        match &variant.fields {
            Fields::Named(named_fields) => {
                let (title_features, named_struct_features) = variant
                    .attrs
                    .parse_features::<NamedFieldStructFeatures>()
                    .into_inner()
                    .map(|features| features.split_for_title())
                    .unwrap_or_default();

                FieldVariantTokens::to_tokens(
                    &EnumVariantTokens(&variant_name, title_features),
                    NamedStructSchema {
                        attributes: &variant.attrs,
                        features: Some(named_struct_features),
                        fields: &named_fields.named,
                        generics: None,
                        alias: None,
                    },
                )
            }
            Fields::Unnamed(unnamed_fields) => {
                let (title_features, unnamed_struct_features) = variant
                    .attrs
                    .parse_features::<UnnamedFieldStructFeatures>()
                    .into_inner()
                    .map(|features| features.split_for_title())
                    .unwrap_or_default();

                FieldVariantTokens::to_tokens(
                    &EnumVariantTokens(&variant_name, title_features),
                    UnnamedStructSchema {
                        attributes: &variant.attrs,
                        features: Some(unnamed_struct_features),
                        fields: &unnamed_fields.unnamed,
                    },
                )
            }
            Fields::Unit => {
                let unit_features = features::parse_schema_features_with(&variant.attrs, |input| {
                    Ok(parse_features!(input as super::features::Title))
                })
                .unwrap_or_default();

                UnitVariantTokens::to_tokens(&EnumVariantTokens(&variant_name, unit_features))
            }
        }
    }

    /// Produce tokens that represent a variant of a [`ComplexEnum`] where serde enum attribute
    /// `tag = ` applies.
    fn tagged_variant_tokens(
        tag: &str,
        variant_name: Cow<'_, str>,
        variant: &Variant,
    ) -> TokenStream {
        match &variant.fields {
            Fields::Named(named_fields) => {
                let (title_features, named_struct_features) = variant
                    .attrs
                    .parse_features::<NamedFieldStructFeatures>()
                    .into_inner()
                    .map(|features| features.split_for_title())
                    .unwrap_or_default();

                let named_enum = NamedStructSchema {
                    attributes: &variant.attrs,
                    features: Some(named_struct_features),
                    fields: &named_fields.named,
                    generics: None,
                    alias: None,
                };
                let title = title_features.to_token_stream();

                let variant_name_tokens =
                    UnitVariantTokens::to_tokens(&EnumVariantTokens(&variant_name, Vec::new()));

                quote! {
                    #named_enum
                        #title
                        .property(#tag, #variant_name_tokens)
                        .required(#tag)
                }
            }
            Fields::Unnamed(unnamed_fields) => {
                if unnamed_fields.unnamed.len() == 1 {
                    let (title_features, unnamed_struct_features) = variant
                        .attrs
                        .parse_features::<UnnamedFieldStructFeatures>()
                        .into_inner()
                        .map(|features| features.split_for_title())
                        .unwrap_or_default();

                    let unnamed_enum = UnnamedStructSchema {
                        attributes: &variant.attrs,
                        features: Some(unnamed_struct_features),
                        fields: &unnamed_fields.unnamed,
                    };

                    let title = title_features.to_token_stream();
                    let variant_name_tokens =
                        UnitVariantTokens::to_tokens(&EnumVariantTokens(&variant_name, Vec::new()));

                    let is_reference = unnamed_fields.unnamed.iter().any(|field| {
                        let ty = TypeTree::from_type(&field.ty);

                        ty.value_type == ValueType::Object
                    });

                    if is_reference {
                        quote! {
                            utoipa::openapi::schema::AllOfBuilder::new()
                                #title
                                .item(#unnamed_enum)
                                .item(utoipa::openapi::schema::ObjectBuilder::new()
                                    .schema_type(utoipa::openapi::schema::SchemaType::Object)
                                    .property(#tag, #variant_name_tokens)
                                    .required(#tag)
                                )
                        }
                    } else {
                        quote! {
                            #unnamed_enum
                                #title
                                .schema_type(utoipa::openapi::schema::SchemaType::Object)
                                .property(#tag, #variant_name_tokens)
                                .required(#tag)
                        }
                    }
                } else {
                    abort!(
                        variant,
                        "Unnamed (tuple) enum variants are unsupported for internally tagged enums using the `tag = ` serde attribute";

                        help = "Try using a different serde enum representation";
                        note = "See more about enum limitations here: `https://serde.rs/enum-representations.html#internally-tagged`"
                    );
                }
            }
            Fields::Unit => {
                let variant_tokens =
                    UnitVariantTokens::to_tokens(&EnumVariantTokens(&variant_name, Vec::new()));

                let unit_features = features::parse_schema_features_with(&variant.attrs, |input| {
                    Ok(parse_features!(input as super::features::Title))
                })
                .unwrap_or_default()
                .to_token_stream();

                quote! {
                    utoipa::openapi::schema::ObjectBuilder::new()
                        #unit_features
                        .property(#tag, #variant_tokens)
                        .required(#tag)
                }
            }
        }
    }
}

impl<'e> EnumTokens<'e> for ComplexEnum<'e> {}

impl ToTokens for ComplexEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        Self::to_enum_tokens(
            self,
            (self.variants, self.attributes, None),
            tokens,
            |tokens, container_rules| {
                let capacity = self.variants.len();
                let tag = container_rules.as_ref().and_then(|rules| {
                    if !rules.tag.is_empty() {
                        Some(&rules.tag)
                    } else {
                        None
                    }
                });

                // serde, externally tagged format supported by now
                let items: TokenStream = self
                    .variants
                    .iter()
                    .filter_map(|variant: &Variant| {
                        let variant_serde_rules = serde::parse_value(&variant.attrs);
                        if is_not_skipped(&variant_serde_rules) {
                            Some((variant, variant_serde_rules))
                        } else {
                            None
                        }
                    })
                    .map(|(variant, variant_serde_rules)| {
                        let variant_name = &*variant.ident.to_string();

                        let name = super::rename::<VariantRename>(
                            variant_name,
                            variant_serde_rules
                                .as_ref()
                                .map(|field_rule| field_rule.rename.as_str()),
                            container_rules
                                .as_ref()
                                .and_then(|container_rule| container_rule.rename_all.as_ref()),
                        )
                        .unwrap_or(Cow::Borrowed(variant_name));

                        if let Some(tag) = tag {
                            Self::tagged_variant_tokens(tag, name, variant)
                        } else {
                            Self::variant_tokens(name, variant)
                        }
                    })
                    .map(|inline_variant| {
                        quote! {
                            .item(#inline_variant)
                        }
                    })
                    .collect();
                // for now just use tag as a discriminator
                let discriminator = tag.map(|tag| {
                    quote! {
                        .discriminator(Some(utoipa::openapi::schema::Discriminator::new(#tag)))
                    }
                });

                tokens.extend(
                    quote! {
                        Into::<utoipa::openapi::schema::OneOfBuilder>::into(utoipa::openapi::OneOf::with_capacity(#capacity))
                            #items
                            #discriminator
                    }
                );
            },
        );
    }
}

struct EnumRepresentation<'e, 'a, T: quote::ToTokens> {
    enum_type: Option<&'e TypePath>,
    values: Array<'a, T>,
    title: &'e Option<TokenStream>,
}

impl<'e, 'a, T> EnumRepresentation<'e, 'a, T>
where
    T: quote::ToTokens,
{
    fn to_tokens(&self) -> TokenStream {
        let EnumRepresentation {
            enum_type,
            values,
            title,
        } = self;
        let len = values.len();
        let (schema_type, enum_type) = if let Some(ty) = *enum_type {
            (
                SchemaType(&ty.path).to_token_stream(),
                ty.into_token_stream(),
            )
        } else {
            (
                SchemaType(&parse_quote!(str)).to_token_stream(),
                quote! {&str},
            )
        };

        quote! {
            utoipa::openapi::ObjectBuilder::new()
                #title
                .schema_type(#schema_type)
                .enum_values::<[#enum_type; #len], #enum_type>(Some(#values))
        }
    }
}

trait TaggedEnumRepresentation {
    fn to_tagged_tokens(&self, tag: &str) -> TokenStream;
}

impl<'e, 'a, T> TaggedEnumRepresentation for EnumRepresentation<'e, 'a, T>
where
    T: quote::ToTokens,
{
    fn to_tagged_tokens(&self, tag: &str) -> TokenStream {
        let EnumRepresentation {
            enum_type, values, ..
        } = self;
        let len = values.len();

        let items = values
            .iter()
            .map(|enum_value| {
                let values = [enum_value];
                let enum_tokens = EnumRepresentation {
                    enum_type: *enum_type,
                    values: Array::Borrowed(&values),
                    title: &None,
                }
                .to_tokens();

                quote! {
                    utoipa::openapi::schema::ObjectBuilder::new()
                        .property(
                            #tag,
                            #enum_tokens
                        )
                        .required(#tag)
                }
            })
            .map(|quote| quote! {.item(#quote)})
            .collect::<TokenStream>();

        quote! {
            Into::<utoipa::openapi::schema::OneOfBuilder>::into(utoipa::openapi::OneOf::with_capacity(#len))
                #items
        }
    }
}

struct EnumVariantTokens<'a>(&'a str, Vec<Feature>);

impl EnumVariantTokens<'_> {
    fn get_title(&self) -> Option<TokenStream> {
        let EnumVariantTokens(_, title) = self;

        Some(title.to_token_stream())
        // title
        //     .as_ref()
        //     .map(ToTokens::into_token_stream)
        //     .or_else(|| Some(quote! {.title(#name)}))
    }
}

trait UnitVariantTokens {
    fn to_tokens(&self) -> TokenStream;
}

impl UnitVariantTokens for EnumVariantTokens<'_> {
    fn to_tokens(&self) -> TokenStream {
        let EnumVariantTokens(name, _) = self;
        let values = [*name];
        let variant_title = self.get_title();

        EnumRepresentation {
            enum_type: None,
            values: Array::Borrowed(&values),
            title: &variant_title,
        }
        .to_tokens()
    }
}

trait FieldVariantTokens {
    fn to_tokens<T: quote::ToTokens>(&self, variant: T) -> TokenStream;
}

impl FieldVariantTokens for EnumVariantTokens<'_> {
    fn to_tokens<T: quote::ToTokens>(&self, variant: T) -> TokenStream {
        let EnumVariantTokens(name, _) = self;
        let variant_title = self.get_title();

        quote! {
            utoipa::openapi::schema::ObjectBuilder::new()
                #variant_title
                .property(#name, #variant)
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq)]
struct TypeTuple<'a, T>(T, &'a Ident);

#[cfg_attr(feature = "debug", derive(Debug))]
struct SchemaProperty<'a> {
    type_tree: &'a TypeTree<'a>,
    comments: Option<&'a CommentAttributes>,
    features: Option<&'a Vec<Feature>>,
    deprecated: Option<&'a Deprecated>,
}

impl<'a> SchemaProperty<'a> {
    fn new(
        type_tree: &'a TypeTree<'a>,
        comments: Option<&'a CommentAttributes>,
        features: Option<&'a Vec<Feature>>,
        deprecated: Option<&'a Deprecated>,
    ) -> Self {
        Self {
            type_tree,
            comments,
            features,
            deprecated,
        }
    }

    /// Check wheter property is required or not
    fn is_option(&self) -> bool {
        matches!(self.type_tree.generic_type, Some(GenericType::Option))
    }
}

impl ToTokens for SchemaProperty<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.type_tree.generic_type {
            Some(GenericType::Map) => {
                // Maps are treated as generic objects with no named properties and
                // additionalProperties denoting the type
                // maps have 2 child schemas and we are interested the second one of them
                // which is used to determine the additional properties
                let schema_property = SchemaProperty {
                    type_tree: self
                        .type_tree
                        .children
                        .as_ref()
                        .expect("SchemaProperty Map type should have children")
                        .iter()
                        .nth(1)
                        .expect("SchemaProperty Map type should have 2 child"),
                    comments: self.comments,
                    features: self.features,
                    deprecated: self.deprecated,
                };

                tokens.extend(quote! {
                    utoipa::openapi::ObjectBuilder::new().additional_properties(Some(#schema_property))
                });

                if let Some(description) = self.comments.and_then(|attributes| attributes.0.first())
                {
                    tokens.extend(quote! {
                        .description(Some(#description))
                    })
                }
            }
            Some(GenericType::Vec) => {
                let empty_features = Vec::new();
                let mut features = self.features.unwrap_or(&empty_features).clone();
                let example = features.pop_by(|feature| matches!(feature, Feature::Example(_)));
                let xml = features.extract_vec_xml_feature(self.type_tree);

                let schema_property = SchemaProperty {
                    type_tree: self
                        .type_tree
                        .children
                        .as_ref()
                        .expect("SchemaProperty Vec should have children")
                        .iter()
                        .next()
                        .expect("SchemaProperty Vec should have 1 child"),
                    comments: self.comments,
                    features: Some(&features),
                    deprecated: self.deprecated,
                };

                tokens.extend(quote! {
                    utoipa::openapi::schema::ArrayBuilder::new()
                        .items(#schema_property)
                });

                if let Some(ref example) = example {
                    tokens.extend(example.to_token_stream());
                }

                if let Some(vec_xml) = xml.as_ref() {
                    tokens.extend(vec_xml.to_token_stream());
                };
            }
            Some(GenericType::Option)
            | Some(GenericType::Cow)
            | Some(GenericType::Box)
            | Some(GenericType::RefCell) => {
                let schema_property = SchemaProperty {
                    type_tree: self
                        .type_tree
                        .children
                        .as_ref()
                        .expect("SchemaProperty generic container type should have children")
                        .iter()
                        .next()
                        .expect("SchemaProperty generic container type should have 1 child"),
                    comments: self.comments,
                    features: self.features,
                    deprecated: self.deprecated,
                };

                tokens.extend(schema_property.into_token_stream())
            }
            None => {
                let type_tree = self.type_tree;

                match type_tree.value_type {
                    ValueType::Primitive => {
                        let type_path = &**type_tree.path.as_ref().unwrap();
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

                        if let Some(description) =
                            self.comments.and_then(|attributes| attributes.0.first())
                        {
                            tokens.extend(quote! {
                                .description(Some(#description))
                            })
                        }

                        if let Some(deprecated) = self.deprecated {
                            tokens.extend(quote! { .deprecated(Some(#deprecated)) });
                        }

                        if let Some(attributes) = self.features {
                            tokens.extend(attributes.to_token_stream())
                        }
                    }
                    ValueType::Object => {
                        let is_inline = self
                            .features
                            .map(|features| features.is_inline())
                            .unwrap_or_default();

                        if type_tree.is_object() {
                            tokens.extend(quote! { utoipa::openapi::ObjectBuilder::new() })
                        } else {
                            let type_path = &**type_tree.path.as_ref().unwrap();
                            if is_inline {
                                tokens.extend(quote_spanned! {type_path.span() =>
                                    <#type_path as utoipa::ToSchema>::schema()
                                });
                            } else {
                                let name = format_path_ref(type_path);
                                tokens.extend(quote! {
                                    utoipa::openapi::Ref::from_schema_name(#name)
                                })
                            }
                        }
                    }
                    // TODO support for tuple types
                    ValueType::Tuple => (),
                }
            }
        }
    }
}

trait SchemaFeatureExt {
    fn split_for_title(self) -> (Vec<Feature>, Vec<Feature>);

    fn extract_vec_xml_feature(&mut self, type_tree: &TypeTree) -> Option<Feature>;
}

impl SchemaFeatureExt for Vec<Feature> {
    fn split_for_title(self) -> (Vec<Feature>, Vec<Feature>) {
        self.into_iter()
            .partition(|feature| matches!(feature, Feature::Title(_)))
    }

    fn extract_vec_xml_feature(&mut self, type_tree: &TypeTree) -> Option<Feature> {
        self.iter_mut().find_map(|feature| match feature {
            Feature::XmlAttr(xml_feature) => {
                let (vec_xml, value_xml) = xml_feature.split_for_vec(type_tree);

                // replace the original xml attribute with splitted value xml
                if let Some(mut xml) = value_xml {
                    mem::swap(xml_feature, &mut xml)
                }

                vec_xml.map(Feature::XmlAttr)
            }
            _ => None,
        })
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

#[inline]
fn is_not_skipped(rule: &Option<SerdeValue>) -> bool {
    rule.as_ref().map(|value| !value.skip).unwrap_or(true)
}

#[inline]
fn is_flatten(rule: &Option<SerdeValue>) -> bool {
    rule.as_ref().map(|value| value.flatten).unwrap_or(false)
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct AliasSchema {
    pub name: String,
    pub ty: Ident,
    pub generics: Generics,
}

impl Parse for AliasSchema {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;

        Ok(Self {
            name: name.to_string(),
            ty: input.parse::<Ident>()?,
            generics: input.parse()?,
        })
    }
}

fn parse_aliases(attributes: &[Attribute]) -> Option<Punctuated<AliasSchema, Comma>> {
    attributes
        .iter()
        .find(|attribute| attribute.path.is_ident("aliases"))
        .map(|aliases| {
            aliases
                .parse_args_with(Punctuated::<AliasSchema, Comma>::parse_terminated)
                .unwrap_or_abort()
        })
}
