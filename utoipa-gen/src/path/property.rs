
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, TypePath};

use crate::{
    component_type::{ComponentFormat, ComponentType},
    schema::component::format_path_ref,
    Type,
};

/// Tokenizable object property. It is used as a object property for components or as property
/// of request or response body or response header.
pub(crate) struct Property<'a>(&'a Type<'a>);

impl<'a> Property<'a> {
    pub fn new(type_definition: &'a Type<'a>) -> Self {
        Self(type_definition)
    }

    pub fn component_type(&'a self) -> ComponentType<'a> {
        ComponentType(&*self.0.ty)
    }
}

impl ToTokens for Property<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let component_type: ComponentType = self.component_type();

        if component_type.is_primitive() {
            let mut component = quote! {
                utoipa::openapi::PropertyBuilder::new().component_type(#component_type)
            };

            let format = ComponentFormat(component_type.0);
            if format.is_known_format() {
                component.extend(quote! {
                    .format(Some(#format))
                })
            }

            tokens.extend(if self.0.is_array {
                quote! {
                    utoipa::openapi::schema::ArrayBuilder::new()
                        .items(#component)
                }
            } else {
                component
            });
        } else {
            let component_name_path: &TypePath = &*component_type.0;
            let name = format_path_ref(&component_name_path.to_token_stream().to_string());

            let component = if self.0.is_inline {
                let assert_component = format_ident!("_Assert{}", name);
                quote_spanned! { component_name_path.span() => {
                        struct #assert_component where #component_name_path: utoipa::Component;

                        <#component_name_path as utoipa::Component>::component()
                    }
                }
            } else {
                quote! {
                    utoipa::openapi::Ref::from_component_name(#name)
                }
            };

            tokens.extend(if self.0.is_array {
                quote! {
                    utoipa::openapi::schema::ArrayBuilder::new()
                        .items(#component)
                }
            } else {
                component
            });
        }
    }
}
