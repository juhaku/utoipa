use proc_macro_error::{abort, ResultExt};
use quote::{quote, ToTokens};
use syn::{
    parse::Parse, punctuated::Punctuated, token::Comma, Attribute, Data, Error, Field, Generics,
    Ident, LitStr, Token,
};

use crate::{
    component_type::{ComponentFormat, ComponentType},
    doc_comment::CommentAttributes,
    parse_utils,
    path::parameter::{ParameterExt, ParameterIn, ParameterStyle},
    Array, Required,
};

use super::{component::format_path_ref, ComponentPart, GenericType, ValueType};

/// Container attribute `#[into_params(...)]`.
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct IntoParamsAttr {
    /// See [`ParameterStyle`].
    style: Option<ParameterStyle>,
    /// Specify names of unnamed fields with `names(...) attribute.`
    names: Option<Vec<String>>,
    /// See [`ParameterIn`].
    parameter_in: Option<ParameterIn>,
}

impl IntoParamsAttr {
    fn merge(mut self, other: Self) -> Self {
        if other.style.is_some() {
            self.style = other.style;
        }

        if other.names.is_some() {
            self.names = other.names;
        }

        if other.parameter_in.is_some() {
            self.parameter_in = other.parameter_in;
        }

        self
    }
}

impl Parse for IntoParamsAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE: &str =
            "unexpected token, expected any of: names, style, parameter_in";

        let punctuated =
            Punctuated::<IntoParamsAttr, Token![,]>::parse_terminated_with(input, |input| {
                let ident: Ident = input.parse::<Ident>().map_err(|error| {
                    Error::new(error.span(), format!("{EXPECTED_ATTRIBUTE}, {error}"))
                })?;

                Ok(match ident.to_string().as_str() {
                    "names" => IntoParamsAttr {
                        names: Some(
                            parse_utils::parse_punctuated_within_parenthesis::<LitStr>(input)?
                                .into_iter()
                                .map(|name| name.value())
                                .collect(),
                        ),
                        ..IntoParamsAttr::default()
                    },
                    "style" => {
                        let style: ParameterStyle =
                            parse_utils::parse_next(input, || input.parse::<ParameterStyle>())?;
                        IntoParamsAttr {
                            style: Some(style),
                            ..IntoParamsAttr::default()
                        }
                    }
                    "parameter_in" => {
                        let parameter_in: ParameterIn =
                            parse_utils::parse_next(input, || input.parse::<ParameterIn>())?;

                        IntoParamsAttr {
                            parameter_in: Some(parameter_in),
                            ..IntoParamsAttr::default()
                        }
                    }
                    _ => return Err(Error::new(ident.span(), EXPECTED_ATTRIBUTE)),
                })
            })?;

        let attributes: IntoParamsAttr = punctuated
            .into_iter()
            .fold(IntoParamsAttr::default(), |acc, next| acc.merge(next));

        Ok(attributes)
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct IntoParams {
    /// Attributes tagged on the whole struct or enum.
    pub attrs: Vec<Attribute>,
    /// Generics required to complete the definition.
    pub generics: Generics,
    /// Data within the struct or enum.
    pub data: Data,
    /// Name of the struct or enum.
    pub ident: Ident,
}

impl ToTokens for IntoParams {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let into_params_attrs: Option<IntoParamsAttr> = self
            .attrs
            .iter()
            .find(|attr| attr.path.is_ident("into_params"))
            .map(|attribute| attribute.parse_args::<IntoParamsAttr>().unwrap_or_abort());

        let params = self
            .get_struct_fields(
                &into_params_attrs
                    .as_ref()
                    .and_then(|params| params.names.as_ref()),
            )
            .enumerate()
            .map(|(index, field)| {
                Param {
                    field,
                    container_attributes: FieldParamContainerAttributes {
                        style: into_params_attrs.as_ref().and_then(|attrs| attrs.style),
                        name: into_params_attrs
                            .as_ref()
                            .and_then(|attrs| attrs.names.as_ref())
                            .map(|names| names.get(index).unwrap_or_else(|| abort!(
                                ident,
                                "There is no name specified in the names(...) container attribute for tuple struct field {}",
                                index
                            ))),
                        parameter_in: into_params_attrs.as_ref().and_then(|attrs| attrs.parameter_in),
                    },
                }
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
                    "struct with unnamed fields must have explicit name declarations.";
                    help = "Try defining `#[into_params(names(...))]` over your type: {}", ident,
                }
            }
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct FieldParamContainerAttributes<'a> {
    /// See [`IntoParamsAttr::style`].
    pub style: Option<ParameterStyle>,
    /// See [`IntoParamsAttr::names`]. The name that applies to this field.
    pub name: Option<&'a String>,
    /// See [`IntoParamsAttr::parameter_in`].
    pub parameter_in: Option<ParameterIn>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct Param<'a> {
    /// Field in the container used to create a single parameter.
    field: &'a Field,
    /// Attributes on the container which are relevant for this macro.
    container_attributes: FieldParamContainerAttributes<'a>,
}

impl ToTokens for Param<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let field = self.field;
        let ident = &field.ident;
        let name = ident
            .as_ref()
            .map(|ident| ident.to_string())
            .or_else(|| self.container_attributes.name.cloned())
            .unwrap_or_else(|| abort!(
                field, "No name specified for unnamed field.";
                help = "Try adding #[into_params(names(...))] container attribute to specify the name for this field"
            ));
        let component_part = ComponentPart::from_type(&field.ty);
        let required: Required =
            (!matches!(&component_part.generic_type, Some(GenericType::Option))).into();

        tokens.extend(quote! { utoipa::openapi::path::ParameterBuilder::new()
            .name(#name)
        });

        tokens.extend(
            if let Some(parameter_in) = self.container_attributes.parameter_in {
                quote! {
                    .parameter_in(#parameter_in)
                }
            } else {
                quote! {
                    .parameter_in(parameter_in_provider().unwrap_or_default())
                }
            },
        );

        tokens.extend(quote! {
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

        let mut parameter_ext = ParameterExt::from(&self.container_attributes);

        // Apply the field attributes if they exist.
        if let Some(p) = field
            .attrs
            .iter()
            .find(|attribute| attribute.path.is_ident("param"))
            .map(|attribute| attribute.parse_args::<ParameterExt>().unwrap_or_abort())
        {
            parameter_ext.merge(p)
        }

        if let Some(ref style) = parameter_ext.style {
            tokens.extend(quote! { .style(Some(#style)) });
        }
        if let Some(ref explode) = parameter_ext.explode {
            tokens.extend(quote! { .explode(Some(#explode)) });
        }
        if let Some(ref allow_reserved) = parameter_ext.allow_reserved {
            tokens.extend(quote! { .allow_reserved(Some(#allow_reserved)) });
        }
        if let Some(ref example) = parameter_ext.example {
            tokens.extend(quote! { .example(Some(#example)) });
        }

        let param_type = ParamType(&component_part);
        tokens.extend(quote! { .schema(Some(#param_type)).build() });
    }
}

struct ParamType<'a>(&'a ComponentPart<'a>);

impl ToTokens for ParamType<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let part: &ComponentPart = self.0;
        match &part.generic_type {
            Some(GenericType::Vec) => {
                let param_type = ParamType(part.child.as_ref().unwrap().as_ref());

                tokens.extend(quote! { #param_type.to_array_builder() });
            }
            None => match part.value_type {
                ValueType::Primitive => {
                    let component_type = ComponentType(&*part.path);

                    tokens.extend(quote! {
                        utoipa::openapi::PropertyBuilder::new().component_type(#component_type)
                    });

                    let format = ComponentFormat(&*part.path);
                    if format.is_known_format() {
                        tokens.extend(quote! {
                            .format(Some(#format))
                        })
                    }
                }
                ValueType::Object => {
                    let name = format_path_ref(&part.path.to_token_stream().to_string());
                    tokens.extend(quote! {
                        utoipa::openapi::Ref::from_component_name(#name)
                    });
                }
            },
            Some(GenericType::Option)
            | Some(GenericType::Cow)
            | Some(GenericType::Box)
            | Some(GenericType::RefCell) => {
                let param_type = ParamType(part.child.as_ref().unwrap().as_ref());

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
