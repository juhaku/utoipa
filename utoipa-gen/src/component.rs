use std::{ops::Deref, rc::Rc};

use proc_macro2::{Group, Ident, Punct, TokenStream as TokenStream2};
use proc_macro_error::{abort, abort_call_site, emit_error};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    punctuated::Punctuated, Attribute, Fields, FieldsNamed, GenericArgument, PathArguments,
    PathSegment, Type, TypePath, Variant,
};

use crate::{
    attribute::{parse_component_attribute, AttributeType, CommentAttributes, ComponentAttribute},
    component_type::{ComponentFormat, ComponentType},
};

pub(crate) fn impl_component(data: syn::Data, attrs: Vec<syn::Attribute>) -> TokenStream2 {
    let component = ComponentVariant::new(data, &attrs);

    quote! {
        use utoipa::openapi::{ComponentType, ComponentFormat};

        #component.into()
    }
}

#[cfg_attr(feature = "all-features", derive(Debug))]
/// Holds the openapi Component implementation which can be added the Schema.
enum ComponentVariant<'a> {
    /// Object variant is rust sturct with Component derive annotation.
    Object(Vec<syn::Field>, &'a [Attribute]),
    /// Enum variant is rust enum with Component derive annotation. **Only supports** enums with
    /// Unit type fields.
    Enum(Vec<syn::Variant>, &'a [Attribute]),
}

impl<'a> ComponentVariant<'a> {
    fn new(data: syn::Data, attributes: &'a [Attribute]) -> ComponentVariant<'a> {
        match data {
            syn::Data::Struct(content) => {
                if let Fields::Named(named_fields) = content.fields {
                    let FieldsNamed { named, .. } = named_fields;

                    ComponentVariant::Object(named.into_iter().collect(), attributes)
                } else {
                    ComponentVariant::Object(vec![], attributes)
                }
            }
            syn::Data::Enum(content) => {
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
        match self {
            Self::Object(fields, attrs) => self.struct_to_tokens(fields, *attrs, tokens),
            Self::Enum(variants, attrs) => self.enum_to_tokens(variants, *attrs, tokens),
        };
    }
}

impl<'a> ComponentVariant<'a> {
    fn struct_to_tokens(
        &self,
        fields: &[syn::Field],
        attributes: &[Attribute],
        tokens: &mut TokenStream2,
    ) {
        tokens.extend(quote! { utoipa::openapi::Object::new() });

        fields.iter().for_each(|field| {
            let field_name = &*field.ident.as_ref().unwrap().to_string();
            let component_part = &ComponentPart::from_type(&field.ty);
            let component = Into::<ComponentPartRef<'_, ComponentPart<'_>>>::into(component_part)
                .collect::<Component>();

            let property = create_property_stream(
                &component,
                CommentAttributes::from_attributes(&field.attrs),
                parse_component_attribute(&field.attrs),
            );

            tokens.extend(quote! {
                .with_property(#field_name, #property)
            });

            if !component.option {
                tokens.extend(quote! {
                    .with_required(#field_name)
                })
            }
        });

        self.append_description(attributes, tokens);
    }

    fn is_not_enum_unit_variant(variant: &&Variant) -> bool {
        !matches!(variant.fields, Fields::Unit)
    }

    fn enum_to_tokens(
        &self,
        variants: &[Variant],
        attributes: &[Attribute],
        tokens: &mut TokenStream2,
    ) {
        variants
            .iter()
            .filter(ComponentVariant::is_not_enum_unit_variant)
            .for_each(|variant| emit_error!(variant.ident.span(), "Currently unsupported enum variant, expected Unit variant without additional fields"));

        let enum_values = &variants
            .iter()
            .filter(|variant| matches!(variant.fields, Fields::Unit))
            .map(|variant| variant.ident.to_string())
            .collect::<EnumValues>();

        tokens.extend(quote! {
            utoipa::openapi::Property::new(ComponentType::String)
                .with_enum_values(#enum_values)
        });

        if let Some(enum_attributes) = parse_component_attribute(attributes) {
            append_attributes(
                tokens,
                enum_attributes
                    .into_iter()
                    .filter(|attribute| !matches!(attribute, AttributeType::Format(..))),
            )
        };

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

/// Tokenizes slice reference (`&[...]`) correctly to OpenAPI JSON.
struct EnumValues(Vec<String>);

impl FromIterator<String> for EnumValues {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self {
            0: iter.into_iter().collect::<Vec<_>>(),
        }
    }
}

impl ToTokens for EnumValues {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.append(Punct::new('&', proc_macro2::Spacing::Joint));
        let items = self
            .0
            .iter()
            .fold(Punctuated::new(), |mut punctuated, item| {
                punctuated.push_value(item);
                punctuated.push_punct(Punct::new(',', proc_macro2::Spacing::Alone));

                punctuated
            });

        tokens.append(Group::new(
            proc_macro2::Delimiter::Bracket,
            items.to_token_stream(),
        ));
    }
}

fn append_attributes<I: Iterator<Item = AttributeType>>(
    token_stream: &mut TokenStream2,
    component_attribute: I,
) {
    component_attribute
        .into_iter()
        .map(|attribute_type| match attribute_type {
            AttributeType::Default(..) => quote! {
                .with_default(#attribute_type)
            },
            AttributeType::Example(..) => quote! {
                .with_example(#attribute_type)
            },
            AttributeType::Format(..) => quote! {
                .with_format(#attribute_type)
            },
        })
        .for_each(|stream| token_stream.extend(stream))
}

fn create_property_stream(
    component: &Component,
    comment_attributes: CommentAttributes,
    component_attribute: Option<ComponentAttribute>,
) -> TokenStream2 {
    let mut property = match component {
        Component {
            generic_type: None,
            value_type:
                Some(TypeTuple(
                    value_type @ ValueType::Primitive | value_type @ ValueType::Object,
                    ident,
                )),
            ..
        } => create_simple_property(value_type, ident, &comment_attributes),
        Component {
            generic_type: Some(generic_type_tuple),
            value_type: Some(value_type_tuple),
            ..
        } => create_complex_property(generic_type_tuple, value_type_tuple, &comment_attributes),
        _ => unreachable!(), // will never occur, there are only complex generic types or simple types with or without generics
    };

    if let Some(component_attribute) = component_attribute {
        append_attributes(&mut property, component_attribute.into_iter())
    }

    property
}

fn create_simple_property(
    value_type: &ValueType,
    ident: &Ident,
    comment_attributes: &CommentAttributes,
) -> TokenStream2 {
    match value_type {
        ValueType::Primitive => {
            let component_type = ComponentType(ident);

            let mut property = quote! {
                utoipa::openapi::Property::new(
                    #component_type
                )
            };

            if let Some(comment) = comment_attributes.0.first() {
                property.extend(quote! {
                    .with_description(#comment)
                })
            }

            let format = ComponentFormat(ident);
            if format.is_known_format() {
                property.extend(quote! {
                    .with_format(#format)
                })
            }

            property
        }
        ValueType::Object => {
            let object_name = &*ident.to_string();

            quote! {
                utoipa::openapi::Ref::from_component_name(#object_name)
            }
        }
    }
}

fn create_complex_property(
    generic_type_tuple: &TypeTuple<GenericType>,
    value_type_tuple: &TypeTuple<ValueType>,
    comment_attributes: &CommentAttributes,
) -> TokenStream2 {
    match generic_type_tuple.0 {
        GenericType::Map => {
            let mut property = quote! {
                utoipa::openapi::Object::new()
            };

            if let Some(comment) = comment_attributes.0.first() {
                property.extend(quote! {
                    .with_description(#comment)
                })
            }

            property
        }
        GenericType::Vec => {
            let property =
                create_simple_property(&value_type_tuple.0, value_type_tuple.1, comment_attributes);

            quote! {
                utoipa::openapi::Array::new(
                    #property
                )
            }
        }
        _ => unreachable!(), //  we do not have option type here
    }
}

#[cfg_attr(feature = "all-features", derive(Debug))]
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

struct ComponentPartRef<'a, T> {
    _inner: Option<&'a T>,
}

impl<'a> Deref for ComponentPartRef<'a, ComponentPart<'a>> {
    type Target = ComponentPart<'a>;

    fn deref(&self) -> &Self::Target {
        self._inner.unwrap() // we can unwrap since it must have value
    }
}

impl<'a> From<&'a ComponentPart<'a>> for ComponentPartRef<'a, ComponentPart<'a>> {
    fn from(component_type: &'a ComponentPart<'_>) -> Self {
        Self {
            _inner: Some(component_type),
        }
    }
}

impl<'a> Iterator for ComponentPartRef<'a, ComponentPart<'a>> {
    type Item = ComponentPartRef<'a, ComponentPart<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self._inner;
        let next = current.and_then(|current| current.child.as_ref());

        if let Some(component) = next {
            self._inner = Some(component.as_ref());
        } else {
            self._inner = None
        }

        current.map(|component_type| ComponentPartRef {
            _inner: Some(component_type),
        })
    }
}

#[cfg_attr(feature = "all-features", derive(Debug))]
#[derive(Clone, Copy)]
enum ValueType {
    Primitive,
    Object,
}

#[cfg_attr(feature = "all-features", derive(Debug))]
#[derive(PartialEq, Clone, Copy)]
enum GenericType {
    Vec,
    Map,
    Option,
}

#[cfg_attr(feature = "all-features", derive(Debug))]
struct TypeTuple<'a, T>(T, &'a Ident);

#[cfg_attr(feature = "all-features", derive(Debug))]
#[derive(Default)]
struct Component<'a> {
    option: bool,
    generic_type: Option<TypeTuple<'a, GenericType>>,
    value_type: Option<TypeTuple<'a, ValueType>>,
}

impl<'a> FromIterator<ComponentPartRef<'a, ComponentPart<'a>>> for Component<'a> {
    fn from_iter<T: IntoIterator<Item = ComponentPartRef<'a, ComponentPart<'a>>>>(iter: T) -> Self {
        let components_iter = iter.into_iter();
        components_iter.fold(Self::default(), |mut component, item| {
            match item.generic_type {
                Some(GenericType::Option) => component.option = true,
                Some(generic_type @ GenericType::Map | generic_type @ GenericType::Vec) => {
                    component.generic_type = Some(TypeTuple(generic_type, item.ident))
                }
                None => (),
            }

            // we are only interested of final concrete value type
            match item.value_type {
                value_type @ ValueType::Object | value_type @ ValueType::Primitive
                    if item.generic_type == None =>
                {
                    component.value_type = Some(TypeTuple(value_type, item.ident))
                }
                _ => (),
            }

            component
        })
    }
}
