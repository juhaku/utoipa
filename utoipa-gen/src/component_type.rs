use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use quote::{quote, ToTokens};
use syn::{parse::Parse, Error, Ident, TypePath};

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

/// Either Rust type component variant or enum variant component variant.
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ComponentFormat<'c> {
    /// [`utoipa::openapi::shcema::ComponentFormat`] enum variant component format.
    Variant(Variant),
    /// Rust type component format.
    Type(Type<'c>),
}

impl ComponentFormat<'_> {
    pub fn is_known_format(&self) -> bool {
        match self {
            Self::Type(ty) => ty.is_known_format(),
            Self::Variant(_) => true,
        }
    }
}

impl<'a> From<&'a TypePath> for ComponentFormat<'a> {
    fn from(type_path: &'a TypePath) -> Self {
        Self::Type(Type(type_path))
    }
}

impl Parse for ComponentFormat<'_> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self::Variant(input.parse()?))
    }
}

impl ToTokens for ComponentFormat<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Type(ty) => ty.to_tokens(tokens),
            Self::Variant(variant) => variant.to_tokens(tokens),
        }
    }
}

/// Tokenizes OpenAPI data type format correctly by given Rust type.
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Type<'a>(&'a syn::TypePath);

impl Type<'_> {
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

impl ToTokens for Type<'_> {
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

/// [`Parse`] and [`ToTokens`] implementation for [`utoipa::openapi::schema::ComponentFormat`].
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Variant {
    Int32,
    Int64,
    Float,
    Double,
    Byte,
    Binary,
    Date,
    DateTime,
    Password,
    #[cfg(feature = "uuid")]
    Uuid,
}

impl Parse for Variant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const FORMATS: [&str; 10] = [
            "Int32", "Int64", "Float", "Double", "Byte", "Binary", "Date", "DateTime", "Password",
            "Uuid",
        ];
        let allowed_formats = FORMATS
            .into_iter()
            .filter(|_format| {
                #[cfg(feature = "uuid")]
                {
                    true
                }
                #[cfg(not(feature = "uuid"))]
                {
                    _format != &"Uuid"
                }
            })
            .collect::<Vec<_>>();
        let expected_formats = format!(
            "unexpected format, expected one of: {}",
            allowed_formats.join(", ")
        );
        let format = input.parse::<Ident>()?;
        let name = &*format.to_string();

        match name {
            "Int32" => Ok(Self::Int32),
            "Int64" => Ok(Self::Int64),
            "Float" => Ok(Self::Float),
            "Double" => Ok(Self::Double),
            "Byte" => Ok(Self::Byte),
            "Binary" => Ok(Self::Binary),
            "Date" => Ok(Self::Date),
            "DateTime" => Ok(Self::DateTime),
            "Password" => Ok(Self::Password),
            #[cfg(feature = "uuid")]
            "Uuid" => Ok(Self::Uuid),
            _ => Err(Error::new(
                format.span(),
                format!("unexpected format: {name}, expected one of: {expected_formats}"),
            )),
        }
    }
}

impl ToTokens for Variant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Int32 => tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Int32)),
            Self::Int64 => tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Int64)),
            Self::Float => tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Float)),
            Self::Double => tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Double)),
            Self::Byte => tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Byte)),
            Self::Binary => tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Binary)),
            Self::Date => tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Date)),
            Self::DateTime => {
                tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::DateTime))
            }
            Self::Password => {
                tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Password))
            }
            #[cfg(feature = "uuid")]
            Self::Uuid => tokens.extend(quote!(utoipa::openapi::schema::ComponentFormat::Uuid)),
        };
    }
}
