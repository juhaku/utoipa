use proc_macro_error::abort;
use quote::{quote, ToTokens};
use syn::{Data, Field, Generics, Ident};

use crate::{
    component::{self, ComponentPart, GenericType, ValueType},
    component_type::{ComponentFormat, ComponentType},
    doc_comment::CommentAttributes,
    Array, Required,
};

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
        let abort = |help: &str| {
            abort! {
                ident.span(),
                "unsupported data type, expected struct with named fields `struct {} {{...}}`",
                ident.to_string();
                help = help
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
            .required(#required)
            .parameter_in(<Self as utoipa::ParameterIn>::parameter_in().unwrap_or_default())
        });

        if let Some(deprecated) = component::get_deprecated(&field.attrs) {
            tokens.extend(quote! { .deprecated(Some(#deprecated)) });
        }

        if let Some(comment) = CommentAttributes::from_attributes(&field.attrs).0.first() {
            tokens.extend(quote! {
                .description(Some(#comment))
            })
        }

        let param_type = ParamType {
            ty: &component_part,
        };

        tokens.extend(quote! { .schema(Some(#param_type)).build() });
    }
}

struct ParamType<'a> {
    ty: &'a ComponentPart<'a>,
}

impl ToTokens for ParamType<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match &self.ty.generic_type {
            Some(GenericType::Vec) => {
                let param_type = ParamType {
                    ty: self.ty.child.as_ref().unwrap(),
                };

                tokens.extend(quote! { #param_type.to_array_builder() });
            }
            None => match self.ty.value_type {
                ValueType::Primitive => {
                    let component_type = ComponentType(self.ty.ident);

                    tokens.extend(quote! {
                        utoipa::openapi::PropertyBuilder::new().component_type(#component_type)
                    });

                    let format = ComponentFormat(self.ty.ident);
                    if format.is_known_format() {
                        tokens.extend(quote! {
                            .format(Some(#format))
                        })
                    }
                }
                ValueType::Object => abort!(
                    self.ty.ident.span(),
                    "unsupported type, only primitive and String types are supported"
                ),
            },
            Some(GenericType::Option)
            | Some(GenericType::Cow)
            | Some(GenericType::Box)
            | Some(GenericType::RefCell) => {
                let param_type = ParamType {
                    ty: self.ty.child.as_ref().unwrap(),
                };

                tokens.extend(param_type.into_token_stream())
            }
            Some(GenericType::Map) => abort!(
                self.ty.ident,
                "maps are not supported parameter receiver types"
            ),
        };
    }
}
