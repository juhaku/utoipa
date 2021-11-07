use std::{rc::Rc, vec::IntoIter};

use proc_macro2::{Ident, TokenStream as TokenStream2};
use proc_macro_error::{abort, abort_call_site};
use quote::quote;
use syn::{Fields, FieldsNamed, GenericArgument, PathArguments, PathSegment, Type, TypePath};

pub fn impl_component(data: syn::Data) -> TokenStream2 {
    let component = get_fields(data)
        .iter()
        .map(|field| {
            (
                // get_component_type(get_type_path(&field.ty)),
                ComponentType::from_type_path(get_type_path(&field.ty)),
                field.ident.as_ref().unwrap().to_string(),
            )
        })
        .fold(
            quote! { utoipa::openapi::Object::new() },
            |mut acc, (field_type, name)| {
                append_tokens(&mut acc, &field_type, &*name);
                // match field_type {
                //     FieldType::Generic(ident, value_type, generic_type) => {
                //         acc.extend(object_append_generic_type(
                //             ident,
                //             &value_type,
                //             &generic_type,
                //             &name,
                //         ));
                //     }
                //     FieldType::Primitive(ident) => {
                //         acc.extend(object_append_primitive_type(ident, &name))
                //     }
                //     FieldType::Object(ident) => {
                //         let object_name = &*ident.to_string();
                //         acc.extend(quote! {
                //             .with_property(#name, utoipa::openapi::Ref::from_component_name(#object_name))
                //             .with_required(#name)
                //         })
                //     }
                // }
                acc
            },
        );

    quote! {
        use utoipa::openapi::{ComponentType, ComponentFormat};

        #component.into()
    }
}

fn append_tokens(
    token_stream: &mut TokenStream2,
    component_type: &ComponentType,
    field_name: &str,
) {
    // let ComponentType {
    //     ident,
    //     optional,
    //     value_type,
    //     generic_type,
    //     child,
    // } = component_type;

    println!(
        "component type: name: {:?}, type: {:#?}",
        field_name, component_type
    );

    match component_type {
        ComponentType {
            child: None,
            value_type: ValueType::Primitive,
            generic_type: None,
            ident,
        } => token_stream.extend(object_append_primitive_type(ident, field_name)),
        ComponentType {
            child: None,
            value_type: ValueType::Object,
            generic_type: None,
            ident,
        } => {
            let object_name = &*ident.to_string();

            token_stream.extend(quote! {
                .with_property(#field_name, utoipa::openapi::Ref::from_component_name(#object_name))
                .with_required(#field_name)
            })
        }
        ComponentType {
            child,
            value_type,
            ident,
            generic_type: Some(generic_type),
        } => token_stream.extend(object_append_generic_type(
            ident,
            value_type,
            generic_type,
            field_name,
            child,
        )),
        // ComponentType {
        //     child: None,
        //     generic_type: Some(generic_type @ GenericType::Option),
        //     ident,
        //     value_type,
        // } => token_stream.extend(object_append_generic_type(
        //     ident,
        //     value_type,
        //     generic_type,
        //     &field_name,
        // )),
        _ => (),
    }
}

fn get_fields(data: syn::Data) -> Vec<syn::Field> {
    match data {
        syn::Data::Struct(content) => {
            if let Fields::Named(named_fields) = content.fields {
                let FieldsNamed { named, .. } = named_fields;

                named.into_iter().collect::<Vec<_>>()
            } else {
                vec![]
            }
        }
        syn::Data::Enum(content) => vec![], // TODO implement enum types
        _ => vec![],                        // throw error here if another type of data
    }
}

fn append_generic_type(component_type: &ComponentType) -> TokenStream2 {
    // TODO

    component_type
        .to_iter()
        .map(|component_type| {
            let cp = &*component_type;

            // TODO make sure that each map iteration will append to the same quote token stream for one type

            match cp {
                ComponentType {
                    ident,
                    value_type,
                    generic_type,
                    child,
                } => (),
                _ => (),
            };

            String::new()
        })
        .collect::<String>();

    quote! {}
}

fn object_append_generic_type(
    ident: &Ident,
    value_type: &ValueType,
    generic_type: &GenericType,
    name: &str,
    child: &Option<Rc<ComponentType>>,
) -> TokenStream2 {
    // TODO get flat type somehow??????
    match generic_type {
        GenericType::Map => {
            quote! {
                .with_property(#name.to_string(), utoipa::openapi::Object::new())
                .with_required(#name.to_string())
            }
        }
        GenericType::Vec => {
            match value_type {
                ValueType::Object => {
                    quote! {
                        .with_property(#name.to_string(), utoipa::openapi::Ref::from_component_name(#ident.to_string()))
                        .with_required(#name.to_string())
                    }
                }
                ValueType::Primitive => {
                    let component_type = resolve_primitive_type(ident);
                    // TODO resolve properties
                    quote! {
                        .with_property(#name.to_string(),
                            utoipa::openapi::Array::new(
                                utoipa::openapi::Property::new(
                                    #component_type
                                )
                            )
                        )
                        .with_required(#name.to_string())
                    }
                }
            }
        }
        GenericType::Option => {
            // TODO if option is generic??? currently unabled to recognize fields suchs as Option<Vec<String>> such as double generics!
            match value_type {
                ValueType::Object => {
                    quote! {
                        .with_property(#name.to_string(), utoipa::openapi::Ref::from_component_name(#ident.to_string()))
                    }
                }
                ValueType::Primitive => {
                    // TODO resolve properties
                    let component_type = resolve_primitive_type(ident);

                    quote! {
                        .with_property(#name.to_string(),
                            utoipa::openapi::Property::new(
                                #component_type
                            )
                            // .with_format(ComponentFormat::Int32)
                            // .with_description("Id of credential")
                            // .with_default("1")
                            // .with_default("Active")
                            // .with_description("Credential status")
                            // .with_enum_values(&["Active", "NotActive", "Locked", "Expired"]),
                        )
                    }
                }
            }
        }
    }
}

fn object_append_primitive_type(ident: &Ident, name: &str) -> TokenStream2 {
    let component_type_quote = resolve_primitive_type(ident);

    // TODO resolve primitive component type properties

    quote! {
        .with_property(#name.to_string(),
            utoipa::openapi::Property::new(
                #component_type_quote
            )
        )
        .with_required(#name.to_string())
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

fn get_component_type(type_path: &TypePath) -> FieldType<'_> {
    get_component_type_from_path(type_path, non_generic_component_type, |path| {
        let segment = get_segment(path);

        if segment.arguments.is_empty() {
            abort!(
                segment.ident.span(),
                "Expected at least one angle bracket argument but was 0"
            );
        };

        println!("got segment: {:#?}", segment);

        get_component_type_from_path(
            get_type_path(get_first_generic_type(segment)),
            |ident| generic_component_type(ident, get_generic_type(segment)),
            |path | {

                println!("path: {:?}", path);
                // TODO 
                get_component_type(path)
            }
            // |type_path| {

            //     // TODO fix this, in double generic situation this is generic type, thus we need to be able to resolve
            //     // types recursively
            //     // abort!(
            //     //     get_segment(type_path).ident.span(),
            //     //     "Is not object or primitive type, cannot resolve ident"
            //     // )
            // },
        )
    })
}

fn non_generic_component_type(ident: &Ident) -> FieldType {
    println!("got primitive ident: {:#?}", ident);

    if is_primitive_type(ident) {
        FieldType::Primitive(ident)
    } else {
        FieldType::Object(ident)
    }
}

fn generic_component_type(ident: &Ident, generic_type: GenericType) -> FieldType {
    if is_primitive_type(ident) {
        FieldType::Generic(ident, ValueType::Primitive, generic_type)
    } else {
        FieldType::Generic(ident, ValueType::Object, generic_type)
    }
}

fn get_component_type_from_path<'a>(
    type_path: &'a TypePath,
    op: impl Fn(&'a Ident) -> FieldType<'a>,
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

fn get_generic_type(segment: &PathSegment) -> GenericType {
    match &*segment.ident.to_string() {
        "HashMap" | "Map" | "BTreeMap" => GenericType::Map,
        "Vec" => GenericType::Vec,
        "Option" => GenericType::Option,
        _ => abort!(
            segment.ident.span(),
            "Unexpected segment type, expected one of: HashMap, BTreeMap, Map, Vec, Option"
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

        // ComponentType::from_type_path_(
        //     generic_type_path,
        //     ComponentType::convert, // if Option<bool> or Vec<String>
        //     |seg| {
        //         // TODO if Option<Vec<String>>

        //         let mut component_type = ComponentType::convert(&seg.ident, seg);
        //         component_type.child = Some(Box::new(ComponentType::resolve_component_type(seg)));

        //         println!("generic component_type: {:#?}", component_type);

        //         component_type
        //     },
        // )
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

// impl<'a> IntoIterator for ComponentType<'a> {
//     type Item = Rc<ComponentType<'a>>;

//     type IntoIter = IntoIter<Self::Item>;

//     fn into_iter(self) -> Self::IntoIter {
//         let mut item = self.child.clone();
//         let mut items = vec![Rc::new(self)];

//         while let Some(component) = item {
//             items.push(component.clone());
//             item = Some(component.clone());
//         }

//         items.into_iter()
//         // self.child.iter().into_iter()
//     }
// }

impl ComponentType<'_> {
    fn to_iter(&self) -> IntoIter<&ComponentType<'_>> {
        let mut item = self.child.as_ref();
        let mut items = vec![self];

        while let Some(component) = item {
            items.push(component.as_ref());
            item = Some(component);
        }

        items.into_iter()
    }
}

#[derive(Debug)]
enum ComponentValue {
    Primitive,
    Struct,
    Enum,
}

#[derive(Debug)]
enum FieldType<'a> {
    Generic(&'a Ident, ValueType, GenericType),
    Primitive(&'a Ident),
    Object(&'a Ident),
}

#[derive(Debug)]
enum ValueType {
    Primitive,
    Object,
}

#[derive(Debug)]
enum GenericType {
    Vec,
    Map,
    Option,
}
