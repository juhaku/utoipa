use std::borrow::Cow;

use proc_macro2::Ident;
use proc_macro_error::{abort, abort_call_site};
use syn::{Attribute, GenericArgument, Path, PathArguments, PathSegment, Type, TypePath};

use crate::{schema_type::SchemaType, Deprecated};

use self::serde::{RenameRule, SerdeContainer, SerdeValue};

pub mod into_params;

mod features;
pub mod schema;
pub mod serde;

/// Check whether either serde `container_rule` or `field_rule` has _`default`_ attribute set.
#[inline]
fn is_default(container_rules: &Option<&SerdeContainer>, field_rule: &Option<&SerdeValue>) -> bool {
    container_rules
        .as_ref()
        .map(|rule| rule.default)
        .unwrap_or(false)
        || field_rule
            .as_ref()
            .map(|rule| rule.default)
            .unwrap_or(false)
}

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

#[cfg_attr(feature = "debug", derive(Debug, PartialEq))]
enum TypeTreeValue<'t> {
    TypePath(&'t TypePath),
    Path(&'t Path),
    /// Slice and array types need to be manually defined, since they cannot be recognized from
    /// generic arguments.
    Array(Vec<TypeTreeValue<'t>>),
}

/// [`TypeTree`] of items which represents a single parsed `type` of a
/// `Schema`, `Parameter` or `FnArg`
#[cfg_attr(feature = "debug", derive(Debug, PartialEq))]
#[derive(Clone)]
pub struct TypeTree<'t> {
    pub path: Option<Cow<'t, Path>>,
    pub value_type: ValueType,
    pub generic_type: Option<GenericType>,
    pub children: Option<Vec<TypeTree<'t>>>,
}

impl<'t> TypeTree<'t> {
    pub fn from_type(ty: &'t Type) -> TypeTree<'t> {
        Self::from_type_paths(Self::get_type_paths(ty))
    }

    fn get_type_paths(ty: &'t Type) -> Vec<TypeTreeValue> {
        match ty {
            Type::Path(path) => {
                vec![TypeTreeValue::TypePath(path)]
            },
            Type::Reference(reference) => Self::get_type_paths(reference.elem.as_ref()),
            Type::Tuple(tuple) => tuple.elems.iter().flat_map(Self::get_type_paths).collect(),
            Type::Group(group) => Self::get_type_paths(group.elem.as_ref()),
            Type::Slice(slice) => vec![TypeTreeValue::Array(Self::get_type_paths(&slice.elem))],
            Type::Array(array) => vec![TypeTreeValue::Array(Self::get_type_paths(&array.elem))],
            Type::TraitObject(trait_object) => {
                trait_object
                    .bounds
                    .iter()
                    .find_map(|bound| {
                        match bound {
                            syn::TypeParamBound::Trait(trait_bound) => Some(&trait_bound.path),
                            syn::TypeParamBound::Lifetime(_) => None
                        }
                    })
                    .map(|path| vec![TypeTreeValue::Path(path)]).unwrap_or_else(Vec::new)
            }
            _ => abort_call_site!(
                "unexpected type in component part get type path, expected one of: Path, Tuple, Reference, Group, Array, Slice, TraitObject"
            ),
        }
    }

    fn from_type_paths(paths: Vec<TypeTreeValue<'t>>) -> TypeTree<'t> {
        if paths.len() > 1 {
            TypeTree {
                path: None,
                children: Some(Self::convert_types(paths).collect()),
                generic_type: None,
                value_type: ValueType::Tuple,
            }
        } else {
            Self::convert_types(paths)
                .into_iter()
                .next()
                .expect("TypeTreeValue from_type_paths expected at least one TypePath")
        }
    }

    fn convert_types(paths: Vec<TypeTreeValue<'t>>) -> impl Iterator<Item = TypeTree<'t>> {
        paths.into_iter().map(|value| {
            let path = match value {
                TypeTreeValue::TypePath(type_path) => &type_path.path,
                TypeTreeValue::Path(path) => path,
                TypeTreeValue::Array(value) => {
                    return TypeTree {
                        path: None,
                        value_type: ValueType::Object,
                        generic_type: Some(GenericType::Vec),
                        children: Some(vec![Self::from_type_paths(value)]),
                    };
                }
            };

            // there will always be one segment at least
            let last_segment = path
                .segments
                .last()
                .expect("at least one segment within path in TypeTree::convert_types");

            if last_segment.arguments.is_empty() {
                Self::convert(path, last_segment)
            } else {
                Self::resolve_schema_type(path, last_segment)
            }
        })
    }

    // Only when type is a generic type we get to this function.
    fn resolve_schema_type(path: &'t Path, last_segment: &'t PathSegment) -> TypeTree<'t> {
        if last_segment.arguments.is_empty() {
            abort!(
                last_segment.ident,
                "expected at least one angle bracket argument but was 0"
            );
        };

        let mut generic_schema_type = Self::convert(path, last_segment);

        let mut generic_types = match &last_segment.arguments {
            PathArguments::AngleBracketed(angle_bracketed_args) => {
                // if all type arguments are lifetimes we ignore the generic type
                if angle_bracketed_args
                    .args
                    .iter()
                    .all(|arg| matches!(arg, GenericArgument::Lifetime(_)))
                {
                    None
                } else {
                    Some(
                        angle_bracketed_args
                            .args
                            .iter()
                            .filter(|arg| !matches!(arg, GenericArgument::Lifetime(_)))
                            .map(|arg| match arg {
                                GenericArgument::Type(arg) => arg,
                                _ => abort!(
                                    arg,
                                    "expected generic argument type or generic argument lifetime"
                                ),
                            }),
                    )
                }
            }
            _ => abort!(
                last_segment.ident,
                "unexpected path argument, expected angle bracketed path argument"
            ),
        };

        generic_schema_type.children = generic_types
            .as_mut()
            .map(|generic_type| generic_type.map(Self::from_type).collect());

        generic_schema_type
    }

    fn convert(path: &'t Path, last_segment: &'t PathSegment) -> TypeTree<'t> {
        let generic_type = Self::get_generic_type(last_segment);
        let is_primitive = SchemaType(path).is_primitive();

        Self {
            path: Some(Cow::Borrowed(path)),
            value_type: if is_primitive {
                ValueType::Primitive
            } else {
                ValueType::Object
            },
            generic_type,
            children: None,
        }
    }

    // TODO should we recognize unknown generic types with `GenericType::Unknown` instead of `None`?
    fn get_generic_type(segment: &PathSegment) -> Option<GenericType> {
        match &*segment.ident.to_string() {
            "HashMap" | "Map" | "BTreeMap" => Some(GenericType::Map),
            "Vec" => Some(GenericType::Vec),
            #[cfg(feature = "smallvec")]
            "SmallVec" => Some(GenericType::Vec),
            "Option" => Some(GenericType::Option),
            "Cow" => Some(GenericType::Cow),
            "Box" => Some(GenericType::Box),
            "RefCell" => Some(GenericType::RefCell),
            _ => None,
        }
    }

    /// Check whether [`TypeTreeValue`]'s [`syn::TypePath`] or any if it's `children`s [`syn::TypePath`]
    /// is a given type as [`str`].
    pub fn is(&self, s: &str) -> bool {
        let mut is = self
            .path
            .as_ref()
            .map(|path| {
                path.segments
                    .last()
                    .expect("expected at least one segment in TreeTypeValue path")
                    .ident
                    == s
            })
            .unwrap_or(false);

        if let Some(ref children) = self.children {
            is = is || children.iter().any(|child| child.is(s));
        }

        is
    }

    fn find_mut_by_ident(&mut self, ident: &'_ Ident) -> Option<&mut Self> {
        let is = self
            .path
            .as_mut()
            .map(|path| path.segments.iter().any(|segment| &segment.ident == ident))
            .unwrap_or(false);

        if is {
            Some(self)
        } else {
            self.children.as_mut().and_then(|children| {
                children
                    .iter_mut()
                    .find_map(|child| Self::find_mut_by_ident(child, ident))
            })
        }
    }

    /// Update current [`TypeTree`] from given `ident`.
    ///
    /// It will update everything else except `children` for the `TypeTree`. This means that the
    /// `TypeTree` will not be changed and will be traveled as before update.
    fn update(&mut self, ident: Ident) {
        let new_path = Path::from(ident);

        let segments = &new_path.segments;
        let last_segment = segments
            .last()
            .expect("TypeTree::update path should have at least one segment");

        let generic_type = Self::get_generic_type(last_segment);
        let value_type = if SchemaType(&new_path).is_primitive() {
            ValueType::Primitive
        } else {
            ValueType::Object
        };

        self.value_type = value_type;
        self.generic_type = generic_type;
        self.path = Some(Cow::Owned(new_path));
    }

    /// `Object` virtual type is used when generic object is required in OpenAPI spec. Typically used
    /// with `value_type` attribute to hinder the actual type.
    pub fn is_object(&self) -> bool {
        self.is("Object")
    }
}

#[cfg(not(feature = "debug"))]
impl PartialEq for TypeTree<'_> {
    fn eq(&self, other: &Self) -> bool {
        use quote::ToTokens;
        let path_eg = match (self.path.as_ref(), other.path.as_ref()) {
            (Some(Cow::Borrowed(self_path)), Some(Cow::Borrowed(other_path))) => {
                self_path.into_token_stream().to_string()
                    == other_path.into_token_stream().to_string()
            }
            (Some(Cow::Owned(self_path)), Some(Cow::Owned(other_path))) => {
                self_path.to_token_stream().to_string()
                    == other_path.into_token_stream().to_string()
            }
            (None, None) => true,
            _ => false,
        };

        path_eg
            && self.value_type == other.value_type
            && self.generic_type == other.generic_type
            && self.children == other.children
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Primitive,
    Object,
    Tuple,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum GenericType {
    Vec,
    Map,
    Option,
    Cow,
    Box,
    RefCell,
}

trait Rename {
    fn rename(rule: &RenameRule, value: &str) -> String;
}

/// Performs a rename for given `value` based on given rules. If no rules were
/// provided returns [`None`]
///
/// Method accepts 3 arguments.
/// * `value` to rename.
/// * `to` Optional rename to value for fields with _`rename`_ property.
/// * `container_rule` which is used to rename containers with _`rename_all`_ property.
fn rename<'r, R: Rename>(
    value: &'r str,
    to: Option<Cow<'r, str>>,
    container_rule: Option<&'r RenameRule>,
) -> Option<Cow<'r, str>> {
    let rename = to.and_then(|to| if !to.is_empty() { Some(to) } else { None });

    rename.or_else(|| {
        container_rule
            .as_ref()
            .map(|container_rule| Cow::Owned(R::rename(container_rule, value)))
    })
}

/// Can be used to perform rename on container level e.g `struct`, `enum` or `enum` `variant` level.
struct VariantRename;

impl Rename for VariantRename {
    fn rename(rule: &RenameRule, value: &str) -> String {
        rule.rename_variant(value)
    }
}

/// Can be used to perform rename on field level of a container e.g `struct`.
struct FieldRename;

impl Rename for FieldRename {
    fn rename(rule: &RenameRule, value: &str) -> String {
        rule.rename(value)
    }
}
