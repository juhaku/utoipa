use std::{borrow::Cow, ops::Deref};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, spanned::Spanned, token::Comma, Fields, TypePath, Variant};

use crate::{
    component::{
        features::{
            attributes::{
                Deprecated, Description, Discriminator, Example, Examples, NoRecursion, Rename,
                RenameAll, Title,
            },
            parse_features, pop_feature, Feature, IntoInner, IsInline, ToTokensExt,
        },
        schema::features::{
            EnumNamedFieldVariantFeatures, EnumUnnamedFieldVariantFeatures, FromAttributes,
        },
        serde::{SerdeContainer, SerdeEnumRepr, SerdeValue},
        FeaturesExt, SchemaReference, TypeTree, ValueType,
    },
    doc_comment::CommentAttributes,
    schema_type::SchemaType,
    Array, AttributesExt, Diagnostics, ToTokensDiagnostics,
};

use super::{features, serde, NamedStructSchema, Root, UnnamedStructSchema};

#[cfg_attr(feature = "debug", derive(Debug))]
enum PlainEnumRepr<'p> {
    Plain(Array<'p, TokenStream>),
    Repr(Array<'p, TokenStream>, syn::TypePath),
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct PlainEnum<'e> {
    pub root: &'e Root<'e>,
    enum_variant: PlainEnumRepr<'e>,
    serde_enum_repr: SerdeEnumRepr,
    features: Vec<Feature>,
    pub description: Option<Description>,
}

impl<'e> PlainEnum<'e> {
    pub fn new(
        root: &'e Root,
        variants: &Punctuated<Variant, Comma>,
        mut features: Vec<Feature>,
    ) -> Result<Self, Diagnostics> {
        #[cfg(feature = "repr")]
        let repr_type_path = PlainEnum::get_repr_type(root.attributes)?;

        #[cfg(not(feature = "repr"))]
        let repr_type_path = None;

        let rename_all = pop_feature!(features => Feature::RenameAll(_) as Option<RenameAll>);
        let description = pop_feature!(features => Feature::Description(_) as Option<Description>);

        let container_rules = serde::parse_container(root.attributes)?;
        let variants_iter = variants
            .iter()
            .map(|variant| match serde::parse_value(&variant.attrs) {
                Ok(variant_rules) => Ok((variant, variant_rules)),
                Err(diagnostics) => Err(diagnostics),
            })
            .collect::<Result<Vec<_>, Diagnostics>>()?
            .into_iter()
            .filter_map(|(variant, variant_rules)| {
                if variant_rules.skip {
                    None
                } else {
                    Some((variant, variant_rules))
                }
            });

        let enum_variant = match repr_type_path {
            Some(repr_type_path) => PlainEnumRepr::Repr(
                variants_iter
                    .map(|(variant, _)| {
                        let ty = &variant.ident;
                        quote! {
                            Self::#ty as #repr_type_path
                        }
                    })
                    .collect::<Array<TokenStream>>(),
                repr_type_path,
            ),
            None => PlainEnumRepr::Plain(
                variants_iter
                    .map(|(variant, variant_rules)| {
                        let parsed_features_result =
                            features::parse_schema_features_with(&variant.attrs, |input| {
                                Ok(parse_features!(input as Rename))
                            });

                        match parsed_features_result {
                            Ok(variant_features) => {
                                Ok((variant, variant_rules, variant_features.unwrap_or_default()))
                            }
                            Err(diagnostics) => Err(diagnostics),
                        }
                    })
                    .collect::<Result<Vec<_>, Diagnostics>>()?
                    .into_iter()
                    .map(|(variant, variant_rules, mut variant_features)| {
                        let name = &*variant.ident.to_string();
                        let renamed = super::rename_enum_variant(
                            name,
                            &mut variant_features,
                            &variant_rules,
                            &container_rules,
                            rename_all.as_ref(),
                        );

                        renamed.unwrap_or(Cow::Borrowed(name)).to_token_stream()
                    })
                    .collect::<Array<TokenStream>>(),
            ),
        };

        Ok(Self {
            root,
            enum_variant,
            features,
            serde_enum_repr: container_rules.enum_repr,
            description,
        })
    }

    #[cfg(feature = "repr")]
    fn get_repr_type(attributes: &[syn::Attribute]) -> Result<Option<syn::TypePath>, syn::Error> {
        attributes
            .iter()
            .find_map(|attr| {
                if attr.path().is_ident("repr") {
                    Some(attr.parse_args::<syn::TypePath>())
                } else {
                    None
                }
            })
            .transpose()
    }
}

impl ToTokens for PlainEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let (variants, schema_type, enum_type) = match &self.enum_variant {
            PlainEnumRepr::Plain(items) => (
                Roo::Ref(items),
                Roo::Owned(SchemaType {
                    nullable: false,
                    path: Cow::Owned(syn::parse_quote!(str)),
                }),
                Roo::Owned(quote! { &str }),
            ),
            PlainEnumRepr::Repr(repr, repr_type) => (
                Roo::Ref(repr),
                Roo::Owned(SchemaType {
                    nullable: false,
                    path: Cow::Borrowed(&repr_type.path),
                }),
                Roo::Owned(repr_type.path.to_token_stream()),
            ),
        };

        match &self.serde_enum_repr {
            SerdeEnumRepr::ExternallyTagged => {
                EnumSchema::<PlainSchema>::with_types(variants, schema_type, enum_type)
                    .to_tokens(tokens);
            }
            SerdeEnumRepr::InternallyTagged { tag } => {
                let items = variants
                    .iter()
                    .map(|item| Array::Owned(vec![item]))
                    .collect::<Array<_>>();
                let schema_type = schema_type.as_ref();
                let enum_type = enum_type.as_ref();

                OneOf {
                    items: &items
                        .iter()
                        .map(|item| {
                            EnumSchema::<PlainSchema>::with_types(
                                Roo::Ref(item),
                                Roo::Ref(schema_type),
                                Roo::Ref(enum_type),
                            )
                            .tagged(tag)
                        })
                        .collect(),
                    discriminator: None,
                }
                .to_tokens(tokens)
            }
            SerdeEnumRepr::Untagged => {
                // Even though untagged enum might have multiple variants, but unit type variants
                // all will result `null` empty schema thus returning one empty schema is
                // sufficient instead of returning one of N * `null` schema.
                EnumSchema::<TokenStream>::untagged().to_tokens(tokens);
            }
            SerdeEnumRepr::AdjacentlyTagged { tag, content } => {
                let items = variants
                    .iter()
                    .map(|item| Array::Owned(vec![item]))
                    .collect::<Array<_>>();
                let schema_type = schema_type.as_ref();
                let enum_type = enum_type.as_ref();

                OneOf {
                    items: &items
                        .iter()
                        .map(|item| {
                            EnumSchema::<ObjectSchema>::adjacently_tagged(
                                PlainSchema::new(
                                    item.deref(),
                                    Roo::Ref(schema_type),
                                    Roo::Ref(enum_type),
                                ),
                                content,
                            )
                            .tag(tag, PlainSchema::for_name(content))
                        })
                        .collect(),
                    discriminator: None,
                }
                .to_tokens(tokens)
            }
            // This should not be possible as serde should not let that happen
            SerdeEnumRepr::UnfinishedAdjacentlyTagged { .. } => {
                unreachable!("Invalid serde enum repr, serde should have panicked and not reach here, plain enum")
            }
        };

        tokens.extend(self.features.to_token_stream());
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct MixedEnum<'p> {
    pub root: &'p Root<'p>,
    pub tokens: TokenStream,
    pub description: Option<Description>,
    pub schema_references: Vec<SchemaReference>,
}

impl<'p> MixedEnum<'p> {
    pub fn new(
        root: &'p Root,
        variants: &Punctuated<Variant, Comma>,
        mut features: Vec<Feature>,
    ) -> Result<Self, Diagnostics> {
        let attributes = root.attributes;
        let container_rules = serde::parse_container(attributes)?;

        let rename_all = pop_feature!(features => Feature::RenameAll(_) as Option<RenameAll>);
        let description = pop_feature!(features => Feature::Description(_) as Option<Description>);
        let discriminator = pop_feature!(features => Feature::Discriminator(_));

        let variants = variants
            .iter()
            .map(|variant| match serde::parse_value(&variant.attrs) {
                Ok(variant_rules) => Ok((variant, variant_rules)),
                Err(diagnostics) => Err(diagnostics),
            })
            .collect::<Result<Vec<_>, Diagnostics>>()?
            .into_iter()
            .filter_map(|(variant, variant_rules)| {
                if variant_rules.skip {
                    None
                } else {
                    let variant_features = match &variant.fields {
                        Fields::Named(_) => {
                            match variant
                                .attrs
                                .parse_features::<EnumNamedFieldVariantFeatures>()
                            {
                                Ok(features) => features.into_inner().unwrap_or_default(),
                                Err(diagnostics) => return Some(Err(diagnostics)),
                            }
                        }
                        Fields::Unnamed(_) => {
                            match variant
                                .attrs
                                .parse_features::<EnumUnnamedFieldVariantFeatures>()
                            {
                                Ok(features) => features.into_inner().unwrap_or_default(),
                                Err(diagnostics) => return Some(Err(diagnostics)),
                            }
                        }
                        Fields::Unit => {
                            let parse_unit_features =
                                features::parse_schema_features_with(&variant.attrs, |input| {
                                    Ok(parse_features!(
                                        input as Title,
                                        Rename,
                                        Example,
                                        Examples,
                                        Deprecated
                                    ))
                                });

                            match parse_unit_features {
                                Ok(features) => features.unwrap_or_default(),
                                Err(diagnostics) => return Some(Err(diagnostics)),
                            }
                        }
                    };

                    Some(Ok((variant, variant_rules, variant_features)))
                }
            })
            .collect::<Result<Vec<_>, Diagnostics>>()?;

        // discriminator is only supported when all variants are unnamed with single non primitive
        // field
        let discriminator_supported = variants
            .iter()
            .all(|(variant, _, features)|
                matches!(&variant.fields, Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1
                && TypeTree::from_type(&unnamed.unnamed.first().unwrap().ty).expect("unnamed field should be valid TypeTree").value_type == ValueType::Object
                && !features.is_inline())
            )
            && matches!(container_rules.enum_repr, SerdeEnumRepr::Untagged);

        if discriminator.is_some() && !discriminator_supported {
            let discriminator: Discriminator =
                IntoInner::<Option<Discriminator>>::into_inner(discriminator).unwrap();
            return Err(Diagnostics::with_span(
                discriminator.get_attribute().span(),
                "Found discriminator in not discriminator supported context",
            ).help("`discriminator` is only supported on enums with `#[serde(untagged)]` having unnamed field variants with single reference field.")
            .note("Unnamed field variants with inlined or primitive schemas does not support discriminator.")
            .note("Read more about discriminators from the specs <https://spec.openapis.org/oas/latest.html#discriminator-object>"));
        }

        let mut items = variants
            .into_iter()
            .map(|(variant, variant_serde_rules, mut variant_features)| {
                if features
                    .iter()
                    .any(|feature| matches!(feature, Feature::NoRecursion(_)))
                {
                    variant_features.push(Feature::NoRecursion(NoRecursion));
                }
                MixedEnumContent::new(
                    variant,
                    root,
                    &container_rules,
                    rename_all.as_ref(),
                    variant_serde_rules,
                    variant_features,
                )
            })
            .collect::<Result<Vec<MixedEnumContent>, Diagnostics>>()?;

        let schema_references = items
            .iter_mut()
            .flat_map(|item| std::mem::take(&mut item.schema_references))
            .collect::<Vec<_>>();

        let one_of_enum = OneOf {
            items: &Array::Owned(items),
            discriminator,
        };

        let _ = pop_feature!(features => Feature::NoRecursion(_));
        let mut tokens = one_of_enum.to_token_stream();
        tokens.extend(features.to_token_stream());

        Ok(Self {
            root,
            tokens,
            description,
            schema_references,
        })
    }
}

impl ToTokens for MixedEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.tokens.to_tokens(tokens);
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct MixedEnumContent {
    tokens: TokenStream,
    schema_references: Vec<SchemaReference>,
}

impl MixedEnumContent {
    fn new(
        variant: &Variant,
        root: &Root,
        serde_container: &SerdeContainer,
        rename_all: Option<&RenameAll>,
        variant_serde_rules: SerdeValue,
        mut variant_features: Vec<Feature>,
    ) -> Result<Self, Diagnostics> {
        let mut tokens = TokenStream::new();
        let name = variant.ident.to_string();
        // TODO support `description = ...` attribute via Feature::Description
        // let description =
        //     pop_feature!(variant_features => Feature::Description(_) as Option<Description>);
        let variant_description =
            CommentAttributes::from_attributes(&variant.attrs).as_formatted_string();
        let description: Option<Description> =
            (!variant_description.is_empty()).then(|| variant_description.into());
        if let Some(description) = description {
            variant_features.push(Feature::Description(description))
        }

        if variant.attrs.has_deprecated() {
            variant_features.push(Feature::Deprecated(true.into()))
        }

        let mut schema_references: Vec<SchemaReference> = Vec::new();
        match &variant.fields {
            Fields::Named(named) => {
                let (variant_tokens, references) =
                    MixedEnumContent::get_named_tokens_with_schema_references(
                        root,
                        MixedEnumVariant {
                            variant,
                            fields: &named.named,
                            name,
                        },
                        variant_features,
                        serde_container,
                        variant_serde_rules,
                        rename_all,
                    )?;
                schema_references.extend(references);
                variant_tokens.to_tokens(&mut tokens);
            }
            Fields::Unnamed(unnamed) => {
                let (variant_tokens, references) =
                    MixedEnumContent::get_unnamed_tokens_with_schema_reference(
                        root,
                        MixedEnumVariant {
                            variant,
                            fields: &unnamed.unnamed,
                            name,
                        },
                        variant_features,
                        serde_container,
                        variant_serde_rules,
                        rename_all,
                    )?;

                schema_references.extend(references);
                variant_tokens.to_tokens(&mut tokens);
            }
            Fields::Unit => {
                let variant_tokens = MixedEnumContent::get_unit_tokens(
                    name,
                    variant_features,
                    serde_container,
                    variant_serde_rules,
                    rename_all,
                );
                variant_tokens.to_tokens(&mut tokens);
            }
        }

        Ok(Self {
            tokens,
            schema_references,
        })
    }

    fn get_named_tokens_with_schema_references(
        root: &Root,
        variant: MixedEnumVariant,
        mut variant_features: Vec<Feature>,
        serde_container: &SerdeContainer,
        variant_serde_rules: SerdeValue,
        rename_all: Option<&RenameAll>,
    ) -> Result<(TokenStream, Vec<SchemaReference>), Diagnostics> {
        let MixedEnumVariant {
            variant,
            fields,
            name,
        } = variant;

        let renamed = super::rename_enum_variant(
            &name,
            &mut variant_features,
            &variant_serde_rules,
            serde_container,
            rename_all,
        );
        let name = renamed.unwrap_or(Cow::Owned(name));

        let root = &Root {
            ident: &variant.ident,
            attributes: &variant.attrs,
            generics: root.generics,
        };

        let tokens_with_schema_references = match &serde_container.enum_repr {
            SerdeEnumRepr::ExternallyTagged => {
                let (enum_features, variant_features) =
                    MixedEnumContent::split_enum_features(variant_features);
                let schema = NamedStructSchema::new(root, fields, variant_features)?;
                let schema_tokens = schema.to_token_stream();

                (
                    EnumSchema::<ObjectSchema>::new(name.as_ref(), schema_tokens)
                        .features(enum_features)
                        .to_token_stream(),
                    schema.fields_references,
                )
            }
            SerdeEnumRepr::InternallyTagged { tag } => {
                let (enum_features, variant_features) =
                    MixedEnumContent::split_enum_features(variant_features);
                let schema = NamedStructSchema::new(root, fields, variant_features)?;

                let mut schema_tokens = schema.to_token_stream();
                (
                    if schema.is_all_of {
                        let object_builder_tokens =
                            quote! { utoipa::openapi::schema::Object::builder() };
                        let enum_schema_tokens =
                            EnumSchema::<ObjectSchema>::tagged(object_builder_tokens)
                                .tag(tag, PlainSchema::for_name(name.as_ref()))
                                .features(enum_features)
                                .to_token_stream();
                        schema_tokens.extend(quote! {
                            .item(#enum_schema_tokens)
                        });
                        schema_tokens
                    } else {
                        EnumSchema::<ObjectSchema>::tagged(schema_tokens)
                            .tag(tag, PlainSchema::for_name(name.as_ref()))
                            .features(enum_features)
                            .to_token_stream()
                    },
                    schema.fields_references,
                )
            }
            SerdeEnumRepr::Untagged => {
                let schema = NamedStructSchema::new(root, fields, variant_features)?;
                (schema.to_token_stream(), schema.fields_references)
            }
            SerdeEnumRepr::AdjacentlyTagged { tag, content } => {
                let (enum_features, variant_features) =
                    MixedEnumContent::split_enum_features(variant_features);
                let schema = NamedStructSchema::new(root, fields, variant_features)?;

                let schema_tokens = schema.to_token_stream();
                (
                    EnumSchema::<ObjectSchema>::adjacently_tagged(schema_tokens, content)
                        .tag(tag, PlainSchema::for_name(name.as_ref()))
                        .features(enum_features)
                        .to_token_stream(),
                    schema.fields_references,
                )
            }
            SerdeEnumRepr::UnfinishedAdjacentlyTagged { .. } => unreachable!(
                "Invalid serde enum repr, serde should have panicked before reaching here"
            ),
        };

        Ok(tokens_with_schema_references)
    }

    fn get_unnamed_tokens_with_schema_reference(
        root: &Root,
        variant: MixedEnumVariant,
        mut variant_features: Vec<Feature>,
        serde_container: &SerdeContainer,
        variant_serde_rules: SerdeValue,
        rename_all: Option<&RenameAll>,
    ) -> Result<(TokenStream, Vec<SchemaReference>), Diagnostics> {
        let MixedEnumVariant {
            variant,
            fields,
            name,
        } = variant;

        let renamed = super::rename_enum_variant(
            &name,
            &mut variant_features,
            &variant_serde_rules,
            serde_container,
            rename_all,
        );
        let name = renamed.unwrap_or(Cow::Owned(name));

        let root = &Root {
            ident: &variant.ident,
            attributes: &variant.attrs,
            generics: root.generics,
        };

        let tokens_with_schema_reference = match &serde_container.enum_repr {
            SerdeEnumRepr::ExternallyTagged => {
                let (enum_features, variant_features) =
                    MixedEnumContent::split_enum_features(variant_features);
                let schema = UnnamedStructSchema::new(root, fields, variant_features)?;

                let schema_tokens = schema.to_token_stream();
                (
                    EnumSchema::<ObjectSchema>::new(name.as_ref(), schema_tokens)
                        .features(enum_features)
                        .to_token_stream(),
                    schema.schema_references,
                )
            }
            SerdeEnumRepr::InternallyTagged { tag } => {
                let (enum_features, variant_features) =
                    MixedEnumContent::split_enum_features(variant_features);
                let schema = UnnamedStructSchema::new(root, fields, variant_features)?;

                let schema_tokens = schema.to_token_stream();

                let is_reference = fields
                    .iter()
                    .map(|field| TypeTree::from_type(&field.ty))
                    .collect::<Result<Vec<TypeTree>, Diagnostics>>()?
                    .iter()
                    .any(|type_tree| type_tree.value_type == ValueType::Object);

                (
                    EnumSchema::<InternallyTaggedUnnamedSchema>::new(schema_tokens, is_reference)
                        .tag(tag, PlainSchema::for_name(name.as_ref()))
                        .features(enum_features)
                        .to_token_stream(),
                    schema.schema_references,
                )
            }
            SerdeEnumRepr::Untagged => {
                let schema = UnnamedStructSchema::new(root, fields, variant_features)?;
                (schema.to_token_stream(), schema.schema_references)
            }
            SerdeEnumRepr::AdjacentlyTagged { tag, content } => {
                if fields.len() > 1 {
                    return Err(Diagnostics::with_span(variant.span(),
                        "Unnamed (tuple) enum variants are unsupported for internally tagged enums using the `tag = ` serde attribute")
                        .help("Try using a different serde enum representation")
                        .note("See more about enum limitations here: `https://serde.rs/enum-representations.html#internally-tagged`")
                    );
                }

                let (enum_features, variant_features) =
                    MixedEnumContent::split_enum_features(variant_features);
                let schema = UnnamedStructSchema::new(root, fields, variant_features)?;

                let schema_tokens = schema.to_token_stream();
                (
                    EnumSchema::<ObjectSchema>::adjacently_tagged(schema_tokens, content)
                        .tag(tag, PlainSchema::for_name(name.as_ref()))
                        .features(enum_features)
                        .to_token_stream(),
                    schema.schema_references,
                )
            }
            SerdeEnumRepr::UnfinishedAdjacentlyTagged { .. } => unreachable!(
                "Invalid serde enum repr, serde should have panicked before reaching here"
            ),
        };

        Ok(tokens_with_schema_reference)
    }

    fn get_unit_tokens(
        name: String,
        mut variant_features: Vec<Feature>,
        serde_container: &SerdeContainer,
        variant_serde_rules: SerdeValue,
        rename_all: Option<&RenameAll>,
    ) -> TokenStream {
        let renamed = super::rename_enum_variant(
            &name,
            &mut variant_features,
            &variant_serde_rules,
            serde_container,
            rename_all,
        );
        let name = renamed.unwrap_or(Cow::Owned(name));

        match &serde_container.enum_repr {
            SerdeEnumRepr::ExternallyTagged => EnumSchema::<PlainSchema>::new(name.as_ref())
                .features(variant_features)
                .to_token_stream(),
            SerdeEnumRepr::InternallyTagged { tag } => {
                EnumSchema::<PlainSchema>::new(name.as_ref())
                    .tagged(tag)
                    .features(variant_features)
                    .to_token_stream()
            }
            SerdeEnumRepr::Untagged => {
                let v: EnumSchema = EnumSchema::untagged().features(variant_features);
                v.to_token_stream()
            }
            SerdeEnumRepr::AdjacentlyTagged { tag, .. } => {
                EnumSchema::<PlainSchema>::new(name.as_ref())
                    .tagged(tag)
                    .features(variant_features)
                    .to_token_stream()
            }
            SerdeEnumRepr::UnfinishedAdjacentlyTagged { .. } => unreachable!(
                "Invalid serde enum repr, serde should have panicked before reaching here"
            ),
        }
    }

    fn split_enum_features(variant_features: Vec<Feature>) -> (Vec<Feature>, Vec<Feature>) {
        let (enum_features, variant_features): (Vec<_>, Vec<_>) =
            variant_features.into_iter().partition(|feature| {
                matches!(
                    feature,
                    Feature::Title(_)
                        | Feature::Example(_)
                        | Feature::Examples(_)
                        | Feature::Default(_)
                        | Feature::Description(_)
                        | Feature::Deprecated(_)
                )
            });

        (enum_features, variant_features)
    }
}

impl ToTokens for MixedEnumContent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.tokens.to_tokens(tokens);
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct MixedEnumVariant<'v> {
    variant: &'v syn::Variant,
    fields: &'v Punctuated<syn::Field, Comma>,
    name: String,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct EnumSchema<T = TokenStream> {
    features: Vec<Feature>,
    untagged: bool,
    content: Option<T>,
}

impl<T> EnumSchema<T> {
    fn untagged() -> EnumSchema<T> {
        Self {
            untagged: true,
            features: Vec::new(),
            content: None,
        }
    }

    fn features(mut self, features: Vec<Feature>) -> Self {
        self.features = features;

        self
    }
}

impl<T> ToTokens for EnumSchema<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(content) = &self.content {
            tokens.extend(content.to_token_stream());
        }

        if self.untagged {
            tokens.extend(quote! {
                utoipa::openapi::schema::Object::builder()
                    .schema_type(utoipa::openapi::schema::Type::Null)
                    .default(Some(utoipa::gen::serde_json::Value::Null))
            })
        }

        tokens.extend(self.features.to_token_stream());
    }
}

impl<'a> EnumSchema<ObjectSchema> {
    fn new<T: ToTokens>(name: &'a str, item: T) -> Self {
        let content = quote! {
            utoipa::openapi::schema::Object::builder()
                .property(#name, #item)
                .required(#name)
        };

        Self {
            content: Some(ObjectSchema(content)),
            features: Vec::new(),
            untagged: false,
        }
    }

    fn tagged<T: ToTokens>(item: T) -> Self {
        let content = item.to_token_stream();

        Self {
            content: Some(ObjectSchema(content)),
            features: Vec::new(),
            untagged: false,
        }
    }

    fn tag(mut self, tag: &'a str, tag_schema: PlainSchema) -> Self {
        let content = self.content.get_or_insert(ObjectSchema::default());

        content.0.extend(quote! {
            .property(#tag, utoipa::openapi::schema::Object::builder() #tag_schema)
            .required(#tag)
        });

        self
    }

    fn adjacently_tagged<T: ToTokens>(item: T, content: &str) -> Self {
        let content = quote! {
            utoipa::openapi::schema::Object::builder()
                .property(#content, #item)
                .required(#content)
        };

        Self {
            content: Some(ObjectSchema(content)),
            features: Vec::new(),
            untagged: false,
        }
    }
}

impl EnumSchema<InternallyTaggedUnnamedSchema> {
    fn new<T: ToTokens>(item: T, is_reference: bool) -> Self {
        let schema = item.to_token_stream();

        let tokens = if is_reference {
            quote! {
                utoipa::openapi::schema::AllOfBuilder::new()
                    .item(#schema)
            }
        } else {
            quote! {
                #schema
                .schema_type(utoipa::openapi::schema::Type::Object)
            }
        };

        Self {
            content: Some(InternallyTaggedUnnamedSchema(tokens, is_reference)),
            untagged: false,
            features: Vec::new(),
        }
    }

    fn tag(mut self, tag: &str, tag_schema: PlainSchema) -> Self {
        let content = self
            .content
            .get_or_insert(InternallyTaggedUnnamedSchema::default());
        let is_reference = content.1;

        if is_reference {
            content.0.extend(quote! {
                .item(
                    utoipa::openapi::schema::Object::builder()
                        .property(#tag, utoipa::openapi::schema::Object::builder() #tag_schema)
                        .required(#tag)
                )
            });
        } else {
            content.0.extend(quote! {
                .property(#tag, utoipa::openapi::schema::Object::builder() #tag_schema)
                .required(#tag)
            });
        }

        self
    }
}

impl<'a> EnumSchema<PlainSchema> {
    fn new<N: ToTokens>(name: N) -> Self {
        let plain_schema = PlainSchema::for_name(name);

        Self {
            content: Some(PlainSchema(quote! {
                utoipa::openapi::schema::Object::builder() #plain_schema
            })),
            untagged: false,
            features: Vec::new(),
        }
    }

    fn with_types<T: ToTokens>(
        items: Roo<'a, Array<'a, T>>,
        schema_type: Roo<'a, SchemaType<'a>>,
        enum_type: Roo<'a, TokenStream>,
    ) -> Self {
        let plain_schema = PlainSchema::new(&items, schema_type, enum_type);

        Self {
            content: Some(PlainSchema(quote! {
                utoipa::openapi::schema::Object::builder() #plain_schema
            })),
            untagged: false,
            features: Vec::new(),
        }
    }

    fn tagged(mut self, tag: &str) -> Self {
        if let Some(content) = self.content {
            let plain_schema = content.0;
            self.content = Some(PlainSchema(
                quote! {
                    utoipa::openapi::schema::Object::builder()
                        .property(#tag, #plain_schema )
                        .required(#tag)
                }
                .to_token_stream(),
            ));
        }

        self
    }
}

#[derive(Default)]
struct ObjectSchema(TokenStream);

impl ToTokens for ObjectSchema {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

#[derive(Default)]
struct InternallyTaggedUnnamedSchema(TokenStream, bool);

impl ToTokens for InternallyTaggedUnnamedSchema {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

#[derive(Default)]
struct PlainSchema(TokenStream);

impl PlainSchema {
    fn get_default_types() -> (Roo<'static, SchemaType<'static>>, Roo<'static, TokenStream>) {
        let type_path: TypePath = syn::parse_quote!(str);
        let schema_type = SchemaType {
            path: Cow::Owned(type_path.path),
            nullable: false,
        };
        let enum_type = quote! { &str };

        (Roo::Owned(schema_type), Roo::Owned(enum_type))
    }

    fn new<'a, T: ToTokens>(
        items: &[T],
        schema_type: Roo<'a, SchemaType<'a>>,
        enum_type: Roo<'a, TokenStream>,
    ) -> Self {
        let schema_type = schema_type.to_token_stream();
        let enum_type = enum_type.as_ref();
        let items = Array::Borrowed(items);
        let len = items.len();

        let plain_enum = quote! {
                .schema_type(#schema_type)
                .enum_values::<[#enum_type; #len], #enum_type>(Some(#items))
        };

        Self(plain_enum.to_token_stream())
    }

    fn for_name<N: ToTokens>(name: N) -> Self {
        let (schema_type, enum_type) = Self::get_default_types();
        let name = &[name.to_token_stream()];
        Self::new(name, schema_type, enum_type)
    }
}

impl ToTokens for PlainSchema {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct OneOf<'a, T: ToTokens> {
    items: &'a Array<'a, T>,
    discriminator: Option<Feature>,
}

impl<'a, T> ToTokens for OneOf<'a, T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let items = self.items;
        let len = items.len();

        // concat items
        let items_as_tokens = items.iter().fold(TokenStream::new(), |mut items, item| {
            items.extend(quote! {
                .item(#item)
            });

            items
        });

        // discriminator tokens will not fail
        let discriminator = self.discriminator.to_token_stream();

        tokens.extend(quote! {
            Into::<utoipa::openapi::schema::OneOfBuilder>::into(utoipa::openapi::OneOf::with_capacity(#len))
                #items_as_tokens
                #discriminator
        });
    }
}

/// `RefOrOwned` is simple `Cow` like type to wrap either `ref` or owned value. This allows passing
/// either owned or referenced values as if they were owned like the `Cow` does but this works with
/// non cloneable types. Thus values cannot be modified but they can be passed down as re-referenced
/// values by dereffing the original value. `Roo::Ref(original.deref())`.
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Roo<'t, T> {
    Ref(&'t T),
    Owned(T),
}

impl<'t, T> Deref for Roo<'t, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Ref(t) => t,
            Self::Owned(t) => t,
        }
    }
}

impl<'t, T> AsRef<T> for Roo<'t, T> {
    fn as_ref(&self) -> &T {
        self.deref()
    }
}
