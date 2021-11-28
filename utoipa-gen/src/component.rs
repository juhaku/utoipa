use std::{ops::Deref, rc::Rc};

use proc_macro2::{Group, Ident, Punct, TokenStream as TokenStream2};
use proc_macro_error::{abort, abort_call_site, emit_error};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    punctuated::Punctuated, Fields, FieldsNamed, GenericArgument, PathArguments, PathSegment, Type,
    TypePath,
};

use crate::{
    attribute::{parse_component_attribute, AttributeType, CommentAttributes, ComponentAttribute},
    component_type::{ComponentFormat, ComponentType},
};

pub fn impl_component(data: syn::Data, attrs: Vec<syn::Attribute>) -> TokenStream2 {
    // println!("Got data: {:#?}", data);
    // println!("Got attributes: {:#?}", attrs);

    let mut component = match ComponentProperties::new(data) {
        ComponentProperties::Fields(fields) => fields.iter().fold(
            quote! { utoipa::openapi::Object::new() },
            |mut object_token_stream, field| {
                let field_name = &*field.ident.as_ref().unwrap().to_string();
                let component_part = &ComponentPart::from_type_path(get_type_path(&field.ty));
                let component =
                    Into::<ComponentPartRef<'_, ComponentPart<'_>>>::into(component_part)
                        .collect::<Component>();
                let component_attribute = parse_component_attribute(&field.attrs);

                // println!("Got component attribute: {:#?}", component_attribute);

                object_token_stream.extend(append_property(
                    &component,
                    field_name,
                    CommentAttributes::from_attributes(&field.attrs),
                    component_attribute,
                ));

                object_token_stream
            },
        ),
        ComponentProperties::Variants(variants) => {
            variants.iter().filter(|variant| !matches!(variant.fields, Fields::Unit))
                .for_each(|unsupported_variant| {
                    emit_error!(unsupported_variant.ident.span(), "Currently unsupported enum variant, expected Unit variant without additional fields")
                });

            let enum_values = &variants
                .iter()
                .filter(|variant| matches!(variant.fields, Fields::Unit))
                .map(|variant| variant.ident.to_string())
                .collect::<EnumValues>();

            let mut enum_stream = quote! {
                utoipa::openapi::Property::new(ComponentType::String)
                    // .with_default("Active")
                    .with_enum_values(#enum_values)
            };

            if let Some(enum_attributes) = parse_component_attribute(&attrs) {
                append_attributes(
                    &mut enum_stream,
                    enum_attributes
                        .into_iter()
                        .filter(|attribute| !matches!(attribute, AttributeType::Format(..))),
                )
            };

            enum_stream
        }
    };

    if let Some(comment) = CommentAttributes::from_attributes(&attrs).0.first() {
        component.extend(quote! {
            .with_description(#comment)
        })
    }

    quote! {
        use utoipa::openapi::{ComponentType, ComponentFormat};

        #component.into()
    }
}

#[derive(Debug)]
enum ComponentProperties {
    Fields(Vec<syn::Field>),
    Variants(Vec<syn::Variant>),
}

impl ComponentProperties {
    fn new(data: syn::Data) -> ComponentProperties {
        match data {
            syn::Data::Struct(content) => {
                if let Fields::Named(named_fields) = content.fields {
                    let FieldsNamed { named, .. } = named_fields;

                    ComponentProperties::Fields(named.into_iter().collect())
                } else {
                    ComponentProperties::Fields(vec![])
                }
            }
            syn::Data::Enum(content) => {
                ComponentProperties::Variants(content.variants.into_iter().collect())
            }
            _ => abort_call_site!(
                "Unexpected data type, expected syn::Data::Struct or syn::Data::Enum"
            ),
        }
    }
}

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

fn append_property(
    component: &Component,
    field_name: &str,
    comment_attributes: CommentAttributes,
    component_attribute: Option<ComponentAttribute>,
) -> TokenStream2 {
    let mut property = match component {
        Component {
            generic_type: None,
            value_type:
                Some(type_tuple @ TypeTuple(ValueType::Primitive | ValueType::Object, ident)),
            ..
        } => resolve_simple_property(type_tuple, ident, &comment_attributes),
        Component {
            generic_type: Some(generic_type_tuple),
            value_type: Some(value_type_tuple),
            ..
        } => resolve_complex_property(generic_type_tuple, value_type_tuple, &comment_attributes),
        _ => unreachable!(),
    };

    if let Some(component_attribute) = component_attribute {
        append_attributes(&mut property, component_attribute.into_iter())
    }

    let mut object = quote! {
        .with_property(#field_name, #property)
    };

    if !component.option {
        object.extend(quote! {
            .with_required(#field_name)
        })
    }

    object
}

fn append_attributes<I: Iterator<Item = AttributeType>>(
    token_stream: &mut TokenStream2,
    component_attribute: I,
) {
    component_attribute
        .into_iter()
        .for_each(|attribute_type| match attribute_type {
            AttributeType::Default(..) => token_stream.extend(quote! {
                .with_default(#attribute_type)
            }),
            AttributeType::Example(..) => token_stream.extend(quote! {
                .with_example(#attribute_type)
            }),
            AttributeType::Format(..) => token_stream.extend(quote! {
                .with_format(#attribute_type)
            }),
        })
}

fn resolve_simple_property(
    type_tuple: &TypeTuple<ValueType>,
    ident: &Ident,
    comment_attributes: &CommentAttributes,
) -> TokenStream2 {
    match type_tuple.0 {
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

fn resolve_complex_property(
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
                resolve_simple_property(value_type_tuple, value_type_tuple.1, comment_attributes);

            quote! {
                utoipa::openapi::Array::new(
                    #property
                )
            }
        }
        _ => unreachable!(), //  we do not have option type here
    }
}

fn get_type_path(ty: &Type) -> &TypePath {
    match ty {
        Type::Path(path) => path,
        _ => abort_call_site!("Unexpected type, expected Type::Path"),
    }
}

fn get_segment(type_path: &TypePath) -> &PathSegment {
    type_path.path.segments.first().unwrap()
}

fn get_first_generic_type(segment: &PathSegment) -> &Type {
    match &segment.arguments {
        PathArguments::AngleBracketed(angle_bracketed_args) => {
            let first_arg = angle_bracketed_args.args.first().unwrap();

            match first_arg {
                GenericArgument::Type(generic_type) => generic_type,
                _ => abort!(
                    segment.ident.span(),
                    "Expected GenericArgument::Type, encountered unexpected type"
                ),
            }
        }
        _ => abort!(
            segment.ident.span(),
            "Unexpected argument type, expected PathArgument::AngleBraketed, found non generic type"
        ),
    }
}

#[derive(Debug)]
struct ComponentPart<'a> {
    ident: &'a Ident,
    value_type: ValueType,
    generic_type: Option<GenericType>,
    child: Option<Rc<ComponentPart<'a>>>,
}

impl<'a> ComponentPart<'a> {
    fn from_type_path(type_path: &'a TypePath) -> ComponentPart<'a> {
        ComponentPart::_from_type_path(
            type_path,
            ComponentPart::convert,
            ComponentPart::resolve_component_type,
        )
    }

    fn _from_type_path(
        type_path: &'a TypePath,
        op: impl Fn(&'a Ident, &'a PathSegment) -> ComponentPart<'a>,
        or_else: impl Fn(&'a PathSegment) -> ComponentPart<'a>,
    ) -> ComponentPart<'a> {
        let segment = get_segment(type_path);

        type_path
            .path
            .get_ident()
            .map(|ident| op(ident, segment))
            .unwrap_or_else(|| or_else(segment))
    }

    fn resolve_component_type(segment: &'a PathSegment) -> ComponentPart<'a> {
        if segment.arguments.is_empty() {
            abort!(
                segment.ident.span(),
                "Expected at least one angle bracket argument but was 0"
            );
        };

        let mut generic_component_type = ComponentPart::convert(&segment.ident, segment);

        generic_component_type.child = Some(Rc::new(ComponentPart::from_type_path(get_type_path(
            get_first_generic_type(segment),
        ))));

        generic_component_type
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

#[derive(Debug, Clone, Copy)]
enum ValueType {
    Primitive,
    Object,
    // Enum
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum GenericType {
    Vec,
    Map,
    Option,
}

#[derive(Debug)]
struct TypeTuple<'a, T>(T, &'a Ident);

#[derive(Debug, Default)]
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
