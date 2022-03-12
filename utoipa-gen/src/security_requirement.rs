use proc_macro2::{Group, TokenStream};
use proc_macro_error::ResultExt;
use quote::{quote, ToTokens};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    LitStr, Token,
};

use crate::{parse_utils, Array};

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct SecurityRequirementAttr {
    name: Option<String>,
    scopes: Option<Vec<String>>,
}

impl Parse for SecurityRequirementAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self {
                ..Default::default()
            });
        }
        let name = input.parse::<LitStr>()?.value();
        input.parse::<Token![=]>()?;

        let scopes_stream;
        bracketed!(scopes_stream in input);
        let scopes = Punctuated::<LitStr, Comma>::parse_terminated(&scopes_stream)?
            .iter()
            .map(LitStr::value)
            .collect::<Vec<_>>();

        Ok(Self {
            name: Some(name),
            scopes: Some(scopes),
        })
    }
}

pub fn parse_security_requirements(
    stream: ParseStream,
) -> syn::Result<Vec<SecurityRequirementAttr>> {
    parse_utils::parse_next(stream, || {
        let content;
        bracketed!(content in stream);

        let groups = Punctuated::<Group, Comma>::parse_terminated(&content)?;

        Ok(groups
            .into_iter()
            .map(|parameter_group| {
                syn::parse2::<SecurityRequirementAttr>(parameter_group.stream()).unwrap_or_abort()
            })
            .collect::<Vec<_>>())
    })
}

pub fn security_requirements_to_tokens(
    security_requirements: &[SecurityRequirementAttr],
) -> TokenStream {
    security_requirements.iter().map(| security | {
        if let (Some(name), Some(scopes)) = (&security.name, &security.scopes) {
            let scopes_array = scopes.iter().collect::<Array<&String>>();
            let scopes_len = scopes.len();

            quote! {
                utoipa::openapi::security::SecurityRequirement::new::<&str, [&str; #scopes_len], &str>(#name, #scopes_array)
            }
        } else {
            quote!{
                utoipa::openapi::security::SecurityRequirement::default()
            }
        }
    }).collect::<Array<TokenStream>>().into_token_stream()
}
