use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, TypePath};

use crate::schema_type::SchemaType;
use crate::Array;

pub trait Variant {
    /// Implement `ToTokens` conversion for the [`Variant`]
    fn to_tokens(&self) -> TokenStream;

    /// Get enum varinat type. By default enum variant is `string`
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
    pub name: Cow<'o, str>,
}

impl<T> Variant for ObjectVariant<'_, T>
where
    T: ToTokens,
{
    fn to_tokens(&self) -> TokenStream {
        let title = &self.title;
        let variant = &self.item;
        let name = &self.name;

        quote! {
            utoipa::openapi::schema::ObjectBuilder::new()
                #title
                .property(#name, #variant)
        }
    }
}

pub struct Enum<'e, T: Variant> {
    pub items: &'e [T],
    pub title: Option<TokenStream>,
}

impl<T> ToTokens for Enum<'_, T>
where
    T: Variant,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let len = self.items.len();
        let title = &self.title;
        let (schema_type, enum_type) = self
            .items
            .iter()
            .next()
            .expect("should have at least one enum variant within `Enum` but None was found")
            .get_type();

        let items = self
            .items
            .iter()
            .map(|variant| variant.to_tokens())
            .collect::<Array<TokenStream>>();

        tokens.extend(quote! {
            utoipa::openapi::ObjectBuilder::new()
                #title
                .schema_type(#schema_type)
                .enum_values::<[#enum_type; #len], #enum_type>(Some(#items))
        })
    }
}

pub struct TaggedEnum<'t, T: Variant> {
    pub items: &'t [T],
    // pub title: Option<Cow<'t, str>>,
    pub tag: Cow<'t, str>,
}

impl<'t, T> ToTokens for TaggedEnum<'t, T>
where
    T: Variant,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let len = self.items.len();
        // let title = &self.title;
        let tag = &self.tag;

        let items = self
            .items
            .iter()
            .map(|variant| {
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

        tokens.extend(quote! {
            Into::<utoipa::openapi::schema::OneOfBuilder>::into(utoipa::openapi::OneOf::with_capacity(#len))
                #items
        })
    }
}
