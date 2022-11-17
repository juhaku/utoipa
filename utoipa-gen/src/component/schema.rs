use std::{borrow::Cow, mem};

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, Data, Field,
    Fields, FieldsNamed, FieldsUnnamed, Generics, Path, PathArguments, Token, TypePath, Variant,
    Visibility,
};

use crate::{
    component::features::{Example, Rename},
    doc_comment::CommentAttributes,
    schema_type::{SchemaFormat, SchemaType},
    Array, Deprecated,
};

use self::{
    enum_variant::{Enum, ObjectVariant, ReprVariant, SimpleEnumVariant, TaggedEnum},
    features::{
        EnumFeatures, FromAttributes, NamedFieldFeatures, NamedFieldStructFeatures,
        UnnamedFieldStructFeatures,
    },
};

use super::{
    features::{
        parse_features, pop_feature, Feature, FeaturesExt, IntoInner, IsInline, RenameAll,
        ToTokensExt,
    },
    serde::{self, SerdeContainer, SerdeValue},
    FieldRename, GenericType, TypeTree, ValueType, VariantRename,
};

mod enum_variant;
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
                    let unnamed_features = attributes
                        .parse_features::<UnnamedFieldStructFeatures>()
                        .into_inner();

                    Self::Unnamed(UnnamedStructSchema {
                        attributes,
                        features: unnamed_features,
                        fields: unnamed,
                    })
                }
                Fields::Named(fields) => {
                    let FieldsNamed { named, .. } = fields;
                    let mut named_features = attributes
                        .parse_features::<NamedFieldStructFeatures>()
                        .into_inner();
                    Self::Named(NamedStructSchema {
                        attributes,
                        rename_all: pop_feature!(named_features => Feature::RenameAll(_)).and_then(
                            |feature| match feature {
                                Feature::RenameAll(rename_all) => Some(rename_all),
                                _ => None,
                            },
                        ),
                        features: named_features,
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
    rename_all: Option<RenameAll>,
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

                with_field_as_schema_property(self, field, |schema_property, rename| {
                    let name = super::rename::<FieldRename>(
                        field_name,
                        field_rule
                            .as_ref()
                            .and_then(|field_rule| field_rule.rename.as_deref())
                            .or(rename.as_deref()),
                        container_rules
                            .as_ref()
                            .and_then(|container_rule| container_rule.rename_all.as_ref())
                            .or_else(|| {
                                self.rename_all
                                    .as_ref()
                                    .map(|rename_all| rename_all.as_rename_rule())
                            }),
                    )
                    .unwrap_or(Cow::Borrowed(field_name));

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
                with_field_as_schema_property(self, field, |schema_property, _| {
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
    yield_: impl FnOnce(SchemaProperty<'_>, Option<Cow<'_, str>>) -> R,
) -> R {
    let type_tree = &mut TypeTree::from_type(&field.ty);

    let mut field_features = field
        .attrs
        .parse_features::<NamedFieldFeatures>()
        .into_inner();

    let rename_field =
        pop_feature!(field_features => Feature::Rename(_)).and_then(|feature| match feature {
            Feature::Rename(rename) => Some(Cow::Owned(rename.into_value())),
            _ => None,
        });

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
        .and_then(|features| features.pop_value_type_feature());
    let override_type_tree = value_type
        .as_ref()
        .map(|value_type| value_type.as_type_tree());
    let comments = CommentAttributes::from_attributes(&field.attrs);

    yield_(
        SchemaProperty::new(
            override_type_tree.as_ref().unwrap_or(type_tree),
            Some(&comments),
            field_features.as_ref(),
            deprecated.as_ref(),
        ),
        rename_field,
    )
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
                .and_then(|features| features.pop_value_type_feature());
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
                            EnumSchemaType::Repr(ReprEnum {
                                variants: self.variants,
                                attributes: self.attributes,
                                enum_type,
                            })
                            .to_token_stream()
                        })
                        .unwrap_or_else(|| {
                            EnumSchemaType::Simple(SimpleEnum {
                                attributes: self.attributes,
                                variants: self.variants,
                            })
                            .to_token_stream()
                        }),
                )
            }

            #[cfg(not(feature = "repr"))]
            {
                tokens.extend(
                    EnumSchemaType::Simple(SimpleEnum {
                        attributes: self.attributes,
                        variants: self.variants,
                    })
                    .to_token_stream(),
                )
            }
        } else {
            tokens.extend(
                EnumSchemaType::Complex(ComplexEnum {
                    attributes: self.attributes,
                    variants: self.variants,
                })
                .to_token_stream(),
            )
        };
    }
}

enum EnumSchemaType<'e> {
    Simple(SimpleEnum<'e>),
    #[cfg(feature = "repr")]
    Repr(ReprEnum<'e>),
    Complex(ComplexEnum<'e>),
}

impl ToTokens for EnumSchemaType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attributes = match self {
            Self::Simple(simple) => {
                simple.to_tokens(tokens);
                simple.attributes
            }
            #[cfg(feature = "repr")]
            Self::Repr(repr) => {
                repr.to_tokens(tokens);
                repr.attributes
            }
            Self::Complex(complex) => {
                complex.to_tokens(tokens);
                complex.attributes
            }
        };

        if let Some(deprecated) = super::get_deprecated(attributes) {
            tokens.extend(quote! { .deprecated(Some(#deprecated)) });
        }

        if let Some(comment) = CommentAttributes::from_attributes(attributes).first() {
            tokens.extend(quote! {
                .description(Some(#comment))
            })
        }
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
impl ToTokens for ReprEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let container_rules = serde::parse_container(self.attributes);
        let repr_enum_features = features::parse_schema_features_with(self.attributes, |input| {
            Ok(parse_features!(input as Example, super::features::Default))
        })
        .unwrap_or_default();

        regular_enum_to_tokens(tokens, &container_rules, repr_enum_features, || {
            self.variants
                .iter()
                .filter_map(|variant| {
                    let variant_type = &variant.ident;
                    let variant_rules = serde::parse_value(&variant.attrs);

                    if is_not_skipped(&variant_rules) {
                        let repr_type = &self.enum_type;
                        Some(ReprVariant {
                            value: quote! { Self::#variant_type as #repr_type },
                            type_path: repr_type,
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<ReprVariant<TokenStream>>>()
        });
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct SimpleEnum<'a> {
    variants: &'a Punctuated<Variant, Comma>,
    attributes: &'a [Attribute],
}

impl ToTokens for SimpleEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let container_rules = serde::parse_container(self.attributes);
        let mut simple_enum_features = self
            .attributes
            .parse_features::<EnumFeatures>()
            .into_inner()
            .unwrap_or_default();
        let rename_all = simple_enum_features.pop_rename_all_feature();

        regular_enum_to_tokens(tokens, &container_rules, simple_enum_features, || {
            self.variants
                .iter()
                .filter_map(|variant| {
                    let name = &*variant.ident.to_string();
                    let variant_rules = serde::parse_value(&variant.attrs);
                    let mut variant_features =
                        features::parse_schema_features_with(&variant.attrs, |input| {
                            Ok(parse_features!(input as Rename))
                        })
                        .unwrap_or_default();
                    let rename = variant_features
                        .pop_rename_feature()
                        .map(|rename| rename.into_value());
                    let rename_to = variant_rules
                        .as_ref()
                        .and_then(|variant_rules| {
                            variant_rules.rename.as_deref().map(Cow::Borrowed)
                        })
                        .or_else(|| rename.map(Cow::Owned));

                    let variant_name = super::rename::<VariantRename>(
                        name,
                        rename_to.as_deref(),
                        container_rules.as_ref().and_then(|container_rule| {
                            container_rule.rename_all.as_ref().or_else(|| {
                                rename_all
                                    .as_ref()
                                    .map(|rename_all| rename_all.as_rename_rule())
                            })
                        }),
                    );

                    if is_not_skipped(&variant_rules) {
                        variant_name
                            .map(|name| SimpleEnumVariant {
                                value: quote! { #name },
                            })
                            .or_else(|| {
                                Some(SimpleEnumVariant {
                                    value: quote! { #name },
                                })
                            })
                    } else {
                        None
                    }
                })
                .collect::<Vec<SimpleEnumVariant<TokenStream>>>()
        });
    }
}

fn regular_enum_to_tokens<T: self::enum_variant::Variant>(
    tokens: &mut TokenStream,
    container_rules: &Option<SerdeContainer>,
    enum_variant_features: Vec<Feature>,
    get_variants_tokens_vec: impl FnOnce() -> Vec<T>,
) {
    let enum_values = get_variants_tokens_vec();

    tokens.extend(match container_rules {
        Some(serde_container) if !serde_container.tag.is_empty() => {
            let tag = &serde_container.tag;
            TaggedEnum {
                items: enum_values.as_slice(),
                tag: Cow::Borrowed(tag),
            }
            .to_token_stream()
        }
        _ => Enum {
            items: enum_values.as_slice(),
            title: None,
        }
        .to_token_stream(),
    });

    tokens.extend(enum_variant_features.to_token_stream());
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
                let (title_features, mut named_struct_features) = variant
                    .attrs
                    .parse_features::<NamedFieldStructFeatures>()
                    .into_inner()
                    .map(|features| features.split_for_title())
                    .unwrap_or_default();
                // TODO rename variant?????

                self::enum_variant::Variant::to_tokens(&ObjectVariant {
                    name: variant_name,
                    title: title_features.first().map(ToTokens::to_token_stream),
                    item: NamedStructSchema {
                        attributes: &variant.attrs,
                        rename_all: pop_feature!(named_struct_features => Feature::RenameAll(_))
                            .and_then(|feature| match feature {
                                Feature::RenameAll(rename_all) => Some(rename_all),
                                _ => None,
                            }),
                        features: Some(named_struct_features),
                        fields: &named_fields.named,
                        generics: None,
                        alias: None,
                    },
                })
            }
            Fields::Unnamed(unnamed_fields) => {
                let (title_features, unnamed_struct_features) = variant
                    .attrs
                    .parse_features::<UnnamedFieldStructFeatures>()
                    .into_inner()
                    .map(|features| features.split_for_title())
                    .unwrap_or_default();
                // TODO rename variant?????

                self::enum_variant::Variant::to_tokens(&ObjectVariant {
                    name: variant_name,
                    title: title_features.first().map(ToTokens::to_token_stream),
                    item: UnnamedStructSchema {
                        attributes: &variant.attrs,
                        features: Some(unnamed_struct_features),
                        fields: &unnamed_fields.unnamed,
                    },
                })
            }
            Fields::Unit => {
                let mut unit_features =
                    features::parse_schema_features_with(&variant.attrs, |input| {
                        Ok(parse_features!(
                            input as super::features::Title,
                            RenameAll,
                            Rename
                        ))
                    })
                    .unwrap_or_default();
                // TODO rename variant?????

                let title = pop_feature!(unit_features => Feature::Title(_));

                // Unit variant is just simple enum with single variant.
                Enum {
                    items: &[SimpleEnumVariant {
                        value: quote! {#variant_name},
                    }],
                    title: title.as_ref().map(ToTokens::to_token_stream),
                }
                .to_token_stream()
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
                let (title_features, mut named_struct_features) = variant
                    .attrs
                    .parse_features::<NamedFieldStructFeatures>()
                    .into_inner()
                    .map(|features| features.split_for_title())
                    .unwrap_or_default();
                // TODO rename variant????

                let named_enum = NamedStructSchema {
                    attributes: &variant.attrs,
                    rename_all: pop_feature!(named_struct_features => Feature::RenameAll(_))
                        .and_then(|feature| match feature {
                            Feature::RenameAll(rename_all) => Some(rename_all),
                            _ => None,
                        }),
                    features: Some(named_struct_features),
                    fields: &named_fields.named,
                    generics: None,
                    alias: None,
                };
                let title = title_features.first().map(ToTokens::to_token_stream);

                // let variant_name_tokens =
                //     UnitVariantTokens::to_tokens(&EnumVariantTokens(&variant_name, Vec::new()));

                let variant_name_tokens = Enum {
                    title: None,
                    items: &[SimpleEnumVariant {
                        value: quote! { #variant_name },
                    }],
                };
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
                    // TODO rename variant????

                    let unnamed_enum = UnnamedStructSchema {
                        attributes: &variant.attrs,
                        features: Some(unnamed_struct_features),
                        fields: &unnamed_fields.unnamed,
                    };

                    let title = title_features.first().map(ToTokens::to_token_stream);
                    // let variant_name_tokens =
                    //     UnitVariantTokens::to_tokens(&EnumVariantTokens(&variant_name, Vec::new()));
                    let variant_name_tokens = Enum {
                        title: None,
                        items: &[SimpleEnumVariant {
                            value: quote! { #variant_name },
                        }],
                    };

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
                // let variant_tokens =
                //     UnitVariantTokens::to_tokens(&EnumVariantTokens(&variant_name, Vec::new()));
                // TODO rename variant????
                // Unit variant is just simple enum with single variant.
                let variant_tokens = Enum {
                    title: None,
                    items: &[SimpleEnumVariant {
                        value: quote! { #variant_name },
                    }],
                };

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

impl ToTokens for ComplexEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attributes = &self.attributes;
        let container_rules = serde::parse_container(attributes);
        let mut enum_features = attributes
            .parse_features::<EnumFeatures>()
            .into_inner()
            .unwrap_or_default();
        let rename_all = enum_features.pop_rename_all_feature();

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
                        .and_then(|field_rule| field_rule.rename.as_deref()),
                    container_rules
                        .as_ref()
                        .and_then(|container_rule| container_rule.rename_all.as_ref())
                        .or_else(|| {
                            rename_all
                                .as_ref()
                                .map(|rename_all| rename_all.as_rename_rule())
                        }),
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

        tokens.extend(enum_features.to_token_stream());
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
