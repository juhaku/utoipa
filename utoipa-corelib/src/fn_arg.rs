use std::borrow::Cow;

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, abort_call_site};
// #[cfg(any(feature = "actix_extras", feature = "axum_extras"))]
use quote::{quote, ToTokens};

use syn::{
    punctuated::Punctuated, token::Comma, GenericArgument, Pat, PatType, PathArguments,
    PathSegment, Type, TypePath,
};

use super::IntoParamsType;

/// [`TypeTree`] of items which represents a single parsed `type` of a
/// `Schema`, `Parameter` or `FnArg`
#[cfg_attr(feature = "debug", derive(Debug, PartialEq))]
pub struct TypeTree<'t> {
    pub path: Option<Cow<'t, TypePath>>,
    pub value_type: ValueType,
    pub generic_type: Option<GenericType>,
    pub children: Option<Vec<TypeTree<'t>>>,
}

impl<'t> TypeTree<'t> {
    pub fn from_type(ty: &'t Type) -> TypeTree<'t> {
        Self::from_type_paths(Self::get_type_paths(ty))
    }

    fn get_type_paths(ty: &'t Type) -> Vec<&'t TypePath> {
        match ty {
            Type::Path(path) => vec![path],
            Type::Reference(reference) => Self::get_type_paths(reference.elem.as_ref()),
            Type::Tuple(tuple) => tuple.elems.iter().flat_map(Self::get_type_paths).collect(),
            Type::Group(group) => Self::get_type_paths(group.elem.as_ref()),
            Type::Array(array) => Self::get_type_paths(&array.elem),
            _ => abort_call_site!(
                "unexpected type in component part get type path, expected one of: Path, Reference, Group"
            ),
        }
    }

    fn from_type_paths(paths: Vec<&'t TypePath>) -> TypeTree<'t> {
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

    fn convert_types(paths: Vec<&'t TypePath>) -> impl Iterator<Item = TypeTree<'t>> {
        paths.into_iter().map(|path| {
            // there will always be one segment at least
            let last_segment = path
                .path
                .segments
                .last()
                .expect("at least one segment within path in TypeTree::convert_types");

            if last_segment.arguments.is_empty() {
                Self::convert(Cow::Borrowed(path), last_segment)
            } else {
                Self::resolve_schema_type(Cow::Borrowed(path), last_segment)
            }
        })
    }

    // Only when type is a generic type we get to this function.
    fn resolve_schema_type(path: Cow<'t, TypePath>, last_segment: &'t PathSegment) -> TypeTree<'t> {
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

    fn convert(path: Cow<'t, TypePath>, last_segment: &'t PathSegment) -> TypeTree<'t> {
        let generic_type = Self::get_generic_type(last_segment);
        let is_primitive = SchemaType(&*path).is_primitive();

        Self {
            path: Some(path),
            value_type: if is_primitive {
                ValueType::Primitive
            } else {
                ValueType::Object
            },
            generic_type,
            children: None,
        }
    }

    // TODO should we recognize unknown generic types with `GenericType::Unkonwn` instead of `None`?
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
                path.path
                    .segments
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
            .map(|path| {
                path.path
                    .segments
                    .iter()
                    .any(|segment| &segment.ident == ident)
            })
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

    fn update_path(&mut self, ident: &'t Ident) {
        self.path = Some(Cow::Owned(TypePath {
            qself: None,
            path: syn::Path::from(ident.clone()),
        }))
    }

    /// `Any` virtual type is used when generic object is required in OpenAPI spec. Typically used
    /// with `value_type` attribute to hinder the actual type.
    fn is_any(&self) -> bool {
        self.is("Any")
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
                self_path.path.to_token_stream().to_string()
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

/// Tokenizes OpenAPI data type correctly according to the Rust type
pub struct SchemaType<'a>(pub &'a syn::TypePath);

impl SchemaType<'_> {
    /// Check whether type is known to be primitive in wich case returns true.
    pub fn is_primitive(&self) -> bool {
        let last_segment = match self.0.path.segments.last() {
            Some(segment) => segment,
            None => return false,
        };
        let name = &*last_segment.ident.to_string();

        #[cfg(not(any(
            feature = "chrono",
            feature = "chrono_with_format",
            feature = "decimal",
            feature = "rocket_extras",
            feature = "uuid",
            feature = "time",
        )))]
        {
            is_primitive(name)
        }

        #[cfg(any(
            feature = "chrono",
            feature = "chrono_with_format",
            feature = "decimal",
            feature = "rocket_extras",
            feature = "uuid",
            feature = "time",
        ))]
        {
            let mut primitive = is_primitive(name);

            #[cfg(any(feature = "chrono", feature = "chrono_with_format"))]
            if !primitive {
                primitive = is_primitive_chrono(name);
            }

            #[cfg(feature = "decimal")]
            if !primitive {
                primitive = is_primitive_rust_decimal(name);
            }

            #[cfg(feature = "rocket_extras")]
            if !primitive {
                primitive = matches!(name, "PathBuf");
            }

            #[cfg(feature = "uuid")]
            if !primitive {
                primitive = matches!(name, "Uuid");
            }

            #[cfg(feature = "time")]
            if !primitive {
                primitive = matches!(
                    name,
                    "Date" | "PrimitiveDateTime" | "OffsetDateTime" | "Duration"
                );
            }

            primitive
        }
    }
}

#[inline]
fn is_primitive(name: &str) -> bool {
    matches!(
        name,
        "String"
            | "str"
            | "char"
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

#[inline]
#[cfg(any(feature = "chrono", feature = "chrono_with_format"))]
fn is_primitive_chrono(name: &str) -> bool {
    matches!(name, "DateTime" | "Date" | "Duration")
}

#[inline]
#[cfg(feature = "decimal")]
fn is_primitive_rust_decimal(name: &str) -> bool {
    matches!(name, "Decimal")
}

impl ToTokens for SchemaType<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let last_segment = self.0.path.segments.last().unwrap_or_else(|| {
            abort_call_site!("expected there to be at least one segment in the path")
        });
        let name = &*last_segment.ident.to_string();

        match name {
            "String" | "str" | "char" => {
                tokens.extend(quote! {utoipa::openapi::SchemaType::String})
            }
            "bool" => tokens.extend(quote! { utoipa::openapi::SchemaType::Boolean }),
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
            | "u128" | "usize" => tokens.extend(quote! { utoipa::openapi::SchemaType::Integer }),
            "f32" | "f64" => tokens.extend(quote! { utoipa::openapi::SchemaType::Number }),
            #[cfg(any(feature = "chrono", feature = "chrono_with_format"))]
            "DateTime" | "Date" | "Duration" => {
                tokens.extend(quote! { utoipa::openapi::SchemaType::String })
            }
            #[cfg(feature = "decimal")]
            "Decimal" => tokens.extend(quote! { utoipa::openapi::SchemaType::String }),
            #[cfg(feature = "rocket_extras")]
            "PathBuf" => tokens.extend(quote! { utoipa::openapi::SchemaType::String }),
            #[cfg(feature = "uuid")]
            "Uuid" => tokens.extend(quote! { utoipa::openapi::SchemaType::String }),
            #[cfg(feature = "time")]
            "Date" | "PrimitiveDateTime" | "OffsetDateTime" => {
                tokens.extend(quote! { utoipa::openapi::SchemaType::String })
            }
            #[cfg(feature = "time")]
            "Duration" => tokens.extend(quote! { utoipa::openapi::SchemaType::String }),
            _ => tokens.extend(quote! { utoipa::openapi::SchemaType::Object }),
        }
    }
}

/// Http operation handler functions fn argument.
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct FnArg<'a> {
    pub ty: TypeTree<'a>,
    pub name: &'a Ident,
}

impl<'a> From<(TypeTree<'a>, &'a Ident)> for FnArg<'a> {
    fn from((ty, name): (TypeTree<'a>, &'a Ident)) -> Self {
        Self { ty, name }
    }
}

impl<'a> Ord for FnArg<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(other.name)
    }
}

impl<'a> PartialOrd for FnArg<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(other.name)
    }
}

impl<'a> PartialEq for FnArg<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.ty == other.ty && self.name == other.name
    }
}

impl<'a> Eq for FnArg<'a> {}

pub fn get_fn_args(fn_args: &Punctuated<syn::FnArg, Comma>) -> impl Iterator<Item = FnArg<'_>> {
    fn_args
        .iter()
        .map(|arg| {
            let pat_type = get_fn_arg_pat_type(arg);

            let arg_name = get_pat_ident(pat_type.pat.as_ref());
            (TypeTree::from_type(&pat_type.ty), arg_name)
        })
        .map(FnArg::from)
}

#[inline]
fn get_pat_ident(pat: &Pat) -> &Ident {
    let arg_name = match pat {
            syn::Pat::Ident(ident) => &ident.ident,
            syn::Pat::TupleStruct(tuple_struct) => {
                get_pat_ident(tuple_struct.pat.elems.first().as_ref().expect(
                    "PatTuple expected to have at least one element, cannot get fn argument",
                ))
            }
            _ => abort!(pat,
                "unexpected syn::Pat, expected syn::Pat::Ident,in get_fn_args, cannot get fn argument name"
            ),
        };
    arg_name
}

#[inline]
fn get_fn_arg_pat_type(fn_arg: &syn::FnArg) -> &PatType {
    match fn_arg {
        syn::FnArg::Typed(value) => value,
        _ => abort!(fn_arg, "unexpected fn argument type, expected FnArg::Typed"),
    }
}

#[cfg(any(feature = "actix_extras", feature = "axum_extras"))]
pub fn with_parameter_in(arg: FnArg<'_>) -> Option<(Option<Cow<'_, TypePath>>, TokenStream)> {
    let parameter_in_provider = if arg.ty.is("Path") {
        quote! { || Some (utoipa::openapi::path::ParameterIn::Path) }
    } else if arg.ty.is("Query") {
        quote! { || Some( utoipa::openapi::path::ParameterIn::Query) }
    } else {
        quote! { || None }
    };

    let type_path = arg
        .ty
        .children
        .expect("FnArg TypeTree generic type Path must have children")
        .into_iter()
        .next()
        .unwrap()
        .path;

    Some((type_path, parameter_in_provider))
}

pub fn into_into_params_type(
    (type_path, parameter_in_provider): (Option<Cow<'_, TypePath>>, TokenStream),
) -> IntoParamsType<'_> {
    IntoParamsType {
        parameter_in_provider,
        type_path,
    }
}

// if type is either Path or Query with direct children as Object types without generics
#[cfg(any(feature = "actix_extras", feature = "axum_extras"))]
pub fn is_into_params(fn_arg: &FnArg) -> bool {
    (fn_arg.ty.is("Path") || fn_arg.ty.is("Query"))
        && fn_arg
            .ty
            .children
            .as_ref()
            .map(|children| {
                children.iter().all(|child| {
                    matches!(child.value_type, ValueType::Object)
                        && matches!(child.generic_type, None)
                })
            })
            .unwrap_or(false)
}
