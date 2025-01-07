use super::*;

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Extensions<'a> {
  phantom_data: std::marker::PhantomData<&'a usize>,
  extensions: Vec<Extension>,
}

impl syn::parse::Parse for Extensions<'_> {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let params;
    parenthesized!(params in input);
    let extensions 
      = Punctuated::<Extension, Token![,]>::parse_terminated(&params)
        .map(|punctuated| punctuated.into_iter().collect::<Vec<Extension>>())?;
    Ok(Extensions { 
      phantom_data: std::marker::PhantomData,
      extensions, 
    })
  }
}

impl ToTokensDiagnostics for Extensions<'_> {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), Diagnostics> {
    let mut quote = quote! {
            utoipa::openapi::extensions::ExtensionsBuilder::new()
    };
    for extension in self.extensions.iter() {
      extension.to_tokens(&mut quote)?;
    }
    tokens.extend(quote! { 
      .extensions(Some(
        #quote
        .build()
      ))
    });
    Ok(())
  }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Extension {
  Tuple(ExtensionTuple),
}

impl Parse for Extension {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let response;
    parenthesized!(response in input);
    Ok(Self::Tuple(response.parse()?))
  }
}

impl ToTokensDiagnostics for Extension {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), Diagnostics> {
    match self {
      Extension::Tuple(e) => e.to_tokens(tokens),
    }
  }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ExtensionTuple {
  property: String,
  value: TokenStream2,
}

impl syn::parse::Parse for ExtensionTuple {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let mut property = None; let mut value = None;
    let span = input.span();

    while ! input.is_empty() {
      let ident = input.parse::<Ident>().map_err(|error| {
        syn::Error::new(
          error.span(),
          format!("Unexpected attribute. {error}"),
        )
      })?;
      let name = &*ident.to_string();
      match name {
        "property" => {
          property = Some(parse_utils::parse_next_literal_str(input)?);
        },
        "value" => {
          value = Some(parse_utils::parse_next(input, || parse_utils::parse_json_token_stream(input))?);
        },
        _ => {
          return Err(syn::Error::new(span, format!("Unexpected attribute {name}")));
        },
      }

      if !input.is_empty() { input.parse::<Token![,]>()?; }
    }

    if let (Some(property), Some(value)) = (property, value) {
      Ok(ExtensionTuple { property, value })
    } else {
      Err(syn::Error::new(span, "Property and/or is not set"))
    }
  }
}

impl ToTokensDiagnostics for ExtensionTuple {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), Diagnostics> {
    let k = self.property.as_str(); let json = &self.value;
    tokens.extend(quote! {
      .add(#k, serde_json::json!(#json))
    });
    Ok(())
  }
}
