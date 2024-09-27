use std::borrow::Cow;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{
    AngleBracketedGenericArguments, Attribute, GenericArgument, GenericParam, Generics, Path,
    PathArguments, PathSegment, Type, TypePath,
};

use crate::doc_comment::CommentAttributes;
use crate::schema_type::{SchemaFormat, SchemaTypeInner};
use crate::{
    as_tokens_or_diagnostics, Array, AttributesExt, Diagnostics, GenericsExt, OptionExt,
    ToTokensDiagnostics,
};
use crate::{schema_type::SchemaType, Deprecated};

use self::features::attributes::{Description, Nullable};
use self::features::validation::Minimum;
use self::features::{
    pop_feature, Feature, FeaturesExt, IntoInner, IsInline, ToTokensExt, Validatable,
};
use self::serde::{RenameRule, SerdeContainer, SerdeValue};

pub mod into_params;

pub mod features;
pub mod schema;
pub mod serde;

/// Check whether either serde `container_rule` or `field_rule` has _`default`_ attribute set.
#[inline]
fn is_default(container_rules: &SerdeContainer, field_rule: &SerdeValue) -> bool {
    container_rules.default || field_rule.default
}

/// Find `#[deprecated]` attribute from given attributes. Typically derive type attributes
/// or field attributes of struct.
fn get_deprecated(attributes: &[Attribute]) -> Option<Deprecated> {
    if attributes.has_deprecated() {
        Some(Deprecated::True)
    } else {
        None
    }
}

/// Check whether field is required based on following rules.
///
/// * If field has not serde's `skip_serializing_if`
/// * Field has not `serde_with` double option
/// * Field is not default
pub fn is_required(field_rule: &SerdeValue, container_rules: &SerdeContainer) -> bool {
    !field_rule.skip_serializing_if
        && !field_rule.double_option
        && !is_default(container_rules, field_rule)
}

#[cfg_attr(feature = "debug", derive(Debug))]
enum TypeTreeValue<'t> {
    TypePath(&'t TypePath),
    Path(&'t Path),
    /// Slice and array types need to be manually defined, since they cannot be recognized from
    /// generic arguments.
    Array(Vec<TypeTreeValue<'t>>, Span),
    UnitType,
    Tuple(Vec<TypeTreeValue<'t>>, Span),
}

impl PartialEq for TypeTreeValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Path(_) => self == other,
            Self::TypePath(_) => self == other,
            Self::Array(array, _) => matches!(other, Self::Array(other, _) if other == array),
            Self::Tuple(tuple, _) => matches!(other, Self::Tuple(other, _) if other == tuple),
            Self::UnitType => self == other,
        }
    }
}

enum TypeTreeValueIter<'a, T> {
    Once(std::iter::Once<T>),
    Empty,
    Iter(Box<dyn std::iter::Iterator<Item = T> + 'a>),
}

impl<'a, T> TypeTreeValueIter<'a, T> {
    fn once(item: T) -> Self {
        Self::Once(std::iter::once(item))
    }

    fn empty() -> Self {
        Self::Empty
    }
}

impl<'a, T> Iterator for TypeTreeValueIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Once(iter) => iter.next(),
            Self::Empty => None,
            Self::Iter(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Once(once) => once.size_hint(),
            Self::Empty => (0, None),
            Self::Iter(iter) => iter.size_hint(),
        }
    }
}

/// [`TypeTree`] of items which represents a single parsed `type` of a
/// `Schema`, `Parameter` or `FnArg`
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct TypeTree<'t> {
    pub path: Option<Cow<'t, Path>>,
    #[allow(unused)]
    pub span: Option<Span>,
    pub value_type: ValueType,
    pub generic_type: Option<GenericType>,
    pub children: Option<Vec<TypeTree<'t>>>,
}

impl TypeTree<'_> {
    pub fn from_type(ty: &Type) -> Result<TypeTree<'_>, Diagnostics> {
        Self::convert_types(Self::get_type_tree_values(ty)?).map(|mut type_tree| {
            type_tree
                .next()
                .expect("TypeTree from type should have one TypeTree parent")
        })
    }

    fn get_type_tree_values(
        ty: &Type,
    ) -> Result<impl Iterator<Item = TypeTreeValue<'_>>, Diagnostics> {
        let type_tree_values = match ty {
            Type::Path(path) => {
                TypeTreeValueIter::once(TypeTreeValue::TypePath(path))
            },
            // NOTE have to put this in the box to avoid compiler bug with recursive functions
            // See here https://github.com/rust-lang/rust/pull/110844 and https://github.com/rust-lang/rust/issues/111906
            // This bug in fixed in Rust 1.79, but in order to support Rust 1.75 these need to be
            // boxed.
            Type::Reference(reference) => TypeTreeValueIter::Iter(Box::new(Self::get_type_tree_values(reference.elem.as_ref())?)),
            // Type::Reference(reference) => Self::get_type_tree_values(reference.elem.as_ref())?,
            Type::Tuple(tuple) => {
                // Detect unit type ()
                if tuple.elems.is_empty() { return Ok(TypeTreeValueIter::once(TypeTreeValue::UnitType)) }
                TypeTreeValueIter::once(TypeTreeValue::Tuple(
                    tuple.elems.iter().map(Self::get_type_tree_values).collect::<Result<Vec<_>, Diagnostics>>()?.into_iter().flatten().collect(),
                    tuple.span()
                ))
            },
            // NOTE have to put this in the box to avoid compiler bug with recursive functions
            // See here https://github.com/rust-lang/rust/pull/110844 and https://github.com/rust-lang/rust/issues/111906
            // This bug in fixed in Rust 1.79, but in order to support Rust 1.75 these need to be
            // boxed.
            Type::Group(group) => TypeTreeValueIter::Iter(Box::new(Self::get_type_tree_values(group.elem.as_ref())?)),
            // Type::Group(group) => Self::get_type_tree_values(group.elem.as_ref())?,
            Type::Slice(slice) => TypeTreeValueIter::once(TypeTreeValue::Array(Self::get_type_tree_values(&slice.elem)?.collect(), slice.bracket_token.span.join())),
            Type::Array(array) => TypeTreeValueIter::once(TypeTreeValue::Array(Self::get_type_tree_values(&array.elem)?.collect(), array.bracket_token.span.join())),
            Type::TraitObject(trait_object) => {
                trait_object
                    .bounds
                    .iter()
                    .find_map(|bound| {
                        match &bound {
                            syn::TypeParamBound::Trait(trait_bound) => Some(&trait_bound.path),
                            syn::TypeParamBound::Lifetime(_) => None,
                            syn::TypeParamBound::Verbatim(_) => None,
                            _ => todo!("TypeTree trait object found unrecognized TypeParamBound"),
                        }
                    })
                    .map(|path| TypeTreeValueIter::once(TypeTreeValue::Path(path))).unwrap_or_else(TypeTreeValueIter::empty)
            }
            unexpected => return Err(Diagnostics::with_span(unexpected.span(), "unexpected type in component part get type path, expected one of: Path, Tuple, Reference, Group, Array, Slice, TraitObject")),
        };

        Ok(type_tree_values)
    }

    fn convert_types<'p, P: IntoIterator<Item = TypeTreeValue<'p>>>(
        paths: P,
    ) -> Result<impl Iterator<Item = TypeTree<'p>>, Diagnostics> {
        paths
            .into_iter()
            .map(|value| {
                let path = match value {
                    TypeTreeValue::TypePath(type_path) => &type_path.path,
                    TypeTreeValue::Path(path) => path,
                    TypeTreeValue::Array(value, span) => {
                        let array: Path = Ident::new("Array", span).into();
                        return Ok(TypeTree {
                            path: Some(Cow::Owned(array)),
                            span: Some(span),
                            value_type: ValueType::Object,
                            generic_type: Some(GenericType::Vec),
                            children: Some(match Self::convert_types(value) {
                                Ok(converted_values) => converted_values.collect(),
                                Err(diagnostics) => return Err(diagnostics),
                            }),
                        });
                    }
                    TypeTreeValue::Tuple(tuple, span) => {
                        return Ok(TypeTree {
                            path: None,
                            span: Some(span),
                            children: Some(match Self::convert_types(tuple) {
                                Ok(converted_values) => converted_values.collect(),
                                Err(diagnostics) => return Err(diagnostics),
                            }),
                            generic_type: None,
                            value_type: ValueType::Tuple,
                        })
                    }
                    TypeTreeValue::UnitType => {
                        return Ok(TypeTree {
                            path: None,
                            span: None,
                            value_type: ValueType::Tuple,
                            generic_type: None,
                            children: None,
                        })
                    }
                };

                // there will always be one segment at least
                let last_segment = path
                    .segments
                    .last()
                    .expect("at least one segment within path in TypeTree::convert_types");

                if last_segment.arguments.is_empty() {
                    Ok(Self::convert(path, last_segment))
                } else {
                    Self::resolve_schema_type(path, last_segment)
                }
            })
            .collect::<Result<Vec<TypeTree<'_>>, Diagnostics>>()
            .map(IntoIterator::into_iter)
    }

    // Only when type is a generic type we get to this function.
    fn resolve_schema_type<'t>(
        path: &'t Path,
        last_segment: &'t PathSegment,
    ) -> Result<TypeTree<'t>, Diagnostics> {
        if last_segment.arguments.is_empty() {
            return Err(Diagnostics::with_span(
                last_segment.ident.span(),
                "expected at least one angle bracket argument but was 0",
            ));
        };

        let mut generic_schema_type = Self::convert(path, last_segment);

        let mut generic_types = match &last_segment.arguments {
            PathArguments::AngleBracketed(angle_bracketed_args) => {
                // if all type arguments are lifetimes we ignore the generic type
                if angle_bracketed_args.args.iter().all(|arg| {
                    matches!(
                        arg,
                        GenericArgument::Lifetime(_) | GenericArgument::Const(_)
                    )
                }) {
                    None
                } else {
                    Some(
                        angle_bracketed_args
                            .args
                            .iter()
                            .filter(|arg| {
                                !matches!(
                                    arg,
                                    GenericArgument::Lifetime(_) | GenericArgument::Const(_)
                                )
                            })
                            .map(|arg| match arg {
                                GenericArgument::Type(arg) => Ok(arg),
                                unexpected => Err(Diagnostics::with_span(
                                    unexpected.span(),
                                    "expected generic argument type or generic argument lifetime",
                                )),
                            })
                            .collect::<Result<Vec<_>, Diagnostics>>()?
                            .into_iter(),
                    )
                }
            }
            _ => {
                return Err(Diagnostics::with_span(
                    last_segment.ident.span(),
                    "unexpected path argument, expected angle bracketed path argument",
                ))
            }
        };

        generic_schema_type.children = generic_types.as_mut().map_try(|generic_type| {
            generic_type
                .map(Self::from_type)
                .collect::<Result<Vec<_>, Diagnostics>>()
        })?;

        Ok(generic_schema_type)
    }

    fn convert<'t>(path: &'t Path, last_segment: &'t PathSegment) -> TypeTree<'t> {
        let generic_type = Self::get_generic_type(last_segment);
        let schema_type = SchemaType {
            path: Cow::Borrowed(path),
            nullable: matches!(generic_type, Some(GenericType::Option)),
        };

        TypeTree {
            path: Some(Cow::Borrowed(path)),
            span: Some(path.span()),
            value_type: if schema_type.is_primitive() {
                ValueType::Primitive
            } else if schema_type.is_value() {
                ValueType::Value
            } else {
                ValueType::Object
            },
            generic_type,
            children: None,
        }
    }

    // TODO should we recognize unknown generic types with `GenericType::Unknown` instead of `None`?
    fn get_generic_type(segment: &PathSegment) -> Option<GenericType> {
        if segment.arguments.is_empty() {
            return None;
        }

        match &*segment.ident.to_string() {
            "HashMap" | "Map" | "BTreeMap" => Some(GenericType::Map),
            #[cfg(feature = "indexmap")]
            "IndexMap" => Some(GenericType::Map),
            "Vec" => Some(GenericType::Vec),
            "BTreeSet" | "HashSet" => Some(GenericType::Set),
            "LinkedList" => Some(GenericType::LinkedList),
            #[cfg(feature = "smallvec")]
            "SmallVec" => Some(GenericType::SmallVec),
            "Option" => Some(GenericType::Option),
            "Cow" => Some(GenericType::Cow),
            "Box" => Some(GenericType::Box),
            #[cfg(feature = "rc_schema")]
            "Arc" => Some(GenericType::Arc),
            #[cfg(feature = "rc_schema")]
            "Rc" => Some(GenericType::Rc),
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

    /// `Object` virtual type is used when generic object is required in OpenAPI spec. Typically used
    /// with `value_type` attribute to hinder the actual type.
    pub fn is_object(&self) -> bool {
        self.is("Object")
    }

    /// `Value` virtual type is used when any JSON value is required in OpenAPI spec. Typically used
    /// with `value_type` attribute for a member of type `serde_json::Value`.
    pub fn is_value(&self) -> bool {
        self.is("Value")
    }

    /// Check whether the [`TypeTree`]'s `generic_type` is [`GenericType::Option`]
    pub fn is_option(&self) -> bool {
        matches!(self.generic_type, Some(GenericType::Option))
    }

    /// Check whether the [`TypeTree`]'s `generic_type` is [`GenericType::Map`]
    pub fn is_map(&self) -> bool {
        matches!(self.generic_type, Some(GenericType::Map))
    }

    /// Get [`syn::Generics`] for current [`TypeTree`]'s [`syn::Path`].
    pub fn get_path_generics(&self) -> syn::Result<Generics> {
        let mut generics = Generics::default();
        let segment = self
            .path
            .as_ref()
            .ok_or_else(|| syn::Error::new(self.path.span(), "cannot get TypeTree::path, did you call this on `tuple` or `unit` type type tree?"))?
            .segments
            .last()
            .expect("Path must have segments");

        fn type_to_generic_params(ty: &Type) -> Vec<GenericParam> {
            match &ty {
                Type::Path(path) => {
                    let mut params_vec: Vec<GenericParam> = Vec::new();
                    let last_segment = path
                        .path
                        .segments
                        .last()
                        .expect("TypePath must have a segment");
                    let ident = &last_segment.ident;
                    params_vec.push(syn::parse_quote!(#ident));

                    params_vec
                }
                Type::Reference(reference) => type_to_generic_params(reference.elem.as_ref()),
                _ => Vec::new(),
            }
        }

        fn angle_bracket_args_to_params(
            args: &AngleBracketedGenericArguments,
        ) -> impl Iterator<Item = GenericParam> + '_ {
            args.args
                .iter()
                .filter_map(move |generic_argument| {
                    match generic_argument {
                        GenericArgument::Type(ty) => Some(type_to_generic_params(ty)),
                        GenericArgument::Lifetime(life) => {
                            Some(vec![GenericParam::Lifetime(syn::parse_quote!(#life))])
                        }
                        _ => None, // other wise ignore
                    }
                })
                .flatten()
        }

        if let PathArguments::AngleBracketed(angle_bracketed_args) = &segment.arguments {
            generics.lt_token = Some(angle_bracketed_args.lt_token);
            generics.params = angle_bracket_args_to_params(angle_bracketed_args).collect();
            generics.gt_token = Some(angle_bracketed_args.gt_token);
        };

        Ok(generics)
    }

    /// Get possible global alias defined in `utoipa_config::Config` for current `TypeTree`.
    pub fn get_alias_type(&self) -> Result<Option<syn::Type>, Diagnostics> {
        #[cfg(feature = "config")]
        {
            self.path
                .as_ref()
                .and_then(|path| path.segments.iter().last())
                .and_then(|last_segment| {
                    crate::CONFIG.aliases.get(&*last_segment.ident.to_string())
                })
                .map_try(|alias| syn::parse_str::<syn::Type>(alias.as_ref()))
                .map_err(|error| Diagnostics::new(error.to_string()))
        }

        #[cfg(not(feature = "config"))]
        Ok(None)
    }
}

impl PartialEq for TypeTree<'_> {
    #[cfg(feature = "debug")]
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.value_type == other.value_type
            && self.generic_type == other.generic_type
            && self.children == other.children
    }

    #[cfg(not(feature = "debug"))]
    fn eq(&self, other: &Self) -> bool {
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
    Value,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum GenericType {
    Vec,
    LinkedList,
    Set,
    #[cfg(feature = "smallvec")]
    SmallVec,
    Map,
    Option,
    Cow,
    Box,
    RefCell,
    #[cfg(feature = "rc_schema")]
    Arc,
    #[cfg(feature = "rc_schema")]
    Rc,
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
fn rename<'s, R: Rename>(
    value: &str,
    to: Option<Cow<'s, str>>,
    container_rule: Option<&RenameRule>,
) -> Option<Cow<'s, str>> {
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

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Container<'c> {
    pub generics: &'c Generics,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ComponentSchemaProps<'c> {
    pub container: &'c Container<'c>,
    pub type_tree: &'c TypeTree<'c>,
    pub features: Vec<Feature>,
    pub description: Option<&'c ComponentDescription<'c>>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ComponentDescription<'c> {
    CommentAttributes(&'c CommentAttributes),
    Description(&'c Description),
}

impl ToTokens for ComponentDescription<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let description = match self {
            Self::CommentAttributes(attributes) => {
                if attributes.is_empty() {
                    TokenStream::new()
                } else {
                    attributes.as_formatted_string().to_token_stream()
                }
            }
            Self::Description(description) => description.to_token_stream(),
        };

        if !description.is_empty() {
            tokens.extend(quote! {
                .description(Some(#description))
            });
        }
    }
}

/// Used to store possible inner field schema name and tokens if field contains any schema
/// references. E.g. field: Vec<Foo> should have name: Foo::name(), tokens: Foo::schema() and
/// references: Foo::schemas()
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Default)]
pub struct SchemaReference {
    pub name: TokenStream,
    pub tokens: TokenStream,
    pub references: TokenStream,
}

impl SchemaReference {
    /// Check whether `SchemaReference` is partial. Partial schema reference occurs in situation
    /// when reference schema tokens cannot be resolved e.g. type in question is generic argument.
    fn is_partial(&self) -> bool {
        self.tokens.is_empty()
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ComponentSchema {
    tokens: TokenStream,
    pub name_tokens: TokenStream,
    pub schema_references: Vec<SchemaReference>,
}

impl ComponentSchema {
    pub fn new(
        ComponentSchemaProps {
            container,
            type_tree,
            mut features,
            description,
        }: ComponentSchemaProps,
    ) -> Result<Self, Diagnostics> {
        let mut tokens = TokenStream::new();
        let mut name_tokens = TokenStream::new();
        let mut schema_references = Vec::<SchemaReference>::new();

        match type_tree.generic_type {
            Some(GenericType::Map) => ComponentSchema::map_to_tokens(
                &mut tokens,
                &mut schema_references,
                container,
                features,
                type_tree,
                description,
            )?,
            Some(GenericType::Vec | GenericType::LinkedList | GenericType::Set) => {
                ComponentSchema::vec_to_tokens(
                    &mut tokens,
                    &mut schema_references,
                    container,
                    features,
                    type_tree,
                    description,
                )?
            }
            #[cfg(feature = "smallvec")]
            Some(GenericType::SmallVec) => ComponentSchema::vec_to_tokens(
                &mut tokens,
                &mut schema_references,
                container,
                features,
                type_tree,
                description,
            )?,
            Some(GenericType::Option) => {
                // Add nullable feature if not already exists. Option is always nullable
                if !features
                    .iter()
                    .any(|feature| matches!(feature, Feature::Nullable(_)))
                {
                    features.push(Nullable::new().into());
                }
                let schema = ComponentSchema::new(ComponentSchemaProps {
                    container,
                    type_tree: type_tree
                        .children
                        .as_ref()
                        .expect("ComponentSchema generic container type should have children")
                        .iter()
                        .next()
                        .expect("ComponentSchema generic container type should have 1 child"),
                    features,
                    description,
                })?;
                schema.to_tokens(&mut tokens);

                schema_references.extend(schema.schema_references);
            }
            Some(GenericType::Cow | GenericType::Box | GenericType::RefCell) => {
                let schema = ComponentSchema::new(ComponentSchemaProps {
                    container,
                    type_tree: type_tree
                        .children
                        .as_ref()
                        .expect("ComponentSchema generic container type should have children")
                        .iter()
                        .next()
                        .expect("ComponentSchema generic container type should have 1 child"),
                    features,
                    description,
                })?;
                schema.to_tokens(&mut tokens);

                schema_references.extend(schema.schema_references);
            }
            #[cfg(feature = "rc_schema")]
            Some(GenericType::Arc) | Some(GenericType::Rc) => {
                let schema = ComponentSchema::new(ComponentSchemaProps {
                    container,
                    type_tree: type_tree
                        .children
                        .as_ref()
                        .expect("ComponentSchema rc generic container type should have children")
                        .iter()
                        .next()
                        .expect("ComponentSchema rc generic container type should have 1 child"),
                    features,
                    description,
                })?;
                schema.to_tokens(&mut tokens);

                schema_references.extend(schema.schema_references);
            }
            None => ComponentSchema::non_generic_to_tokens(
                &mut tokens,
                &mut name_tokens,
                &mut schema_references,
                container,
                features,
                type_tree,
                description,
            )?,
        };

        Ok(Self {
            tokens,
            name_tokens,
            schema_references,
        })
    }

    /// Create `.schema_type(...)` override token stream if nullable is true from given [`SchemaTypeInner`].
    fn get_schema_type_override(
        nullable: Option<Nullable>,
        schema_type_inner: SchemaTypeInner,
    ) -> Option<TokenStream> {
        if let Some(nullable) = nullable {
            let nullable_schema_type = nullable.into_schema_type_token_stream();
            let schema_type = if nullable.value() && !nullable_schema_type.is_empty() {
                Some(
                    quote! { utoipa::openapi::schema::SchemaType::from_iter([#schema_type_inner, #nullable_schema_type]) },
                )
            } else {
                None
            };

            schema_type.map(|schema_type| quote! { .schema_type(#schema_type) })
        } else {
            None
        }
    }

    fn map_to_tokens(
        tokens: &mut TokenStream,
        schema_references: &mut Vec<SchemaReference>,
        container: &Container,
        mut features: Vec<Feature>,
        type_tree: &TypeTree,
        description_stream: Option<&ComponentDescription<'_>>,
    ) -> Result<(), Diagnostics> {
        let example = features.pop_by(|feature| matches!(feature, Feature::Example(_)));
        let additional_properties = pop_feature!(features => Feature::AdditionalProperties(_));
        let nullable: Option<Nullable> =
            pop_feature!(features => Feature::Nullable(_)).into_inner();
        let default = pop_feature!(features => Feature::Default(_));
        let default_tokens = as_tokens_or_diagnostics!(&default);
        let deprecated = pop_feature!(features => Feature::Deprecated(_)).try_to_token_stream()?;

        let additional_properties = additional_properties
            .as_ref()
            .map_try(|feature| Ok(as_tokens_or_diagnostics!(feature)))?
            .or_else_try(|| {
                // Maps are treated as generic objects with no named properties and
                // additionalProperties denoting the type
                // maps have 2 child schemas and we are interested the second one of them
                // which is used to determine the additional properties
                let schema_property = ComponentSchema::new(ComponentSchemaProps {
                    container,
                    type_tree: type_tree
                        .children
                        .as_ref()
                        .expect("ComponentSchema Map type should have children")
                        .get(1)
                        .expect("ComponentSchema Map type should have 2 child"),
                    features,
                    description: None,
                })?;
                let schema_tokens = schema_property.to_token_stream();

                schema_references.extend(schema_property.schema_references);

                Result::<Option<TokenStream>, Diagnostics>::Ok(Some(
                    quote! { .additional_properties(Some(#schema_tokens)) },
                ))
            })?;

        let schema_type =
            ComponentSchema::get_schema_type_override(nullable, SchemaTypeInner::Object);

        tokens.extend(quote! {
            utoipa::openapi::ObjectBuilder::new()
                #schema_type
                #additional_properties
                #description_stream
                #deprecated
                #default_tokens
        });

        example.to_tokens(tokens)
    }

    fn vec_to_tokens(
        tokens: &mut TokenStream,
        schema_references: &mut Vec<SchemaReference>,
        container: &Container,
        mut features: Vec<Feature>,
        type_tree: &TypeTree,
        description_stream: Option<&ComponentDescription<'_>>,
    ) -> Result<(), Diagnostics> {
        let example = pop_feature!(features => Feature::Example(_));
        let xml = features.extract_vec_xml_feature(type_tree)?;
        let max_items = pop_feature!(features => Feature::MaxItems(_));
        let min_items = pop_feature!(features => Feature::MinItems(_));
        let nullable: Option<Nullable> =
            pop_feature!(features => Feature::Nullable(_)).into_inner();
        let default = pop_feature!(features => Feature::Default(_));
        let deprecated = pop_feature!(features => Feature::Deprecated(_)).try_to_token_stream()?;

        let child = type_tree
            .children
            .as_ref()
            .expect("ComponentSchema Vec should have children")
            .iter()
            .next()
            .expect("ComponentSchema Vec should have 1 child");

        #[cfg(feature = "smallvec")]
        let child = if type_tree.generic_type == Some(GenericType::SmallVec) {
            child
                .children
                .as_ref()
                .expect("SmallVec should have children")
                .iter()
                .next()
                .expect("SmallVec should have 1 child")
        } else {
            child
        };

        let component_schema = ComponentSchema::new(ComponentSchemaProps {
            container,
            type_tree: child,
            features,
            description: None,
        })?;
        let component_schema_tokens = component_schema.to_token_stream();

        schema_references.extend(component_schema.schema_references);

        let unique = match matches!(type_tree.generic_type, Some(GenericType::Set)) {
            true => quote! {
                .unique_items(true)
            },
            false => quote! {},
        };
        let schema_type =
            ComponentSchema::get_schema_type_override(nullable, SchemaTypeInner::Array);

        let schema = quote! {
            utoipa::openapi::schema::ArrayBuilder::new()
                #schema_type
                .items(#component_schema_tokens)
            #unique
        };

        let validate = |feature: &Feature| {
            let type_path = &**type_tree.path.as_ref().unwrap();
            let schema_type = SchemaType {
                path: Cow::Borrowed(type_path),
                nullable: nullable
                    .map(|nullable| nullable.value())
                    .unwrap_or_default(),
            };
            feature.validate(&schema_type, type_tree);
        };

        tokens.extend(quote! {
            #schema
            #deprecated
            #description_stream
        });

        if let Some(max_items) = max_items {
            validate(&max_items);
            tokens.extend(max_items.to_token_stream())
        }

        if let Some(min_items) = min_items {
            validate(&min_items);
            tokens.extend(min_items.to_token_stream())
        }

        if let Some(default) = default {
            tokens.extend(default.to_token_stream())
        }

        example.to_tokens(tokens)?;
        xml.to_tokens(tokens)?;

        Ok(())
    }

    fn non_generic_to_tokens(
        tokens: &mut TokenStream,
        name_tokens: &mut TokenStream,
        schema_references: &mut Vec<SchemaReference>,
        container: &Container,
        mut features: Vec<Feature>,
        type_tree: &TypeTree,
        description_stream: Option<&ComponentDescription<'_>>,
    ) -> Result<(), Diagnostics> {
        let nullable_feat: Option<Nullable> =
            pop_feature!(features => Feature::Nullable(_)).into_inner();
        let nullable = nullable_feat
            .map(|nullable| nullable.value())
            .unwrap_or_default();
        let deprecated = pop_feature!(features => Feature::Deprecated(_)).try_to_token_stream()?;

        match type_tree.value_type {
            ValueType::Primitive => {
                let type_path = &**type_tree.path.as_ref().unwrap();
                let schema_type = SchemaType {
                    path: Cow::Borrowed(type_path),
                    nullable,
                };
                if schema_type.is_unsigned_integer() {
                    // add default minimum feature only when there is no explicit minimum
                    // provided
                    if !features
                        .iter()
                        .any(|feature| matches!(&feature, Feature::Minimum(_)))
                    {
                        features.push(Minimum::new(0f64, type_path.span()).into());
                    }
                }

                let schema_type_tokens = as_tokens_or_diagnostics!(&schema_type);
                tokens.extend(quote! {
                    utoipa::openapi::ObjectBuilder::new().schema_type(#schema_type_tokens)
                });

                let format: SchemaFormat = (type_path).into();
                if format.is_known_format() {
                    tokens.extend(quote! {
                        .format(Some(#format))
                    })
                }

                description_stream.to_tokens(tokens);
                tokens.extend(deprecated);
                for feature in features.iter().filter(|feature| feature.is_validatable()) {
                    feature.validate(&schema_type, type_tree);
                }
                tokens.extend(features.to_token_stream()?);
            }
            ValueType::Value => {
                // since OpenAPI 3.1 the type is an array, thus nullable should not be necessary
                // for value type that is going to allow all types of content.
                if type_tree.is_value() {
                    tokens.extend(quote! {
                        utoipa::openapi::ObjectBuilder::new()
                            .schema_type(utoipa::openapi::schema::SchemaType::AnyValue)
                            #description_stream #deprecated
                    })
                }
            }
            ValueType::Object => {
                let is_inline = features.is_inline();

                if type_tree.is_object() {
                    let nullable_schema_type = ComponentSchema::get_schema_type_override(
                        nullable_feat,
                        SchemaTypeInner::Object,
                    );
                    tokens.extend(quote! {
                        utoipa::openapi::ObjectBuilder::new()
                            #nullable_schema_type
                            #description_stream #deprecated
                    })
                } else {
                    fn nullable_all_of_item(nullable: bool) -> Option<TokenStream> {
                        if nullable {
                            Some(
                                quote! { .item(utoipa::openapi::schema::ObjectBuilder::new().schema_type(utoipa::openapi::schema::Type::Null)) },
                            )
                        } else {
                            None
                        }
                    }
                    let type_path = &**type_tree.path.as_ref().unwrap();
                    let nullable_item = nullable_all_of_item(nullable);
                    let mut object_schema_reference = SchemaReference::default();

                    let mut component_name_buffer = String::new();
                    if let Some(children) = &type_tree.children {
                        component_name_buffer.push('_');
                        component_name_buffer.push_str(&Self::compose_name(children))
                    }
                    name_tokens.extend(quote! { format!("{}{}", < #type_path as utoipa::ToSchema >::name(), #component_name_buffer) });

                    object_schema_reference.name = name_tokens.clone();

                    if is_inline {
                        let default = pop_feature!(features => Feature::Default(_));
                        let default_tokens = as_tokens_or_diagnostics!(&default);

                        let items_tokens = if let Some(children) = &type_tree.children {
                            schema_references.extend(Self::compose_child_references(children));

                            let composed_generics =
                                Self::compose_generics(children).collect::<Array<_>>();
                            quote_spanned! {type_path.span()=>
                                <#type_path as utoipa::__dev::ComposeSchema>::compose(#composed_generics.to_vec())
                            }
                        } else {
                            quote_spanned! {type_path.span()=>
                                <#type_path as utoipa::PartialSchema>::schema()
                            }
                        };
                        object_schema_reference.tokens = items_tokens.clone();
                        object_schema_reference.references = quote! { <#type_path as utoipa::__dev::SchemaReferences>::schemas(schemas) };

                        let schema = if default.is_some() || nullable {
                            quote_spanned! {type_path.span()=>
                                utoipa::openapi::schema::AllOfBuilder::new()
                                    #nullable_item
                                    .item(#items_tokens)
                                #default_tokens
                            }
                        } else {
                            items_tokens
                        };

                        schema.to_tokens(tokens);
                    } else {
                        let default = pop_feature!(features => Feature::Default(_));
                        let default_tokens = as_tokens_or_diagnostics!(&default);

                        let index = container.generics.get_generic_type_param_index(type_tree);
                        // only set schema references for concrete non generic types
                        if index.is_none() {
                            object_schema_reference.tokens =
                                quote! {<#type_path as utoipa::PartialSchema>::schema() };
                            object_schema_reference.references = quote! { <#type_path as utoipa::__dev::SchemaReferences>::schemas(schemas) };
                        }
                        let composed_or_ref = |item_tokens: TokenStream| -> TokenStream {
                            if let Some(index) = &index {
                                quote_spanned! {type_path.span()=>
                                    {
                                        let _ = <#type_path as utoipa::PartialSchema>::schema;

                                        if let Some(composed) = generics.get_mut(#index) {
                                            std::mem::take(composed)
                                        } else {
                                            #item_tokens.into()
                                        }
                                    }
                                }
                            } else {
                                quote_spanned! {type_path.span()=>
                                    #item_tokens
                                }
                            }
                        };

                        // TODO: refs support `summary` field but currently there is no such field
                        // on schemas more over there is no way to distinct the `summary` from
                        // `description` of the ref. Should we consider supporting the summary?
                        let schema = if default.is_some() || nullable {
                            composed_or_ref(quote_spanned! {type_path.span()=>
                                utoipa::openapi::schema::AllOfBuilder::new()
                                    #nullable_item
                                    .item(utoipa::openapi::schema::RefBuilder::new()
                                        #description_stream
                                        .ref_location_from_schema_name(#name_tokens)
                                    )
                                    #default_tokens
                            })
                        } else {
                            composed_or_ref(quote_spanned! {type_path.span()=>
                                utoipa::openapi::schema::RefBuilder::new()
                                    #description_stream
                                    .ref_location_from_schema_name(#name_tokens)
                            })
                        };

                        schema.to_tokens(tokens);
                    }

                    schema_references.push(object_schema_reference);
                }
            }
            ValueType::Tuple => {
                type_tree
                    .children
                    .as_ref()
                    .map_try(|children| {
                        let all_of = children
                            .iter()
                            .map(|child| {
                                let features = if child.is_option() {
                                    vec![Feature::Nullable(Nullable::new())]
                                } else {
                                    Vec::new()
                                };

                                match ComponentSchema::new(ComponentSchemaProps {
                                    container,
                                    type_tree: child,
                                    features,
                                    description: None,
                                }) {
                                    Ok(child) => Ok(child.to_token_stream()),
                                    Err(diagnostics) => Err(diagnostics),
                                }
                            })
                            .collect::<Result<Vec<_>, Diagnostics>>()?
                            .into_iter()
                            .fold(
                                quote! { utoipa::openapi::schema::AllOfBuilder::new() },
                                |mut all_of, child_tokens| {
                                    all_of.extend(quote!( .item(#child_tokens) ));

                                    all_of
                                },
                            );

                        let nullable_schema_type = ComponentSchema::get_schema_type_override(
                            nullable_feat,
                            SchemaTypeInner::Array,
                        );
                        Result::<TokenStream, Diagnostics>::Ok(quote! {
                            utoipa::openapi::schema::ArrayBuilder::new()
                                #nullable_schema_type
                                .items(#all_of)
                                #description_stream
                                #deprecated
                        })
                    })?
                    .unwrap_or_else(|| quote!(utoipa::openapi::schema::empty())) // TODO should
                    // this bee type "null"?
                    .to_tokens(tokens);
                tokens.extend(features.to_token_stream());
            }
        }
        Ok(())
    }

    fn compose_name<'tr, I>(children: I) -> String
    where
        I: IntoIterator<Item = &'tr TypeTree<'tr>>,
    {
        children
            .into_iter()
            .map(|type_tree| {
                let mut name = type_tree
                    .path
                    .as_ref()
                    .expect("Generic ValueType::Object must have path")
                    .segments
                    .last()
                    .expect("Generic path must have one segment")
                    .ident
                    .to_string();

                if let Some(children) = &type_tree.children {
                    name.push('_');
                    name.push_str(&Self::compose_name(children));

                    name
                } else {
                    name
                }
            })
            .collect::<Vec<_>>()
            .join("_")
    }

    fn compose_generics<'v, I: IntoIterator<Item = &'v TypeTree<'v>>>(
        children: I,
    ) -> impl Iterator<Item = TokenStream> + 'v
    where
        <I as std::iter::IntoIterator>::IntoIter: 'v,
    {
        children.into_iter().map(|child| {
            let path = child
                .path
                .as_deref()
                .expect("inline TypeTree ValueType::Object must have child path if generic");

            if let Some(children) = &child.children {
                let items = Self::compose_generics(children).collect::<Array<_>>();
                quote! { <#path as utoipa::__dev::ComposeSchema>::compose(#items.to_vec()) }
            } else {
                quote! { <#path as utoipa::PartialSchema>::schema() }
            }
        })
    }

    fn compose_child_references<'a, I: IntoIterator<Item = &'a TypeTree<'a>> + 'a>(
        children: I,
    ) -> impl Iterator<Item = SchemaReference> + 'a {
        children.into_iter().flat_map(|type_tree| {
            if let Some(children) = &type_tree.children {
                ChildRefIter::Iter(Box::new(Self::compose_child_references(children)))
            } else if type_tree.value_type == ValueType::Object {
                let type_path = type_tree
                    .path
                    .as_ref()
                    .expect("Object TypePath must have type path, compose child references").as_ref();

                ChildRefIter::Once(std::iter::once(SchemaReference {
                    name: quote! { String::from(< #type_path as utoipa::ToSchema >::name().as_ref()) },
                    tokens: quote! { <#type_path as utoipa::PartialSchema>::schema() },
                    references: quote !{ <#type_path as utoipa::__dev::SchemaReferences>::schemas(schemas) },
                }))
            } else {
                ChildRefIter::Empty
            }
        })
    }
}

impl ToTokens for ComponentSchema {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.tokens.to_tokens(tokens);
    }
}

enum ChildRefIter<'c, T> {
    Iter(Box<dyn std::iter::Iterator<Item = T> + 'c>),
    Once(std::iter::Once<T>),
    Empty,
}

impl<'a, T> Iterator for ChildRefIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Iter(iter) => iter.next(),
            Self::Once(once) => once.next(),
            Self::Empty => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Iter(iter) => iter.size_hint(),
            Self::Once(once) => once.size_hint(),
            Self::Empty => (0, None),
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct FlattenedMapSchema {
    tokens: TokenStream,
}

impl FlattenedMapSchema {
    pub fn new(
        ComponentSchemaProps {
            container,
            type_tree,
            mut features,
            description,
        }: ComponentSchemaProps,
    ) -> Result<Self, Diagnostics> {
        let mut tokens = TokenStream::new();
        let deprecated = pop_feature!(features => Feature::Deprecated(_)).try_to_token_stream()?;

        let example = features.pop_by(|feature| matches!(feature, Feature::Example(_)));
        let nullable = pop_feature!(features => Feature::Nullable(_));
        let default = pop_feature!(features => Feature::Default(_));
        let default_tokens = as_tokens_or_diagnostics!(&default);

        // Maps are treated as generic objects with no named properties and
        // additionalProperties denoting the type
        // maps have 2 child schemas and we are interested the second one of them
        // which is used to determine the additional properties
        let schema_property = ComponentSchema::new(ComponentSchemaProps {
            container,
            type_tree: type_tree
                .children
                .as_ref()
                .expect("ComponentSchema Map type should have children")
                .get(1)
                .expect("ComponentSchema Map type should have 2 child"),
            features,
            description: None,
        })?;
        let schema_tokens = schema_property.to_token_stream();

        tokens.extend(quote! {
            #schema_tokens
                #description
                #deprecated
                #default_tokens
        });

        example.to_tokens(&mut tokens)?;
        nullable.to_tokens(&mut tokens)?;

        Ok(Self { tokens })
    }
}

impl ToTokensDiagnostics for FlattenedMapSchema {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        self.tokens.to_tokens(tokens);
        Ok(())
    }
}
