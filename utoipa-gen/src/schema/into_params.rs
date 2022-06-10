use proc_macro_error::{abort, ResultExt};
use quote::{quote, ToTokens};
use syn::{
    parse::Parse, punctuated::Punctuated, token::Comma, Attribute, Data, Error, Field, Generics,
    Ident, LitStr,
};

use crate::{
    component_type::{ComponentFormat, ComponentType},
    doc_comment::CommentAttributes,
    parse_utils,
    path::parameter::ParameterExt,
    Array, Required,
};

use super::{ComponentPart, GenericType, ValueType};

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct IntoParamsAttr {
    names: Vec<String>,
}

impl Parse for IntoParamsAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE: &str = "unexpected token, expected: names";

        input
            .parse::<Ident>()
            .map_err(|error| Error::new(error.span(), format!("{EXPECTED_ATTRIBUTE}, {error}")))
            .and_then(|ident| {
                if ident != "names" {
                    Err(Error::new(ident.span(), EXPECTED_ATTRIBUTE))
                } else {
                    Ok(ident)
                }
            })?;

        Ok(IntoParamsAttr {
            names: parse_utils::parse_punctuated_within_parenthesis::<LitStr>(input)?
                .into_iter()
                .map(|name| name.value())
                .collect(),
        })
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct IntoParams {
    pub generics: Generics,
    pub data: Data,
    pub ident: Ident,
    pub attrs: Vec<Attribute>,
}

impl ToTokens for IntoParams {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let into_params_attrs = &mut self
            .attrs
            .iter()
            .find(|attr| attr.path.is_ident("into_params"))
            .map(|attribute| attribute.parse_args::<IntoParamsAttr>().unwrap_or_abort());

        let params = self
            .get_struct_fields(
                &into_params_attrs
                    .as_mut()
                    .map(|params| params.names.as_ref()),
            )
            .enumerate()
            .map(|(index, field)| {
                Param(
                    field,
                    into_params_attrs
                        .as_ref()
                        .and_then(|param| param.names.get(index)),
                )
            })
            .collect::<Array<Param>>();

        tokens.extend(quote! {
            impl #impl_generics utoipa::IntoParams for #ident #ty_generics #where_clause {

                fn into_params(parameter_in_provider: impl Fn() -> Option<utoipa::openapi::path::ParameterIn>) -> Vec<utoipa::openapi::path::Parameter> {
                    #params.to_vec()
                }

            }
        });
    }
}

impl IntoParams {
    fn get_struct_fields(
        &self,
        field_names: &Option<&Vec<String>>,
    ) -> impl Iterator<Item = &Field> {
        let ident = &self.ident;
        let abort = |note: &str| {
            abort! {
                ident,
                "unsupported data type, expected struct with named fields `struct {} {{...}}` or unnamed fields `struct {}(...)`",
                ident.to_string(),
                ident.to_string();
                note = note
            }
        };

        match &self.data {
            Data::Struct(data_struct) => match &data_struct.fields {
                syn::Fields::Named(named_fields) => {
                    if field_names.is_some() {
                        abort! {ident, "`#[into_params(names(...))]` is not supported attribute on a struct with named fields"}
                    }
                    named_fields.named.iter()
                }
                syn::Fields::Unnamed(unnamed_fields) => {
                    self.validate_unnamed_field_names(&unnamed_fields.unnamed, field_names);
                    unnamed_fields.unnamed.iter()
                }
                _ => abort("Unit type struct is not supported"),
            },
            _ => abort("Only struct type is supported"),
        }
    }

    fn validate_unnamed_field_names(
        &self,
        unnamed_fields: &Punctuated<Field, Comma>,
        field_names: &Option<&Vec<String>>,
    ) {
        let ident = &self.ident;
        match field_names {
            Some(names) => {
                if names.len() != unnamed_fields.len() {
                    abort! {
                        ident,
                        "declared names amount '{}' does not match to the unnamed fields amount '{}' in type: {}",
                            names.len(), unnamed_fields.len(), ident;
                        help = r#"Did you forget to add a field name to `#[into_params(names(... , "field_name"))]`"#;
                        help = "Or have you added extra name but haven't defined a type?"
                    }
                }
            }
            None => {
                abort! {
                    ident,
                    "struct with unnamed fields must have explisit name declarations.";
                    help = "Try defining `#[into_params(names(...))]` over your type: {}", ident,
                }
            }
        }
    }
}

struct Param<'a>(&'a Field, Option<&'a String>);

impl ToTokens for Param<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let unnamed_field_name = self.1;
        let field = self.0;
        let ident = &field.ident;
        let name = ident
            .as_ref()
            .map(|ident| ident.to_string())
            .or_else(|| unnamed_field_name.map(ToString::to_string))
            .unwrap_or_default();
        let component_part = ComponentPart::from_type(&field.ty);
        let required: Required =
            (!matches!(&component_part.generic_type, Some(GenericType::Option))).into();

        tokens.extend(quote! { utoipa::openapi::path::ParameterBuilder::new()
            .name(#name)
            .parameter_in(parameter_in_provider().unwrap_or_default())
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
                let param_type = ParamType(ty.child.as_ref().unwrap().as_ref());

                tokens.extend(param_type.into_token_stream())
            }
            Some(GenericType::Map) => {
                // Maps are treated just as generic objects without types. There is no Map type in OpenAPI spec.
                tokens.extend(quote! {
                    utoipa::openapi::ObjectBuilder::new()
                });
            }
        };
    }
}
