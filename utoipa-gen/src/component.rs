use std::borrow::Cow;

use proc_macro2::Ident;
use proc_macro_error::{abort, abort_call_site};
use syn::{Attribute, GenericArgument, Path, PathArguments, PathSegment, Type, TypePath};

use crate::{schema_type::SchemaType, Deprecated};

pub mod into_params;

pub mod schema;

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
}

/// [`TypeTree`] of items which represents a single parsed `type` of a
/// `Schema`, `Parameter` or `FnArg`
#[cfg_attr(feature = "debug", derive(Debug, PartialEq))]
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
            Type::Array(array) => Self::get_type_paths(&array.elem),
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
                "unexpected type in component part get type path, expected one of: Path, Reference, Group"
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

    fn update_path(&mut self, ident: &'_ Ident) {
        self.path = Some(Cow::Owned(Path::from(ident.clone())))
    }

    /// `Object` virtual type is used when generic object is required in OpenAPI spec. Typically used
    /// with `value_type` attribute to hinder the actual type.
    fn is_object(&self) -> bool {
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

pub mod serde {
    //! Provides serde related features parsing serde attributes from types.

    use std::str::FromStr;

    use proc_macro2::{Ident, Span, TokenTree};
    use proc_macro_error::ResultExt;
    use syn::{buffer::Cursor, Attribute, Error};

    #[inline]
    fn parse_next_lit_str(next: Cursor) -> Option<(String, Span)> {
        match next.token_tree() {
            Some((tt, next)) => match tt {
                TokenTree::Punct(punct) if punct.as_char() == '=' => parse_next_lit_str(next),
                TokenTree::Literal(literal) => {
                    Some((literal.to_string().replace('\"', ""), literal.span()))
                }
                _ => None,
            },
            _ => None,
        }
    }

    #[derive(Default)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct SerdeValue {
        pub skip: Option<bool>,
        pub rename: Option<String>,
        pub default: Option<bool>,
    }

    impl SerdeValue {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let mut value = Self::default();

            input.step(|cursor| {
                let mut rest = *cursor;
                while let Some((tt, next)) = rest.token_tree() {
                    match tt {
                        TokenTree::Ident(ident) if ident == "skip" => value.skip = Some(true),
                        TokenTree::Ident(ident) if ident == "rename" => {
                            if let Some((literal, _)) = parse_next_lit_str(next) {
                                value.rename = Some(literal)
                            };
                        }
                        TokenTree::Ident(ident) if ident == "default" => value.default = Some(true),
                        _ => (),
                    }

                    rest = next;
                }
                Ok(((), rest))
            })?;

            Ok(value)
        }
    }

    /// Attributes defined within a `#[serde(...)]` container attribute.
    #[derive(Default)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct SerdeContainer {
        pub rename_all: Option<RenameRule>,
        pub tag: Option<String>,
        pub default: Option<bool>,
    }

    impl SerdeContainer {
        /// Parse a single serde attribute, currently `rename_all = ...`, `tag = ...` and
        /// `defaut = ...` attributes are supported.
        fn parse_attribute(&mut self, ident: Ident, next: Cursor) -> syn::Result<()> {
            match ident.to_string().as_str() {
                "rename_all" => {
                    if let Some((literal, span)) = parse_next_lit_str(next) {
                        self.rename_all = Some(
                            literal
                                .parse::<RenameRule>()
                                .map_err(|error| Error::new(span, error.to_string()))?,
                        );
                    };
                }
                "tag" => {
                    if let Some((literal, _span)) = parse_next_lit_str(next) {
                        self.tag = Some(literal)
                    }
                }
                "default" => {
                    self.default = Some(true);
                }
                _ => {}
            }
            Ok(())
        }

        /// Parse the attributes inside a `#[serde(...)]` container attribute.
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let mut container = Self::default();

            input.step(|cursor| {
                let mut rest = *cursor;
                while let Some((tt, next)) = rest.token_tree() {
                    if let TokenTree::Ident(ident) = tt {
                        container.parse_attribute(ident, next)?
                    }

                    rest = next;
                }
                Ok(((), rest))
            })?;

            Ok(container)
        }
    }

    pub fn parse_value(attributes: &[Attribute]) -> Option<SerdeValue> {
        attributes
            .iter()
            .find(|attribute| attribute.path.is_ident("serde"))
            .map(|serde_attribute| {
                serde_attribute
                    .parse_args_with(SerdeValue::parse)
                    .unwrap_or_abort()
            })
    }

    pub fn parse_container(attributes: &[Attribute]) -> Option<SerdeContainer> {
        attributes
            .iter()
            .find(|attribute| attribute.path.is_ident("serde"))
            .map(|serde_attribute| {
                serde_attribute
                    .parse_args_with(SerdeContainer::parse)
                    .unwrap_or_abort()
            })
    }

    #[cfg_attr(feature = "debug", derive(Debug))]
    pub enum RenameRule {
        Lower,
        Upper,
        Camel,
        Snake,
        ScreamingSnake,
        Pascal,
        Kebab,
        ScreamingKebab,
    }

    impl RenameRule {
        pub fn rename(&self, value: &str) -> String {
            match self {
                RenameRule::Lower => value.to_ascii_lowercase(),
                RenameRule::Upper => value.to_ascii_uppercase(),
                RenameRule::Camel => {
                    let mut camel_case = String::new();

                    let mut upper = false;
                    for letter in value.chars() {
                        if letter == '_' {
                            upper = true;
                            continue;
                        }

                        if upper {
                            camel_case.push(letter.to_ascii_uppercase());
                            upper = false;
                        } else {
                            camel_case.push(letter)
                        }
                    }

                    camel_case
                }
                RenameRule::Snake => value.to_string(),
                RenameRule::ScreamingSnake => Self::Snake.rename(value).to_ascii_uppercase(),
                RenameRule::Pascal => {
                    let mut pascal_case = String::from(&value[..1].to_ascii_uppercase());
                    pascal_case.push_str(&Self::Camel.rename(&value[1..]));

                    pascal_case
                }
                RenameRule::Kebab => Self::Snake.rename(value).replace('_', "-"),
                RenameRule::ScreamingKebab => Self::Kebab.rename(value).to_ascii_uppercase(),
            }
        }

        pub fn rename_variant(&self, variant: &str) -> String {
            match self {
                RenameRule::Lower => variant.to_ascii_lowercase(),
                RenameRule::Upper => variant.to_ascii_uppercase(),
                RenameRule::Camel => {
                    let mut snake_case = String::from(&variant[..1].to_ascii_lowercase());
                    snake_case.push_str(&variant[1..]);

                    snake_case
                }
                RenameRule::Snake => {
                    let mut snake_case = String::new();

                    for (index, letter) in variant.char_indices() {
                        if index > 0 && letter.is_uppercase() {
                            snake_case.push('_');
                        }
                        snake_case.push(letter);
                    }

                    snake_case.to_ascii_lowercase()
                }
                RenameRule::ScreamingSnake => {
                    Self::Snake.rename_variant(variant).to_ascii_uppercase()
                }
                RenameRule::Pascal => variant.to_string(),
                RenameRule::Kebab => Self::Snake.rename_variant(variant).replace('_', "-"),
                RenameRule::ScreamingKebab => {
                    Self::Kebab.rename_variant(variant).to_ascii_uppercase()
                }
            }
        }
    }

    impl FromStr for RenameRule {
        type Err = Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            [
                ("lowercase", RenameRule::Lower),
                ("UPPERCASE", RenameRule::Upper),
                ("PascalCase", RenameRule::Pascal),
                ("camelCase", RenameRule::Camel),
                ("snake_case", RenameRule::Snake),
                ("SCREAMING_SNAKE_CASE", RenameRule::ScreamingSnake),
                ("kebab-case", RenameRule::Kebab),
                ("SCREAMING-KEBAB-CASE", RenameRule::ScreamingKebab),
            ]
            .into_iter()
            .find_map(|(case, rule)| if case == s { Some(rule) } else { None })
            .ok_or_else(|| {
                Error::new(
                    Span::call_site(),
                    r#"unexpected rename rule, expected one of: "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "SCREAMING_SNAKE_CASE", "kebab-case", "SCREAMING-KEBAB-CASE""#,
                )
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::serde::RenameRule;

    macro_rules! test_rename_rule {
        ( $($case:expr=> $value:literal = $expected:literal)* ) => {
            #[test]
            fn rename_all_rename_rules() {
                $(
                    let value = $case.rename($value);
                    assert_eq!(value, $expected, "expected case: {} => {} != {}", stringify!($case), $value, $expected);
                )*
            }
        };
    }

    macro_rules! test_rename_variant_rule {
        ( $($case:expr=> $value:literal = $expected:literal)* ) => {
            #[test]
            fn rename_all_rename_variant_rules() {
                $(
                    let value = $case.rename_variant($value);
                    assert_eq!(value, $expected, "expected case: {} => {} != {}", stringify!($case), $value, $expected);
                )*
            }
        };
    }

    test_rename_rule! {
        RenameRule::Lower=> "single" = "single"
        RenameRule::Upper=> "single" = "SINGLE"
        RenameRule::Pascal=> "single" = "Single"
        RenameRule::Camel=> "single" = "single"
        RenameRule::Snake=> "single" = "single"
        RenameRule::ScreamingSnake=> "single" = "SINGLE"
        RenameRule::Kebab=> "single" = "single"
        RenameRule::ScreamingKebab=> "single" = "SINGLE"

        RenameRule::Lower=> "multi_value" = "multi_value"
        RenameRule::Upper=> "multi_value" = "MULTI_VALUE"
        RenameRule::Pascal=> "multi_value" = "MultiValue"
        RenameRule::Camel=> "multi_value" = "multiValue"
        RenameRule::Snake=> "multi_value" = "multi_value"
        RenameRule::ScreamingSnake=> "multi_value" = "MULTI_VALUE"
        RenameRule::Kebab=> "multi_value" = "multi-value"
        RenameRule::ScreamingKebab=> "multi_value" = "MULTI-VALUE"
    }

    test_rename_variant_rule! {
        RenameRule::Lower=> "Single" = "single"
        RenameRule::Upper=> "Single" = "SINGLE"
        RenameRule::Pascal=> "Single" = "Single"
        RenameRule::Camel=> "Single" = "single"
        RenameRule::Snake=> "Single" = "single"
        RenameRule::ScreamingSnake=> "Single" = "SINGLE"
        RenameRule::Kebab=> "Single" = "single"
        RenameRule::ScreamingKebab=> "Single" = "SINGLE"

        RenameRule::Lower=> "MultiValue" = "multivalue"
        RenameRule::Upper=> "MultiValue" = "MULTIVALUE"
        RenameRule::Pascal=> "MultiValue" = "MultiValue"
        RenameRule::Camel=> "MultiValue" = "multiValue"
        RenameRule::Snake=> "MultiValue" = "multi_value"
        RenameRule::ScreamingSnake=> "MultiValue" = "MULTI_VALUE"
        RenameRule::Kebab=> "MultiValue" = "multi-value"
        RenameRule::ScreamingKebab=> "MultiValue" = "MULTI-VALUE"
    }

    #[test]
    fn test_serde_rename_rule_from_str() {
        for s in [
            "lowercase",
            "UPPERCASE",
            "PascalCase",
            "camelCase",
            "snake_case",
            "SCREAMING_SNAKE_CASE",
            "kebab-case",
            "SCREAMING-KEBAB-CASE",
        ] {
            s.parse::<RenameRule>().unwrap();
        }
    }
}
