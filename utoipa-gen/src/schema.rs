use std::rc::Rc;

use proc_macro2::Ident;
use proc_macro_error::{abort, abort_call_site};
use syn::{
    AngleBracketedGenericArguments, Attribute, GenericArgument, PathArguments, PathSegment, Type,
    TypePath,
};

use crate::{component_type::ComponentType, Deprecated};

#[cfg(feature = "actix_extras")]
pub mod into_params;

pub mod component;

/// Find `#[deprecated]` attribute from given attributes. Typically derive type attributes
/// or field attributes of struct.
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
    pub ident: &'a Ident,
    pub value_type: ValueType,
    pub generic_type: Option<GenericType>,
    pub child: Option<Rc<ComponentPart<'a>>>,
}

impl<'a> ComponentPart<'a> {
    pub fn from_type(ty: &'a Type) -> ComponentPart<'a> {
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

    fn from_ident(ty: &'a Ident) -> ComponentPart<'a> {
        ComponentPart {
            child: None,
            generic_type: None,
            ident: ty,
            value_type: if ComponentType(ty).is_primitive() {
                ValueType::Primitive
            } else {
                ValueType::Object
            },
        }
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
