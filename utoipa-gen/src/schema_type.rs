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

            #[cfg(feature = "jiff_0_2")]
            if !primitive {
                primitive = matches!(name, "Zoned" | "Date");
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
                tokens.extend(quote! {
                    {
                        use std::iter::FromIterator;
                        utoipa::openapi::schema::SchemaType::from_iter([
                            #schema_type,
                            utoipa::openapi::schema::Type::Null
                        ])
                    }
                })
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

            #[cfg(any(feature = "chrono", feature = "time", feature = "jiff_0_2"))]
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
            #[cfg(feature = "jiff_0_2")]
            "Zoned" => schema_type_tokens(tokens, SchemaTypeInner::String, self.nullable),
            _ => schema_type_tokens(tokens, SchemaTypeInner::Object, self.nullable),
        };

        Ok(())
    }
}

/// [`Parse`] and [`ToTokens`] implementation for [`utoipa::openapi::schema::SchemaFormat`].
#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum KnownFormat {
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
    Duration,
    Password,
    #[cfg(feature = "uuid")]
    Uuid,
    #[cfg(feature = "ulid")]
    Ulid,
    #[cfg(feature = "url")]
    Uri,
    #[cfg(feature = "url")]
    UriReference,
    #[cfg(feature = "url")]
    Iri,
    #[cfg(feature = "url")]
    IriReference,
    Email,
    IdnEmail,
    Hostname,
    IdnHostname,
    Ipv4,
    Ipv6,
    UriTemplate,
    JsonPointer,
    RelativeJsonPointer,
    Regex,
    /// Custom format is reserved only for manual entry.
    Custom(String),
    /// This is not tokenized, but is present for purpose of having some format in
    /// case we do not know the format. E.g. We cannot determine the format based on type path.
    #[allow(unused)]
    Unknown,
}

impl KnownFormat {
    pub fn from_path(path: &syn::Path) -> Result<Self, Diagnostics> {
        let last_segment = path.segments.last().ok_or_else(|| {
            Diagnostics::with_span(
                path.span(),
                "type should have at least one segment in the path",
            )
        })?;
        let name = &*last_segment.ident.to_string();

        let variant = match name {
            #[cfg(feature = "non_strict_integers")]
            "i8" => Self::Int8,
            #[cfg(feature = "non_strict_integers")]
            "u8" => Self::UInt8,
            #[cfg(feature = "non_strict_integers")]
            "i16" => Self::Int16,
            #[cfg(feature = "non_strict_integers")]
            "u16" => Self::UInt16,
            #[cfg(feature = "non_strict_integers")]
            "u32" => Self::UInt32,
            #[cfg(feature = "non_strict_integers")]
            "u64" => Self::UInt64,

            #[cfg(not(feature = "non_strict_integers"))]
            "i8" | "i16" | "u8" | "u16" | "u32" => Self::Int32,

            #[cfg(not(feature = "non_strict_integers"))]
            "u64" => Self::Int64,

            "i32" => Self::Int32,
            "i64" => Self::Int64,
            "f32" => Self::Float,
            "f64" => Self::Double,

            #[cfg(feature = "chrono")]
            "NaiveDate" => Self::Date,

            #[cfg(feature = "chrono")]
            "DateTime" | "NaiveDateTime" => Self::DateTime,

            #[cfg(any(feature = "chrono", feature = "time", feature = "jiff_0_2"))]
            "Date" => Self::Date,

            #[cfg(feature = "decimal_float")]
            "Decimal" => Self::Double,

            #[cfg(feature = "uuid")]
            "Uuid" => Self::Uuid,

            #[cfg(feature = "ulid")]
            "Ulid" => Self::Ulid,

            #[cfg(feature = "url")]
            "Url" => Self::Uri,

            #[cfg(feature = "time")]
            "PrimitiveDateTime" | "OffsetDateTime" => Self::DateTime,

            #[cfg(feature = "jiff_0_2")]
            "Zoned" => Self::DateTime,
            _ => Self::Unknown,
        };

        Ok(variant)
    }

    pub fn is_known_format(&self) -> bool {
        !matches!(self, Self::Unknown)
    }

    fn get_allowed_formats() -> String {
        let default_formats = [
            "Int32",
            "Int64",
            "Float",
            "Double",
            "Byte",
            "Binary",
            "Date",
            "DateTime",
            "Duration",
            "Password",
            #[cfg(feature = "uuid")]
            "Uuid",
            #[cfg(feature = "ulid")]
            "Ulid",
            #[cfg(feature = "url")]
            "Uri",
            #[cfg(feature = "url")]
            "UriReference",
            #[cfg(feature = "url")]
            "Iri",
            #[cfg(feature = "url")]
            "IriReference",
            "Email",
            "IdnEmail",
            "Hostname",
            "IdnHostname",
            "Ipv4",
            "Ipv6",
            "UriTemplate",
            "JsonPointer",
            "RelativeJsonPointer",
            "Regex",
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

        formats
    }
}

impl Parse for KnownFormat {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let formats = KnownFormat::get_allowed_formats();

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
                "Duration" => Ok(Self::Duration),
                "Password" => Ok(Self::Password),
                #[cfg(feature = "uuid")]
                "Uuid" => Ok(Self::Uuid),
                #[cfg(feature = "ulid")]
                "Ulid" => Ok(Self::Ulid),
                #[cfg(feature = "url")]
                "Uri" => Ok(Self::Uri),
                #[cfg(feature = "url")]
                "UriReference" => Ok(Self::UriReference),
                #[cfg(feature = "url")]
                "Iri" => Ok(Self::Iri),
                #[cfg(feature = "url")]
                "IriReference" => Ok(Self::IriReference),
                "Email" => Ok(Self::Email),
                "IdnEmail" => Ok(Self::IdnEmail),
                "Hostname" => Ok(Self::Hostname),
                "IdnHostname" => Ok(Self::IdnHostname),
                "Ipv4" => Ok(Self::Ipv4),
                "Ipv6" => Ok(Self::Ipv6),
                "UriTemplate" => Ok(Self::UriTemplate),
                "JsonPointer" => Ok(Self::JsonPointer),
                "RelativeJsonPointer" => Ok(Self::RelativeJsonPointer),
                "Regex" => Ok(Self::Regex),
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

impl ToTokens for KnownFormat {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            #[cfg(feature = "non_strict_integers")]
            Self::Int8 => tokens.extend(quote! {utoipa::openapi::schema::SchemaFormat::KnownFormat(utoipa::openapi::schema::KnownFormat::Int8)}),
            #[cfg(feature = "non_strict_integers")]
            Self::Int16 => tokens.extend(quote! {utoipa::openapi::schema::SchemaFormat::KnownFormat(utoipa::openapi::schema::KnownFormat::Int16)}),
            Self::Int32 => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Int32
            ))),
            Self::Int64 => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Int64
            ))),
            #[cfg(feature = "non_strict_integers")]
            Self::UInt8 => tokens.extend(quote! {utoipa::openapi::schema::SchemaFormat::KnownFormat(utoipa::openapi::schema::KnownFormat::UInt8)}),
            #[cfg(feature = "non_strict_integers")]
            Self::UInt16 => tokens.extend(quote! {utoipa::openapi::schema::SchemaFormat::KnownFormat(utoipa::openapi::schema::KnownFormat::UInt16)}),
            #[cfg(feature = "non_strict_integers")]
            Self::UInt32 => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::UInt32
            ))),
            #[cfg(feature = "non_strict_integers")]
            Self::UInt64 => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::UInt64
            ))),
            Self::Float => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Float
            ))),
            Self::Double => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Double
            ))),
            Self::Byte => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Byte
            ))),
            Self::Binary => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Binary
            ))),
            Self::Date => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Date
            ))),
            Self::DateTime => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::DateTime
            ))),
            Self::Duration => tokens.extend(quote! {utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Duration
            ) }),
            Self::Password => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Password
            ))),
            #[cfg(feature = "uuid")]
            Self::Uuid => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Uuid
            ))),
            #[cfg(feature = "ulid")]
            Self::Ulid => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Ulid
            ))),
            #[cfg(feature = "url")]
            Self::Uri => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Uri
            ))),
            #[cfg(feature = "url")]
            Self::UriReference => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::UriReference
            ))),
            #[cfg(feature = "url")]
            Self::Iri => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Iri
            ))),
            #[cfg(feature = "url")]
            Self::IriReference => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::IriReference
            ))),
            Self::Email => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Email
            ))),
            Self::IdnEmail => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::IdnEmail
            ))),
            Self::Hostname => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Hostname
            ))),
            Self::IdnHostname => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::IdnHostname
            ))),
            Self::Ipv4 => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Ipv4
            ))),
            Self::Ipv6 => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Ipv6
            ))),
            Self::UriTemplate => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::UriTemplate
            ))),
            Self::JsonPointer => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::JsonPointer
            ))),
            Self::RelativeJsonPointer => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::RelativeJsonPointer
            ))),
            Self::Regex => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::KnownFormat(
                utoipa::openapi::schema::KnownFormat::Regex
            ))),
            Self::Custom(value) => tokens.extend(quote!(utoipa::openapi::schema::SchemaFormat::Custom(
                String::from(#value)
            ))),
            Self::Unknown => (), // unknown we just skip it
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

            #[cfg(any(feature = "chrono", feature = "time", feature = "jiff_0_2"))]
            "Date" => {
                syn::parse_quote!(String)
            }

            #[cfg(any(feature = "chrono", feature = "time"))]
            "Duration" => {
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

            #[cfg(feature = "jiff_0_2")]
            "Zoned" => {
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
