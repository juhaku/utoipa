use proc_macro2::Ident;
use proc_macro_error::ResultExt;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Error, ExprPath, Token,
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};

use crate::parse_utils;

mod info;

const PATH_STRUCT_PREFIX: &str = "__path_";

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct OpenApiAttr {
    handlers: Vec<ExprPath>,
    components: Vec<Ident>,
}

pub fn parse_openapi_attributes_from_attributes(attrs: &[Attribute]) -> Option<OpenApiAttr> {
    attrs
        .iter()
        .find(|attribute| attribute.path.is_ident("openapi"))
        .map(|attribute| attribute.parse_args::<OpenApiAttr>().unwrap_or_abort())
}

impl Parse for OpenApiAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut openapi = OpenApiAttr::default();

        loop {
            let ident = input
                .parse::<Ident>()
                .expect_or_abort("unaparseable OpenApi, expected Ident");
            let attribute = &*ident.to_string();

            match attribute {
                "handlers" => {
                    openapi.handlers = parse_handlers(input).unwrap_or_abort();
                }
                "components" => {
                    openapi.components = parse_components(input).unwrap_or_abort();
                }
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        format!(
                            "unexpected attribute: {}, expected: handlers, components",
                            ident
                        ),
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>().unwrap_or_abort();
            }
            if input.is_empty() {
                break;
            }
        }

        Ok(openapi)
    }
}

fn parse_handlers(input: ParseStream) -> syn::Result<Vec<ExprPath>> {
    parse_utils::parse_next(input, || {
        if input.peek(syn::token::Bracket) {
            let content;
            bracketed!(content in input);
            let tokens = Punctuated::<ExprPath, Token![,]>::parse_terminated(&content)?;

            Ok(tokens.into_iter().collect::<Vec<_>>())
        } else {
            Err(Error::new(
                input.span(),
                "unparseable handlers, expected Bracket Token [...]",
            ))
        }
    })
}

fn parse_components(input: ParseStream) -> syn::Result<Vec<Ident>> {
    parse_utils::parse_next(input, || {
        if input.peek(syn::token::Bracket) {
            let content;
            bracketed!(content in input);
            let tokens = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?;

            Ok(tokens.into_iter().collect::<Vec<_>>())
        } else {
            Err(syn::Error::new(
                input.span(),
                "unparseable components, expected Bracket Token [...]",
            ))
        }
    })
}

pub(crate) struct OpenApi(pub OpenApiAttr, pub Ident);

impl ToTokens for OpenApi {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let OpenApi(attributes, ident) = self;

        let info = info::impl_info();

        let schema = attributes.components.iter().fold(
            quote! { utoipa::openapi::Schema::new() },
            |mut schema, component| {
                let span = component.span();
                let component_name = &*component.to_string();

                let assert_component = format_ident!("_AssertComponent{}", component_name);
                tokens.extend(quote_spanned! {span=>
                    struct #assert_component where #component: utoipa::Component;
                });

                schema.extend(quote! {
                    .with_component(#component_name, #component::component())
                });

                schema
            },
        );

        let path_items = impl_paths(&attributes.handlers, tokens);

        tokens.extend(quote! {
            use utoipa::openapi::schema::ToArray;
            impl utoipa::OpenApi for #ident {
                fn openapi() -> utoipa::openapi::OpenApi {
                    utoipa::openapi::OpenApi::new(#info, #path_items)
                        .with_components(#schema)
                }
            }
        });
    }
}

fn impl_paths(handler_paths: &[ExprPath], quote: &mut TokenStream) -> TokenStream {
    quote.extend(quote! {
        use utoipa::Path as OpenApiPath;
    });
    handler_paths.iter().fold(
        quote! { utoipa::openapi::path::Paths::new() },
        |mut paths, handler| {
            let segments = handler.path.segments.iter().collect::<Vec<_>>();
            let handler_fn_name = &*segments.last().unwrap().ident.to_string();

            let tag = segments
                .iter()
                .take(segments.len() - 1)
                .map(|part| part.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            let handler_ident = format_ident!("{}{}", PATH_STRUCT_PREFIX, handler_fn_name);
            let handler_ident_name = &*handler_ident.to_string();

            let usage = syn::parse_str::<ExprPath>(
                &vec![
                    if tag.starts_with("crate") {
                        None
                    } else {
                        Some("crate")
                    },
                    if tag.is_empty() { None } else { Some(&tag) },
                    Some(handler_ident_name),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join("::"),
            )
            .unwrap();

            let assert_handler_ident = format_ident!("__assert_{}", handler_ident_name);
            quote.extend(quote! {
                struct #assert_handler_ident where #handler_ident : utoipa::Path;
                use #usage;
                impl utoipa::DefaultTag for #handler_ident {
                    fn tag() -> &'static str {
                        #tag
                    }
                }
            });
            paths.extend(quote! {
                .append(#handler_ident::path(), #handler_ident::path_item())
            });

            paths
        },
    )
}
