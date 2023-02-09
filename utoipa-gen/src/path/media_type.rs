use std::ops::Deref;

use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;

use crate::{
    component::{schema, GenericType, TypeTree, ValueType},
    schema_type::{SchemaFormat, SchemaType},
};

pub(super) struct MediaTypeSchema<'t> {
    pub(super) type_tree: &'t TypeTree<'t>,
    pub(super) is_inline: bool,
}

impl ToTokens for MediaTypeSchema<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let MediaTypeSchema {
            type_tree,
            is_inline,
        } = &self;
        match type_tree.generic_type {
            Some(GenericType::Vec) => {
                let child = type_tree
                    .children
                    .as_ref()
                    .expect("Vec must have children")
                    .first()
                    .expect("Vec must have one child type");

                if child
                    .path
                    .as_ref()
                    .map(|path| SchemaType(path).is_byte())
                    .unwrap_or(false)
                {
                    tokens.extend(quote! {
                        utoipa::openapi::ObjectBuilder::new()
                            .schema_type(utoipa::openapi::schema::SchemaType::String)
                            .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(utoipa::openapi::KnownFormat::Binary)))
                    })
                } else {
                    let media_type_schema = MediaTypeSchema {
                        type_tree: type_tree
                            .children
                            .as_ref()
                            .expect("Vec must have children")
                            .first()
                            .expect("Vec must have one child type"),
                        is_inline: *is_inline,
                    };
                    tokens.extend(quote! {
                        utoipa::openapi::schema::ArrayBuilder::new()
                            .items(#media_type_schema)
                    })
                }
            }
            Some(GenericType::Map) => {
                let media_type_schema = MediaTypeSchema {
                    type_tree: type_tree
                        .children
                        .as_ref()
                        .expect("Map should have children")
                        .iter()
                        .nth(1)
                        .expect("Map should have 2 child types"),
                    is_inline: *is_inline,
                };

                tokens.extend(quote! {
                    utoipa::openapi::ObjectBuilder::new()
                        .additional_properties(Some(#media_type_schema))
                });
            }
            Some(GenericType::Option)
            | Some(GenericType::Box)
            | Some(GenericType::RefCell)
            | Some(GenericType::Cow) => {
                let media_type_schema = MediaTypeSchema {
                    type_tree: type_tree
                        .children
                        .as_ref()
                        .expect("Box, RefCell, Cow, Option must have children")
                        .first()
                        .expect("Box, RefCell, Cow, Option, must have one child type"),
                    is_inline: *is_inline,
                };

                tokens.extend(media_type_schema.to_token_stream())
            }
            None => {
                match type_tree.value_type {
                    ValueType::Primitive => {
                        let path = type_tree
                            .path
                            .as_ref()
                            .expect("ValueType::Primitive must have path")
                            .deref();

                        let schema_type = SchemaType(path);
                        tokens.extend(quote! {
                            utoipa::openapi::ObjectBuilder::new()
                                .schema_type(#schema_type)
                        });
                        let format: SchemaFormat = path.into();
                        if format.is_known_format() {
                            tokens.extend(quote! {
                                .format(Some(#format))
                            })
                        }
                    }
                    ValueType::Object => {
                        let path = type_tree
                            .path
                            .as_ref()
                            .expect("ValueType::Object must have path")
                            .deref();

                        if type_tree.is_object() {
                            tokens.extend(quote! {
                                utoipa::openapi::ObjectBuilder::new()
                            })
                        } else if *is_inline {
                            tokens.extend(quote_spanned! {path.span()=>
                                <#path as utoipa::ToSchema>::schema().1
                            })
                        } else {
                            let name = type_tree
                                .path
                                .as_ref()
                                .expect("ValueType::Object must have path");

                            let name = schema::format_path_ref(name);
                            tokens.extend(quote! {
                                utoipa::openapi::Ref::from_schema_name(#name)
                            });
                        }
                    }
                    // TODO support for tuple types
                    ValueType::Tuple => {
                        // Detect unit type ()
                        if type_tree.children.is_none() {
                            tokens.extend(quote! {
                                utoipa::openapi::schema::empty()
                            })
                        };
                    }
                }
            }
        };
    }
}
