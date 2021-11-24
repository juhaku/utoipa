use std::{ops::Deref, rc::Rc};

use proc_macro2::{Group, Ident, Punct, TokenStream as TokenStream2};
use proc_macro_error::{abort, abort_call_site, emit_error};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    punctuated::Punctuated, Fields, FieldsNamed, GenericArgument, PathArguments, PathSegment, Type,
    TypePath,
};

pub fn impl_component(data: syn::Data) -> TokenStream2 {
    let component = match ComponentProperties::new(data) {
        ComponentProperties::Fields(fields) => fields
            .iter()
            .map(|field| {
                (
                    ComponentType::from_type_path(get_type_path(&field.ty)),
                    field.ident.as_ref().unwrap().to_string(),
                )
            })
            .fold(
                quote! { utoipa::openapi::Object::new() },
                |mut object_token_stream, (component_type, field_name)| {
                    object_token_stream.extend(append_property(&component_type, &*field_name));

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

            quote! {
                utoipa::openapi::Property::new(ComponentType::String)
                    // .with_default("Active")
                    // .with_description("Credential status")
                    .with_enum_values(#enum_values)
            }
        }
    };

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

fn append_property(component_type: &ComponentType, field_name: &str) -> TokenStream2 {
    let component = Into::<ComponentTypeRef<'_, ComponentType<'_>>>::into(component_type)
        .collect::<Component>();

    println!("Got component: {:?}", component);

    match component {
        Component {
            option,
            generic_type: None,
            value_type:
                Some(type_tuple @ TypeTuple(ValueType::Primitive | ValueType::Object, ident)),
        } => {
            let mut property = match type_tuple.0 {
                ValueType::Primitive => {
                    let component_type_quote = resolve_primitive_type(ident);
                    // TODO resolve other properties

                    quote! {
                        .with_property(#field_name,
                            utoipa::openapi::Property::new(
                                #component_type_quote
                            )
                        )
                    }
                }
                ValueType::Object => {
                    let object_name = &*ident.to_string();

                    quote! {
                        .with_property(#field_name, utoipa::openapi::Ref::from_component_name(#object_name))
                    }
                }
            };
            if !option {
                property.extend(quote! {
                    .with_required(#field_name)
                })
            }

            property
        }
        Component {
            option,
            generic_type: Some(generic_type_tuple),
            value_type: Some(value_type_tupple),
        } => {
            let mut property = match generic_type_tuple.0 {
                GenericType::Map => quote! {
                    .with_property(#field_name, utoipa::openapi::Object::new())
                },
                GenericType::Vec => {
                    let property = match value_type_tupple.0 {
                        ValueType::Object => {
                            let value_name = &*value_type_tupple.1;

                            quote! {
                                utoipa::openapi::Ref::from_component_name(#value_name)
                            }
                        }
                        ValueType::Primitive => {
                            let item_type = resolve_primitive_type(value_type_tupple.1);

                            quote! {
                                utoipa::openapi::Property::new(
                                    #item_type
                                )
                            }
                        }
                    };

                    quote! {
                        .with_property(#field_name,
                            utoipa::openapi::Array::new(
                                #property
                            )
                        )
                    }
                }
                _ => unreachable!(), //  we do not have option type here
            };

            if !option {
                property.extend(quote! {
                    .with_required(#field_name)
                })
            }

            property
        }
        _ => unreachable!(),
    }
}

fn resolve_primitive_type(ident: &Ident) -> TokenStream2 {
    let primitive_name = &*ident.to_string();
    quote! {
        match #primitive_name {
            "String" | "str" | "char" => utoipa::openapi::ComponentType::String,
            "bool" => utoipa::openapi::ComponentType::Boolean,
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => utoipa::openapi::ComponentType::Integer,
            "f32" | "f64"  => utoipa::openapi::ComponentType::Number,
            _ => utoipa::openapi::ComponentType::Object // TODO is this object for sure???
        }
    }
}

fn is_primitive_type(ident: &Ident) -> bool {
    let name = &*ident.to_string();

    matches!(
        name,
        "String"
            | "str"
            | "&str"
            | "char"
            | "&char"
            | "bool"
            | "usize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "isize"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "f32"
            | "f64"
    )
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
struct ComponentType<'a> {
    ident: &'a Ident,
    value_type: ValueType,
    generic_type: Option<GenericType>,
    child: Option<Rc<ComponentType<'a>>>,
}

impl<'a> ComponentType<'a> {
    fn from_type_path(type_path: &'a TypePath) -> ComponentType<'a> {
        ComponentType::from_type_path_(
            type_path,
            ComponentType::convert,
            ComponentType::resolve_component_type,
        )
    }

    fn from_type_path_(
        type_path: &'a TypePath,
        op: impl Fn(&'a Ident, &'a PathSegment) -> ComponentType<'a>,
        or_else: impl Fn(&'a PathSegment) -> ComponentType<'a>,
    ) -> ComponentType<'a> {
        let segment = get_segment(type_path);

        type_path
            .path
            .get_ident()
            .map(|ident| op(ident, segment))
            .unwrap_or_else(|| or_else(segment))
    }

    fn resolve_component_type(segment: &'a PathSegment) -> ComponentType<'a> {
        if segment.arguments.is_empty() {
            abort!(
                segment.ident.span(),
                "Expected at least one angle bracket argument but was 0"
            );
        };

        println!("got segment: {:#?}", segment);

        let mut generic_component_type = ComponentType::convert(&segment.ident, segment);

        generic_component_type.child = Some(Rc::new(ComponentType::from_type_path(get_type_path(
            get_first_generic_type(segment),
        ))));

        generic_component_type
    }

    fn convert(ident: &'a Ident, segment: &PathSegment) -> ComponentType<'a> {
        let generic_type = ComponentType::get_generic(segment);

        println!(
            "converting ident: {:?} to generic type: {:?}",
            ident, generic_type
        );

        Self {
            ident,
            value_type: if is_primitive_type(ident) {
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

struct ComponentTypeRef<'a, T> {
    _inner: Option<&'a T>,
}

impl<'a> Deref for ComponentTypeRef<'a, ComponentType<'a>> {
    type Target = ComponentType<'a>;

    fn deref(&self) -> &Self::Target {
        self._inner.unwrap() // we can unwrap since it must have value
    }
}

impl<'a> From<&'a ComponentType<'a>> for ComponentTypeRef<'a, ComponentType<'a>> {
    fn from(component_type: &'a ComponentType<'_>) -> Self {
        Self {
            _inner: Some(component_type),
        }
    }
}

impl<'a> Iterator for ComponentTypeRef<'a, ComponentType<'a>> {
    type Item = ComponentTypeRef<'a, ComponentType<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self._inner;
        let next = current.and_then(|current| current.child.as_ref());

        if let Some(component) = next {
            self._inner = Some(component.as_ref());
        } else {
            self._inner = None
        }

        current.map(|component_type| ComponentTypeRef {
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

impl<'a> FromIterator<ComponentTypeRef<'a, ComponentType<'a>>> for Component<'a> {
    fn from_iter<T: IntoIterator<Item = ComponentTypeRef<'a, ComponentType<'a>>>>(iter: T) -> Self {
        let components_iter = iter.into_iter();
        components_iter.fold(Self::default(), |mut acc, item| {
            match item.generic_type {
                Some(GenericType::Option) => acc.option = true,
                Some(generic_type @ GenericType::Map | generic_type @ GenericType::Vec) => {
                    acc.generic_type = Some(TypeTuple(generic_type, item.ident))
                }
                None => (),
            }

            // we are only interested of final concrete value type
            match item.value_type {
                value_type @ ValueType::Object | value_type @ ValueType::Primitive
                    if item.generic_type == None =>
                {
                    acc.value_type = Some(TypeTuple(value_type, item.ident))
                }
                _ => (),
            }

            acc
        })
    }
}
