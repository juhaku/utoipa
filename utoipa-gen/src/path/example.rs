use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parenthesized, Error, LitStr, Token, Type};

use crate::{parse_utils, AnyValue, Array, Diagnostics, ExamplesFrom};

// (name = (summary = "...", description = "...", value = "..", data_value = "..", serialized_value = "..", external_value = "..."))
#[derive(Default, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Example {
    pub(super) name: String,
    pub(super) summary: Option<String>,
    pub(super) description: Option<String>,
    pub(super) value: Option<AnyValue>,
    pub(super) data_value: Option<AnyValue>,
    pub(super) serialized_value: Option<String>,
    pub(super) external_value: Option<String>,
}

impl Parse for Example {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let example_stream;
        parenthesized!(example_stream in input);
        let mut example = Example {
            name: example_stream.parse::<LitStr>()?.value(),
            ..Default::default()
        };
        example_stream.parse::<Token![=]>()?;

        let content;
        parenthesized!(content in example_stream);

        while !content.is_empty() {
            let ident = content.parse::<Ident>()?;
            let attribute_name = &*ident.to_string();
            match attribute_name {
                "summary" => {
                    example.summary = Some(
                        parse_utils::parse_next(&content, || content.parse::<LitStr>())?
                            .value(),
                    )
                }
                "description" => {
                    example.description = Some(
                        parse_utils::parse_next(&content, || content.parse::<LitStr>())?
                            .value(),
                    )
                }
                "value" => {
                    example.value = Some(parse_utils::parse_next(&content, || {
                        AnyValue::parse_json(&content)
                    })?)
                }
                "data_value" => {
                    example.data_value = Some(parse_utils::parse_next(&content, || {
                        AnyValue::parse_json(&content)
                    })?)
                }
                "serialized_value" => {
                    example.serialized_value = Some(
                        parse_utils::parse_next(&content, || content.parse::<LitStr>())?
                            .value(),
                    )
                }
                "external_value" => {
                    example.external_value = Some(
                        parse_utils::parse_next(&content, || content.parse::<LitStr>())?
                            .value(),
                    )
                }
                _ => {
                    return Err(
                        Error::new(
                            ident.span(),
                            format!("unexpected attribute: {attribute_name}, expected one of: summary, description, value, data_value, serialized_value, external_value")
                        )
                    )
                }
            }

            if !content.is_empty() {
                content.parse::<Comma>()?;
            }
        }

        Ok(example)
    }
}

impl ToTokens for Example {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let summary = self
            .summary
            .as_ref()
            .map(|summary| quote!(.summary(#summary)));
        let description = self
            .description
            .as_ref()
            .map(|description| quote!(.description(#description)));
        let value = self
            .value
            .as_ref()
            .map(|value| quote!(.value(Some(#value))));
        let data_value = self
            .data_value
            .as_ref()
            .map(|data_value| quote!(.data_value(Some(#data_value))));
        let serialized_value = self
            .serialized_value
            .as_ref()
            .map(|serialized_value| quote!(.serialized_value(Some(#serialized_value))));
        let external_value = self
            .external_value
            .as_ref()
            .map(|external_value| quote!(.external_value(#external_value)));

        tokens.extend(quote! {
            utoipa::openapi::example::ExampleBuilder::new()
                #summary
                #description
                #value
                #data_value
                #serialized_value
                #external_value
        })
    }
}

/// Named examples of a media type or a parameter, either listed as
/// `examples(("name" = (...)), ...)` or given as `examples(from = <expr>)` where each item of
/// `<expr>` is a `(name, value)` tuple.
#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Examples {
    List(Punctuated<Example, Comma>),
    From(ExamplesFrom),
}

impl Default for Examples {
    fn default() -> Self {
        Self::List(Punctuated::new())
    }
}

impl Parse for Examples {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);

        if ExamplesFrom::peek(&content) {
            ExamplesFrom::parse(&content).map(Self::From)
        } else {
            Punctuated::parse_terminated(&content).map(Self::List)
        }
    }
}

impl Examples {
    /// Get `.examples_from_iter(...)` builder call, if any examples are defined. `ty` is the
    /// type values of `examples(from = ...)` are converted to.
    pub fn to_examples_from_iter(
        &self,
        ty: Option<&Type>,
    ) -> Result<Option<TokenStream>, Diagnostics> {
        match self {
            Self::List(examples) if examples.is_empty() => Ok(None),
            Self::List(examples) => {
                let examples = examples
                    .iter()
                    .map(|example| {
                        let name = &example.name;
                        quote!( (#name, #example) )
                    })
                    .collect::<Array<TokenStream>>();
                Ok(Some(quote!( .examples_from_iter(#examples) )))
            }
            Self::From(from) => {
                let ty = ty.ok_or_else(|| {
                    Diagnostics::with_span(
                        from.span(),
                        "`examples(from = ...)` requires a type to check the examples against",
                    )
                    .help("Define the type explicitly, e.g. `content = Pet` or `body = Pet`")
                })?;
                // `[T]` is used for arrays in content, examples of it are `Vec<T>`
                let ty = match ty {
                    Type::Slice(slice) => {
                        let elem = &slice.elem;
                        quote!(::std::vec::Vec<#elem>)
                    }
                    ty => ty.to_token_stream(),
                };
                let examples = from.to_named_examples(&ty);
                Ok(Some(quote!( .examples_from_iter(#examples) )))
            }
        }
    }
}
