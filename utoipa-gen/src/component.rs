use std::rc::Rc;

use proc_macro2::{Ident, TokenStream as TokenStream2};
use proc_macro_error::{abort, abort_call_site};
use quote::{quote, ToTokens};
use syn::{
    punctuated::Punctuated, token::Comma, AngleBracketedGenericArguments, Attribute, Data, Field,
    Fields, FieldsNamed, FieldsUnnamed, GenericArgument, Generics, PathArguments, PathSegment,
    Type, TypePath, Variant,
};

use crate::{
    component_type::{ComponentFormat, ComponentType},
    doc_comment::CommentAttributes,
    Array, Deprecated,
};

use self::{
    attr::{ComponentAttr, Enum, NamedField, UnnamedFieldStruct},
    xml::Xml,
};

mod attr;
mod xml;

pub struct Component<'a> {
    ident: &'a Ident,
    variant: ComponentVariant<'a>,
    generics: &'a Generics,
}

impl<'a> Component<'a> {
    pub fn new(
        data: &'a Data,
        attributes: &'a [Attribute],
        ident: &'a Ident,
        generics: &'a Generics,
    ) -> Self {
        Self {
            ident,
            variant: ComponentVariant::new(data, attributes, ident),
            generics,
        }
    }
}

impl ToTokens for Component<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ident = self.ident;
        let variant = &self.variant;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        tokens.extend(quote! {
            impl #impl_generics utoipa::Component for #ident #ty_generics #where_clause {
                fn component() -> utoipa::openapi::schema::Component {
                    #variant.into()
                }
            }
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
    fn to_tokens(&self, tokens: &mut TokenStream2) {
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
}

impl ToTokens for NamedStructComponent<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(quote! { utoipa::openapi::ObjectBuilder::new() });
        self.fields.iter().for_each(|field| {
            let field_name = &*field.ident.as_ref().unwrap().to_string();

            let component_part = &ComponentPart::from_type(&field.ty);
            let deprecated = get_deprecated(&field.attrs);
            let attrs = ComponentAttr::<NamedField>::from_attributes_validated(
                &field.attrs,
                component_part,
            );
            let xml_value = attrs
                .as_ref()
                .and_then(|named_field| named_field.as_ref().xml.as_ref());
            let comments = CommentAttributes::from_attributes(&field.attrs);

            let component = ComponentProperty::new(
                component_part,
                Some(&comments),
                attrs.as_ref(),
                deprecated.as_ref(),
                xml_value,
            );

            tokens.extend(quote! {
                .property(#field_name, #component)
            });

            if !component.is_option() {
                tokens.extend(quote! {
                    .required(#field_name)
                })
            }
        });

        if let Some(deprecated) = get_deprecated(self.attributes) {
            tokens.extend(quote! { .deprecated(Some(#deprecated)) });
        }

        let attrs = ComponentAttr::<attr::Struct>::from_attributes_validated(self.attributes);
        if let Some(attrs) = attrs {
            tokens.extend(attrs.to_token_stream());
        }

        if let Some(comment) = CommentAttributes::from_attributes(self.attributes)
            .0
            .first()
        {
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
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let fields_len = self.fields.len();
        let first_field = self.fields.first().unwrap();
        let first_part = &ComponentPart::from_type(&first_field.ty);

        let all_fields_are_same = fields_len == 1
            || self.fields.iter().skip(1).all(|field| {
                let component_part = &ComponentPart::from_type(&field.ty);

                first_part == component_part
            });

        let attrs =
            attr::parse_component_attr::<ComponentAttr<UnnamedFieldStruct>>(self.attributes);
        let deprecated = get_deprecated(self.attributes);
        if all_fields_are_same {
            tokens.extend(
                ComponentProperty::new(first_part, None, attrs.as_ref(), deprecated.as_ref(), None)
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

        if let Some(comment) = CommentAttributes::from_attributes(self.attributes)
            .0
            .first()
        {
            tokens.extend(quote! {
                .description(Some(#comment))
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
struct EnumComponent<'a> {
    variants: &'a Punctuated<Variant, Comma>,
    attributes: &'a [Attribute],
}

impl ToTokens for EnumComponent<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
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

impl ToTokens for SimpleEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let enum_values = self
            .variants
            .iter()
            .filter(|variant| matches!(variant.fields, Fields::Unit))
            .map(|variant| variant.ident.to_string())
            .collect::<Array<String>>();

        tokens.extend(quote! {
            utoipa::openapi::PropertyBuilder::new()
            .component_type(utoipa::openapi::ComponentType::String)
            .enum_values(Some(#enum_values))
        });

        let attrs = attr::parse_component_attr::<ComponentAttr<Enum>>(self.attributes);
        if let Some(attributes) = attrs {
            tokens.extend(attributes.to_token_stream());
        }

        if let Some(deprecated) = get_deprecated(self.attributes) {
            tokens.extend(quote! { .deprecated(Some(#deprecated)) });
        }

        if let Some(comment) = CommentAttributes::from_attributes(self.attributes)
            .0
            .first()
        {
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

impl ToTokens for ComplexEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
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

        let capasity = self.variants.len();
        tokens.extend(quote! {
            Into::<utoipa::openapi::schema::OneOfBuilder>::into(utoipa::openapi::OneOf::with_capacity(#capasity))
        });

        // serde, externally tagged format supported by now
        self.variants
            .iter()
            .map(|variant| match &variant.fields {
                Fields::Named(named_fields) => {
                    let named_enum = NamedStructComponent {
                        attributes: &variant.attrs,
                        fields: &named_fields.named,
                    };
                    let name = &*variant.ident.to_string();

                    quote! {
                        utoipa::openapi::schema::ObjectBuilder::new()
                            .property(#name, #named_enum)
                    }
                }
                Fields::Unnamed(unnamed_fields) => {
                    let unnamed_enum = UnnamedStructComponent {
                        attributes: &variant.attrs,
                        fields: &unnamed_fields.unnamed,
                    };
                    let name = &*variant.ident.to_string();

                    quote! {
                        utoipa::openapi::schema::ObjectBuilder::new()
                            .property(#name, #unnamed_enum)
                    }
                }
                Fields::Unit => {
                    let mut enum_values = Punctuated::<Variant, Comma>::new();
                    enum_values.push(variant.clone());

                    SimpleEnum {
                        attributes: &variant.attrs,
                        variants: &enum_values,
                    }
                    .to_token_stream()
                }
            })
            .for_each(|inline_variant| {
                tokens.extend(quote! {
                    .item(#inline_variant)
                })
            });

        if let Some(comment) = CommentAttributes::from_attributes(self.attributes)
            .0
            .first()
        {
            tokens.extend(quote! {
                .description(Some(#comment))
            })
        }
    }
}

fn get_deprecated(attributes: &[Attribute]) -> Option<Deprecated> {
    attributes.iter().find_map(|attribute| {
        if *attribute.path.get_ident().unwrap() == "deprecated" {
            Some(Deprecated::True)
        } else {
            None
        }
    })
}

#[derive(PartialEq)]
#[cfg_attr(feature = "debug", derive(Debug))]
/// Linked list of implementing types of a field in a struct.
struct ComponentPart<'a> {
    ident: &'a Ident,
    value_type: ValueType,
    generic_type: Option<GenericType>,
    child: Option<Rc<ComponentPart<'a>>>,
}

impl<'a> ComponentPart<'a> {
    fn from_type(ty: &'a Type) -> ComponentPart<'a> {
        ComponentPart::from_type_path(
            match ty {
                Type::Path(path) => path,
                Type::Reference(reference) => match reference.elem.as_ref() {
                    Type::Path(path) => path,
                    _ => abort_call_site!("unexpected type in reference, expected Type:Path"),
                },
                _ => abort_call_site!("unexpected type, expected Type::Path"),
            },
            ComponentPart::convert,
            ComponentPart::resolve_component_type,
        )
    }

    fn from_type_path(
        type_path: &'a TypePath,
        op: impl Fn(&'a Ident, &'a PathSegment) -> ComponentPart<'a>,
        or_else: impl Fn(&'a PathSegment) -> ComponentPart<'a>,
    ) -> ComponentPart<'a> {
        let segment = type_path.path.segments.first().unwrap();

        type_path
            .path
            .get_ident()
            .map(|ident| op(ident, segment))
            .unwrap_or_else(|| or_else(segment))
    }

    // Only when type is a generic type we get to this function.
    fn resolve_component_type(segment: &'a PathSegment) -> ComponentPart<'a> {
        if segment.arguments.is_empty() {
            abort!(
                segment.ident,
                "expected at least one angle bracket argument but was 0"
            );
        };

        let mut generic_component_type = ComponentPart::convert(&segment.ident, segment);

        generic_component_type.child = Some(Rc::new(ComponentPart::from_type(
            match &segment.arguments {
                PathArguments::AngleBracketed(angle_bracketed_args) => {
                    ComponentPart::get_generic_arg_type(0, angle_bracketed_args)
                }
                _ => abort!(
                    segment.ident,
                    "unexpected path argument, expected angle bracketed path argument"
                ),
            },
        )));

        generic_component_type
    }

    fn get_generic_arg_type(index: usize, args: &'a AngleBracketedGenericArguments) -> &'a Type {
        let generic_arg = args.args.iter().nth(index);

        match generic_arg {
            Some(GenericArgument::Type(generic_type)) => generic_type,
            Some(GenericArgument::Lifetime(_)) => {
                ComponentPart::get_generic_arg_type(index + 1, args)
            }
            _ => abort!(
                generic_arg,
                "expected generic argument type or generic argument lifetime"
            ),
        }
    }

    fn convert(ident: &'a Ident, segment: &PathSegment) -> ComponentPart<'a> {
        let generic_type = ComponentPart::get_generic(segment);

        Self {
            ident,
            value_type: if ComponentType(ident).is_primitive() {
                ValueType::Primitive
            } else {
                ValueType::Object
            },
            generic_type,
            child: None,
        }
    }

    fn get_generic(segment: &PathSegment) -> Option<GenericType> {
        match &*segment.ident.to_string() {
            "HashMap" | "Map" | "BTreeMap" => Some(GenericType::Map),
            "Vec" => Some(GenericType::Vec),
            "Option" => Some(GenericType::Option),
            "Cow" => Some(GenericType::Cow),
            "Box" => Some(GenericType::Box),
            "RefCell" => Some(GenericType::RefCell),
            _ => None,
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
enum ValueType {
    Primitive,
    Object,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq, Clone, Copy)]
enum GenericType {
    Vec,
    Map,
    Option,
    Cow,
    Box,
    RefCell,
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
    T: Sized + quote::ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream2) {
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
                    self.component_part.child.as_ref().unwrap(),
                    self.comments,
                    self.attrs,
                    self.deprecated,
                    self.xml,
                );

                tokens.extend(quote! {
                    #component_property.to_array_builder()
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
                    self.component_part.child.as_ref().unwrap(),
                    self.comments,
                    self.attrs,
                    self.deprecated,
                    self.xml,
                );

                tokens.extend(component_property.into_token_stream())
            }
            None => match self.component_part.value_type {
                ValueType::Primitive => {
                    let component_type = ComponentType(self.component_part.ident);

                    tokens.extend(quote! {
                        utoipa::openapi::PropertyBuilder::new().component_type(#component_type)
                    });

                    let format = ComponentFormat(self.component_part.ident);
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
                    let name = &*self.component_part.ident.to_string();

                    tokens.extend(quote! {
                        utoipa::openapi::Ref::from_component_name(#name)
                    })
                }
            },
        }
    }
}
