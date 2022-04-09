use std::fmt::Display;

use quote::{quote, ToTokens};

/// Tokenizes OpenAPI data type correctly according to the Rust type
pub(crate) struct ComponentType<'a, T: Display>(pub &'a T);

impl<'a, T> ComponentType<'a, T>
where
    T: Display,
{
    /// Check whether type is known to be primitive in wich case returns true.
    pub(crate) fn is_primitive(&self) -> bool {
        let name = &*self.0.to_string();

        let primitive = is_primitive(name);

        #[cfg(any(feature = "chrono_types", feature = "chrono_types_with_format"))]
        let mut primitive = primitive;

        #[cfg(any(feature = "chrono_types", feature = "chrono_types_with_format"))]
        if !primitive {
            primitive = is_primitive_chrono(name);
        }

        primitive
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
#[cfg(any(feature = "chrono_types", feature = "chrono_types_with_format"))]
fn is_primitive_chrono(name: &str) -> bool {
    matches!(name, "DateTime" | "Date" | "Duration")
}

impl<'a, T> ToTokens for ComponentType<'a, T>
where
    T: Display,
{
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
            #[cfg(any(feature = "chrono_types", feature = "chrono_types_with_format"))]
            "DateTime" | "Date" | "Duration" => {
                tokens.extend(quote! { utoipa::openapi::ComponentType::String })
            }
            _ => tokens.extend(quote! {utoipa::openapi::ComponentType::Object}),
        }
    }
}

/// Tokenizes OpenAPI data type format correctly by given Rust type.
pub(crate) struct ComponentFormat<T: Display>(pub(crate) T);

impl<T: Display> ComponentFormat<T> {
    /// Check is the format know format. Known formats can be used within `quote! {...}` statements.
    pub(crate) fn is_known_format(&self) -> bool {
        let name = &*self.0.to_string();

        let known_format = is_known_format(name);

        #[cfg(feature = "chrono_types_with_format")]
        let mut known_format = known_format;

        #[cfg(feature = "chrono_types_with_format")]
        if !known_format {
            known_format = matches!(name, "DateTime" | "Date");
        }

        known_format
    }
}

#[inline]
fn is_known_format(name: &str) -> bool {
    matches!(
        name,
        "i8" | "i16" | "i32" | "u8" | "u16" | "u32" | "i64" | "u64" | "f32" | "f64"
    )
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
            #[cfg(feature = "chrono_types_with_format")]
            "DateTime" => tokens.extend(quote! { utoipa::openapi::ComponentFormat::DateTime}),
            #[cfg(feature = "chrono_types_with_format")]
            "Date" => tokens.extend(quote! { utoipa::openapi::ComponentFormat::Date}),
            _ => (),
        }
    }
}
