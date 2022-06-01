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
        let t = match &self.type_definition {
            TypeDefinition::Component(t) => t,
            TypeDefinition::Inline(t) => t,
        };
        ComponentType(&t.ty)
    }
}

impl ToTokens for Property<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match &self.type_definition {
            TypeDefinition::Component(component_type_definition) => {
                let component_type = ComponentType(&component_type_definition.ty);
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

                    tokens.extend(component);
                } else {
                    let name = &*component_type.0.to_string();

                    tokens.extend(quote! {
                        utoipa::openapi::Ref::from_component_name(#name)
                    })
                };

                if component_type_definition.is_array {
                    tokens.extend(quote! {
                        .to_array_builder()
                    });
                }
            }
            TypeDefinition::Inline(_) => todo!(),
        }
    }
}
