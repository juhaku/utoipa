use std::borrow::Borrow;

use proc_macro_error::{abort, ResultExt};
use quote::{quote, ToTokens};
use syn::{Data, Field, Generics, Ident};

use crate::{
    component_type::{ComponentFormat, ComponentType},
    doc_comment::CommentAttributes,
    path::parameter::ParameterExt,
    Array, Required,
};

use super::{ComponentPart, GenericType, ValueType};

pub struct IntoParams {
    pub generics: Generics,
    pub data: Data,
    pub ident: Ident,
}

impl ToTokens for IntoParams {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let params = self
            .get_struct_fields()
            .map(Param)
            .collect::<Array<Param>>();

        tokens.extend(quote! {
            impl #impl_generics utoipa::IntoParams for #ident #ty_generics #where_clause {

                fn into_params() -> Vec<utoipa::openapi::path::Parameter> {
                    #params.to_vec()
                }

            }
        });
    }
}

impl IntoParams {
    fn get_struct_fields(&self) -> impl Iterator<Item = &Field> {
        let ident = &self.ident;
        let abort = |note: &str| {
            abort! {
                ident,
                "unsupported data type, expected struct with named fields `struct {} {{...}}`",
                ident.to_string();
                note = note
            }
        };

        match &self.data {
            Data::Struct(data_struct) => match &data_struct.fields {
                syn::Fields::Named(named_fields) => named_fields.named.iter(),
                _ => abort("Only struct with named fields is supported"),
            },
            _ => abort("Only struct type is supported"),
        }
    }
}

struct Param<'a>(&'a Field);

impl ToTokens for Param<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let field = self.0;
        let ident = &field.ident;
        let name = ident
            .as_ref()
            .map(|ident| ident.to_string())
            .unwrap_or_else(String::new);
        let component_part = ComponentPart::from_type(&field.ty);
        let required: Required =
            (!matches!(&component_part.generic_type, Some(GenericType::Option))).into();

        tokens.extend(quote! { utoipa::openapi::path::ParameterBuilder::new()
            .name(#name)
            .parameter_in(<Self as utoipa::ParameterIn>::parameter_in().unwrap_or_default())
            .required(#required)
        });

        if let Some(deprecated) = super::get_deprecated(&field.attrs) {
            tokens.extend(quote! { .deprecated(Some(#deprecated)) });
        }

        if let Some(comment) = CommentAttributes::from_attributes(&field.attrs).first() {
            tokens.extend(quote! {
                .description(Some(#comment))
            })
        }

        let parameter_ext = field
            .attrs
            .iter()
            .find(|attribute| attribute.path.is_ident("param"))
            .map(|attribute| attribute.parse_args::<ParameterExt>().unwrap_or_abort());

        if let Some(ext) = parameter_ext {
            if let Some(ref style) = ext.style {
                tokens.extend(quote! { .style(Some(#style)) });
            }
            if let Some(ref explode) = ext.explode {
                tokens.extend(quote! { .explode(Some(#explode)) });
            }
            if let Some(ref allow_reserved) = ext.allow_reserved {
                tokens.extend(quote! { .allow_reserved(Some(#allow_reserved)) });
            }
            if let Some(ref example) = ext.example {
                tokens.extend(quote! { .example(Some(#example)) });
            }
        }

        let param_type = ParamType(&component_part);
        tokens.extend(quote! { .schema(Some(#param_type)).build() });
    }
}

struct ParamType<'a>(&'a ComponentPart<'a>);

impl ToTokens for ParamType<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty = self.0;
        match &ty.generic_type {
            Some(GenericType::Vec) => {
                let param_type = ParamType(ty.child.as_ref().unwrap().as_ref());

                tokens.extend(quote! { #param_type.to_array_builder() });
            }
            None => match ty.value_type {
                ValueType::Primitive => {
                    let component_type = ComponentType(ty.ident);

                    tokens.extend(quote! {
                        utoipa::openapi::PropertyBuilder::new().component_type(#component_type)
                    });

                    let format = ComponentFormat(ty.ident);
                    if format.is_known_format() {
                        tokens.extend(quote! {
                            .format(Some(#format))
                        })
                    }
                }
                ValueType::Object => {
                    let name = ty.ident.to_string();
                    tokens.extend(quote! {
                        utoipa::openapi::Ref::from_component_name(#name)
                    });
                }
            },
            Some(GenericType::Option)
            | Some(GenericType::Cow)
            | Some(GenericType::Box)
            | Some(GenericType::RefCell) => {
                let param_type = ParamType(&ty.child.as_ref().unwrap().as_ref());

                tokens.extend(param_type.into_token_stream())
            }
            Some(GenericType::Map) => {
                abort!(ty.ident, "maps are not supported parameter receiver types")
            }
        };
    }
}
