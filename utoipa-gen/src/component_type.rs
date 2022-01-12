use std::fmt::Display;

use proc_macro2::Ident;
use quote::{quote, ToTokens};

/// Tokenizes OpenAPI data type correctly according to the Rust type
pub(crate) struct ComponentType<'a>(pub &'a Ident);

impl<'a> ComponentType<'a> {
    /// Check whether type is known to be primitive in wich case returns true.
    pub(crate) fn is_primitive(&self) -> bool {
        let name = &*self.0.to_string();

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
}

impl<'a> ToTokens for ComponentType<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &*self.0.to_string();

        match name {
            "String" | "str" | "char" => {
                tokens.extend(quote! {utoipa::openapi::ComponentType::String})
            }
            "bool" => tokens.extend(quote! {utoipa::openapi::ComponentType::Boolean}),
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
            | "u128" | "usize" => tokens.extend(quote! {utoipa::openapi::ComponentType::Integer}),
            "f32" | "f64" => tokens.extend(quote! {utoipa::openapi::ComponentType::Number}),
            _ => tokens.extend(quote! {utoipa::openapi::ComponentType::Object}),
        }
    }
}

// TODO add extendability to this so that custom types can also be tokenized?
/// Tokenizes OpenAPI data type format correctly by given Rust type.
pub(crate) struct ComponentFormat<T: Display>(pub(crate) T);

impl<T: Display> ComponentFormat<T> {
    /// Check is the format know format. Known formats can be used within `quote! {...}` statements.
    pub(crate) fn is_known_format(&self) -> bool {
        let name = &*self.0.to_string();

        matches!(
            name,
            "i8" | "i16" | "i32" | "u8" | "u16" | "u32" | "i64" | "u64" | "f32" | "f64"
        )
    }
}

impl<T: Display> ToTokens for ComponentFormat<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &*self.0.to_string();

        match name {
            "i8" | "i16" | "i32" | "u8" | "u16" | "u32" => {
                tokens.extend(quote! {utoipa::openapi::ComponentFormat::Int32})
            }
            "i64" | "u64" => tokens.extend(quote! {utoipa::openapi::ComponentFormat::Int64}),
            "f32" | "f64" => tokens.extend(quote! {utoipa::openapi::ComponentFormat::Float}),
            _ => (),
        }
    }
}
