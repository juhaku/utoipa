use proc_macro_error::abort_call_site;
use quote::{quote, ToTokens};

/// Tokenizes OpenAPI data type correctly according to the Rust type
pub struct ComponentType<'a>(pub &'a syn::TypePath);

impl ComponentType<'_> {
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
            feature = "uuid"
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
#[cfg(feature = "chrono")]
fn is_primitive_rust_decimal(name: &str) -> bool {
    matches!(name, "Decimal")
}

impl ToTokens for ComponentType<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let last_segment = self.0.path.segments.last().unwrap_or_else(|| {
            abort_call_site!("expected there to be at least one segment in the path")
        });
        let name = &*last_segment.ident.to_string();

        match name {
            "String" | "str" | "char" => {
                tokens.extend(quote! {utoipa::openapi::ComponentType::String})
            }
            "bool" => tokens.extend(quote! { utoipa::openapi::ComponentType::Boolean }),
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
            | "u128" | "usize" => tokens.extend(quote! { utoipa::openapi::ComponentType::Integer }),
            "f32" | "f64" => tokens.extend(quote! { utoipa::openapi::ComponentType::Number }),
            #[cfg(any(feature = "chrono", feature = "chrono_with_format"))]
            "DateTime" | "Date" | "Duration" => {
                tokens.extend(quote! { utoipa::openapi::ComponentType::String })
            }
            #[cfg(feature = "decimal")]
            "Decimal" => tokens.extend(quote! { utoipa::openapi::ComponentType::String }),
            #[cfg(feature = "rocket_extras")]
            "PathBuf" => tokens.extend(quote! { utoipa::openapi::ComponentType::String }),
            #[cfg(feature = "uuid")]
            "Uuid" => tokens.extend(quote! { utoipa::openapi::ComponentType::String }),
            _ => tokens.extend(quote! { utoipa::openapi::ComponentType::Object }),
        }
    }
}

/// Tokenizes OpenAPI data type format correctly by given Rust type.
pub struct ComponentFormat<'a>(pub(crate) &'a syn::TypePath);

impl ComponentFormat<'_> {
    /// Check is the format know format. Known formats can be used within `quote! {...}` statements.
    pub fn is_known_format(&self) -> bool {
        let last_segment = match self.0.path.segments.last() {
            Some(segment) => segment,
            None => return false,
        };
        let name = &*last_segment.ident.to_string();

        #[cfg(not(any(feature = "chrono_with_format", feature = "uuid")))]
        {
            is_known_format(name)
        }

        #[cfg(any(feature = "chrono_with_format", feature = "uuid"))]
        {
            let mut known_format = is_known_format(name);

            #[cfg(feature = "chrono_with_format")]
            if !known_format {
                known_format = matches!(name, "DateTime" | "Date");
            }

            #[cfg(feature = "uuid")]
            if !known_format {
                known_format = matches!(name, "Uuid");
            }

            known_format
        }
    }
}

#[inline]
fn is_known_format(name: &str) -> bool {
    matches!(
        name,
        "i8" | "i16" | "i32" | "u8" | "u16" | "u32" | "i64" | "u64" | "f32" | "f64"
    )
}

impl ToTokens for ComponentFormat<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let last_segment = self.0.path.segments.last().unwrap_or_else(|| {
            abort_call_site!("expected there to be at least one segment in the path")
        });
        let name = &*last_segment.ident.to_string();

        match name {
            "i8" | "i16" | "i32" | "u8" | "u16" | "u32" => {
                tokens.extend(quote! { utoipa::openapi::ComponentFormat::Int32 })
            }
            "i64" | "u64" => tokens.extend(quote! { utoipa::openapi::ComponentFormat::Int64 }),
            "f32" | "f64" => tokens.extend(quote! { utoipa::openapi::ComponentFormat::Float }),
            #[cfg(feature = "chrono_with_format")]
            "DateTime" => tokens.extend(quote! { utoipa::openapi::ComponentFormat::DateTime }),
            #[cfg(feature = "chrono_with_format")]
            "Date" => tokens.extend(quote! { utoipa::openapi::ComponentFormat::Date }),
            #[cfg(feature = "uuid")]
            "Uuid" => tokens.extend(quote! { utoipa::openapi::ComponentFormat::Uuid }),
            _ => (),
        }
    }
}
