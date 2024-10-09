use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{parse::Parse, Error, Ident, LitStr, Path};

use crate::{Diagnostics, ToTokensDiagnostics};

/// Represents data type of [`Schema`].
#[cfg_attr(feature = "debug", derive(Debug))]
#[allow(dead_code)]
pub enum SchemaTypeInner {
    /// Generic schema type allows "properties" with custom types
    Object,
    /// Indicates string type of content.
    String,
    /// Indicates integer type of content.    
    Integer,
    /// Indicates floating point number type of content.
    Number,
    /// Indicates boolean type of content.
    Boolean,
    /// Indicates array type of content.
    Array,
    /// Null type. Used together with other type to indicate nullable values.
    Null,
}

impl ToTokens for SchemaTypeInner {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = match self {
            Self::Object => quote! { utoipa::openapi::schema::Type::Object },
            Self::String => quote! { utoipa::openapi::schema::Type::String },
            Self::Integer => quote! { utoipa::openapi::schema::Type::Integer },
            Self::Number => quote! { utoipa::openapi::schema::Type::Number },
            Self::Boolean => quote! { utoipa::openapi::schema::Type::Boolean },
            Self::Array => quote! { utoipa::openapi::schema::Type::Array },
            Self::Null => quote! { utoipa::openapi::schema::Type::Null },
        };
        tokens.extend(ty)
    }
}

/// Tokenizes OpenAPI data type correctly according to the Rust type
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct SchemaType<'a> {
    pub path: std::borrow::Cow<'a, syn::Path>,
    pub nullable: bool,
}

impl SchemaType<'_> {
    fn last_segment_to_string(&self) -> String {
        self.path
            .segments
            .last()
            .expect("Expected at least one segment is_integer")
            .ident
            .to_string()
    }

    pub fn is_value(&self) -> bool {
        matches!(&*self.last_segment_to_string(), "Value")
    }

    /// Check whether type is known to be primitive in which case returns true.
    pub fn is_primitive(&self) -> bool {
        let SchemaType { path, .. } = self;
        let last_segment = match path.segments.last() {
            Some(segment) => segment,
            None => return false,
        };
        let name = &*last_segment.ident.to_string();

        #[cfg(not(any(
            feature = "chrono",
            feature = "decimal",
            feature = "decimal_float",
            feature = "rocket_extras",
            feature = "uuid",
            feature = "ulid",
            feature = "url",
            feature = "time",
        )))]
        {
            is_primitive(name)
        }

        #[cfg(any(
            feature = "chrono",
            feature = "decimal",
            feature = "decimal_float",
            feature = "rocket_extras",
            feature = "uuid",
            feature = "ulid",
            feature = "url",
            feature = "time",
        ))]
        {
            let mut primitive = is_primitive(name);

            #[cfg(feature = "chrono")]
            if !primitive {
                primitive = is_primitive_chrono(name);
            }

            #[cfg(any(feature = "decimal", feature = "decimal_float"))]
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

            #[cfg(feature = "ulid")]
            if !primitive {
                primitive = matches!(name, "Ulid");
            }

            #[cfg(feature = "url")]
            if !primitive {
                primitive = matches!(name, "Url");
            }

            #[cfg(feature = "time")]
            if !primitive {
                primitive = matches!(
                    name,
                    "Date" | "PrimitiveDateTime" | "OffsetDateTime" | "Duration"
                );
            }

            primitive
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            &*self.last_segment_to_string(),
            "i8" | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
        )
    }

    pub fn is_unsigned_integer(&self) -> bool {
        matches!(
            &*self.last_segment_to_string(),
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
        )
    }

    pub fn is_number(&self) -> bool {
        match &*self.last_segment_to_string() {
            "f32" | "f64" => true,
            _ if self.is_integer() => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(&*self.last_segment_to_string(), "str" | "String")
    }

    pub fn is_byte(&self) -> bool {
        matches!(&*self.last_segment_to_string(), "u8")
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
#[cfg(feature = "chrono")]
fn is_primitive_chrono(name: &str) -> bool {
    matches!(
        name,
        "DateTime" | "Date" | "NaiveDate" | "NaiveTime" | "Duration" | "NaiveDateTime"
    )
}

#[inline]
#[cfg(any(feature = "decimal", feature = "decimal_float"))]
fn is_primitive_rust_decimal(name: &str) -> bool {
    matches!(name, "Decimal")
}

impl ToTokensDiagnostics for SchemaType<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), Diagnostics> {
        let last_segment = self.path.segments.last().ok_or_else(|| {
            Diagnostics::with_span(
                self.path.span(),
                "schema type should have at least one segment in the path",
            )
        })?;
        let name = &*last_segment.ident.to_string();

        fn schema_type_tokens(
            tokens: &mut TokenStream,
            schema_type: SchemaTypeInner,
            nullable: bool,
        ) {
            if nullable {
                tokens.extend(quote! { utoipa::openapi::schema::SchemaType::from_iter([
                    #schema_type,
                    utoipa::openapi::schema::Type::Null
                ])})
            } else {
                tokens.extend(quote! { utoipa::openapi::schema::SchemaType::new(#schema_type)});
            }
        }

        match name {
            "String" | "str" | "char" => {
                schema_type_tokens(tokens, SchemaTypeInner::String, self.nullable)
            }

            "bool" => schema_type_tokens(tokens, SchemaTypeInner::Boolean, self.nullable),

            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
            | "u128" | "usize" => {
                schema_type_tokens(tokens, SchemaTypeInner::Integer, self.nullable)
            }
            "f32" | "f64" => schema_type_tokens(tokens, SchemaTypeInner::Number, self.nullable),

            #[cfg(feature = "chrono")]
            "DateTime" | "NaiveDateTime" | "NaiveDate" | "NaiveTime" => {
                schema_type_tokens(tokens, SchemaTypeInner::String, self.nullable)
            }

            #[cfg(any(feature = "chrono", feature = "time"))]
            "Date" | "Duration" => {
                schema_type_tokens(tokens, SchemaTypeInner::String, self.nullable)
            }

            #[cfg(feature = "decimal")]
            "Decimal" => schema_type_tokens(tokens, SchemaTypeInner::String, self.nullable),

            #[cfg(feature = "decimal_float")]
            "Decimal" => schema_type_tokens(tokens, SchemaTypeInner::Number, self.nullable),

            #[cfg(feature = "rocket_extras")]
            "PathBuf" => schema_type_tokens(tokens, SchemaTypeInner::String, self.nullable),

            #[cfg(feature = "uuid")]
            "Uuid" => schema_type_tokens(tokens, SchemaTypeInner::String, self.nullable),

            #[cfg(feature = "ulid")]
            "Ulid" => schema_type_tokens(tokens, SchemaTypeInner::String, self.nullable),

            #[cfg(feature = "url")]
            "Url" => schema_type_tokens(tokens, SchemaTypeInner::String, self.nullable),

            #[cfg(feature = "time")]
            "PrimitiveDateTime" | "OffsetDateTime" => {
                schema_type_tokens(tokens, SchemaTypeInner::String, self.nullable)
            }
            _ => schema_type_tokens(tokens, SchemaTypeInner::Object, self.nullable),
        };

        Ok(())
    }
}

/// Either Rust type component variant or enum variant schema variant.
#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum SchemaFormat<'c> {
    /// [`utoipa::openapi::schema::SchemaFormat`] enum variant schema format.
    Variant(Variant),
    /// Rust type schema format.
    Type(Type<'c>),
}

impl SchemaFormat<'_> {
    pub fn is_known_format(&self) -> bool {
        match self {
            Self::Type(ty) => ty.is_known_format(),
            Self::Variant(_) => true,
        }
    }
}

impl<'a> From<&'a Path> for SchemaFormat<'a> {
    fn from(path: &'a Path) -> Self {
        Self::Type(Type(path))
    }
}

impl Parse for SchemaFormat<'_> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self::Variant(input.parse()?))
    }
}

impl ToTokens for SchemaFormat<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Type(ty) => {
                if let Err(diagnostics) = ty.to_tokens(tokens) {
                    diagnostics.to_tokens(tokens)
                }
            }
            Self::Variant(variant) => variant.to_tokens(tokens),
        }
    }
}

/// Tokenizes OpenAPI data type format correctly by given Rust type.
#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Type<'a>(&'a syn::Path);

impl Type<'_> {
    /// Check is the format know format. Known formats can be used within `quote! {...}` statements.
    pub fn is_known_format(&self) -> bool {
        let last_segment = match self.0.segments.last() {
            Some(segment) => segment,
            None => return false,
        };
        let name = &*last_segment.ident.to_string();

        #[cfg(not(any(
            feature = "chrono",
            feature = "decimal_float",
            feature = "uuid",
            feature = "ulid",
            feature = "url",
            feature = "time"
        )))]
        {
            is_known_format(name)
        }

        #[cfg(any(
            feature = "chrono",
            feature = "decimal_float",
            feature = "uuid",
            feature = "ulid",
            feature = "url",
            feature = "time"
        ))]
        {
            let mut known_format = is_known_format(name);

            #[cfg(feature = "chrono")]
            if !known_format {
                known_format = matches!(name, "DateTime" | "Date" | "NaiveDate" | "NaiveDateTime");
            }

            #[cfg(feature = "decimal_float")]
            if !known_format {
                known_format = matches!(name, "Decimal");
            }

            #[cfg(feature = "uuid")]
            if !known_format {
                known_format = matches!(name, "Uuid");
            }

            #[cfg(feature = "ulid")]
            if !known_format {
                known_format = matches!(name, "Ulid");
            }

            #[cfg(feature = "url")]
            if !known_format {
                known_format = matches!(name, "Url");
            }

            #[cfg(feature = "time")]
            if !known_format {
                known_format = matches!(name, "Date" | "PrimitiveDateTime" | "OffsetDateTime");
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

impl ToTokensDiagnostics for Type<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), Diagnostics> {
        let last_segment = self.0.segments.last().ok_or_else(|| {
            Diagnostics::with_span(
                self.0.span(),
                "type should have at least one segment in the path",
            )
        })?;
        let name = &*last_segment.ident.to_string();

        match name {
            #[cfg(feature="non_strict_integers")]
            "i8" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Int8) }),
            #[cfg(feature="non_strict_integers")]
            "u8" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::UInt8) }),
            #[cfg(feature="non_strict_integers")]
            "i16" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Int16) }),
            #[cfg(feature="non_strict_integers")]
            "u16" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::UInt16) }),
            #[cfg(feature="non_strict_integers")]
            #[cfg(feature="non_strict_integers")]
            "u32" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::UInt32) }),
            #[cfg(feature="non_strict_integers")]
            "u64" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::UInt64) }),

            #[cfg(not(feature="non_strict_integers"))]
            "i8" | "i16" | "u8" | "u16" | "u32" => {
                tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Int32) })
            }

            #[cfg(not(feature="non_strict_integers"))]
            "u64" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Int64) }),

            "i32" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Int32) }),
            "i64" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Int64) }),
            "f32" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Float) }),
            "f64" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Double) }),

            #[cfg(feature = "chrono")]
            "NaiveDate" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Date) }),

            #[cfg(feature = "chrono")]
            "DateTime" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::DateTime) }),

            #[cfg(feature = "chrono")]
            "NaiveDateTime" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::DateTime) }),

            #[cfg(any(feature = "chrono", feature = "time"))]
            "Date" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Date) }),

            #[cfg(feature = "decimal_float")]
            "Decimal" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Double) }),

            #[cfg(feature = "uuid")]
            "Uuid" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Uuid) }),

            #[cfg(feature = "ulid")]
            "Ulid" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Ulid) }),

            #[cfg(feature = "url")]
            "Url" => tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Uri) }),

            #[cfg(feature = "time")]
            "PrimitiveDateTime" | "OffsetDateTime" => {
                tokens.extend(quote! { utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::DateTime) })
            }
            _ => (),
        };

        Ok(())
    }
}

/// [`Parse`] and [`ToTokens`] implementation for [`utoipa::openapi::schema::SchemaFormat`].
#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Variant {
    #[cfg(feature = "non_strict_integers")]
    Int8,
    #[cfg(feature = "non_strict_integers")]
    Int16,
    Int32,
    Int64,
    #[cfg(feature = "non_strict_integers")]
    UInt8,
    #[cfg(feature = "non_strict_integers")]
    UInt16,
    #[cfg(feature = "non_strict_integers")]
    UInt32,
    #[cfg(feature = "non_strict_integers")]
    UInt64,
    Float,
    Double,
    Byte,
    Binary,
    Date,
    DateTime,
    Password,
    #[cfg(feature = "uuid")]
    Uuid,
    #[cfg(feature = "ulid")]
    Ulid,
    #[cfg(feature = "url")]
    Uri,
    Custom(String),
}

impl Parse for Variant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let default_formats = [
            "Int32",
            "Int64",
            "Float",
            "Double",
            "Byte",
            "Binary",
            "Date",
            "DateTime",
            "Password",
            #[cfg(feature = "uuid")]
            "Uuid",
            #[cfg(feature = "ulid")]
            "Ulid",
            #[cfg(feature = "url")]
            "Uri",
        ];
        #[cfg(feature = "non_strict_integers")]
        let non_strict_integer_formats = [
            "Int8", "Int16", "Int32", "Int64", "UInt8", "UInt16", "UInt32", "UInt64",
        ];

        #[cfg(feature = "non_strict_integers")]
        let formats = {
            let mut formats = default_formats
                .into_iter()
                .chain(non_strict_integer_formats)
                .collect::<Vec<_>>();
            formats.sort_unstable();
            formats.join(", ")
        };
        #[cfg(not(feature = "non_strict_integers"))]
        let formats = {
            let formats = default_formats.into_iter().collect::<Vec<_>>();
            formats.join(", ")
        };

        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            let format = input.parse::<Ident>()?;
            let name = &*format.to_string();

            match name {
                #[cfg(feature = "non_strict_integers")]
                "Int8" => Ok(Self::Int8),
                #[cfg(feature = "non_strict_integers")]
                "Int16" => Ok(Self::Int16),
                "Int32" => Ok(Self::Int32),
                "Int64" => Ok(Self::Int64),
                #[cfg(feature = "non_strict_integers")]
                "UInt8" => Ok(Self::UInt8),
                #[cfg(feature = "non_strict_integers")]
                "UInt16" => Ok(Self::UInt16),
                #[cfg(feature = "non_strict_integers")]
                "UInt32" => Ok(Self::UInt32),
                #[cfg(feature = "non_strict_integers")]
                "UInt64" => Ok(Self::UInt64),
                "Float" => Ok(Self::Float),
                "Double" => Ok(Self::Double),
                "Byte" => Ok(Self::Byte),
                "Binary" => Ok(Self::Binary),
                "Date" => Ok(Self::Date),
                "DateTime" => Ok(Self::DateTime),
                "Password" => Ok(Self::Password),
                #[cfg(feature = "uuid")]
                "Uuid" => Ok(Self::Uuid),
                #[cfg(feature = "ulid")]
                "Ulid" => Ok(Self::Ulid),
                #[cfg(feature = "url")]
                "Uri" => Ok(Self::Uri),
                _ => Err(Error::new(
                    format.span(),
                    format!("unexpected format: {name}, expected one of: {formats}"),
                )),
            }
        } else if lookahead.peek(LitStr) {
            let value = input.parse::<LitStr>()?.value();
            Ok(Self::Custom(value))
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for Variant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            #[cfg(feature = "non_strict_integers")]
            Self::Int8 => tokens.extend(quote! {utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Int8)}),
            #[cfg(feature = "non_strict_integers")]
            Self::Int16 => tokens.extend(quote! {utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Int16)}),
            Self::Int32 => tokens.extend(quote!(utoipa::openapi::SchemaFormat::KnownFormat(
                utoipa::openapi::KnownFormat::Int32
            ))),
            Self::Int64 => tokens.extend(quote!(utoipa::openapi::SchemaFormat::KnownFormat(
                utoipa::openapi::KnownFormat::Int64
            ))),
            #[cfg(feature = "non_strict_integers")]
            Self::UInt8 => tokens.extend(quote! {utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::UInt8)}),
            #[cfg(feature = "non_strict_integers")]
            Self::UInt16 => tokens.extend(quote! {utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::UInt16)}),
            #[cfg(feature = "non_strict_integers")]
            Self::UInt32 => tokens.extend(quote!(utoipa::openapi::SchemaFormat::KnownFormat(
                utoipa::openapi::KnownFormat::UInt32
            ))),
            #[cfg(feature = "non_strict_integers")]
            Self::UInt64 => tokens.extend(quote!(utoipa::openapi::SchemaFormat::KnownFormat(
                utoipa::openapi::KnownFormat::UInt64
            ))),
            Self::Float => tokens.extend(quote!(utoipa::openapi::SchemaFormat::KnownFormat(
                utoipa::openapi::KnownFormat::Float
            ))),
            Self::Double => tokens.extend(quote!(utoipa::openapi::SchemaFormat::KnownFormat(
                utoipa::openapi::KnownFormat::Double
            ))),
            Self::Byte => tokens.extend(quote!(utoipa::openapi::SchemaFormat::KnownFormat(
                utoipa::openapi::KnownFormat::Byte
            ))),
            Self::Binary => tokens.extend(quote!(utoipa::openapi::SchemaFormat::KnownFormat(
                utoipa::openapi::KnownFormat::Binary
            ))),
            Self::Date => tokens.extend(quote!(utoipa::openapi::SchemaFormat::KnownFormat(
                utoipa::openapi::KnownFormat::Date
            ))),
            Self::DateTime => tokens.extend(quote!(utoipa::openapi::SchemaFormat::KnownFormat(
                utoipa::openapi::KnownFormat::DateTime
            ))),
            Self::Password => tokens.extend(quote!(utoipa::openapi::SchemaFormat::KnownFormat(
                utoipa::openapi::KnownFormat::Password
            ))),
            #[cfg(feature = "uuid")]
            Self::Uuid => tokens.extend(quote!(utoipa::openapi::SchemaFormat::KnownFormat(
                utoipa::openapi::KnownFormat::Uuid
            ))),
            #[cfg(feature = "ulid")]
            Self::Ulid => tokens.extend(quote!(utoipa::openapi::SchemaFormat::KnownFormat(
                utoipa::openapi::KnownFormat::Ulid
            ))),
            #[cfg(feature = "url")]
            Self::Uri => tokens.extend(quote!(utoipa::openapi::SchemaFormat::KnownFormat(
                utoipa::openapi::KnownFormat::Uri
            ))),
            Self::Custom(value) => tokens.extend(quote!(utoipa::openapi::SchemaFormat::Custom(
                String::from(#value)
            ))),
        };
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct PrimitiveType {
    pub ty: syn::Type,
}

impl PrimitiveType {
    pub fn new(path: &Path) -> Option<PrimitiveType> {
        let last_segment = path.segments.last().unwrap_or_else(|| {
            panic!(
                "Path for DefaultType must have at least one segment: `{path}`",
                path = path.to_token_stream()
            )
        });

        let name = &*last_segment.ident.to_string();

        let ty: syn::Type = match name {
            "String" | "str" | "char" => syn::parse_quote!(#path),

            "bool" => syn::parse_quote!(#path),

            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
            | "u128" | "usize" => syn::parse_quote!(#path),
            "f32" | "f64" => syn::parse_quote!(#path),

            #[cfg(feature = "chrono")]
            "DateTime" | "NaiveDateTime" | "NaiveDate" | "NaiveTime" => {
                syn::parse_quote!(String)
            }

            #[cfg(any(feature = "chrono", feature = "time"))]
            "Date" | "Duration" => {
                syn::parse_quote!(String)
            }

            #[cfg(feature = "decimal")]
            "Decimal" => {
                syn::parse_quote!(String)
            }

            #[cfg(feature = "decimal_float")]
            "Decimal" => {
                syn::parse_quote!(f64)
            }

            #[cfg(feature = "rocket_extras")]
            "PathBuf" => {
                syn::parse_quote!(String)
            }

            #[cfg(feature = "uuid")]
            "Uuid" => {
                syn::parse_quote!(String)
            }

            #[cfg(feature = "ulid")]
            "Ulid" => {
                syn::parse_quote!(String)
            }

            #[cfg(feature = "url")]
            "Url" => {
                syn::parse_quote!(String)
            }

            #[cfg(feature = "time")]
            "PrimitiveDateTime" | "OffsetDateTime" => {
                syn::parse_quote!(String)
            }
            _ => {
                // not a primitive type
                return None;
            }
        };

        Some(Self { ty })
    }
}
