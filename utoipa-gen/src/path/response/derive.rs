use std::borrow::Cow;
use std::{iter, mem};

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::{abort, emit_error};
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{Attribute, Data, Field, Fields, Generics, Path, Type, TypePath};

use crate::component::schema::NamedStructSchema;
use crate::doc_comment::CommentAttributes;
use crate::path::{InlineType, PathType};
use crate::Array;

use super::{
    DeriveIntoResponsesValue, DeriveResponseValue, ResponseTuple, ResponseTupleInner, ResponseValue,
};

pub struct IntoResponses {
    pub attributes: Vec<Attribute>,
    pub data: Data,
    pub generics: Generics,
    pub ident: Ident,
}

impl ToTokens for IntoResponses {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let responses = match &self.data {
            Data::Struct(struct_value) => match &struct_value.fields {
                Fields::Named(fields) => {
                    let response =
                        NamedStructResponse::new(&self.attributes, &self.ident, &fields.named).0;
                    let status = &response.status_code;

                    Array::from_iter(iter::once(quote!((#status, #response))))
                }
                Fields::Unnamed(fields) => {
                    let field = fields
                        .unnamed
                        .iter()
                        .next()
                        .expect("Unnamed struct must have 1 field");

                    let response =
                        UnnamedStructResponse::new(&self.attributes, &field.ty, &field.attrs).0;
                    let status = &response.status_code;

                    Array::from_iter(iter::once(quote!((#status, #response))))
                }
                Fields::Unit => {
                    let response = UnitStructResponse::new(&self.attributes).0;
                    let status = &response.status_code;

                    Array::from_iter(iter::once(quote!((#status, #response))))
                }
            },
            Data::Enum(enum_value) => enum_value
                .variants
                .iter()
                .map(|variant| match &variant.fields {
                    Fields::Named(fields) => {
                        NamedStructResponse::new(&variant.attrs, &variant.ident, &fields.named).0
                    }
                    Fields::Unnamed(fields) => {
                        let field = fields
                            .unnamed
                            .iter()
                            .next()
                            .expect("Unnamed enum variant must have 1 field");
                        UnnamedStructResponse::new(&variant.attrs, &field.ty, &field.attrs).0
                    }
                    Fields::Unit => UnitStructResponse::new(&variant.attrs).0,
                })
                .map(|response| {
                    let status = &response.status_code;
                    quote!((#status, utoipa::openapi::RefOr::from(#response)))
                })
                .collect::<Array<TokenStream>>(),
            Data::Union(_) => abort!(self.ident, "`IntoReponses` does not support `Union` type"),
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
        })
    }
}

trait Response {
    fn to_type(ident: &Ident) -> Type {
        let path = Path::from(ident.clone());
        let type_path = TypePath { path, qself: None };
        Type::Path(type_path)
    }

    fn has_no_field_attributes(attribute: &Attribute) -> (bool, &'static str) {
        const ERROR: &str = "Unexpected field attribute, field attributes are only supported unnamed field structs or enum variants";

        let ident = attribute.path.get_ident().unwrap();
        match &*ident.to_string() {
            "to_schema" => (false, ERROR),
            "ref_response" => (false, ERROR),
            "to_response" => (false, ERROR),
            _ => (true, ERROR),
        }
    }

    fn validate_attributes<'a, I: IntoIterator<Item = &'a Attribute>>(
        attributes: I,
        validate: impl Fn(&Attribute) -> (bool, &'static str),
    ) {
        for attribute in attributes {
            let (valid, message) = validate(attribute);
            if !valid {
                emit_error!(attribute, message)
            }
        }
    }
}

fn create_response_value(
    description: String,
    response_value: DeriveIntoResponsesValue,
    response_type: Option<PathType>,
) -> ResponseValue {
    ResponseValue {
        description: if response_value.description.is_empty() && !description.is_empty() {
            description
        } else {
            response_value.description
        },
        headers: response_value.headers,
        example: response_value.example.map(|(example, _)| example),
        examples: response_value.examples.map(|(examples, _)| examples),
        content_type: response_value.content_type,
        response_type,
        ..Default::default()
    }
}

struct UnnamedStructResponse<'u>(ResponseTuple<'u>);

impl Response for UnnamedStructResponse<'_> {}

impl<'u> UnnamedStructResponse<'u> {
    fn new(attributes: &[Attribute], ty: &'u Type, inner_attributes: &[Attribute]) -> Self {
        let is_inline = inner_attributes
            .iter()
            .any(|attribute| attribute.path.get_ident().unwrap() == "to_schema");
        let ref_response = inner_attributes
            .iter()
            .any(|attribute| attribute.path.get_ident().unwrap() == "ref_response");
        let to_response = inner_attributes
            .iter()
            .any(|attribute| attribute.path.get_ident().unwrap() == "to_response");

        if is_inline && (ref_response || to_response) {
            abort!(
                ty.span(),
                "Attribute `to_schema` cannot be used with `ref_response` and `to_response` attribute"
            )
        }
        let mut derive_value = DeriveIntoResponsesValue::from_attributes(attributes)
            .expect("`IntoResponses` must have `#[response(...)]` attribute");
        let description = CommentAttributes::from_attributes(attributes).as_formatted_string();
        let status_code = mem::take(&mut derive_value.status);

        match (ref_response, to_response) {
            (false, false) => {
                let response = create_response_value(
                    description,
                    derive_value,
                    Some(PathType::MediaType(InlineType {
                        ty: Cow::Borrowed(ty),
                        is_inline,
                    })),
                );
                Self(ResponseTuple {
                    inner: Some(super::ResponseTupleInner::Value(response)),
                    status_code,
                })
            }
            (true, false) => Self(ResponseTuple {
                inner: Some(ResponseTupleInner::Ref(InlineType {
                    ty: Cow::Borrowed(ty),
                    is_inline: false,
                })),
                status_code,
            }),
            (false, true) => Self(ResponseTuple {
                inner: Some(ResponseTupleInner::Ref(InlineType {
                    ty: Cow::Borrowed(ty),
                    is_inline: true,
                })),
                status_code,
            }),
            (true, true) => {
                abort!(
                    ty.span(),
                    "Cannot define `ref_response` and `to_response` attribute simultaneously"
                );
            }
        }
    }
}

struct NamedStructResponse<'n>(ResponseTuple<'n>);

impl Response for NamedStructResponse<'_> {}

impl NamedStructResponse<'_> {
    fn new(attributes: &[Attribute], ident: &Ident, fields: &Punctuated<Field, Comma>) -> Self {
        Self::validate_attributes(attributes, Self::has_no_field_attributes);
        Self::validate_attributes(
            fields.iter().flat_map(|field| &field.attrs),
            Self::has_no_field_attributes,
        );

        let mut derive_value = DeriveIntoResponsesValue::from_attributes(attributes)
            .expect("`IntoResponses` must have `#[response(...)]` attribute");
        let description = CommentAttributes::from_attributes(attributes).as_formatted_string();
        let status_code = mem::take(&mut derive_value.status);

        let inline_schema = NamedStructSchema {
            attributes,
            fields,
            alias: None,
            features: None,
            generics: None,
            rename_all: None,
            struct_name: Cow::Owned(ident.to_string()),
        };

        let ty = Self::to_type(ident);
        let response_value = create_response_value(
            description,
            derive_value,
            Some(PathType::InlineSchema(inline_schema.to_token_stream(), ty)),
        );

        Self(ResponseTuple {
            status_code,
            inner: Some(ResponseTupleInner::Value(response_value)),
        })
    }
}

struct UnitStructResponse<'u>(ResponseTuple<'u>);

impl Response for UnitStructResponse<'_> {}

impl UnitStructResponse<'_> {
    fn new(attributes: &[Attribute]) -> Self {
        Self::validate_attributes(attributes, Self::has_no_field_attributes);

        let mut derive_value = DeriveIntoResponsesValue::from_attributes(attributes)
            .expect("`IntoResponses` must have `#[response(...)]` attribute");
        let status_code = mem::take(&mut derive_value.status);
        let description = CommentAttributes::from_attributes(attributes).as_formatted_string();
        let response_value = create_response_value(description, derive_value, None);

        Self(ResponseTuple {
            status_code,
            inner: Some(ResponseTupleInner::Value(response_value)),
        })
    }
}
