use std::borrow::Cow;

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, Data, Field,
    Fields, FieldsNamed, FieldsUnnamed, Generics, Path, PathArguments, Token, Variant, Visibility,
};

use crate::{
    component::features::Rename,
    doc_comment::CommentAttributes,
    schema_type::{SchemaFormat, SchemaType},
    Array, Deprecated,
};

use self::{
    enum_variant::{CustomEnum, Enum, ObjectVariant, SimpleEnumVariant, TaggedEnum},
    features::{
        ComplexEnumFeatures, EnumFeatures, EnumNamedFieldVariantFeatures,
        EnumUnnamedFieldVariantFeatures, FromAttributes, NamedFieldFeatures,
        NamedFieldStructFeatures, UnnamedFieldStructFeatures,
    },
};

use super::{
    features::{
        parse_features, pop_feature, Feature, FeaturesExt, IntoInner, IsInline, RenameAll,
        ToTokensExt, Validatable,
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
                .collect::<TokenStream>()
        });

        tokens.extend(quote! {
            impl #impl_generics utoipa::ToSchema for #ident #ty_generics #where_clause {
                fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
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
    Unit(UnitStructVariant),
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
                        struct_name: Cow::Owned(ident.to_string()),
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
                        struct_name: Cow::Owned(ident.to_string()),
                        attributes,
                        rename_all: named_features.pop_rename_all_feature(),
                        features: named_features,
                        fields: named,
                        generics: Some(generics),
                        alias,
                    })
                }
                Fields::Unit => Self::Unit(UnitStructVariant),
            },
            Data::Enum(content) => Self::Enum(EnumSchema {
                enum_name: Cow::Owned(ident.to_string()),
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
            Self::Unit(unit) => unit.to_tokens(tokens),
        }
    }
}

struct UnitStructVariant;

impl ToTokens for UnitStructVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! {
            utoipa::openapi::schema::ObjectBuilder::new()
                .nullable(true)
                .default(Some(serde_json::Value::Null))
                .example(Some(serde_json::Value::Null))
        });
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct NamedStructSchema<'a> {
    pub struct_name: Cow<'a, str>,
    pub fields: &'a Punctuated<Field, Comma>,
    pub attributes: &'a [Attribute],
    pub features: Option<Vec<Feature>>,
    pub rename_all: Option<RenameAll>,
    pub generics: Option<&'a Generics>,
    pub alias: Option<&'a AliasSchema>,
}

impl NamedStructSchema<'_> {
    fn field_as_schema_property<R>(
        &self,
        field: &Field,
        yield_: impl FnOnce(Property<'_>, Option<Cow<'_, str>>) -> R,
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

        if let Some((generic_types, alias)) = self.generics.zip(self.alias) {
            generic_types
                .type_params()
                .enumerate()
                .for_each(|(index, generic)| {
                    if let Some(generic_type) = type_tree.find_mut_by_ident(&generic.ident) {
                        generic_type.update(
                            alias
                                .generics
                                .type_params()
                                .nth(index)
                                .unwrap()
                                .ident
                                .clone(),
                        );
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
        let with_schema = pop_feature!(field_features => Feature::SchemaWith(_));

        yield_(
            if let Some(with_schema) = with_schema {
                Property::WithSchema(with_schema)
            } else {
                Property::Schema(SchemaProperty::new(
                    override_type_tree.as_ref().unwrap_or(type_tree),
                    Some(&comments),
                    field_features.as_ref(),
                    deprecated.as_ref(),
                    self.struct_name.as_ref(),
                ))
            },
            rename_field,
        )
    }
}

impl ToTokens for NamedStructSchema<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let container_rules = serde::parse_container(self.attributes);

        let object_tokens = self
            .fields
            .iter()
            .filter_map(|field| {
                let field_rule = serde::parse_value(&field.attrs);

                if is_not_skipped(&field_rule) && !is_flatten(&field_rule) {
                    Some((field, field_rule))
                } else {
                    None
                }
            })
            .fold(
                quote! { utoipa::openapi::ObjectBuilder::new() },
                |mut object_tokens, (field, field_rule)| {
                    let mut field_name = &*field.ident.as_ref().unwrap().to_string();

                    if field_name.starts_with("r#") {
                        field_name = &field_name[2..];
                    }

                    self.field_as_schema_property(field, |property, rename| {
                        let rename_to = field_rule
                            .as_ref()
                            .and_then(|field_rule| field_rule.rename.as_deref().map(Cow::Borrowed))
                            .or(rename);
                        let rename_all = container_rules
                            .as_ref()
                            .and_then(|container_rule| container_rule.rename_all.as_ref())
                            .or_else(|| {
                                self.rename_all
                                    .as_ref()
                                    .map(|rename_all| rename_all.as_rename_rule())
                            });

                        let name = super::rename::<FieldRename>(field_name, rename_to, rename_all)
                            .unwrap_or(Cow::Borrowed(field_name));

                        object_tokens.extend(quote! {
                            .property(#name, #property)
                        });

                        if let Property::Schema(schema_property) = property {
                            if !schema_property.is_option()
                                && !super::is_default(
                                    &container_rules.as_ref(),
                                    &field_rule.as_ref(),
                                )
                            {
                                object_tokens.extend(quote! {
                                    .required(#name)
                                })
                            }
                        }

                        object_tokens
                    })
                },
            );

        let flatten_fields: Vec<&Field> = self
            .fields
            .iter()
            .filter(|field| {
                let field_rule = serde::parse_value(&field.attrs);
                is_flatten(&field_rule)
            })
            .collect();

        if !flatten_fields.is_empty() {
            tokens.extend(quote! {
                utoipa::openapi::AllOfBuilder::new()
            });

            for field in flatten_fields {
                self.field_as_schema_property(field, |schema_property, _| {
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

        let description = CommentAttributes::from_attributes(self.attributes).as_formatted_string();
        if !description.is_empty() {
            tokens.extend(quote! {
                .description(Some(#description))
            })
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct UnnamedStructSchema<'a> {
    struct_name: Cow<'a, str>,
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
                    self.struct_name.as_ref(),
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

        let description = CommentAttributes::from_attributes(self.attributes).as_formatted_string();
        if !description.is_empty() && !is_object {
            tokens.extend(quote! {
                .description(Some(#description))
            })
        }

        if fields_len > 1 {
            tokens.extend(
                quote! { .to_array_builder().max_items(Some(#fields_len)).min_items(Some(#fields_len)) },
            )
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct EnumSchema<'a> {
   pub enum_name: Cow<'a, str>,
   pub variants: &'a Punctuated<Variant, Comma>,
   pub attributes: &'a [Attribute],
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
                                attribute.parse_args::<syn::TypePath>().ok()
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
                    enum_name: self.enum_name.as_ref(),
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

        let description = CommentAttributes::from_attributes(attributes).as_formatted_string();
        if !description.is_empty() {
            tokens.extend(quote! {
                .description(Some(#description))
            })
        }
    }
}

#[cfg(feature = "repr")]
#[cfg_attr(feature = "debug", derive(Debug))]
struct ReprEnum<'a> {
    variants: &'a Punctuated<Variant, Comma>,
    attributes: &'a [Attribute],
    enum_type: syn::TypePath,
}

#[cfg(feature = "repr")]
impl ToTokens for ReprEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let container_rules = serde::parse_container(self.attributes);
        let repr_enum_features = features::parse_schema_features_with(self.attributes, |input| {
            Ok(parse_features!(
                input as super::features::Example,
                super::features::Default
            ))
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
                        Some(enum_variant::ReprVariant {
                            value: quote! { Self::#variant_type as #repr_type },
                            type_path: repr_type,
                        })
                    } else {
                        None
                    }
                })
                .collect::<Vec<enum_variant::ReprVariant<TokenStream>>>()
        });
    }
}

fn rename_enum_variant<'a>(
    name: &'a str,
    features: &mut Vec<Feature>,
    variant_rules: &'a Option<SerdeValue>,
    container_rules: &'a Option<SerdeContainer>,
    rename_all: &'a Option<RenameAll>,
) -> Option<Cow<'a, str>> {
    let rename = features
        .pop_rename_feature()
        .map(|rename| rename.into_value());
    let rename_to = variant_rules
        .as_ref()
        .and_then(|variant_rules| variant_rules.rename.as_deref().map(Cow::Borrowed))
        .or_else(|| rename.map(Cow::Owned));

    let rename_all = container_rules
        .as_ref()
        .and_then(|container_rules| container_rules.rename_all.as_ref())
        .or_else(|| {
            rename_all
                .as_ref()
                .map(|rename_all| rename_all.as_rename_rule())
        });

    super::rename::<VariantRename>(name, rename_to, rename_all)
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
                    let variant_rules = serde::parse_value(&variant.attrs);

                    if is_not_skipped(&variant_rules) {
                        Some((variant, variant_rules))
                    } else {
                        None
                    }
                })
                .flat_map(|(variant, variant_rules)| {
                    let name = &*variant.ident.to_string();
                    let mut variant_features =
                        features::parse_schema_features_with(&variant.attrs, |input| {
                            Ok(parse_features!(input as Rename))
                        })
                        .unwrap_or_default();
                    let variant_name = rename_enum_variant(
                        name,
                        &mut variant_features,
                        &variant_rules,
                        &container_rules,
                        &rename_all,
                    );

                    variant_name
                        .map(|name| SimpleEnumVariant {
                            value: name.to_token_stream(),
                        })
                        .or_else(|| {
                            Some(SimpleEnumVariant {
                                value: name.to_token_stream(),
                            })
                        })
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
            TaggedEnum::new(
                enum_values
                    .into_iter()
                    .map(|variant| (Cow::Borrowed(&**tag), variant)),
            )
            .to_token_stream()
        }
        _ => Enum::new(enum_values).to_token_stream(),
    });

    tokens.extend(enum_variant_features.to_token_stream());
}

struct ComplexEnum<'a> {
    variants: &'a Punctuated<Variant, Comma>,
    attributes: &'a [Attribute],
    enum_name: &'a str,
}

impl ComplexEnum<'_> {
    /// Produce tokens that represent a variant of a [`ComplexEnum`].
    fn variant_tokens(
        &self,
        name: Cow<'_, str>,
        variant: &Variant,
        variant_rules: &Option<SerdeValue>,
        container_rules: &Option<SerdeContainer>,
        rename_all: &Option<RenameAll>,
    ) -> TokenStream {
        // TODO need to be able to split variant.attrs for variant and the struct representation!
        match &variant.fields {
            Fields::Named(named_fields) => {
                let (title_features, mut named_struct_features) = variant
                    .attrs
                    .parse_features::<EnumNamedFieldVariantFeatures>()
                    .into_inner()
                    .map(|features| features.split_for_title())
                    .unwrap_or_default();
                let variant_name = rename_enum_variant(
                    name.as_ref(),
                    &mut named_struct_features,
                    variant_rules,
                    container_rules,
                    rename_all,
                );

                self::enum_variant::Variant::to_tokens(&ObjectVariant {
                    name: variant_name.unwrap_or(Cow::Borrowed(&name)),
                    title: title_features.first().map(ToTokens::to_token_stream),
                    item: NamedStructSchema {
                        struct_name: Cow::Borrowed(self.enum_name),
                        attributes: &variant.attrs,
                        rename_all: named_struct_features.pop_rename_all_feature(),
                        features: Some(named_struct_features),
                        fields: &named_fields.named,
                        generics: None,
                        alias: None,
                    },
                })
            }
            Fields::Unnamed(unnamed_fields) => {
                let (title_features, mut unnamed_struct_features) = variant
                    .attrs
                    .parse_features::<EnumUnnamedFieldVariantFeatures>()
                    .into_inner()
                    .map(|features| features.split_for_title())
                    .unwrap_or_default();
                let variant_name = rename_enum_variant(
                    name.as_ref(),
                    &mut unnamed_struct_features,
                    variant_rules,
                    container_rules,
                    rename_all,
                );

                self::enum_variant::Variant::to_tokens(&ObjectVariant {
                    name: variant_name.unwrap_or(Cow::Borrowed(&name)),
                    title: title_features.first().map(ToTokens::to_token_stream),
                    item: UnnamedStructSchema {
                        struct_name: Cow::Borrowed(self.enum_name),
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
                let title = pop_feature!(unit_features => Feature::Title(_));
                let variant_name = rename_enum_variant(
                    name.as_ref(),
                    &mut unit_features,
                    variant_rules,
                    container_rules,
                    rename_all,
                );

                // Unit variant is just simple enum with single variant.
                Enum::new([SimpleEnumVariant {
                    value: variant_name
                        .unwrap_or(Cow::Borrowed(&name))
                        .to_token_stream(),
                }])
                .with_title(title.as_ref().map(ToTokens::to_token_stream))
                .to_token_stream()
            }
        }
    }

    /// Produce tokens that represent a variant of a [`ComplexEnum`] where serde enum attribute
    /// `tag = ` applies.
    fn tagged_variant_tokens(
        &self,
        tag: &str,
        name: Cow<'_, str>,
        variant: &Variant,
        variant_rules: &Option<SerdeValue>,
        container_rules: &Option<SerdeContainer>,
        rename_all: &Option<RenameAll>,
    ) -> TokenStream {
        match &variant.fields {
            Fields::Named(named_fields) => {
                let (title_features, mut named_struct_features) = variant
                    .attrs
                    .parse_features::<EnumNamedFieldVariantFeatures>()
                    .into_inner()
                    .map(|features| features.split_for_title())
                    .unwrap_or_default();
                let variant_name = rename_enum_variant(
                    name.as_ref(),
                    &mut named_struct_features,
                    variant_rules,
                    container_rules,
                    rename_all,
                );

                let named_enum = NamedStructSchema {
                    struct_name: Cow::Borrowed(self.enum_name),
                    attributes: &variant.attrs,
                    rename_all: named_struct_features.pop_rename_all_feature(),
                    features: Some(named_struct_features),
                    fields: &named_fields.named,
                    generics: None,
                    alias: None,
                };
                let title = title_features.first().map(ToTokens::to_token_stream);

                let variant_name_tokens = Enum::new([SimpleEnumVariant {
                    value: variant_name
                        .unwrap_or(Cow::Borrowed(&name))
                        .to_token_stream(),
                }]);
                quote! {
                    #named_enum
                        #title
                        .property(#tag, #variant_name_tokens)
                        .required(#tag)
                }
            }
            Fields::Unnamed(unnamed_fields) => {
                if unnamed_fields.unnamed.len() == 1 {
                    let (title_features, mut unnamed_struct_features) = variant
                        .attrs
                        .parse_features::<EnumUnnamedFieldVariantFeatures>()
                        .into_inner()
                        .map(|features| features.split_for_title())
                        .unwrap_or_default();
                    let variant_name = rename_enum_variant(
                        name.as_ref(),
                        &mut unnamed_struct_features,
                        variant_rules,
                        container_rules,
                        rename_all,
                    );

                    let unnamed_enum = UnnamedStructSchema {
                        struct_name: Cow::Borrowed(self.enum_name),
                        attributes: &variant.attrs,
                        features: Some(unnamed_struct_features),
                        fields: &unnamed_fields.unnamed,
                    };

                    let title = title_features.first().map(ToTokens::to_token_stream);
                    let variant_name_tokens = Enum::new([SimpleEnumVariant {
                        value: variant_name
                            .unwrap_or(Cow::Borrowed(&name))
                            .to_token_stream(),
                    }]);

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
                let mut unit_features =
                    features::parse_schema_features_with(&variant.attrs, |input| {
                        Ok(parse_features!(input as super::features::Title, Rename))
                    })
                    .unwrap_or_default();
                let title = pop_feature!(unit_features => Feature::Title(_));

                let variant_name = rename_enum_variant(
                    name.as_ref(),
                    &mut unit_features,
                    variant_rules,
                    container_rules,
                    rename_all,
                );

                // Unit variant is just simple enum with single variant.
                let variant_tokens = Enum::new([SimpleEnumVariant {
                    value: variant_name
                        .unwrap_or(Cow::Borrowed(&name))
                        .to_token_stream(),
                }]);

                quote! {
                    utoipa::openapi::schema::ObjectBuilder::new()
                        #title
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
            .parse_features::<ComplexEnumFeatures>()
            .into_inner()
            .unwrap_or_default();

        let rename_all = enum_features.pop_rename_all_feature();

        let tag = container_rules.as_ref().and_then(|rules| {
            if !rules.tag.is_empty() {
                Some(&rules.tag)
            } else {
                None
            }
        });

        // serde, externally tagged format supported by now
        self.variants
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

                if let Some(tag) = tag {
                    self.tagged_variant_tokens(
                        tag,
                        Cow::Borrowed(variant_name),
                        variant,
                        &variant_serde_rules,
                        &container_rules,
                        &rename_all,
                    )
                } else {
                    self.variant_tokens(
                        Cow::Borrowed(variant_name),
                        variant,
                        &variant_serde_rules,
                        &container_rules,
                        &rename_all,
                    )
                }
            })
            .collect::<CustomEnum<'_, TokenStream>>()
            .with_discriminator(tag.map(|tag| Cow::Borrowed(tag.as_str())))
            .to_tokens(tokens);

        tokens.extend(enum_features.to_token_stream());
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq)]
struct TypeTuple<'a, T>(T, &'a Ident);

#[cfg_attr(feature = "debug", derive(Debug))]
enum Property<'a> {
    Schema(SchemaProperty<'a>),
    WithSchema(Feature),
}

impl ToTokens for Property<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Schema(schema) => schema.to_tokens(tokens),
            Self::WithSchema(with_schema) => with_schema.to_tokens(tokens),
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct SchemaProperty<'a> {
    type_tree: &'a TypeTree<'a>,
    comments: Option<&'a CommentAttributes>,
    features: Option<&'a Vec<Feature>>,
    deprecated: Option<&'a Deprecated>,
    object_name: &'a str,
}

impl<'a> SchemaProperty<'a> {
    fn new(
        type_tree: &'a TypeTree<'a>,
        comments: Option<&'a CommentAttributes>,
        features: Option<&'a Vec<Feature>>,
        deprecated: Option<&'a Deprecated>,
        object_name: &'a str,
    ) -> Self {
        Self {
            type_tree,
            comments,
            features,
            deprecated,
            object_name,
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
                let empty_features = Vec::new();
                let mut features = self.features.unwrap_or(&empty_features).clone();
                let example = features.pop_by(|feature| matches!(feature, Feature::Example(_)));

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
                    features: Some(&features),
                    deprecated: self.deprecated,
                    object_name: self.object_name,
                };

                tokens.extend(quote! {
                    utoipa::openapi::ObjectBuilder::new().additional_properties(Some(#schema_property))
                });

                if let Some(ref example) = example {
                    tokens.extend(example.to_token_stream());
                }

                if let Some(description) = self.comments.map(CommentAttributes::as_formatted_string)
                {
                    if !description.is_empty() {
                        tokens.extend(quote! { .description(Some(#description))})
                    }
                }
            }
            Some(GenericType::Vec) => {
                let empty_features = Vec::new();
                let mut features = self.features.unwrap_or(&empty_features).clone();
                let example = pop_feature!(features => Feature::Example(_));
                let xml = features.extract_vec_xml_feature(self.type_tree);
                let max_items = pop_feature!(features => Feature::MaxItems(_));
                let min_items = pop_feature!(features => Feature::MinItems(_));

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
                    object_name: self.object_name,
                };

                let validate = |feature: &Feature| {
                    let type_path = &**self.type_tree.path.as_ref().unwrap();
                    let schema_type = SchemaType(type_path);
                    feature.validate(&schema_type, self.type_tree);
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

                if let Some(max_items) = max_items {
                    validate(&max_items);
                    tokens.extend(max_items.to_token_stream())
                }

                if let Some(min_items) = min_items {
                    validate(&min_items);
                    tokens.extend(min_items.to_token_stream())
                }
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
                    object_name: self.object_name,
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
                            self.comments.map(CommentAttributes::as_formatted_string)
                        {
                            if !description.is_empty() {
                                tokens.extend(quote! {.description(Some(#description))})
                            }
                        };

                        if let Some(deprecated) = self.deprecated {
                            tokens.extend(quote! { .deprecated(Some(#deprecated)) });
                        }

                        if let Some(features) = self.features {
                            for feature in
                                features.iter().filter(|feature| feature.is_validatable())
                            {
                                feature.validate(&schema_type, type_tree);
                            }
                            tokens.extend(features.to_token_stream())
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
                                let mut name = Cow::Owned(format_path_ref(type_path));
                                if name == "Self" {
                                    name = Cow::Borrowed(self.object_name);
                                }
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
}

impl SchemaFeatureExt for Vec<Feature> {
    fn split_for_title(self) -> (Vec<Feature>, Vec<Feature>) {
        self.into_iter()
            .partition(|feature| matches!(feature, Feature::Title(_)))
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
