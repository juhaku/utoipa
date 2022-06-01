use std::borrow::Cow;

use proc_macro2::Ident;
use quote::{quote, ToTokens};

use crate::{
    component_type::{ComponentFormat, ComponentType},
    TypeDefinition,
};

/// Tokenizable object property. It is used as a object property for components or as property
/// of request or response body or response header.
// TODO: Switch this to support either references or inline definition.
// file:///home/luke/programming/rust/utoipa/target/doc/utoipa/openapi/schema/enum.Component.html
pub(crate) struct Property<'a> {
    type_definition: TypeDefinition<'a>,
}

impl<'a> Property<'a> {
    pub fn new(type_definition: TypeDefinition<'a>) -> Self {
        Self { type_definition }
    }

    pub fn component_type(&self) -> ComponentType<'a, Cow<Ident>> {
        ComponentType(&self.type_definition_type().ty)
    }

    pub fn type_definition_type(&self) -> &crate::Type<'a> {
        match &self.type_definition {
            TypeDefinition::Component(t) => t,
            TypeDefinition::Inline(t) => t,
        }
    }
}

impl ToTokens for Property<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let component_type: ComponentType<_> = self.component_type();
        let type_definition_type = self.type_definition_type();

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

            if type_definition_type.is_array {
                component.extend(quote! {
                    .to_array_builder()
                });
            }

            tokens.extend(component);
        } else {
            let component_name_ident: &Ident = &*component_type.0;
            let name = component_name_ident.to_string();

            match self.type_definition {
                TypeDefinition::Component(_) => {
                    tokens.extend(quote! {
                        utoipa::openapi::Ref::from_component_name(#name)
                    });

                    if type_definition_type.is_array {
                        tokens.extend(quote! {
                            .to_array_builder()
                        });
                    }
                }
                TypeDefinition::Inline(_) => tokens.extend(quote! {
                    <#component_name_ident as utoipa::Component>::component()
                }),
            }
        }
    }
}
