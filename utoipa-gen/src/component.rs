use std::ops::Deref;

use proc_macro2::{Ident, TokenStream as TokenStream2};
use proc_macro_error::{abort, abort_call_site};
use quote::quote;
use syn::{Fields, FieldsNamed, GenericArgument, PathArguments, PathSegment, Type, TypePath};

pub fn impl_component(data: syn::Data) -> TokenStream2 {
    let mut component = quote! {
        utoipa::openapi::Object::new()
    };

    match data {
        syn::Data::Struct(content) => {
            if let Fields::Named(named_fields) = content.fields {
                let FieldsNamed { named, .. } = named_fields;

                named.iter().for_each(|field| {
                    let name = &*field.ident.as_ref().unwrap().to_string();
                    let component_type = get_component_type(&field.ty);

                    // let is_primitive = is_primitive_type(type_ident);

                    println!("ident: {:?}, name: {}", component_type, name);

                    // TODO if type value is not primitive create ref component
                    // TODO if type is vec then add array component
                    // TODO if type is map add empty object component

                    let component_quote = match component_type {
                        FieldType::Generic(ident, value_type) => {
                            match value_type {
                                ValueType::Object => {
                                    quote! {
                                        utoipa::openapi::Array::new(utoipa::openapi::Ref::from_component_name(#ident.to_string()))
                                    }
                                },
                                ValueType::Primitive => {
                                    // TODO resolve component_type and properties
                                     quote! {
                                        utoipa::openapi::Array::new(
                                            utoipa::openapi::Property::new(
                                                ComponentType::Integer
                                                // Some(ComponentFormat::Int32),
                                                // Some("1"),
                                                // Some("Id of credential"),
                                                // None,
                                            )
                                        )
                                    }
                                }
                            }
                            // let component_type_quote = quote! {
                            //     utoipa::openapi::ComponentType::Array
                            // };

                            // // if this is array of strings? or other primitive type?
                            // // if this is array of object types??
                            // // if this is map of objects?? currently not supported at all

                            // // ArrayComponent::new(RefComponent::from_component_name("component name")) = generic object
                            // // ArrayComponent::new(Component::new(ComponentType::String, ....)) = generic primitive

                            // component_type_quote
                        }
                        FieldType::Primitive(ident) => {
                            let primitive_name = &*ident.to_string();
                            let component_type_quote = quote! {
                                match #primitive_name {
                                    "String" | "str" | "char" => utoipa::openapi::ComponentType::String,
                                    "bool" => utoipa::openapi::ComponentType::Boolean,
                                    "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => utoipa::openapi::ComponentType::Integer,
                                    "f32" | "f64"  => utoipa::openapi::ComponentType::Number,
                                    _ => utoipa::openapi::ComponentType::Object // TODO is this object for sure???
                                }
                            };

                            // TODO resolve primitive component type properties

                            quote! {
                                utoipa::openapi::Property::new(
                                    #component_type_quote
                                    // Some(ComponentFormat::Int32),
                                    // Some("1"),
                                    // Some("Id of credential"),
                                    // None,
                                )
                            }
                        }
                        FieldType::Struct(ident) => {
                            // RefComponent::from_component_name("component name")
                            // if this is
                            // RefComponent::from_component_name("type name").into()

                            quote! {
                                utoipa::openapi::Ref::from_component_name(#ident.to_string())
                            }
                        }
                    };

                    let field_quote = quote! {
                        .with_property(#name, #component_quote)
                    };

                    component.extend(field_quote)
                });
            }
        }
        syn::Data::Enum(content) => (), // TODO implement enum types
        _ => (),                        // throw error here if another type of data
    }

    quote! {
        use utoipa::openapi::{ComponentType, ComponentFormat};

        #component.into()
    }

    // quote! {
    //     #component
    //         .with_property(
    //         "id",
    //         utoipa::openapi::Component::new(
    //             utoipa::openapi::ComponentType::Integer,
    //             Some(utoipa::openapi::ComponentFormat::Int32),
    //             Some("1"), // resolve default value
    //             Some("Id of credential"), // resolve description
    //             None, // Resolve enum values
    //         )
    //     ).into()
    // }
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

fn get_component_type(ty: &Type) -> FieldType<'_> {
    get_component_type_from_path(get_type_path(ty), non_generic_component_type, |path| {
        let segment = get_segment(path);

        if segment.arguments.is_empty() {
            abort!(
                segment.ident.span(),
                "Expected at least one angle bracket argument but was 0"
            );
        };

        get_component_type_from_path(
            get_type_path(get_first_generic_type(segment)),
            generic_component_type,
            |type_path| {
                abort!(
                    get_segment(type_path).ident.span(),
                    "Is not object or primitive type, cannot resolve ident"
                )
            },
        )
    })
}

fn non_generic_component_type(ident: &Ident) -> FieldType {
    if is_primitive_type(ident) {
        FieldType::Primitive(ident)
    } else {
        FieldType::Struct(ident)
    }
}

fn generic_component_type(ident: &Ident) -> FieldType {
    if is_primitive_type(ident) {
        FieldType::Generic(ident, ValueType::Primitive)
    } else {
        FieldType::Generic(ident, ValueType::Object)
    }
}

fn get_component_type_from_path<'a>(
    type_path: &'a TypePath,
    op: fn(&'a Ident) -> FieldType<'a>,
    or_else: impl Fn(&'a TypePath) -> FieldType<'a>,
) -> FieldType<'a> {
    type_path
        .path
        .get_ident()
        .map(op)
        .unwrap_or_else(|| or_else(type_path))
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
enum FieldType<'a> {
    Generic(&'a Ident, ValueType),
    Primitive(&'a Ident),
    Struct(&'a Ident),
}

impl<'a> Deref for FieldType<'a> {
    type Target = Ident;

    fn deref(&self) -> &Self::Target {
        match *self {
            Self::Generic(ident, ..) | Self::Primitive(ident) | Self::Struct(ident) => ident,
        }
    }
}

#[derive(Debug)]
enum ValueType {
    Primitive,
    Object,
}
