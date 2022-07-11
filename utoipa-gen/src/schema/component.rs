use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, ResultExt};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::Parse, punctuated::Punctuated, token::Comma, Attribute, Data, Field, Fields,
    FieldsNamed, FieldsUnnamed, Generics, Token, Variant, Visibility,
};

use crate::{
    component_type::{ComponentFormat, ComponentType},
    doc_comment::CommentAttributes,
    Array, Deprecated,
};

use self::{
    attr::{ComponentAttr, Enum, IsInline, NamedField, UnnamedFieldStruct},
    xml::Xml,
};

use super::{
    serde::{self, RenameRule, SerdeContainer, SerdeValue},
    ComponentPart, GenericType, ValueType,
};

mod attr;
mod xml;

pub struct Component<'a> {
    ident: &'a Ident,
    attributes: &'a [Attribute],
    generics: &'a Generics,
    aliases: Option<Punctuated<AliasComponent, Comma>>,
    data: &'a Data,
    vis: &'a Visibility,
}

impl<'a> Component<'a> {
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

impl ToTokens for Component<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = self.ident;
        let variant = ComponentVariant::new(self.data, self.attributes, ident, self.generics, None);
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let aliases = self.aliases.as_ref().map(|aliases| {
            let alias_components = aliases
                .iter()
                .map(|alias| {
                    let name = &*alias.name;

                    let variant = ComponentVariant::new(
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
                fn aliases() -> Vec<(&'static str, utoipa::openapi::schema::Component)> {
                    #alias_components.to_vec()
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
            impl #impl_generics utoipa::Component for #ident #ty_generics #where_clause {
                fn component() -> utoipa::openapi::schema::Component {
                    #variant.into()
                }

                #aliases
            }

            #type_aliases
        })
    }
}

enum ComponentVariant<'a> {
    Named(NamedStructComponent<'a>),
    Unnamed(UnnamedStructComponent<'a>),
    Enum(EnumComponent<'a>),
}

impl<'a> ComponentVariant<'a> {
    pub fn new(
        data: &'a Data,
        attributes: &'a [Attribute],
        ident: &'a Ident,
        generics: &'a Generics,
        alias: Option<&'a AliasComponent>,
    ) -> ComponentVariant<'a> {
        match data {
            Data::Struct(content) => match &content.fields {
                Fields::Unnamed(fields) => {
                    let FieldsUnnamed { unnamed, .. } = fields;
                    Self::Unnamed(UnnamedStructComponent {
                        attributes,
                        fields: unnamed,
                    })
                }
                Fields::Named(fields) => {
                    let FieldsNamed { named, .. } = fields;
                    Self::Named(NamedStructComponent {
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
            Data::Enum(content) => Self::Enum(EnumComponent {
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

impl ToTokens for ComponentVariant<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Enum(component) => component.to_tokens(tokens),
            Self::Named(component) => component.to_tokens(tokens),
            Self::Unnamed(component) => component.to_tokens(tokens),
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct NamedStructComponent<'a> {
    fields: &'a Punctuated<Field, Comma>,
    attributes: &'a [Attribute],
    generics: Option<&'a Generics>,
    alias: Option<&'a AliasComponent>,
}

impl ToTokens for NamedStructComponent<'_> {
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

                let component_part = &mut ComponentPart::from_type(&field.ty);

                if let Some((generic_types, alias)) = self.generics.zip(self.alias) {
                    generic_types
                        .type_params()
                        .enumerate()
                        .for_each(|(index, generic)| {
                            if let Some(generic_type) =
                                component_part.find_mut_by_ident(&generic.ident)
                            {
                                generic_type.update_ident(
                                    &alias.generics.type_params().nth(index).unwrap().ident,
                                );
                            };
                        })
                }

                let deprecated = super::get_deprecated(&field.attrs);
                let attrs = ComponentAttr::<NamedField>::from_attributes_validated(
                    &field.attrs,
                    component_part,
                );

                let override_component_part = attrs
                    .as_ref()
                    .and_then(|field| field.as_ref().value_type.as_ref())
                    .map(|value_type| value_type.get_component_part());
                let xml_value = attrs
                    .as_ref()
                    .and_then(|named_field| named_field.as_ref().xml.as_ref());
                let comments = CommentAttributes::from_attributes(&field.attrs);

                let component = ComponentProperty::new(
                    override_component_part.as_ref().unwrap_or(component_part),
                    Some(&comments),
                    attrs.as_ref(),
                    deprecated.as_ref(),
                    xml_value,
                );

                tokens.extend(quote! {
                    .property(#name, #component)
                });

                if !component.is_option() {
                    tokens.extend(quote! {
                        .required(#name)
                    })
                }
            });

        if let Some(deprecated) = super::get_deprecated(self.attributes) {
            tokens.extend(quote! { .deprecated(Some(#deprecated)) });
        }

        let attrs = ComponentAttr::<attr::Struct>::from_attributes_validated(self.attributes);
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

#[cfg_attr(feature = "debug", derive(Debug))]
struct UnnamedStructComponent<'a> {
    fields: &'a Punctuated<Field, Comma>,
    attributes: &'a [Attribute],
}

impl ToTokens for UnnamedStructComponent<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fields_len = self.fields.len();
        let first_field = self.fields.first().unwrap();
        let first_part = &ComponentPart::from_type(&first_field.ty);

        let mut is_object = matches!(first_part.value_type, ValueType::Object);

        let all_fields_are_same = fields_len == 1
            || self.fields.iter().skip(1).all(|field| {
                let component_part = &ComponentPart::from_type(&field.ty);

                first_part == component_part
            });

        let attrs =
            attr::parse_component_attr::<ComponentAttr<UnnamedFieldStruct>>(self.attributes);
        let deprecated = super::get_deprecated(self.attributes);
        if all_fields_are_same {
            let override_component = attrs
                .as_ref()
                .and_then(|unnamed_struct| unnamed_struct.as_ref().value_type.as_ref())
                .map(|value_type| value_type.get_component_part());

            if override_component.is_some() {
                is_object = override_component
                    .as_ref()
                    .map(|override_type| matches!(override_type.value_type, ValueType::Object))
                    .unwrap_or_default();
            }

            tokens.extend(
                ComponentProperty::new(
                    override_component.as_ref().unwrap_or(first_part),
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
struct EnumComponent<'a> {
    variants: &'a Punctuated<Variant, Comma>,
    attributes: &'a [Attribute],
}

impl ToTokens for EnumComponent<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self
            .variants
            .iter()
            .all(|variant| matches!(variant.fields, Fields::Unit))
        {
            tokens.extend(
                SimpleEnum {
                    attributes: self.attributes,
                    variants: self.variants,
                }
                .to_token_stream(),
            )
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
                            utoipa::openapi::schema::PropertyBuilder::new()
                                .component_type(utoipa::openapi::ComponentType::String)
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
            utoipa::openapi::PropertyBuilder::new()
            .component_type(utoipa::openapi::ComponentType::String)
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

        let attrs = attr::parse_component_attr::<ComponentAttr<Enum>>(self.attributes);
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
    fn unit_variant_tokens(variant_name: String) -> TokenStream {
        quote! {
            utoipa::openapi::PropertyBuilder::new()
                .component_type(utoipa::openapi::ComponentType::String)
                .enum_values::<[&str; 1], &str>(Some([#variant_name]))
        }
    }
    /// Produce tokens that represent a variant of a [`ComplexEnum`].
    fn variant_tokens(variant_name: String, variant: &Variant) -> TokenStream {
        match &variant.fields {
            Fields::Named(named_fields) => {
                let named_enum = NamedStructComponent {
                    attributes: &variant.attrs,
                    fields: &named_fields.named,
                    generics: None,
                    alias: None,
                };

                quote! {
                    utoipa::openapi::schema::ObjectBuilder::new()
                        .property(#variant_name, #named_enum)
                }
            }
            Fields::Unnamed(unnamed_fields) => {
                let unnamed_enum = UnnamedStructComponent {
                    attributes: &variant.attrs,
                    fields: &unnamed_fields.unnamed,
                };

                quote! {
                    utoipa::openapi::schema::ObjectBuilder::new()
                        .property(#variant_name, #unnamed_enum)
                }
            }
            Fields::Unit => Self::unit_variant_tokens(variant_name),
        }
    }

    /// Produce tokens that represent a variant of a [`ComplexEnum`] where serde enum attribute
    /// `tag = ` applies.
    fn tagged_variant_tokens(tag: &str, variant_name: String, variant: &Variant) -> TokenStream {
        match &variant.fields {
            Fields::Named(named_fields) => {
                let named_enum = NamedStructComponent {
                    attributes: &variant.attrs,
                    fields: &named_fields.named,
                    generics: None,
                    alias: None,
                };

                let variant_name_tokens = Self::unit_variant_tokens(variant_name);

                quote! {
                    #named_enum
                        .property(#tag, #variant_name_tokens)
                        .required(#tag)
                }
            }
            Fields::Unnamed(_) => {
                abort!(
                    variant,
                    "Unnamed (tuple) enum variants are unsupported for internally tagged enums using the `tag = ` serde attribute";

                    help = "Try using a different serde enum representation";
                );
            }
            Fields::Unit => {
                let variant_tokens = Self::unit_variant_tokens(variant_name);
                quote! {
                    utoipa::openapi::schema::ObjectBuilder::new()
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
            .any(|attribute| attribute.path.get_ident().unwrap() == "component")
        {
            abort!(
                self.attributes.first().unwrap(),
                "component macro attribute not expected on complex enum";

                help = "Try adding the #[component(...)] on variant of the enum";
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

                if let Some(tag) = &tag {
                    Self::tagged_variant_tokens(tag, variant_name, variant)
                } else {
                    Self::variant_tokens(variant_name, variant)
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
struct ComponentProperty<'a, T> {
    component_part: &'a ComponentPart<'a>,
    comments: Option<&'a CommentAttributes>,
    attrs: Option<&'a ComponentAttr<T>>,
    deprecated: Option<&'a Deprecated>,
    xml: Option<&'a Xml>,
}

impl<'a, T: Sized + ToTokens> ComponentProperty<'a, T> {
    fn new(
        component_part: &'a ComponentPart<'a>,
        comments: Option<&'a CommentAttributes>,
        attrs: Option<&'a ComponentAttr<T>>,
        deprecated: Option<&'a Deprecated>,
        xml: Option<&'a Xml>,
    ) -> Self {
        Self {
            component_part,
            comments,
            attrs,
            deprecated,
            xml,
        }
    }

    /// Check wheter property is required or not
    fn is_option(&self) -> bool {
        matches!(self.component_part.generic_type, Some(GenericType::Option))
    }
}

impl<T> ToTokens for ComponentProperty<'_, T>
where
    T: Sized + quote::ToTokens + IsInline,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.component_part.generic_type {
            Some(GenericType::Map) => {
                // Maps are treated just as generic objects without types. There is no Map type in OpenAPI spec.
                tokens.extend(quote! {
                    utoipa::openapi::ObjectBuilder::new()
                });

                if let Some(description) = self.comments.and_then(|attributes| attributes.0.first())
                {
                    tokens.extend(quote! {
                        .description(Some(#description))
                    })
                }
            }
            Some(GenericType::Vec) => {
                let component_property = ComponentProperty::new(
                    self.component_part.child.as_ref().unwrap().as_ref(),
                    self.comments,
                    self.attrs,
                    self.deprecated,
                    self.xml,
                );

                tokens.extend(quote! {
                    utoipa::openapi::schema::ArrayBuilder::new()
                        .items(#component_property)
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
                let component_property = ComponentProperty::new(
                    self.component_part.child.as_ref().unwrap().as_ref(),
                    self.comments,
                    self.attrs,
                    self.deprecated,
                    self.xml,
                );

                tokens.extend(component_property.into_token_stream())
            }
            None => {
                let component_part = self.component_part;

                match component_part.value_type {
                    ValueType::Primitive => {
                        let component_type = ComponentType(component_part.ident);

                        tokens.extend(quote! {
                            utoipa::openapi::PropertyBuilder::new().component_type(#component_type)
                        });

                        let format = ComponentFormat(component_part.ident);
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
                        let component_ident = component_part.ident;
                        let name = &*component_ident.to_string();

                        // When users wishes to hinder the actual type with Any type render a generic `object`
                        if component_part.is_any() {
                            tokens.extend(quote! { utoipa::openapi::ObjectBuilder::new() })
                        } else if is_inline {
                            tokens.extend(quote_spanned! {component_ident.span()=>
                                {
                                    <#component_ident as utoipa::Component>::component()
                                }
                            });
                        } else {
                            tokens.extend(quote! {
                                utoipa::openapi::Ref::from_component_name(#name)
                            })
                        }
                    }
                }
            }
        }
    }
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
pub struct AliasComponent {
    pub name: String,
    pub ty: Ident,
    pub generics: Generics,
}

impl Parse for AliasComponent {
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

fn parse_aliases(attributes: &[Attribute]) -> Option<Punctuated<AliasComponent, Comma>> {
    attributes
        .iter()
        .find(|attribute| attribute.path.is_ident("aliases"))
        .map(|aliases| {
            aliases
                .parse_args_with(Punctuated::<AliasComponent, Comma>::parse_terminated)
                .unwrap_or_abort()
        })
}
