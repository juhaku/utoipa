use std::borrow::Cow;

use quote::{quote, ToTokens};
use syn::TypePath;

use crate::{
    component_type::{ComponentFormat, ComponentType},
    schema::component::format_path_ref,
    Type,
};

/// Tokenizable object property. It is used as a object property for components or as property
/// of request or response body or response header.
pub(crate) struct Property<'a> {
    type_definition: Type<'a>,
}

impl<'a> Property<'a> {
    pub fn new(type_definition: Type<'a>) -> Self {
        Self { type_definition }
    }

    pub fn component_type(&self) -> ComponentType<'a, Cow<TypePath>> {
        ComponentType(&self.type_definition.ty)
    }
}

impl ToTokens for Property<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let component_type: ComponentType<_> = self.component_type();

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

            if self.type_definition.is_array {
                component.extend(quote! {
                    .to_array_builder()
                });
            }

            tokens.extend(component);
        } else {
            let component_name_path: &TypePath = &*component_type.0;
            let name = format_path_ref(&component_name_path.to_token_stream().to_string());

            if self.type_definition.is_inline {
                let component = quote! {
                    <#component_name_path as utoipa::Component>::component()
                };

                if self.type_definition.is_array {
                    let array_component = quote! {
                        utoipa::openapi::schema::ArrayBuilder::new()
                            .items(#component)
                    };

                    tokens.extend(array_component);
                } else {
                    tokens.extend(component);
                }
            } else {
                tokens.extend(quote! {
                    utoipa::openapi::Ref::from_component_name(#name)
                });

                if self.type_definition.is_array {
                    tokens.extend(quote! {
                        .to_array_builder()
                    });
                }
            }
        }
    }
}
