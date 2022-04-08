use std::fmt::Display;

use quote::{quote, ToTokens};

use crate::component_type::{ComponentFormat, ComponentType};

/// Tokenizable object property. It is used as a object property for components or as property
/// of request or response body or response header.
pub(crate) struct Property<'a, T: Display> {
    pub(crate) is_array: bool,
    pub(crate) component_type: ComponentType<'a, T>,
}

impl<'a, T> Property<'a, T>
where
    T: Display,
{
    pub fn new(is_array: bool, ident: &'a T) -> Self {
        Self {
            is_array,
            component_type: ComponentType(ident),
        }
    }
}

impl<T> ToTokens for Property<'_, T>
where
    T: Display,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.component_type.is_primitive() {
            let component_type = &self.component_type;
            let mut component = quote! {
                utoipa::openapi::PropertyBuilder::new().component_type(#component_type)
            };

            let format = ComponentFormat(self.component_type.0);
            if format.is_known_format() {
                component.extend(quote! {
                    .format(Some(#format))
                })
            }

            tokens.extend(component);
        } else {
            let name = &*self.component_type.0.to_string();

            tokens.extend(quote! {
                utoipa::openapi::Ref::from_component_name(#name)
            })
        };

        if self.is_array {
            tokens.extend(quote! {
                .to_array_builder()
            });
        }
    }
}
