use std::borrow::Cow;
use std::marker::PhantomData;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, TypePath};

use crate::component::features::Feature;
use crate::schema_type::SchemaType;
use crate::Array;

pub trait Variant {
    /// Implement `ToTokens` conversion for the [`Variant`]
    fn to_tokens(&self) -> TokenStream;

    /// Get enum variant type. By default enum variant is `string`
    fn get_type(&self) -> (TokenStream, TokenStream) {
        (
            SchemaType(&parse_quote!(str)).to_token_stream(),
            quote! {&str},
        )
    }
}

pub struct SimpleEnumVariant<T: ToTokens> {
    pub value: T,
}

impl<T> Variant for SimpleEnumVariant<T>
where
    T: ToTokens,
{
    fn to_tokens(&self) -> TokenStream {
        self.value.to_token_stream()
    }
}

pub struct ReprVariant<'r, T: ToTokens> {
    pub value: T,
    pub type_path: &'r TypePath,
}

impl<'r, T> Variant for ReprVariant<'r, T>
where
    T: ToTokens,
{
    fn to_tokens(&self) -> TokenStream {
        self.value.to_token_stream()
    }

    fn get_type(&self) -> (TokenStream, TokenStream) {
        (
            SchemaType(&self.type_path.path).to_token_stream(),
            self.type_path.to_token_stream(),
        )
    }
}

pub struct ObjectVariant<'o, T: ToTokens> {
    pub item: T,
    pub title: Option<TokenStream>,
    pub example: Option<TokenStream>,
    pub name: Cow<'o, str>,
}

impl<T> Variant for ObjectVariant<'_, T>
where
    T: ToTokens,
{
    fn to_tokens(&self) -> TokenStream {
        let title = &self.title;
        let example = &self.example;
        let variant = &self.item;
        let name = &self.name;

        quote! {
            utoipa::openapi::schema::ObjectBuilder::new()
                #title
                #example
                .property(#name, #variant)
                .required(#name)
        }
    }
}

pub struct Enum<'e, V: Variant> {
    title: Option<TokenStream>,
    example: Option<TokenStream>,
    len: usize,
    items: Array<'e, TokenStream>,
    schema_type: TokenStream,
    enum_type: TokenStream,
    _p: PhantomData<V>,
}

impl<V: Variant> Enum<'_, V> {
    pub fn new<I: IntoIterator<Item = V>>(items: I) -> Self {
        items.into_iter().collect()
    }

    pub fn with_title<I: Into<TokenStream>>(mut self, title: Option<I>) -> Self {
        self.title = title.map(|title| title.into());

        self
    }

    pub fn with_example<I: Into<TokenStream>>(mut self, example: Option<I>) -> Self {
        self.example = example.map(|example| example.into());

        self
    }
}

impl<T> ToTokens for Enum<'_, T>
where
    T: Variant,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let len = &self.len;
        let title = &self.title;
        let example = &self.example;
        let items = &self.items;
        let schema_type = &self.schema_type;
        let enum_type = &self.enum_type;

        tokens.extend(quote! {
            utoipa::openapi::ObjectBuilder::new()
                #title
                #example
                .schema_type(#schema_type)
                .enum_values::<[#enum_type; #len], #enum_type>(Some(#items))
        })
    }
}

impl<V: Variant> FromIterator<V> for Enum<'_, V> {
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        let mut len = 0;
        let mut schema_type: TokenStream = quote! {};
        let mut enum_type: TokenStream = quote! {};

        let items = iter
            .into_iter()
            .enumerate()
            .map(|(index, variant)| {
                if index == 0 {
                    (schema_type, enum_type) = variant.get_type();
                }
                len = index + 1;
                variant.to_tokens()
            })
            .collect::<Array<TokenStream>>();

        Self {
            title: None,
            example: None,
            len,
            items,
            schema_type,
            enum_type,
            _p: PhantomData,
        }
    }
}

pub struct TaggedEnum<T: Variant> {
    items: TokenStream,
    len: usize,
    _p: PhantomData<T>,
}

impl<V: Variant> TaggedEnum<V> {
    pub fn new<'t, I: IntoIterator<Item = (Cow<'t, str>, V)>>(items: I) -> Self {
        items.into_iter().collect()
    }
}

impl<T> ToTokens for TaggedEnum<T>
where
    T: Variant,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let len = &self.len;
        let items = &self.items;

        tokens.extend(quote! {
            Into::<utoipa::openapi::schema::OneOfBuilder>::into(utoipa::openapi::OneOf::with_capacity(#len))
                #items
        })
    }
}

impl<'t, V: Variant> FromIterator<(Cow<'t, str>, V)> for TaggedEnum<V> {
    fn from_iter<T: IntoIterator<Item = (Cow<'t, str>, V)>>(iter: T) -> Self {
        let mut len = 0;

        let items = iter
            .into_iter()
            .enumerate()
            .map(|(index, (tag, variant))| {
                len = index + 1;

                let (schema_type, enum_type) = variant.get_type();
                let item = variant.to_tokens();
                quote! {
                    .item(
                        utoipa::openapi::schema::ObjectBuilder::new()
                            .property(
                                #tag,
                                utoipa::openapi::schema::ObjectBuilder::new()
                                    .schema_type(#schema_type)
                                    .enum_values::<[#enum_type; 1], #enum_type>(Some([#item]))
                            )
                            .required(#tag)
                    )
                }
            })
            .collect::<TokenStream>();

        Self {
            items,
            len,
            _p: PhantomData,
        }
    }
}

pub struct UntaggedEnum {
    title: Option<Feature>,
}

impl UntaggedEnum {
    pub fn new() -> Self {
        Self { title: None }
    }

    pub fn with_title(title: Option<Feature>) -> Self {
        Self { title }
    }
}

impl ToTokens for UntaggedEnum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let title = &self.title;

        tokens.extend(quote! {
            utoipa::openapi::schema::ObjectBuilder::new()
                .nullable(true)
                .default(Some(serde_json::Value::Null))
                #title
        })
    }
}

pub struct AdjacentlyTaggedEnum<T: Variant> {
    items: TokenStream,
    len: usize,
    _p: PhantomData<T>,
}

impl<V: Variant> AdjacentlyTaggedEnum<V> {
    pub fn new<'t, I: IntoIterator<Item = (Cow<'t, str>, Cow<'t, str>, V)>>(items: I) -> Self {
        items.into_iter().collect()
    }
}

impl<T> ToTokens for AdjacentlyTaggedEnum<T>
where
    T: Variant,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let len = &self.len;
        let items = &self.items;

        tokens.extend(quote! {
            Into::<utoipa::openapi::schema::OneOfBuilder>::into(utoipa::openapi::OneOf::with_capacity(#len))
                #items
        })
    }
}

impl<'t, V: Variant> FromIterator<(Cow<'t, str>, Cow<'t, str>, V)> for AdjacentlyTaggedEnum<V> {
    fn from_iter<T: IntoIterator<Item = (Cow<'t, str>, Cow<'t, str>, V)>>(iter: T) -> Self {
        let mut len = 0;

        let items = iter
            .into_iter()
            .enumerate()
            .map(|(index, (tag, content, variant))| {
                len = index + 1;

                let (schema_type, enum_type) = variant.get_type();
                let item = variant.to_tokens();
                quote! {
                    .item(
                        utoipa::openapi::schema::ObjectBuilder::new()
                            .property(
                                #tag,
                                utoipa::openapi::schema::ObjectBuilder::new()
                                    .schema_type(utoipa::openapi::schema::SchemaType::String)
                                    .enum_values::<[#enum_type; 1], #enum_type>(Some([#content]))
                            )
                            .required(#tag)
                            .property(
                                #content,
                                utoipa::openapi::schema::ObjectBuilder::new()
                                    .schema_type(#schema_type)
                                    .enum_values::<[#enum_type; 1], #enum_type>(Some([#item]))
                            )
                            .required(#content)
                    )
                }
            })
            .collect::<TokenStream>();

        Self {
            items,
            len,
            _p: PhantomData,
        }
    }
}

/// Used to create complex enums with varying Object types.
///
/// Will create `oneOf` object with discriminator field for referenced schemas.
pub struct CustomEnum<'c, T: ToTokens> {
    // pub items: Cow<'c, >,
    items: T,
    tag: Option<Cow<'c, str>>,
}

impl<'c, T: ToTokens> CustomEnum<'c, T> {
    pub fn with_discriminator(mut self, discriminator: Option<Cow<'c, str>>) -> Self {
        self.tag = discriminator;

        self
    }
}

impl<'c, T> ToTokens for CustomEnum<'c, T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.items.to_tokens(tokens);

        // currently uses serde `tag` attribute as a discriminator. This discriminator
        // feature needs some refinement.
        let discriminator = self.tag.as_ref().map(|tag| {
            quote! {
                .discriminator(Some(utoipa::openapi::schema::Discriminator::new(#tag)))
            }
        });

        tokens.extend(quote! {
            #discriminator
        });
    }
}

impl FromIterator<TokenStream> for CustomEnum<'_, TokenStream> {
    fn from_iter<T: IntoIterator<Item = TokenStream>>(iter: T) -> Self {
        let mut len = 0;

        let items = iter
            .into_iter()
            .enumerate()
            .map(|(index, variant)| {
                len = index + 1;
                quote! {
                    .item(
                        #variant
                    )
                }
            })
            .collect::<TokenStream>();

        let mut tokens = TokenStream::new();

        tokens.extend(quote! {
            Into::<utoipa::openapi::schema::OneOfBuilder>::into(utoipa::openapi::OneOf::with_capacity(#len))
                #items
        });

        CustomEnum {
            items: tokens,
            tag: None,
        }
    }
}
