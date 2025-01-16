use std::borrow::Cow;
use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Comma, Paren};
use syn::{Error, Generics, Ident, Token, Type};

use crate::component::features::attributes::{Extensions, Inline};
use crate::component::features::Feature;
use crate::component::{ComponentSchema, ComponentSchemaProps, Container, TypeTree, ValueType};
use crate::ext::ExtSchema;
use crate::{parse_utils, AnyValue, Array, Diagnostics, ToTokensDiagnostics};

use super::example::Example;
use super::PathTypeTree;

pub mod encoding;

use encoding::Encoding;

/// Parse OpenAPI Media Type object params
/// ( Schema )
/// ( Schema = "content/type" )
/// ( "content/type", ),
/// ( "content/type", example = ..., examples(..., ...), encoding(("exampleField" = (...)), ...), extensions(("x-ext" = json!(...))) )
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct MediaTypeAttr<'m> {
    pub content_type: Option<parse_utils::LitStrOrExpr>, // if none, true guess
    pub schema: Schema<'m>,
    pub example: Option<AnyValue>,
    pub examples: Punctuated<Example, Comma>,
    pub encoding: BTreeMap<String, Encoding>,
    pub extensions: Option<Extensions>,
}

impl Parse for MediaTypeAttr<'_> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut media_type = MediaTypeAttr::default();

        let fork = input.fork();
        let is_schema = fork.parse::<DefaultSchema>().is_ok();
        if is_schema {
            let schema = input.parse::<DefaultSchema>()?;

            let content_type = if input.parse::<Option<Token![=]>>()?.is_some() {
                Some(
                    input
                        .parse::<parse_utils::LitStrOrExpr>()
                        .map_err(|error| {
                            Error::new(
                                error.span(),
                                format!(
                                    r#"missing content type e.g. `"application/json"`, {error}"#
                                ),
                            )
                        })?,
                )
            } else {
                None
            };
            media_type.schema = Schema::Default(schema);
            media_type.content_type = content_type;
        } else {
            // if schema, the content type is required
            let content_type = input
                .parse::<parse_utils::LitStrOrExpr>()
                .map_err(|error| {
                    Error::new(
                        error.span(),
                        format!("unexpected content, should be `schema`, `schema = content_type` or `content_type`, {error}"),
                    )
                })?;
            media_type.content_type = Some(content_type);
        }

        if !input.is_empty() {
            input.parse::<Comma>()?;
        }

        while !input.is_empty() {
            let attribute = input.parse::<Ident>()?;
            MediaTypeAttr::parse_named_attributes(&mut media_type, input, &attribute)?;
        }

        Ok(media_type)
    }
}

impl<'m> MediaTypeAttr<'m> {
    pub fn parse_schema(input: ParseStream) -> syn::Result<DefaultSchema<'m>> {
        input.parse()
    }

    pub fn parse_named_attributes(
        &mut self,
        input: ParseStream,
        attribute: &Ident,
    ) -> syn::Result<()> {
        let name = &*attribute.to_string();

        match name {
            "example" => {
                self.example = Some(parse_utils::parse_next(input, || {
                    AnyValue::parse_any(input)
                })?)
            }
            "examples" => {
                self.examples = parse_utils::parse_comma_separated_within_parenthesis(input)?
            }
            "encoding" => {
                struct KV {
                    k: String,
                    v: Encoding,
                }

                impl Parse for KV {
                    fn parse(input: ParseStream) -> syn::Result<Self> {
                        let key_val;

                        syn::parenthesized!(key_val in input);

                        let k = key_val.parse::<syn::LitStr>()?.value();

                        key_val.parse::<Token![=]>()?;

                        let v = key_val.parse::<Encoding>()?;

                        if !key_val.is_empty() {
                            key_val.parse::<Comma>()?;
                        }

                        Ok(KV{k, v})
                    }
                }

                let fields = parse_utils::parse_comma_separated_within_parenthesis::<KV>(input)?;

                self.encoding = fields.into_iter().map(|x| (x.k, x.v)).collect();
            }
            "extensions" => {
                self.extensions = Some(input.parse::<Extensions>()?);
            }
            unexpected => {
                return Err(syn::Error::new(
                    attribute.span(),
                    format!(
                        "unexpected attribute: {unexpected}, expected any of: example, examples, encoding(...), extensions(...)"
                    ),
                ))
            }
        }

        if !input.is_empty() {
            input.parse::<Comma>()?;
        }

        Ok(())
    }
}

impl ToTokensDiagnostics for MediaTypeAttr<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), Diagnostics> {
        let schema = &self.schema.try_to_token_stream()?;
        let schema_tokens = if schema.is_empty() {
            None
        } else {
            Some(quote! { .schema(Some(#schema)) })
        };
        let example = self
            .example
            .as_ref()
            .map(|example| quote!( .example(Some(#example)) ));

        let examples = self
            .examples
            .iter()
            .map(|example| {
                let name = &example.name;
                quote!( (#name, #example) )
            })
            .collect::<Array<TokenStream>>();
        let examples = if !examples.is_empty() {
            Some(quote!( .examples_from_iter(#examples) ))
        } else {
            None
        };
        let encoding = self
            .encoding
            .iter()
            .map(|(field_name, encoding)| quote!(.encoding(#field_name, #encoding)));
        let extensions = self
            .extensions
            .as_ref()
            .map(|e| quote! { .extensions(Some(#e)) });

        tokens.extend(quote! {
            utoipa::openapi::content::ContentBuilder::new()
                #schema_tokens
                #example
                #examples
                #(#encoding)*
                #extensions
                .into()
        });

        Ok(())
    }
}

pub trait MediaTypePathExt<'a> {
    fn get_component_schema(&self) -> Result<Option<ComponentSchema>, Diagnostics>;
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[allow(unused)]
pub enum Schema<'a> {
    Default(DefaultSchema<'a>),
    Ext(ExtSchema<'a>),
}

impl Default for Schema<'_> {
    fn default() -> Self {
        Self::Default(DefaultSchema::None)
    }
}

impl Schema<'_> {
    pub fn get_type_tree(&self) -> Result<Option<Cow<TypeTree<'_>>>, Diagnostics> {
        match self {
            Self::Default(def) => def.get_type_tree(),
            Self::Ext(ext) => ext.get_type_tree(),
        }
    }

    pub fn get_default_content_type(&self) -> Result<Cow<'static, str>, Diagnostics> {
        match self {
            Self::Default(def) => def.get_default_content_type(),
            Self::Ext(ext) => ext.get_default_content_type(),
        }
    }

    pub fn get_component_schema(&self) -> Result<Option<ComponentSchema>, Diagnostics> {
        match self {
            Self::Default(def) => def.get_component_schema(),
            Self::Ext(ext) => ext.get_component_schema(),
        }
    }

    pub fn is_inline(&self) -> bool {
        match self {
            Self::Default(def) => match def {
                DefaultSchema::TypePath(parsed) => parsed.is_inline,
                _ => false,
            },
            Self::Ext(_) => false,
        }
    }
}

impl ToTokensDiagnostics for Schema<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        match self {
            Self::Default(def) => def.to_tokens(tokens)?,
            Self::Ext(ext) => ext.to_tokens(tokens)?,
        }

        Ok(())
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Default)]
pub enum DefaultSchema<'d> {
    Ref(parse_utils::LitStrOrExpr),
    TypePath(ParsedType<'d>),
    /// for cases where the schema is irrelevant but we just want to return generic
    /// `content_type` without actual schema.
    #[default]
    None,
    /// Support for raw tokens as Schema. Used in response derive.
    Raw {
        tokens: TokenStream,
        ty: Cow<'d, Type>,
    },
}

impl ToTokensDiagnostics for DefaultSchema<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), Diagnostics> {
        match self {
            Self::Ref(reference) => tokens.extend(quote! {
                utoipa::openapi::schema::Ref::new(#reference)
            }),
            Self::TypePath(parsed) => {
                let is_inline = parsed.is_inline;
                let type_tree = &parsed.to_type_tree()?;

                let component_tokens = ComponentSchema::new(ComponentSchemaProps {
                    type_tree,
                    features: vec![Inline::from(is_inline).into()],
                    description: None,
                    container: &Container {
                        generics: &Generics::default(),
                    },
                })?
                .to_token_stream();

                component_tokens.to_tokens(tokens);
            }
            Self::Raw {
                tokens: raw_tokens, ..
            } => {
                raw_tokens.to_tokens(tokens);
            }
            // nada
            Self::None => (),
        }

        Ok(())
    }
}

impl<'a> MediaTypePathExt<'a> for TypeTree<'a> {
    fn get_component_schema(&self) -> Result<Option<ComponentSchema>, Diagnostics> {
        let generics = &if matches!(self.value_type, ValueType::Tuple) {
            Generics::default()
        } else {
            self.get_path_generics()?
        };

        let component_schema = ComponentSchema::new(ComponentSchemaProps {
            container: &Container { generics },
            type_tree: self,
            description: None,
            // get the actual schema, not the reference
            features: vec![Feature::Inline(true.into())],
        })?;

        Ok(Some(component_schema))
    }
}

impl DefaultSchema<'_> {
    pub fn get_default_content_type(&self) -> Result<Cow<'static, str>, Diagnostics> {
        match self {
            Self::TypePath(path) => {
                let type_tree = path.to_type_tree()?;
                Ok(type_tree.get_default_content_type())
            }
            Self::Ref(_) => Ok(Cow::Borrowed("application/json")),
            Self::Raw { ty, .. } => {
                let type_tree = TypeTree::from_type(ty.as_ref())?;
                Ok(type_tree.get_default_content_type())
            }
            Self::None => Ok(Cow::Borrowed("")),
        }
    }

    pub fn get_component_schema(&self) -> Result<Option<ComponentSchema>, Diagnostics> {
        match self {
            Self::TypePath(path) => {
                let type_tree = path.to_type_tree()?;
                let v = type_tree.get_component_schema()?;

                Ok(v)
            }
            _ => Ok(None),
        }
    }

    pub fn get_type_tree(&self) -> Result<Option<Cow<'_, TypeTree<'_>>>, Diagnostics> {
        match self {
            Self::TypePath(path) => path
                .to_type_tree()
                .map(|type_tree| Some(Cow::Owned(type_tree))),
            _ => Ok(None),
        }
    }
}

impl Parse for DefaultSchema<'_> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        let is_ref = if (fork.parse::<Option<Token![ref]>>()?).is_some() {
            fork.peek(Paren)
        } else {
            false
        };

        if is_ref {
            input.parse::<Token![ref]>()?;
            let ref_stream;
            syn::parenthesized!(ref_stream in input);

            ref_stream.parse().map(Self::Ref)
        } else {
            input.parse().map(Self::TypePath)
        }
    }
}

impl<'r> From<ParsedType<'r>> for Schema<'r> {
    fn from(value: ParsedType<'r>) -> Self {
        Self::Default(DefaultSchema::TypePath(value))
    }
}

// inline(syn::TypePath) | syn::TypePath
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ParsedType<'i> {
    pub ty: Cow<'i, Type>,
    pub is_inline: bool,
}

impl ParsedType<'_> {
    /// Get's the underlying [`syn::Type`] as [`TypeTree`].
    fn to_type_tree(&self) -> Result<TypeTree, Diagnostics> {
        TypeTree::from_type(&self.ty)
    }
}

impl Parse for ParsedType<'_> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        let is_inline = if let Some(ident) = fork.parse::<Option<syn::Ident>>()? {
            ident == "inline" && fork.peek(Paren)
        } else {
            false
        };

        let ty = if is_inline {
            input.parse::<syn::Ident>()?;
            let inlined;
            syn::parenthesized!(inlined in input);

            inlined.parse::<Type>()?
        } else {
            input.parse::<Type>()?
        };

        Ok(ParsedType {
            ty: Cow::Owned(ty),
            is_inline,
        })
    }
}
