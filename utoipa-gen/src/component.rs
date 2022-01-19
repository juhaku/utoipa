use std::rc::Rc;

use proc_macro2::{Ident, TokenStream as TokenStream2};
use proc_macro_error::{abort, abort_call_site, emit_error};
use quote::{quote, ToTokens};
use syn::{
    Attribute, Data, Field, Fields, FieldsNamed, FieldsUnnamed, GenericArgument, PathArguments,
    PathSegment, Type, TypePath, Variant,
};

use crate::{
    component_type::{ComponentFormat, ComponentType},
    doc_comment::CommentAttributes,
    ValueArray,
};

use self::attr::{ComponentAttr, Enum, NamedField};

mod attr;

pub struct Component<'a> {
    ident: &'a Ident,
    variant: ComponentVariant<'a>,
}

impl<'a> Component<'a> {
    pub fn new(data: Data, attributes: &'a [Attribute], ident: &'a Ident) -> Self {
        Self {
            ident,
            variant: ComponentVariant::new(data, attributes),
        }
    }
}

impl ToTokens for Component<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ident = self.ident;
        let component_variant_impl = &self.variant;

        tokens.extend(quote! {
            impl utoipa::Component for #ident {
                fn component() -> utoipa::openapi::schema::Component {
                    #component_variant_impl.into()
                }
            }
        })
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum FieldType {
    Named,
    Unnamed,
}

#[cfg_attr(feature = "debug", derive(Debug))]
/// Holds the OpenAPI Component implementation which can be added the Schema.
pub enum ComponentVariant<'a> {
    /// Object variant is rust sturct with Component derive annotation.
    Object(Vec<Field>, &'a [Attribute], FieldType),
    /// Enum variant is rust enum with Component derive annotation. **Only supports** enums with
    /// Unit type fields.
    Enum(Vec<Variant>, &'a [Attribute]),
}

impl<'a> ComponentVariant<'a> {
    pub fn new(data: Data, attributes: &'a [Attribute]) -> ComponentVariant<'a> {
        match data {
            Data::Struct(content) => {
                let (fields , field_type ) = match content.fields {
                    Fields::Unnamed(fields) => {
                        let FieldsUnnamed { unnamed, .. } = fields;
                        (unnamed , FieldType::Unnamed)
                    }
                    Fields::Named(fields) => {
                        let FieldsNamed { named, .. } = fields;
                        (named, FieldType::Named)
                    }
                    Fields::Unit => abort_call_site!("Expected struct with either named or unnamed fields, unit type unsupported")
                };
                ComponentVariant::Object(fields.into_iter().collect(), attributes, field_type)
            }
            Data::Enum(content) => {
                ComponentVariant::Enum(content.variants.into_iter().collect(), attributes)
            }
            _ => abort_call_site!(
                "Unexpected data type, expected syn::Data::Struct or syn::Data::Enum"
            ),
        }
    }
}

impl<'a> ToTokens for ComponentVariant<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(quote! {
            use utoipa::openapi::{ComponentType, ComponentFormat};
        });

        match self {
            Self::Object(fields, attrs, field_type) => {
                self.struct_to_tokens(fields, *attrs, tokens, field_type)
            }
            Self::Enum(variants, attrs) => self.enum_to_tokens(variants, *attrs, tokens),
        };
    }
}

impl<'a> ComponentVariant<'a> {
    fn struct_to_tokens(
        &self,
        fields: &[Field],
        attributes: &[Attribute],
        tokens: &mut TokenStream2,
        field_type: &FieldType,
    ) {
        match field_type {
            FieldType::Named => self.named_fields_struct_to_tokens(fields, tokens),
            FieldType::Unnamed => self.unnamed_fields_struct_to_tokens(fields, tokens),
        }

        self.append_description(attributes, tokens);
    }

    fn named_fields_struct_to_tokens(&self, fields: &[Field], tokens: &mut TokenStream2) {
        tokens.extend(quote! { utoipa::openapi::Object::new() });

        fields.iter().for_each(|field| {
            let field_name = &*field.ident.as_ref().unwrap().to_string();

            let component_part = &ComponentPart::from_type(&field.ty);

            let attrs = attr::parse_component_attr::<ComponentAttr<NamedField>>(&field.attrs);
            let comments = CommentAttributes::from_attributes(&field.attrs);
            let component = ComponentProperty::new(component_part, Some(&comments), attrs.as_ref());

            tokens.extend(quote! {
                .with_property(#field_name, #component)
            });

            if !component.is_option() {
                tokens.extend(quote! {
                    .with_required(#field_name)
                })
            }
        });
    }

    fn unnamed_fields_struct_to_tokens(&self, fields: &[Field], tokens: &mut TokenStream2) {
        let fields_len = fields.len();
        let first_field = fields.first().unwrap();
        let first_part = &ComponentPart::from_type(&first_field.ty);

        let all_fields_are_same = fields_len == 1
            || fields.iter().skip(1).all(|field| {
                let component_part = &ComponentPart::from_type(&field.ty);

                first_part == component_part
            });

        if all_fields_are_same {
            tokens.extend(
                ComponentProperty::new(first_part, None, None::<&ComponentAttr<attr::Struct>>)
                    .to_token_stream(),
            );
            if fields_len > 1 {
                tokens.extend(quote! { .to_array() })
            }
        } else {
            // Struct that has multiple unnamed fields is serialized to array by default with serde.
            // See: https://serde.rs/json.html
            // Typically OpenAPI does not support multi type arrays thus we simply consider the case
            // as generic object array
            tokens.extend(quote! {
                utoipa::openapi::Object::new().to_array()
            });
        };
    }

    fn warn_unsupported_enum_variants(&self, variants: &[Variant]) {
        variants
            .iter()
            .filter(|variant| !matches!(variant.fields, Fields::Unit))
            .for_each(|variant| emit_error!(variant.ident.span(), "Currently unsupported enum variant, expected Unit variant without additional fields"));
    }

    fn enum_to_tokens(
        &self,
        variants: &[Variant],
        attributes: &[Attribute],
        tokens: &mut TokenStream2,
    ) {
        self.warn_unsupported_enum_variants(variants);

        let enum_values = &variants
            .iter()
            .filter(|variant| matches!(variant.fields, Fields::Unit))
            .map(|variant| variant.ident.to_string())
            .collect::<ValueArray<String>>();

        tokens.extend(quote! {
            utoipa::openapi::Property::new(ComponentType::String)
                .with_enum_values(#enum_values)
        });

        let attrs = attr::parse_component_attr::<ComponentAttr<Enum>>(attributes);
        if let Some(attributes) = attrs {
            tokens.extend(attributes.to_token_stream());
        }

        self.append_description(attributes, tokens);
    }

    fn append_description(&self, attributes: &[Attribute], tokens: &mut TokenStream2) {
        if let Some(comment) = CommentAttributes::from_attributes(attributes).0.first() {
            tokens.extend(quote! {
                .with_description(#comment)
            })
        }
    }
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
                _ => abort_call_site!("Unexpected type, expected Type::Path"),
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
                segment.ident.span(),
                "Expected at least one angle bracket argument but was 0"
            );
        };

        let mut generic_component_type = ComponentPart::convert(&segment.ident, segment);

        generic_component_type.child = Some(Rc::new(ComponentPart::from_type(
            ComponentPart::get_first_generic_type(segment),
        )));

        generic_component_type
    }

    fn get_first_generic_type(segment: &PathSegment) -> &Type {
        match &segment.arguments {
            PathArguments::AngleBracketed(angle_bracketed_args) => {
                let first_arg = angle_bracketed_args.args.first().unwrap();

                match first_arg {
                    GenericArgument::Type(generic_type) => generic_type,
                    _ => abort!(segment.ident, "Expected GenericArgument::Type"),
                }
            }
            _ => abort!(
                segment.ident,
                "Unexpected PathArgument, expected PathArgument::AngleBracketed"
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
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq)]
struct TypeTuple<'a, T>(T, &'a Ident);

struct ComponentProperty<'a, T> {
    component_part: &'a ComponentPart<'a>,
    comments: Option<&'a CommentAttributes>,
    attrs: Option<&'a ComponentAttr<T>>,
}

impl<'a, T: Sized + ToTokens> ComponentProperty<'a, T> {
    fn new(
        component_part: &'a ComponentPart<'a>,
        comments: Option<&'a CommentAttributes>,
        attrs: Option<&'a ComponentAttr<T>>,
    ) -> Self {
        Self {
            component_part,
            comments,
            attrs,
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
                    utoipa::openapi::Object::new()
                });

                if let Some(description) = self.comments.and_then(|attributes| attributes.0.first())
                {
                    tokens.extend(quote! {
                        .with_description(#description)
                    })
                }
            }
            Some(GenericType::Vec) => {
                let component_property = ComponentProperty::new(
                    self.component_part.child.as_ref().unwrap(),
                    self.comments,
                    self.attrs,
                );

                tokens.extend(quote! {
                    #component_property.to_array()
                });
            }
            Some(GenericType::Option) => {
                let component_property = ComponentProperty::new(
                    self.component_part.child.as_ref().unwrap(),
                    self.comments,
                    self.attrs,
                );

                tokens.extend(component_property.into_token_stream())
            }
            None => match self.component_part.value_type {
                ValueType::Primitive => {
                    let component_type = ComponentType(self.component_part.ident);

                    tokens.extend(quote! {
                        utoipa::openapi::Property::new(#component_type)
                    });

                    let format = ComponentFormat(self.component_part.ident);
                    if format.is_known_format() {
                        tokens.extend(quote! {
                            .with_format(#format)
                        })
                    }

                    if let Some(description) =
                        self.comments.and_then(|attributes| attributes.0.first())
                    {
                        tokens.extend(quote! {
                            .with_description(#description)
                        })
                    }

                    if let Some(attributes) = self.attrs {
                        tokens.extend(attributes.to_token_stream())
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
