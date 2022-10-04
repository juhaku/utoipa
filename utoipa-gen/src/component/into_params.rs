use proc_macro2::TokenStream;
use proc_macro_error::{abort, ResultExt};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::Parse, punctuated::Punctuated, spanned::Spanned, token::Comma, Attribute, Data, Error,
    Field, Generics, Ident, LitStr, Token,
};

use crate::{
    doc_comment::CommentAttributes,
    parse_utils,
    path::parameter::{ParameterExt, ParameterIn, ParameterStyle},
    schema_type::{SchemaFormat, SchemaType},
    Array, Required,
};

use super::{GenericType, TypeTree, ValueType};

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

        // #[params] is only supported over fields
        if self.attrs.iter().any(|attr| attr.path.is_ident("param")) {
            abort! {
                ident,
                "found `param` attribute in unsupported context";
                help = "Did you mean `into_params`?",
            }
        }

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
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field = self.field;
        let ident = &field.ident;
        let mut name = &*ident
            .as_ref()
            .map(|ident| ident.to_string())
            .or_else(|| self.container_attributes.name.cloned())
            .unwrap_or_else(|| abort!(
                field, "No name specified for unnamed field.";
                help = "Try adding #[into_params(names(...))] container attribute to specify the name for this field"
            ));

        if name.starts_with("r#") {
            name = &name[2..];
        }

        let type_tree = TypeTree::from_type(&field.ty);
        let field_param_attrs = field
            .attrs
            .iter()
            .find(|attribute| attribute.path.is_ident("param"))
            .map(|attribute| {
                attribute
                    .parse_args::<IntoParamsFieldParamsAttr>()
                    .unwrap_or_abort()
            });

        if let Some(IntoParamsFieldParamsAttr { rename: Some(ref name), ..}) = field_param_attrs {
            tokens.extend(quote! { utoipa::openapi::path::ParameterBuilder::new()
                .name(#name)
            });
        } else {
            tokens.extend(quote! { utoipa::openapi::path::ParameterBuilder::new()
                .name(#name)
            });
        }

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
        if let Some(field_params_attrs) = field
            .attrs
            .iter()
            .find(|attribute| attribute.path.is_ident("param"))
            .map(|attribute| {
                attribute
                    .parse_args::<IntoParamsFieldParamsAttr>()
                    .unwrap_or_abort()
            })
        {
            parameter_ext.merge(field_params_attrs.parameter_ext.unwrap_or_default())
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
        let component = &field_param_attrs
            .as_ref()
            .and_then(|field_params| field_params.value_type.as_ref().map(TypeTree::from_type))
            .unwrap_or(type_tree);
        let required: Required =
            (!matches!(&component.generic_type, Some(GenericType::Option))).into();

        tokens.extend(quote! {
            .required(#required)
        });

        let schema = ParamType {
            component,
            field_param_attrs: &field_param_attrs,
        };
        tokens.extend(quote! { .schema(Some(#schema)).build() });
    }
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
struct IntoParamsFieldParamsAttr {
    inline: bool,
    value_type: Option<syn::Type>,
    parameter_ext: Option<ParameterExt>,
    rename: Option<String>,
}

impl Parse for IntoParamsFieldParamsAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const EXPECTED_ATTRIBUTE_MESSAGE: &str = "unexpected attribute, expected any of: style, explode, allow_reserved, example, inline, value_type, rename";
        let mut param = IntoParamsFieldParamsAttr::default();

        while !input.is_empty() {
            if ParameterExt::is_parameter_ext(input) {
                let param_ext = param.parameter_ext.get_or_insert(ParameterExt::default());
                param_ext.merge(input.call(ParameterExt::parse_once)?);
            } else {
                let ident = input.parse::<Ident>()?;
                let name = &*ident.to_string();

                match name {
                    "inline" => param.inline = parse_utils::parse_bool_or_true(input)?,
                    "value_type" => {
                        param.value_type = Some(parse_utils::parse_next(input, || {
                            input.parse::<syn::Type>()
                        })?)
                    }
                    "rename" => {
                        param.rename = Some(parse_utils::parse_next_literal_str(input)?)
                    }
                    _ => return Err(Error::new(ident.span(), EXPECTED_ATTRIBUTE_MESSAGE)),
                }

                if !input.is_empty() {
                    input.parse::<Comma>()?;
                }
            }
        }

        Ok(param)
    }
}

struct ParamType<'a> {
    component: &'a TypeTree<'a>,
    field_param_attrs: &'a Option<IntoParamsFieldParamsAttr>,
}

impl ToTokens for ParamType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let component = self.component;

        match &component.generic_type {
            Some(GenericType::Vec) => {
                let param_type = ParamType {
                    component: component
                        .children
                        .as_ref()
                        .expect("Vec ParamType should have children")
                        .iter()
                        .next()
                        .expect("Vec ParamType should have 1 child"),
                    field_param_attrs: self.field_param_attrs,
                };

                tokens.extend(quote! {
                    utoipa::openapi::Schema::Array(
                        utoipa::openapi::ArrayBuilder::new().items(#param_type).build()
                    )
                });
            }
            Some(GenericType::Option)
            | Some(GenericType::Cow)
            | Some(GenericType::Box)
            | Some(GenericType::RefCell) => {
                let param_type = ParamType {
                    component: component
                        .children
                        .as_ref()
                        .expect("Generic container ParamType should have children")
                        .iter()
                        .next()
                        .expect("Generic container ParamType should have 1 child"),
                    field_param_attrs: self.field_param_attrs,
                };

                tokens.extend(param_type.into_token_stream())
            }
            Some(GenericType::Map) => {
                // Maps are treated as generic objects with no named properties and
                // additionalProperties denoting the type

                let component_property = ParamType {
                    component: component
                        .children
                        .as_ref()
                        .expect("Map ParamType should have children")
                        .iter()
                        .nth(1)
                        .expect("Map Param type should have 2 child"),
                    field_param_attrs: self.field_param_attrs,
                };

                tokens.extend(quote! {
                    utoipa::openapi::ObjectBuilder::new().additional_properties(Some(#component_property))
                });
            }
            None => {
                let inline = matches!(self.field_param_attrs, Some(params) if params.inline);

                match component.value_type {
                    ValueType::Primitive => {
                        let type_path = &**component.path.as_ref().unwrap();
                        let schema_type = SchemaType(type_path);

                        tokens.extend(quote! {
                            utoipa::openapi::ObjectBuilder::new().schema_type(#schema_type)
                        });

                        let format: SchemaFormat = (type_path).into();
                        if format.is_known_format() {
                            tokens.extend(quote! {
                                .format(Some(#format))
                            })
                        }
                    }
                    ValueType::Object => {
                        let component_path = &**component
                            .path
                            .as_ref()
                            .expect("component should have a path");
                        if inline {
                            tokens.extend(quote_spanned! {component_path.span()=>
                                <#component_path as utoipa::ToSchema>::schema()
                            })
                        } else if component.is_object() {
                            tokens.extend(quote! {
                                utoipa::openapi::ObjectBuilder::new()
                            });
                        } else {
                            let name: String = component_path
                                .segments
                                .last()
                                .expect("Expected there to be at least one element in the path")
                                .ident
                                .to_string();
                            tokens.extend(quote! {
                                utoipa::openapi::Ref::from_schema_name(#name)
                            });
                        }
                    }
                    // TODO support for tuple types
                    ValueType::Tuple => (),
                }
            }
        };
    }
}
