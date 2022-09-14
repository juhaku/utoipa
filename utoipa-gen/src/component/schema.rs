use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, Data, Field,
    Fields, FieldsNamed, FieldsUnnamed, Generics, PathArguments, Token, TypePath, Variant,
    Visibility,
};

use crate::{
    doc_comment::CommentAttributes,
    schema_type::{SchemaFormat, SchemaType},
    Array, Deprecated,
};

use self::{
    attr::{Enum, IsInline, NamedField, SchemaAttr, Title, UnnamedFieldStruct},
    xml::Xml,
};

use super::{
    serde::{self, RenameRule, SerdeContainer, SerdeValue},
    GenericType, TypeTree, ValueType,
};

mod attr;
mod xml;

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
                        fields: unnamed,
                    })
                }
                Fields::Named(fields) => {
                    let FieldsNamed { named, .. } = fields;
                    Self::Named(NamedStructSchema {
                        attributes,
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
    generics: Option<&'a Generics>,
    alias: Option<&'a AliasSchema>,
}

impl ToTokens for NamedStructSchema<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let container_rules = serde::parse_container(self.attributes);

        tokens.extend(quote! { utoipa::openapi::ObjectBuilder::new() });

        self.fields
            .iter()
            .filter_map(|field| {
                let field_rule = serde::parse_value(&field.attrs);

                if is_not_skipped(&field_rule) {
                    Some((field, field_rule))
                } else {
                    None
                }
            })
            .for_each(|(field, mut field_rule)| {
                let mut field_name = &*field.ident.as_ref().unwrap().to_string();

                if field_name.starts_with("r#") {
                    field_name = &field_name[2..];
                }

                let name = &rename_field(&container_rules, &mut field_rule, field_name)
                    .unwrap_or_else(|| String::from(field_name));

                let type_tree = &mut TypeTree::from_type(&field.ty);

                if let Some((generic_types, alias)) = self.generics.zip(self.alias) {
                    generic_types
                        .type_params()
                        .enumerate()
                        .for_each(|(index, generic)| {
                            if let Some(generic_type) = type_tree.find_mut_by_ident(&generic.ident)
                            {
                                generic_type.update_path(
                                    &alias.generics.type_params().nth(index).unwrap().ident,
                                );
                            };
                        })
                }

                let deprecated = super::get_deprecated(&field.attrs);
                let attrs =
                    SchemaAttr::<NamedField>::from_attributes_validated(&field.attrs, type_tree);

                let override_type_tree = attrs
                    .as_ref()
                    .and_then(|field| field.as_ref().value_type.as_ref().map(TypeTree::from_type));

                let xml_value = attrs
                    .as_ref()
                    .and_then(|named_field| named_field.as_ref().xml.as_ref());
                let comments = CommentAttributes::from_attributes(&field.attrs);

                let schema_property = SchemaProperty::new(
                    override_type_tree.as_ref().unwrap_or(type_tree),
                    Some(&comments),
                    attrs.as_ref(),
                    deprecated.as_ref(),
                    xml_value,
                );

                tokens.extend(quote! {
                    .property(#name, #schema_property)
                });

                if !schema_property.is_option() && !is_default(&container_rules, &field_rule) {
                    tokens.extend(quote! {
                        .required(#name)
                    })
                }
            });

        if let Some(deprecated) = super::get_deprecated(self.attributes) {
            tokens.extend(quote! { .deprecated(Some(#deprecated)) });
        }

        let attrs = SchemaAttr::<attr::Struct>::from_attributes_validated(self.attributes);
        if let Some(attrs) = attrs {
            tokens.extend(attrs.to_token_stream());
        }

        if let Some(comment) = CommentAttributes::from_attributes(self.attributes).first() {
            tokens.extend(quote! {
                .description(Some(#comment))
            })
        }
    }
}

#[inline]
fn is_default(container_rules: &Option<SerdeContainer>, field_rule: &Option<SerdeValue>) -> bool {
    *container_rules
        .as_ref()
        .and_then(|rule| rule.default.as_ref())
        .unwrap_or(&false)
        || *field_rule
            .as_ref()
            .and_then(|rule| rule.default.as_ref())
            .unwrap_or(&false)
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct UnnamedStructSchema<'a> {
    fields: &'a Punctuated<Field, Comma>,
    attributes: &'a [Attribute],
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

        let attrs = attr::parse_schema_attr::<SchemaAttr<UnnamedFieldStruct>>(self.attributes);
        let deprecated = super::get_deprecated(self.attributes);
        if all_fields_are_same {
            let override_schema = attrs.as_ref().and_then(|unnamed_struct| {
                unnamed_struct
                    .as_ref()
                    .value_type
                    .as_ref()
                    .map(TypeTree::from_type)
            });

            if override_schema.is_some() {
                is_object = override_schema
                    .as_ref()
                    .map(|override_type| matches!(override_type.value_type, ValueType::Object))
                    .unwrap_or_default();
            }

            tokens.extend(
                SchemaProperty::new(
                    override_schema.as_ref().unwrap_or(first_part),
                    None,
                    attrs.as_ref(),
                    deprecated.as_ref(),
                    None,
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

            if let Some(attrs) = attrs {
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
            let repr_match = self.attributes
                .iter()
                .find_map(|attr| {
                    if attr.path.is_ident("repr") {
                        attr.parse_args::<Ident>().ok()
                    } else {
                        None
                    }
                });

            if let Some(ty) = repr_match {
                tokens.extend(
                    ReprEnum {
                        attributes: self.attributes,
                        variants: self.variants,
                        rtype: ty,
                    }
                    .to_token_stream()
                )
            } else {
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

#[cfg_attr(feature = "debug", derive(Debug))]
struct ReprEnum<'a> {
    variants: &'a Punctuated<Variant, Comma>,
    attributes: &'a [Attribute],
    rtype: Ident,
}

impl ReprEnum<'_> {
    /// Produce tokens that represent each variant.
    fn variants_tokens(&self) -> TokenStream {
        let iter = self.variants.iter().map(|variant| &variant.ident);
        let ty = self.rtype.clone();
        let enum_values = quote!{
            [
                #(Self::#iter as #ty,)*
            ]
        };

        quote! {
            utoipa::openapi::ObjectBuilder::new()
            .schema_type(utoipa::openapi::SchemaType::Integer)
            .enum_values(Some(#enum_values))
        }
    }
}

impl ToTokens for ReprEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.variants_tokens());

        let attrs = attr::parse_schema_attr::<SchemaAttr<Enum>>(self.attributes);
        if let Some(attributes) = attrs {
            tokens.extend(attributes.to_token_stream());
        }

        if let Some(deprecated) = super::get_deprecated(self.attributes) {
            tokens.extend(quote! { .deprecated(Some(#deprecated)) });
        }

        if let Some(comment) = CommentAttributes::from_attributes(self.attributes).first() {
            tokens.extend(quote! {
                .description(Some(#comment))
            })
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct SimpleEnum<'a> {
    variants: &'a Punctuated<Variant, Comma>,
    attributes: &'a [Attribute],
}

impl SimpleEnum<'_> {
    /// Produce tokens that represent each variant for the situation where the serde enum tag =
    /// "<tag>" attribute applies.
    fn tagged_variants_tokens(tag: String, enum_values: Array<String>) -> TokenStream {
        let len = enum_values.len();
        let items: TokenStream = enum_values
            .iter()
            .map(|enum_value: &String| {
                quote! {
                    utoipa::openapi::schema::ObjectBuilder::new()
                        .property(
                            #tag,
                            utoipa::openapi::schema::ObjectBuilder::new()
                                .schema_type(utoipa::openapi::SchemaType::String)
                                .enum_values::<[&str; 1], &str>(Some([#enum_value]))
                        )
                        .required(#tag)
                }
            })
            .map(|object: TokenStream| {
                quote! {
                    .item(#object)
                }
            })
            .collect();
        quote! {
            Into::<utoipa::openapi::schema::OneOfBuilder>::into(utoipa::openapi::OneOf::with_capacity(#len))
                #items
        }
    }

    /// Produce tokens that represent each variant.
    fn variants_tokens(enum_values: Array<String>) -> TokenStream {
        let len = enum_values.len();
        quote! {
            utoipa::openapi::ObjectBuilder::new()
            .schema_type(utoipa::openapi::SchemaType::String)
            .enum_values::<[&str; #len], &str>(Some(#enum_values))
        }
    }
}

impl ToTokens for SimpleEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let container_rules = serde::parse_container(self.attributes);

        let enum_values = self
            .variants
            .iter()
            .filter_map(|variant| {
                let mut variant_rules = serde::parse_value(&variant.attrs);

                if is_not_skipped(&variant_rules) {
                    let name = &*variant.ident.to_string();
                    let renamed = rename_variant(&container_rules, &mut variant_rules, name);

                    renamed.or_else(|| Some(String::from(name)))
                } else {
                    None
                }
            })
            .collect::<Array<String>>();

        tokens.extend(match container_rules {
            Some(serde_container) if serde_container.tag.is_some() => {
                let tag = serde_container.tag.expect("Expected tag to be present");
                Self::tagged_variants_tokens(tag, enum_values)
            }
            _ => Self::variants_tokens(enum_values),
        });

        let attrs = attr::parse_schema_attr::<SchemaAttr<Enum>>(self.attributes);
        if let Some(attributes) = attrs {
            tokens.extend(attributes.to_token_stream());
        }

        if let Some(deprecated) = super::get_deprecated(self.attributes) {
            tokens.extend(quote! { .deprecated(Some(#deprecated)) });
        }

        if let Some(comment) = CommentAttributes::from_attributes(self.attributes).first() {
            tokens.extend(quote! {
                .description(Some(#comment))
            })
        }
    }
}

struct ComplexEnum<'a> {
    variants: &'a Punctuated<Variant, Comma>,
    attributes: &'a [Attribute],
}

impl ComplexEnum<'_> {
    fn unit_variant_tokens(
        variant_name: String,
        variant_title: Option<SchemaAttr<Title>>,
    ) -> TokenStream {
        quote! {
            utoipa::openapi::ObjectBuilder::new()
                #variant_title
                .schema_type(utoipa::openapi::SchemaType::String)
                .enum_values::<[&str; 1], &str>(Some([#variant_name]))
        }
    }
    /// Produce tokens that represent a variant of a [`ComplexEnum`].
    fn variant_tokens(
        variant_name: String,
        variant_title: Option<SchemaAttr<Title>>,
        variant: &Variant,
    ) -> TokenStream {
        match &variant.fields {
            Fields::Named(named_fields) => {
                let named_enum = NamedStructSchema {
                    attributes: &variant.attrs,
                    fields: &named_fields.named,
                    generics: None,
                    alias: None,
                };

                quote! {
                    utoipa::openapi::schema::ObjectBuilder::new()
                        #variant_title
                        .property(#variant_name, #named_enum)
                }
            }
            Fields::Unnamed(unnamed_fields) => {
                let unnamed_enum = UnnamedStructSchema {
                    attributes: &variant.attrs,
                    fields: &unnamed_fields.unnamed,
                };

                quote! {
                    utoipa::openapi::schema::ObjectBuilder::new()
                        #variant_title
                        .property(#variant_name, #unnamed_enum)
                }
            }
            Fields::Unit => Self::unit_variant_tokens(variant_name, variant_title),
        }
    }

    /// Produce tokens that represent a variant of a [`ComplexEnum`] where serde enum attribute
    /// `tag = ` applies.
    fn tagged_variant_tokens(
        tag: &str,
        variant_name: String,
        variant_title: Option<SchemaAttr<Title>>,
        variant: &Variant,
    ) -> TokenStream {
        match &variant.fields {
            Fields::Named(named_fields) => {
                let named_enum = NamedStructSchema {
                    attributes: &variant.attrs,
                    fields: &named_fields.named,
                    generics: None,
                    alias: None,
                };

                let variant_name_tokens = Self::unit_variant_tokens(variant_name, None);

                quote! {
                    #named_enum
                        #variant_title
                        .property(#tag, #variant_name_tokens)
                        .required(#tag)
                }
            }
            Fields::Unnamed(unnamed_fields) => {
                if unnamed_fields.unnamed.len() == 1 {
                    let unnamed_enum = UnnamedStructSchema {
                        attributes: &variant.attrs,
                        fields: &unnamed_fields.unnamed,
                    };

                    quote! {
                        utoipa::openapi::schema::ObjectBuilder::new()
                            #variant_title
                            .property(#variant_name, #unnamed_enum)
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
                let variant_tokens = Self::unit_variant_tokens(variant_name, None);
                quote! {
                    utoipa::openapi::schema::ObjectBuilder::new()
                        #variant_title
                        .property(#tag, #variant_tokens)
                        .required(#tag)
                }
            }
        }
    }
}

impl ToTokens for ComplexEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self
            .attributes
            .iter()
            .any(|attribute| attribute.path.get_ident().unwrap() == "schema")
        {
            abort!(
                self.attributes.first().unwrap(),
                "schema macro attribute not expected on complex enum";

                help = "Try adding the #[schema(...)] on variant of the enum";
            );
        }

        let capacity = self.variants.len();

        let mut container_rules = serde::parse_container(self.attributes);
        let tag: Option<String> = if let Some(serde_container) = &mut container_rules {
            serde_container.tag.take()
        } else {
            None
        };

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
            .map(|(variant, mut variant_serde_rules)| {
                let variant_name = &*variant.ident.to_string();
                let variant_name =
                    rename_variant(&container_rules, &mut variant_serde_rules, variant_name)
                        .unwrap_or_else(|| String::from(variant_name));
                let variant_title = attr::parse_schema_attr::<SchemaAttr<Title>>(&variant.attrs);

                if let Some(tag) = &tag {
                    Self::tagged_variant_tokens(tag, variant_name, variant_title, variant)
                } else {
                    Self::variant_tokens(variant_name, variant_title, variant)
                }
            })
            .map(|inline_variant| {
                quote! {
                    .item(#inline_variant)
                }
            })
            .collect();

        tokens.extend(
            quote! {
                Into::<utoipa::openapi::schema::OneOfBuilder>::into(utoipa::openapi::OneOf::with_capacity(#capacity))
                    #items
            }
        );

        if let Some(comment) = CommentAttributes::from_attributes(self.attributes).first() {
            tokens.extend(quote! {
                .description(Some(#comment))
            })
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq)]
struct TypeTuple<'a, T>(T, &'a Ident);

#[cfg_attr(feature = "debug", derive(Debug))]
struct SchemaProperty<'a, T> {
    schema_part: &'a TypeTree<'a>,
    comments: Option<&'a CommentAttributes>,
    attrs: Option<&'a SchemaAttr<T>>,
    deprecated: Option<&'a Deprecated>,
    xml: Option<&'a Xml>,
}

impl<'a, T: Sized + ToTokens> SchemaProperty<'a, T> {
    fn new(
        schema_part: &'a TypeTree<'a>,
        comments: Option<&'a CommentAttributes>,
        attrs: Option<&'a SchemaAttr<T>>,
        deprecated: Option<&'a Deprecated>,
        xml: Option<&'a Xml>,
    ) -> Self {
        Self {
            schema_part,
            comments,
            attrs,
            deprecated,
            xml,
        }
    }

    /// Check wheter property is required or not
    fn is_option(&self) -> bool {
        matches!(self.schema_part.generic_type, Some(GenericType::Option))
    }
}

impl<T> ToTokens for SchemaProperty<'_, T>
where
    T: Sized + quote::ToTokens + IsInline,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.schema_part.generic_type {
            Some(GenericType::Map) => {
                // Maps are treated as generic objects with no named properties and
                // additionalProperties denoting the type
                // maps have 2 child schemas and we are interested the second one of them
                // which is used to determine the additional properties
                let schema_property = SchemaProperty::new(
                    self.schema_part
                        .children
                        .as_ref()
                        .expect("SchemaProperty Map type should have children")
                        .iter()
                        .nth(1)
                        .expect("SchemaProperty Map type should have 2 child"),
                    self.comments,
                    self.attrs,
                    self.deprecated,
                    self.xml,
                );

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
                let schema_property = SchemaProperty::new(
                    self.schema_part
                        .children
                        .as_ref()
                        .expect("SchemaProperty Vec should have children")
                        .iter()
                        .next()
                        .expect("SchemaProperty Vec should have 1 child"),
                    self.comments,
                    self.attrs,
                    self.deprecated,
                    self.xml,
                );

                tokens.extend(quote! {
                    utoipa::openapi::schema::ArrayBuilder::new()
                        .items(#schema_property)
                });

                if let Some(xml_value) = self.xml {
                    match xml_value {
                        Xml::Slice { vec, value: _ } => tokens.extend(quote! {
                            .xml(Some(#vec))
                        }),
                        Xml::NonSlice(_) => (),
                    }
                }
            }
            Some(GenericType::Option)
            | Some(GenericType::Cow)
            | Some(GenericType::Box)
            | Some(GenericType::RefCell) => {
                let schema_property = SchemaProperty::new(
                    self.schema_part
                        .children
                        .as_ref()
                        .expect("SchemaProperty generic container type should have children")
                        .iter()
                        .next()
                        .expect("SchemaProperty generic container type should have 1 child"),
                    self.comments,
                    self.attrs,
                    self.deprecated,
                    self.xml,
                );

                tokens.extend(schema_property.into_token_stream())
            }
            None => {
                let type_tree = self.schema_part;

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

                        if let Some(attributes) = self.attrs {
                            tokens.extend(attributes.to_token_stream())
                        }

                        if let Some(xml_value) = self.xml {
                            match xml_value {
                                Xml::Slice { vec: _, value } => tokens.extend(quote! {
                                    .xml(Some(#value))
                                }),
                                Xml::NonSlice(xml) => tokens.extend(quote! {
                                    .xml(Some(#xml))
                                }),
                            }
                        }
                    }
                    ValueType::Object => {
                        let is_inline: bool = self
                            .attrs
                            .map(|attributes| attributes.is_inline())
                            .unwrap_or(false);
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

/// Reformat a path reference string that was generated using [`quote`] to be used as a nice compact schema reference,
/// by removing spaces between colon punctuation and `::` and the path segments.
pub(crate) fn format_path_ref(path: &TypePath) -> String {
    let mut path: TypePath = path.clone();

    // Generics and path arguments are unsupported
    if let Some(last_segment) = path.path.segments.last_mut() {
        last_segment.arguments = PathArguments::None;
    }

    // :: are not officially supported in the spec
    // See: https://github.com/juhaku/utoipa/pull/187#issuecomment-1173101405
    path.to_token_stream().to_string().replace(" :: ", ".")
}

#[inline]
fn is_not_skipped(rule: &Option<SerdeValue>) -> bool {
    rule.as_ref()
        .map(|value| value.skip.is_none())
        .unwrap_or(true)
}

/// Resolves the appropriate [`RenameRule`] to apply to the specified `struct` `field` name given a
/// `container_rule` (`struct` or `enum` level) and `field_rule` (`struct` field or `enum` variant
/// level). Returns `Some` of the result of the `rename_op` if a rename is required by the supplied
/// rules.
#[inline]
fn rename_field<'a>(
    container_rule: &'a Option<SerdeContainer>,
    field_rule: &'a mut Option<SerdeValue>,
    field: &str,
) -> Option<String> {
    rename(container_rule, field_rule, &|rule| rule.rename(field))
}

/// Resolves the appropriate [`RenameRule`] to apply to the specified `enum` `variant` name given a
/// `container_rule` (`struct` or `enum` level) and `field_rule` (`struct` field or `enum` variant
/// level). Returns `Some` of the result of the `rename_op` if a rename is required by the supplied
/// rules.
#[inline]
fn rename_variant<'a>(
    container_rule: &'a Option<SerdeContainer>,
    field_rule: &'a mut Option<SerdeValue>,
    variant: &str,
) -> Option<String> {
    rename(container_rule, field_rule, &|rule| {
        rule.rename_variant(variant)
    })
}

/// Resolves the appropriate [`RenameRule`] to apply during a `rename_op` given a `container_rule`
/// (`struct` or `enum` level) and `field_rule` (`struct` field or `enum` variant level). Returns
/// `Some` of the result of the `rename_op` if a rename is required by the supplied rules.
#[inline]
fn rename<'a>(
    container_rule: &'a Option<SerdeContainer>,
    field_rule: &'a mut Option<SerdeValue>,
    rename_op: &impl Fn(&RenameRule) -> String,
) -> Option<String> {
    field_rule
        .as_mut()
        .and_then(|value| value.rename.take())
        .or_else(|| {
            container_rule
                .as_ref()
                .and_then(|container| container.rename_all.as_ref().map(rename_op))
        })
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
