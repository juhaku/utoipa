use syn::punctuated::Punctuated;
use syn::Token;

use crate::component::features::{Feature, Parse};
use crate::features::impl_feature;
use crate::{parse_utils, AnyValue};
use quote::ToTokens;

impl_feature! {
    /// Parse the following into a set of extensions:
    /// ```text
    /// extensions(
    ///   ("foo_extension" = json!("foo")),
    ///   ("bar_extension" = json!("bar")),
    /// )
    /// ```
    #[derive(Clone, Default)]
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct Extensions {
        extensions: Vec<Extension>,
    }
}

impl Parse for Extensions {
    fn parse(input: syn::parse::ParseStream, _: proc_macro2::Ident) -> syn::Result<Self> {
        syn::parse::Parse::parse(input)
    }
}

impl syn::parse::Parse for Extensions {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let params;
        syn::parenthesized!(params in input);
        let extensions = Punctuated::<Extension, Token![,]>::parse_terminated(&params)
            .map(|punctuated| punctuated.into_iter().collect::<Vec<Extension>>())?;
        Ok(Extensions { extensions })
    }
}

impl ToTokens for Extensions {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let extensions = &self.extensions;
        tokens.extend(quote::quote! {
          utoipa::openapi::extensions::ExtensionsBuilder::new() #(#extensions)* .build()
        });
    }
}

impl From<Extensions> for Feature {
    fn from(value: Extensions) -> Self {
        Feature::Extensions(value)
    }
}

/// Parse the following into an extension:
/// ```text
/// ("foo_extension" = json!("value"))
/// ```
#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Extension {
    name: parse_utils::LitStrOrExpr,
    value: AnyValue, // <- Expect variant AnyValue::Json
}

impl syn::parse::Parse for Extension {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let inner;
        syn::parenthesized!(inner in input);
        let name = inner.parse::<parse_utils::LitStrOrExpr>()?;

        inner.parse::<Token![=]>()?;

        let value = AnyValue::parse_json(&inner)?;
        Ok(Extension { name, value })
    }
}

impl ToTokens for Extension {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let value = &self.value;
        tokens.extend(quote::quote! {
            .add(#name, #value)
        });
    }
}
