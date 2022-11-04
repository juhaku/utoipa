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
    pub skip: bool,
    pub rename: String,
    pub default: bool,
    pub flatten: bool,
}

impl SerdeValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut value = Self::default();

        input.step(|cursor| {
            let mut rest = *cursor;
            while let Some((tt, next)) = rest.token_tree() {
                match tt {
                    TokenTree::Ident(ident) if ident == "skip" => value.skip = true,
                    TokenTree::Ident(ident) if ident == "flatten" => value.flatten = true,
                    TokenTree::Ident(ident) if ident == "rename" => {
                        if let Some((literal, _)) = parse_next_lit_str(next) {
                            value.rename = literal
                        };
                    }
                    TokenTree::Ident(ident) if ident == "default" => value.default = true,
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
    pub tag: String,
    pub default: bool,
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
                    self.tag = literal
                }
            }
            "default" => {
                self.default = true;
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
            RenameRule::ScreamingSnake => Self::Snake.rename_variant(variant).to_ascii_uppercase(),
            RenameRule::Pascal => variant.to_string(),
            RenameRule::Kebab => Self::Snake.rename_variant(variant).replace('_', "-"),
            RenameRule::ScreamingKebab => Self::Kebab.rename_variant(variant).to_ascii_uppercase(),
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

#[cfg(test)]
mod tests {
    use super::RenameRule;

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
