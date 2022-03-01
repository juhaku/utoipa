use proc_macro2::Ident;
use proc_macro_error::ResultExt;
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{And, Comma},
    Attribute, Error, ExprPath, GenericParam, Generics, Token,
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};

use crate::{
    parse_utils,
    security_requirement::{self, SecurityRequirementAttr},
};

mod info;

const PATH_STRUCT_PREFIX: &str = "__path_";

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct OpenApiAttr {
    handlers: Vec<ExprPath>,
    components: Vec<Component>,
    modifiers: Punctuated<Modifier, Comma>,
    security: Option<Vec<SecurityRequirementAttr>>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct Component {
    ty: Ident,
    generics: Generics,
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct Modifier {
    and: And,
    ident: Ident,
}

impl ToTokens for Modifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let and = &self.and;
        let ident = &self.ident;
        tokens.extend(quote! {
            #and #ident
        })
    }
}

impl Parse for Modifier {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            and: input.parse()?,
            ident: input.parse()?,
        })
    }
}

impl Component {
    fn has_lifetime_generics(&self) -> bool {
        self.generics
            .params
            .iter()
            .any(|generic| matches!(generic, GenericParam::Lifetime(_)))
    }
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
                    openapi.handlers = parse_handlers(input)?;
                }
                "components" => {
                    openapi.components = parse_components(input)?;
                }
                "modifiers" => {
                    openapi.modifiers = parse_modifiers(input)?;
                }
                "security" => {
                    openapi.security = Some(
                        security_requirement::parse_security_requirements(input)?
                            .into_iter()
                            .collect::<Vec<_>>(),
                    )
                }
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        format!(
                            "unexpected attribute: {}, expected: handlers, components, modifiers, security",
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

fn parse_components(input: ParseStream) -> syn::Result<Vec<Component>> {
    parse_utils::parse_next(input, || {
        if input.peek(syn::token::Bracket) {
            let content;
            bracketed!(content in input);

            let mut components = Vec::new();
            loop {
                components.push(Component {
                    ty: content.parse()?,
                    generics: content.parse()?,
                });

                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
                if content.is_empty() {
                    break;
                }
            }

            Ok(components)
        } else {
            Err(syn::Error::new(
                input.span(),
                "unparseable components, expected Bracket Token [...]",
            ))
        }
    })
}

fn parse_modifiers(input: ParseStream) -> syn::Result<Punctuated<Modifier, Comma>> {
    parse_utils::parse_next(input, || {
        let content;
        bracketed!(content in input);

        Punctuated::<Modifier, Comma>::parse_terminated(&content)
    })
}

pub(crate) struct OpenApi(pub OpenApiAttr, pub Ident);

impl ToTokens for OpenApi {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let OpenApi(attributes, ident) = self;

        let info = info::impl_info();

        let components = attributes.components.iter().fold(
            quote! { utoipa::openapi::Components::new() },
            |mut schema, component| {
                let ident = &component.ty;
                let span = ident.span();
                let component_name = &*ident.to_string();
                let (_, ty_generics, _) = component.generics.split_for_impl();

                let assert_ty_generics = if component.has_lifetime_generics() {
                    Some(quote! {<'static>})
                } else {
                    Some(ty_generics.to_token_stream())
                };
                let assert_component = format_ident!("_AssertComponent{}", component_name);
                tokens.extend(quote_spanned! {span=>
                    struct #assert_component where #ident #assert_ty_generics: utoipa::Component;
                });

                let ty_generics = if component.has_lifetime_generics() {
                    None
                } else {
                    Some(ty_generics)
                };
                schema.extend(quote! {
                    .with_component(#component_name, <#ident #ty_generics>::component())
                });

                schema
            },
        );

        let modifiers = &self.0.modifiers;
        let modifiers_len = modifiers.len();

        modifiers.iter().for_each(|modifier| {
            let assert_modifier = format_ident!("_Assert{}", modifier.ident);
            let ident = &modifier.ident;
            quote_spanned! {modifier.ident.span()=>
                struct #assert_modifier where #ident : utoipa::Modify;
            };
        });

        let path_items = impl_paths(&attributes.handlers);

        let securities = if let Some(ref securities) = self.0.security {
            let securities_tokens =
                security_requirement::security_requirements_to_tokens(securities);
            Some(quote! {
                .with_securities(#securities_tokens)
            })
        } else {
            None
        };

        tokens.extend(quote! {
            impl utoipa::OpenApi for #ident {
                fn openapi() -> utoipa::openapi::OpenApi {
                    use utoipa::{Component, Path};
                    let mut openapi = utoipa::openapi::OpenApi::new(#info, #path_items)
                        .with_components(#components)
                        #securities;

                    let _mods: [&dyn utoipa::Modify; #modifiers_len] = [#modifiers];
                    _mods.iter().for_each(|modifier| modifier.modify(&mut openapi));

                    openapi
                }
            }
        });
    }
}

fn impl_paths(handler_paths: &[ExprPath]) -> TokenStream {
    handler_paths.iter().fold(
        quote! { utoipa::openapi::path::Paths::new() },
        |mut paths, handler| {
            let segments = handler.path.segments.iter().collect::<Vec<_>>();
            let handler_fn_name = &*segments.last().unwrap().ident.to_string();

            let tag = &*segments
                .iter()
                .take(segments.len() - 1)
                .map(|part| part.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            let handler_ident = format_ident!("{}{}", PATH_STRUCT_PREFIX, handler_fn_name);
            let handler_ident_name = &*handler_ident.to_string();

            let usage = syn::parse_str::<ExprPath>(
                &vec![
                    if tag.is_empty() { None } else { Some(tag) },
                    Some(handler_ident_name),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join("::"),
            )
            .unwrap();

            paths.extend(quote! {
                .append(#usage::path(), #usage::path_item(Some(#tag)))
            });

            paths
        },
    )
}
