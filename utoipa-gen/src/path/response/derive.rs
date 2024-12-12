use std::borrow::Cow;
use std::{iter, mem};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{
    Attribute, Data, Field, Fields, Generics, Lifetime, LifetimeParam, LitStr, Path, Type,
    TypePath, Variant,
};

use crate::component::schema::{EnumSchema, NamedStructSchema, Root};
use crate::doc_comment::CommentAttributes;
use crate::path::media_type::{DefaultSchema, MediaTypeAttr, ParsedType, Schema};
use crate::{
    as_tokens_or_diagnostics, parse_utils, Array, Diagnostics, OptionExt, ToTokensDiagnostics,
};

use super::{
    DeriveIntoResponsesValue, DeriveResponseValue, DeriveToResponseValue, ResponseTuple,
    ResponseTupleInner, ResponseValue,
};

pub struct ToResponse<'r> {
    ident: Ident,
    lifetime: Lifetime,
    generics: Generics,
    response: ResponseTuple<'r>,
}

impl<'r> ToResponse<'r> {
    const LIFETIME: &'static str = "'__r";

    pub fn new(
        attributes: Vec<Attribute>,
        data: &'r Data,
        generics: Generics,
        ident: Ident,
    ) -> Result<ToResponse<'r>, Diagnostics> {
        let response = match &data {
            Data::Struct(struct_value) => match &struct_value.fields {
                Fields::Named(fields) => {
                    ToResponseNamedStructResponse::new(&attributes, &ident, &fields.named)?.0
                }
                Fields::Unnamed(fields) => {
                    let field = fields
                        .unnamed
                        .iter()
                        .next()
                        .expect("Unnamed struct must have 1 field");

                    ToResponseUnnamedStructResponse::new(&attributes, &field.ty, &field.attrs)?.0
                }
                Fields::Unit => ToResponseUnitStructResponse::new(&attributes)?.0,
            },
            Data::Enum(enum_value) => {
                EnumResponse::new(&ident, &enum_value.variants, &attributes)?.0
            }
            Data::Union(_) => {
                return Err(Diagnostics::with_span(
                    ident.span(),
                    "`ToResponse` does not support `Union` type",
                ))
            }
        };

        let lifetime = Lifetime::new(ToResponse::LIFETIME, Span::call_site());

        Ok(Self {
            ident,
            lifetime,
            generics,
            response,
        })
    }
}

impl ToTokensDiagnostics for ToResponse<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        let (_, ty_generics, where_clause) = self.generics.split_for_impl();

        let lifetime = &self.lifetime;
        let ident = &self.ident;
        let name = ident.to_string();
        let response = as_tokens_or_diagnostics!(&self.response);

        let mut to_response_generics = self.generics.clone();
        to_response_generics
            .params
            .push(syn::GenericParam::Lifetime(LifetimeParam::new(
                lifetime.clone(),
            )));
        let (to_response_impl_generics, _, _) = to_response_generics.split_for_impl();

        tokens.extend(quote! {
            impl #to_response_impl_generics utoipa::ToResponse <#lifetime> for #ident #ty_generics #where_clause {
                fn response() -> (& #lifetime str, utoipa::openapi::RefOr<utoipa::openapi::response::Response>) {
                    (#name, #response.into())
                }
            }
        });

        Ok(())
    }
}

pub struct IntoResponses {
    pub attributes: Vec<Attribute>,
    pub data: Data,
    pub generics: Generics,
    pub ident: Ident,
}

impl ToTokensDiagnostics for IntoResponses {
    fn to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostics> {
        let responses = match &self.data {
            Data::Struct(struct_value) => match &struct_value.fields {
                Fields::Named(fields) => {
                    let response =
                        NamedStructResponse::new(&self.attributes, &self.ident, &fields.named)?.0;
                    let status = &response.status_code;
                    let response_tokens = as_tokens_or_diagnostics!(&response);

                    Array::from_iter(iter::once(quote!((#status, #response_tokens))))
                }
                Fields::Unnamed(fields) => {
                    let field = fields
                        .unnamed
                        .iter()
                        .next()
                        .expect("Unnamed struct must have 1 field");

                    let response =
                        UnnamedStructResponse::new(&self.attributes, &field.ty, &field.attrs)?.0;
                    let status = &response.status_code;
                    let response_tokens = as_tokens_or_diagnostics!(&response);

                    Array::from_iter(iter::once(quote!((#status, #response_tokens))))
                }
                Fields::Unit => {
                    let response = UnitStructResponse::new(&self.attributes)?.0;
                    let status = &response.status_code;
                    let response_tokens = as_tokens_or_diagnostics!(&response);

                    Array::from_iter(iter::once(quote!((#status, #response_tokens))))
                }
            },
            Data::Enum(enum_value) => enum_value
                .variants
                .iter()
                .map(|variant| match &variant.fields {
                    Fields::Named(fields) => Ok(NamedStructResponse::new(
                        &variant.attrs,
                        &variant.ident,
                        &fields.named,
                    )?
                    .0),
                    Fields::Unnamed(fields) => {
                        let field = fields
                            .unnamed
                            .iter()
                            .next()
                            .expect("Unnamed enum variant must have 1 field");
                        match UnnamedStructResponse::new(&variant.attrs, &field.ty, &field.attrs) {
                            Ok(response) => Ok(response.0),
                            Err(diagnostics) => Err(diagnostics),
                        }
                    }
                    Fields::Unit => Ok(UnitStructResponse::new(&variant.attrs)?.0),
                })
                .collect::<Result<Vec<ResponseTuple>, Diagnostics>>()?
                .iter()
                .map(|response| {
                    let status = &response.status_code;
                    let response_tokens = as_tokens_or_diagnostics!(response);
                    Ok(quote!((#status, utoipa::openapi::RefOr::from(#response_tokens))))
                })
                .collect::<Result<Array<TokenStream>, Diagnostics>>()?,
            Data::Union(_) => {
                return Err(Diagnostics::with_span(
                    self.ident.span(),
                    "`IntoResponses` does not support `Union` type",
                ))
            }
        };

        let ident = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let responses = if responses.len() > 0 {
            Some(quote!( .responses_from_iter(#responses)))
        } else {
            None
        };
        tokens.extend(quote!{
                impl #impl_generics utoipa::IntoResponses for #ident #ty_generics #where_clause {
                    fn responses() -> std::collections::BTreeMap<String, utoipa::openapi::RefOr<utoipa::openapi::response::Response>> {
                        utoipa::openapi::response::ResponsesBuilder::new()
                            #responses
                            .build()
                            .into()
                    }
                }
            });

        Ok(())
    }
}

trait Response {
    fn to_type(ident: &Ident) -> Type {
        let path = Path::from(ident.clone());
        let type_path = TypePath { path, qself: None };
        Type::Path(type_path)
    }

    fn has_no_field_attributes(attribute: &Attribute) -> (bool, &'static str) {
        const ERROR: &str =
            "Unexpected field attribute, field attributes are only supported at unnamed fields";

        let ident = attribute.path().get_ident().unwrap();
        match &*ident.to_string() {
            "to_schema" => (false, ERROR),
            "ref_response" => (false, ERROR),
            "content" => (false, ERROR),
            "to_response" => (false, ERROR),
            _ => (true, ERROR),
        }
    }

    fn validate_attributes<'a, I: IntoIterator<Item = &'a Attribute>>(
        attributes: I,
        validate: impl Fn(&Attribute) -> (bool, &'static str) + 'a,
    ) -> impl Iterator<Item = Diagnostics> {
        attributes.into_iter().filter_map(move |attribute| {
            let (valid, error_message) = validate(attribute);
            if !valid {
                Some(Diagnostics::with_span(attribute.span(), error_message))
            } else {
                None
            }
        })
    }
}

struct UnnamedStructResponse<'u>(ResponseTuple<'u>);

impl Response for UnnamedStructResponse<'_> {}

impl<'u> UnnamedStructResponse<'u> {
    fn new(
        attributes: &[Attribute],
        ty: &'u Type,
        inner_attributes: &[Attribute],
    ) -> Result<Self, Diagnostics> {
        let is_inline = inner_attributes
            .iter()
            .any(|attribute| attribute.path().get_ident().unwrap() == "to_schema");
        let ref_response = inner_attributes
            .iter()
            .any(|attribute| attribute.path().get_ident().unwrap() == "ref_response");
        let to_response = inner_attributes
            .iter()
            .any(|attribute| attribute.path().get_ident().unwrap() == "to_response");

        if is_inline && (ref_response || to_response) {
            return Err(Diagnostics::with_span(ty.span(), "Attribute `to_schema` cannot be used with `ref_response` and `to_response` attribute"));
        }
        let mut derive_value = DeriveIntoResponsesValue::from_attributes(attributes)?
            .expect("`IntoResponses` must have `#[response(...)]` attribute");
        let description = {
            let s = CommentAttributes::from_attributes(attributes).as_formatted_string();
            parse_utils::LitStrOrExpr::LitStr(LitStr::new(&s, Span::call_site()))
        };
        let status_code = mem::take(&mut derive_value.status);

        let response = match (ref_response, to_response) {
            (false, false) => Self(
                (
                    status_code,
                    ResponseValue::from_derive_into_responses_value(
                        derive_value,
                        ParsedType {
                            ty: Cow::Borrowed(ty),
                            is_inline,
                        },
                        description,
                    ),
                )
                    .into(),
            ),
            (true, false) => Self(ResponseTuple {
                inner: Some(ResponseTupleInner::Ref(ParsedType {
                    ty: Cow::Borrowed(ty),
                    is_inline: false,
                })),
                status_code,
            }),
            (false, true) => Self(ResponseTuple {
                inner: Some(ResponseTupleInner::Ref(ParsedType {
                    ty: Cow::Borrowed(ty),
                    is_inline: true,
                })),
                status_code,
            }),
            (true, true) => {
                return Err(Diagnostics::with_span(
                    ty.span(),
                    "Cannot define `ref_response` and `to_response` attribute simultaneously",
                ))
            }
        };

        Ok(response)
    }
}

struct NamedStructResponse<'n>(ResponseTuple<'n>);

impl Response for NamedStructResponse<'_> {}

impl NamedStructResponse<'_> {
    fn new(
        attributes: &[Attribute],
        ident: &Ident,
        fields: &Punctuated<Field, Comma>,
    ) -> Result<Self, Diagnostics> {
        if let Some(diagnostics) =
            Self::validate_attributes(attributes, Self::has_no_field_attributes)
                .chain(Self::validate_attributes(
                    fields.iter().flat_map(|field| &field.attrs),
                    Self::has_no_field_attributes,
                ))
                .collect::<Option<Diagnostics>>()
        {
            return Err(diagnostics);
        }

        let mut derive_value = DeriveIntoResponsesValue::from_attributes(attributes)?
            .expect("`IntoResponses` must have `#[response(...)]` attribute");
        let description = {
            let s = CommentAttributes::from_attributes(attributes).as_formatted_string();
            parse_utils::LitStrOrExpr::LitStr(LitStr::new(&s, Span::call_site()))
        };
        let status_code = mem::take(&mut derive_value.status);
        let inline_schema = NamedStructSchema::new(
            &Root {
                ident,
                attributes,
                generics: &Generics::default(),
            },
            fields,
            Vec::new(),
        )?;

        let ty = Self::to_type(ident);

        Ok(Self(
            (
                status_code,
                ResponseValue::from_derive_into_responses_value(
                    derive_value,
                    Schema::Default(DefaultSchema::Raw {
                        tokens: inline_schema.to_token_stream(),
                        ty: Cow::Owned(ty),
                    }),
                    description,
                ),
            )
                .into(),
        ))
    }
}

struct UnitStructResponse<'u>(ResponseTuple<'u>);

impl Response for UnitStructResponse<'_> {}

impl UnitStructResponse<'_> {
    fn new(attributes: &[Attribute]) -> Result<Self, Diagnostics> {
        if let Some(diagnostics) =
            Self::validate_attributes(attributes, Self::has_no_field_attributes)
                .collect::<Option<Diagnostics>>()
        {
            return Err(diagnostics);
        }

        let mut derive_value = DeriveIntoResponsesValue::from_attributes(attributes)?
            .expect("`IntoResponses` must have `#[response(...)]` attribute");
        let status_code = mem::take(&mut derive_value.status);
        let description = {
            let s = CommentAttributes::from_attributes(attributes).as_formatted_string();
            parse_utils::LitStrOrExpr::LitStr(LitStr::new(&s, Span::call_site()))
        };

        Ok(Self(
            (
                status_code,
                ResponseValue::from_derive_into_responses_value(
                    derive_value,
                    Schema::Default(DefaultSchema::None),
                    description,
                ),
            )
                .into(),
        ))
    }
}

struct ToResponseNamedStructResponse<'p>(ResponseTuple<'p>);

impl Response for ToResponseNamedStructResponse<'_> {}

impl<'p> ToResponseNamedStructResponse<'p> {
    fn new(
        attributes: &[Attribute],
        ident: &Ident,
        fields: &Punctuated<Field, Comma>,
    ) -> Result<Self, Diagnostics> {
        if let Some(diagnostics) =
            Self::validate_attributes(attributes, Self::has_no_field_attributes)
                .chain(Self::validate_attributes(
                    fields.iter().flat_map(|field| &field.attrs),
                    Self::has_no_field_attributes,
                ))
                .collect::<Option<Diagnostics>>()
        {
            return Err(diagnostics);
        }

        let derive_value = DeriveToResponseValue::from_attributes(attributes)?;
        let description = {
            let s = CommentAttributes::from_attributes(attributes).as_formatted_string();
            parse_utils::LitStrOrExpr::LitStr(LitStr::new(&s, Span::call_site()))
        };
        let ty = Self::to_type(ident);

        let inline_schema = NamedStructSchema::new(
            &Root {
                ident,
                attributes,
                generics: &Generics::default(),
            },
            fields,
            Vec::new(),
        )?;

        let response_value = if let Some(derive_value) = derive_value {
            ResponseValue::from_derive_to_response_value(
                derive_value,
                Schema::Default(DefaultSchema::Raw {
                    tokens: inline_schema.to_token_stream(),
                    ty: Cow::Owned(ty),
                }),
                description,
            )
        } else {
            ResponseValue::from_schema(
                Schema::Default(DefaultSchema::Raw {
                    tokens: inline_schema.to_token_stream(),
                    ty: Cow::Owned(ty),
                }),
                description,
            )
        };
        // response_value.response_type = Some(response_type);

        Ok(Self(response_value.into()))
    }
}

struct ToResponseUnnamedStructResponse<'c>(ResponseTuple<'c>);

impl Response for ToResponseUnnamedStructResponse<'_> {}

impl<'u> ToResponseUnnamedStructResponse<'u> {
    fn new(
        attributes: &[Attribute],
        ty: &'u Type,
        inner_attributes: &[Attribute],
    ) -> Result<Self, Diagnostics> {
        if let Some(diagnostics) =
            Self::validate_attributes(attributes, Self::has_no_field_attributes)
                .chain(Self::validate_attributes(inner_attributes, |attribute| {
                    const ERROR: &str =
                "Unexpected attribute, `content` is only supported on unnamed field enum variant";
                    if attribute.path().get_ident().unwrap() == "content" {
                        (false, ERROR)
                    } else {
                        (true, ERROR)
                    }
                }))
                .collect::<Option<Diagnostics>>()
        {
            return Err(diagnostics);
        }
        let derive_value = DeriveToResponseValue::from_attributes(attributes)?;
        let description = {
            let s = CommentAttributes::from_attributes(attributes).as_formatted_string();
            parse_utils::LitStrOrExpr::LitStr(LitStr::new(&s, Span::call_site()))
        };

        let is_inline = inner_attributes
            .iter()
            .any(|attribute| attribute.path().get_ident().unwrap() == "to_schema");

        let response_value = if let Some(derive_value) = derive_value {
            ResponseValue::from_derive_to_response_value(
                derive_value,
                ParsedType {
                    ty: Cow::Borrowed(ty),
                    is_inline,
                },
                description,
            )
        } else {
            ResponseValue::from_schema(
                ParsedType {
                    ty: Cow::Borrowed(ty),
                    is_inline,
                },
                description,
            )
        };

        Ok(Self(response_value.into()))
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct VariantAttributes<'r> {
    type_and_content: Option<(&'r Type, String)>,
    derive_value: Option<DeriveToResponseValue>,
    is_inline: bool,
}

struct EnumResponse<'r>(ResponseTuple<'r>);

impl Response for EnumResponse<'_> {}

impl<'r> EnumResponse<'r> {
    fn new(
        ident: &Ident,
        variants: &'r Punctuated<Variant, Comma>,
        attributes: &[Attribute],
    ) -> Result<Self, Diagnostics> {
        if let Some(diagnostics) =
            Self::validate_attributes(attributes, Self::has_no_field_attributes)
                .chain(Self::validate_attributes(
                    variants.iter().flat_map(|variant| &variant.attrs),
                    Self::has_no_field_attributes,
                ))
                .collect::<Option<Diagnostics>>()
        {
            return Err(diagnostics);
        }

        let ty = Self::to_type(ident);
        let description = {
            let s = CommentAttributes::from_attributes(attributes).as_formatted_string();
            parse_utils::LitStrOrExpr::LitStr(LitStr::new(&s, Span::call_site()))
        };

        let content = variants
            .into_iter()
            .map(Self::parse_variant_attributes)
            .collect::<Result<Vec<VariantAttributes>, Diagnostics>>()?
            .into_iter()
            .filter(|variant| variant.type_and_content.is_some())
            .collect::<Vec<_>>();

        let derive_value = DeriveToResponseValue::from_attributes(attributes)?;
        if let Some(derive_value) = &derive_value {
            if (!content.is_empty() && derive_value.example.is_some())
                || (!content.is_empty() && derive_value.examples.is_some())
            {
                let ident = derive_value
                    .example
                    .as_ref()
                    .map(|(_, ident)| ident)
                    .or_else(|| derive_value.examples.as_ref().map(|(_, ident)| ident))
                    .expect("Expected `example` or `examples` to be present");
                return Err(
                    Diagnostics::with_span(ident.span(),
                        "Enum with `#[content]` attribute in variant cannot have enum level `example` or `examples` defined")
                    .help(format!("Try defining `{}` on the enum variant", ident))
                );
            }
        }

        let generics = Generics::default();
        let root = &Root {
            ident,
            attributes,
            generics: &generics,
        };
        let inline_schema = EnumSchema::new(root, variants)?;

        let response_value = if content.is_empty() {
            if let Some(derive_value) = derive_value {
                ResponseValue::from_derive_to_response_value(
                    derive_value,
                    Schema::Default(DefaultSchema::None),
                    description,
                )
            } else {
                ResponseValue::from_schema(
                    Schema::Default(DefaultSchema::Raw {
                        tokens: inline_schema.to_token_stream(),
                        ty: Cow::Owned(ty),
                    }),
                    description,
                )
            }
        } else {
            let content = content
                .into_iter()
                .map(
                    |VariantAttributes {
                         type_and_content,
                         derive_value,
                         is_inline,
                     }| {
                        let (content_type, schema) = if let Some((ty, content)) = type_and_content {
                            (
                                Some(content.into()),
                                Some(Schema::Default(DefaultSchema::TypePath(ParsedType {
                                    ty: Cow::Borrowed(ty),
                                    is_inline,
                                }))),
                            )
                        } else {
                            (None, None)
                        };
                        let (example, examples) = if let Some(derive_value) = derive_value {
                            (
                                derive_value.example.map(|(example, _)| example),
                                derive_value.examples.map(|(examples, _)| examples),
                            )
                        } else {
                            (None, None)
                        };

                        MediaTypeAttr {
                            content_type,
                            schema: schema.unwrap_or_else(|| Schema::Default(DefaultSchema::None)),
                            example,
                            examples: examples.unwrap_or_default(),
                            ..MediaTypeAttr::default()
                        }
                    },
                )
                .collect::<Vec<_>>();

            let mut response = if let Some(derive_value) = derive_value {
                ResponseValue::from_derive_to_response_value(
                    derive_value,
                    Schema::Default(DefaultSchema::None),
                    description,
                )
            } else {
                ResponseValue::from_schema(
                    Schema::Default(DefaultSchema::Raw {
                        tokens: inline_schema.to_token_stream(),
                        ty: Cow::Owned(ty),
                    }),
                    description,
                )
            };
            response.content = content;

            response
        };

        Ok(Self(response_value.into()))
    }

    fn parse_variant_attributes(variant: &Variant) -> Result<VariantAttributes, Diagnostics> {
        let variant_derive_response_value =
            DeriveToResponseValue::from_attributes(variant.attrs.as_slice())?;
        // named enum variant should not have field attributes
        if let Fields::Named(named_fields) = &variant.fields {
            if let Some(diagnostics) = Self::validate_attributes(
                named_fields.named.iter().flat_map(|field| &field.attrs),
                Self::has_no_field_attributes,
            )
            .collect::<Option<Diagnostics>>()
            {
                return Err(diagnostics);
            }
        };

        let field = variant.fields.iter().next();

        let content_type = field.and_then_try(|field| {
            field
                .attrs
                .iter()
                .find(|attribute| attribute.path().get_ident().unwrap() == "content")
                .map_try(|attribute| {
                    attribute
                        .parse_args_with(|input: ParseStream| input.parse::<LitStr>())
                        .map(|content| content.value())
                        .map_err(Diagnostics::from)
                })
        })?;

        let is_inline = field
            .map(|field| {
                field
                    .attrs
                    .iter()
                    .any(|attribute| attribute.path().get_ident().unwrap() == "to_schema")
            })
            .unwrap_or(false);

        Ok(VariantAttributes {
            type_and_content: field.map(|field| &field.ty).zip(content_type),
            derive_value: variant_derive_response_value,
            is_inline,
        })
    }
}

struct ToResponseUnitStructResponse<'u>(ResponseTuple<'u>);

impl Response for ToResponseUnitStructResponse<'_> {}

impl ToResponseUnitStructResponse<'_> {
    fn new(attributes: &[Attribute]) -> Result<Self, Diagnostics> {
        if let Some(diagnostics) =
            Self::validate_attributes(attributes, Self::has_no_field_attributes)
                .collect::<Option<Diagnostics>>()
        {
            return Err(diagnostics);
        }

        let derive_value = DeriveToResponseValue::from_attributes(attributes)?;
        let description = {
            let s = CommentAttributes::from_attributes(attributes).as_formatted_string();
            parse_utils::LitStrOrExpr::LitStr(LitStr::new(&s, Span::call_site()))
        };

        let response_value = if let Some(derive_value) = derive_value {
            ResponseValue::from_derive_to_response_value(
                derive_value,
                Schema::Default(DefaultSchema::None),
                description,
            )
        } else {
            ResponseValue {
                description,
                ..Default::default()
            }
        };

        Ok(Self(response_value.into()))
    }
}
