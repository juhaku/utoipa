use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;

use crate::{
    component::schema,
    schema_type::{SchemaFormat, SchemaType},
    Type,
};

/// Tokenizable object property. It is used as a object property for components or as property
/// of request or response body or response header.
pub(crate) struct Property<'a>(&'a Type<'a>);

impl<'a> Property<'a> {
    pub fn new(type_definition: &'a Type<'a>) -> Self {
        Self(type_definition)
    }

    pub fn schema_type(&'a self) -> SchemaType<'a> {
        SchemaType(&self.0.ty)
    }
}

impl ToTokens for Property<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let schema_type = self.schema_type();

        if schema_type.is_primitive() {
            let mut schema = quote! {
                utoipa::openapi::ObjectBuilder::new().schema_type(#schema_type)
            };

            let format: SchemaFormat = schema_type.0.into();
            if format.is_known_format() {
                schema.extend(quote! {
                    .format(Some(#format))
                })
            }

            tokens.extend(if self.0.is_array {
                quote! {
                    utoipa::openapi::schema::ArrayBuilder::new()
                        .items(#schema)
                }
            } else {
                schema
            });
        } else {
            let schema_name_path = schema_type.0;

            let schema = if self.0.is_inline {
                quote_spanned! { schema_name_path.span()=>
                    <#schema_name_path as utoipa::ToSchema>::schema()
                }
            } else {
                let name = schema::format_path_ref(schema_name_path);
                quote! {
                    utoipa::openapi::Ref::from_schema_name(#name)
                }
            };

            tokens.extend(if self.0.is_array {
                quote! {
                    utoipa::openapi::schema::ArrayBuilder::new()
                        .items(#schema)
                }
            } else {
                schema
            });
        }
    }
}
