use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    Token,
};

use crate::parse_utils;

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct SecurityRequirementsAttrItem {
    pub name: Option<String>,
    pub scopes: Option<Vec<parse_utils::LitStrOrExpr>>,
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct SecurityRequirementsAttr(Punctuated<SecurityRequirementsAttrItem, Comma>);

impl Parse for SecurityRequirementsAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Punctuated::<SecurityRequirementsAttrItem, Comma>::parse_terminated(input)
            .map(|o| Self(o.into_iter().collect()))
    }
}

impl Parse for SecurityRequirementsAttrItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::LitStr>()?.value();

        input.parse::<Token![=]>()?;

        let scopes_stream;
        bracketed!(scopes_stream in input);

        let scopes =
            Punctuated::<parse_utils::LitStrOrExpr, Comma>::parse_terminated(&scopes_stream)?
                .into_iter()
                .collect::<Vec<_>>();

        Ok(Self {
            name: Some(name),
            scopes: Some(scopes),
        })
    }
}

impl ToTokens for SecurityRequirementsAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! {
            utoipa::openapi::security::SecurityRequirement::default()
        });

        for requirement in &self.0 {
            if let (Some(name), Some(scopes)) = (&requirement.name, &requirement.scopes) {
                let scopes_tokens = scopes.iter().map(|scope| match scope {
                    parse_utils::LitStrOrExpr::LitStr(lit) => quote! { #lit.to_string() },
                    parse_utils::LitStrOrExpr::Expr(expr) => quote! { #expr.to_string() },
                });
                let scopes_len = scopes.len();

                tokens.extend(quote! {
                    .add::<&str, [String; #scopes_len], String>(#name, [#(#scopes_tokens),*])
                });
            }
        }
    }
}
