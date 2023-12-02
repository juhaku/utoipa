use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    LitStr, Token,
};

use crate::Array;

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct SecurityRequirementsAttrItem {
    pub name: String,
    pub scopes: Vec<String>,
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct SecurityRequirementsAttr(Vec<SecurityRequirementsAttrItem>);

impl Parse for SecurityRequirementsAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();

        if input.is_empty() {
            return Ok(Self(items));
        }

        items.push(input.parse::<SecurityRequirementsAttrItem>()?);

        while input.lookahead1().peek(Token![,]) {
            input.parse::<Token![,]>()?;
            items.push(input.parse::<SecurityRequirementsAttrItem>()?);
        }

        Ok(Self(items))
    }
}

impl Parse for SecurityRequirementsAttrItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<LitStr>()?.value();

        if input.lookahead1().peek(Token![=]) {
            input.parse::<Token![=]>()?;

            let scopes_stream;
            bracketed!(scopes_stream in input);

            let scopes = Punctuated::<LitStr, Comma>::parse_terminated(&scopes_stream)?
                .iter()
                .map(LitStr::value)
                .collect::<Vec<_>>();

            Ok(Self { name, scopes })
        } else {
            Ok(Self {
                name,
                scopes: vec![],
            })
        }
    }
}

impl ToTokens for SecurityRequirementsAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! {
            utoipa::openapi::security::SecurityRequirement::new()
        });

        for requirement in &self.0 {
            let name = &requirement.name;
            let scopes = requirement.scopes.iter().collect::<Array<&String>>();
            let scopes_len = scopes.len();

            tokens.extend(quote! {
                .add::<&str, [&str; #scopes_len], &str>(#name, #scopes)
            });
        }
    }
}
