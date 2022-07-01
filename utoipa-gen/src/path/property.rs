use std::borrow::Cow;

use proc_macro2::Ident;
use quote::{format_ident, quote, quote_spanned, ToTokens};

use crate::{
    component_type::{ComponentFormat, ComponentType},
    Type,
};

/// Tokenizable object property. It is used as a object property for components or as property
/// of request or response body or response header.
pub(crate) struct Property<'a>(&'a Type<'a>);

impl<'a> Property<'a> {
    pub fn new(type_definition: &'a Type<'a>) -> Self {
        Self(type_definition)
    }

    pub fn component_type(&self) -> ComponentType<'a, Cow<Ident>> {
        ComponentType(&self.0.ty)
    }
}

impl ToTokens for Property<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let component_type = self.component_type();

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
            let component_ident = &*component_type.0;
            let name = component_ident.to_string();

            let component = if self.0.is_inline {
                let assert_component = format_ident!("_Assert{}", name);
                quote_spanned! {component_ident.span()=>
                    {
                        struct #assert_component where #component_ident: utoipa::Component;

                        <#component_ident as utoipa::Component>::component()
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
