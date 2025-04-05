use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::Token;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    Error, Ident, LitStr,
};

use crate::{parse_utils, Array};

// (url = "http:://url", description = "description", variables(...))
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Server {
    url: parse_utils::LitStrOrExpr,
    description: Option<String>,
    variables: Punctuated<ServerVariable, Comma>,
}

impl Parse for Server {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let server_stream;
        parenthesized!(server_stream in input);
        let mut server = Server::default();
        while !server_stream.is_empty() {
            let ident = server_stream.parse::<Ident>()?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "url" => {
                    server.url = parse_utils::parse_next_literal_str_or_expr(&server_stream)?
                }
                "description" => {
                    server.description =
                        Some(parse_utils::parse_next(&server_stream, || server_stream.parse::<LitStr>())?.value())
                }
                "variables" => {
                    server.variables = parse_utils::parse_comma_separated_within_parenthesis(&server_stream)?
                }
                _ => {
                    return Err(Error::new(ident.span(), format!("unexpected attribute: {attribute_name}, expected one of: url, description, variables")))
                }
            }

            if !server_stream.is_empty() {
                server_stream.parse::<Comma>()?;
            }
        }

        Ok(server)
    }
}

impl ToTokens for Server {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let url = &self.url;
        let description = &self
            .description
            .as_ref()
            .map(|description| quote! { .description(Some(#description)) });

        let parameters = self
            .variables
            .iter()
            .map(|variable| {
                let name = &variable.name;
                let default_value = &variable.default;
                let description = &variable
                    .description
                    .as_ref()
                    .map(|description| quote! { .description(Some(#description)) });
                let enum_values = &variable.enum_values.as_ref().map(|enum_values| {
                    let enum_values = enum_values.iter().collect::<Array<&LitStr>>();

                    quote! { .enum_values(Some(#enum_values)) }
                });

                quote! {
                    .parameter(#name, utoipa::openapi::server::ServerVariableBuilder::new()
                        .default_value(#default_value)
                        #description
                        #enum_values
                    )
                }
            })
            .collect::<TokenStream>();

        tokens.extend(quote! {
            utoipa::openapi::server::ServerBuilder::new()
                .url(#url)
                #description
                #parameters
                .build()
        })
    }
}

// ("username" = (default = "demo", description = "This is default username for the API")),
// ("port" = (enum_values = (8080, 5000, 4545)))
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
struct ServerVariable {
    name: String,
    default: String,
    description: Option<String>,
    enum_values: Option<Punctuated<LitStr, Comma>>,
}

impl Parse for ServerVariable {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let variable_stream;
        parenthesized!(variable_stream in input);
        let mut server_variable = ServerVariable {
            name: variable_stream.parse::<LitStr>()?.value(),
            ..ServerVariable::default()
        };

        variable_stream.parse::<Token![=]>()?;
        let content;
        parenthesized!(content in variable_stream);

        while !content.is_empty() {
            let ident = content.parse::<Ident>()?;
            let attribute_name = &*ident.to_string();

            match attribute_name {
                "default" => {
                    server_variable.default =
                        parse_utils::parse_next(&content, || content.parse::<LitStr>())?.value()
                }
                "description" => {
                    server_variable.description =
                        Some(parse_utils::parse_next(&content, || content.parse::<LitStr>())?.value())
                }
                "enum_values" => {
                    server_variable.enum_values =
                        Some(parse_utils::parse_comma_separated_within_parenthesis(&content)?)
                }
                _ => {
                    return Err(Error::new(ident.span(), format!( "unexpected attribute: {attribute_name}, expected one of: default, description, enum_values")))
                }
            }

            if !content.is_empty() {
                content.parse::<Comma>()?;
            }
        }

        Ok(server_variable)
    }
}
