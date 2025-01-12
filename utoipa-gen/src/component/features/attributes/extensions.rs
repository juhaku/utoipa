use super::*;

/// Parse the following into a set of extensions:
/// ```
/// extensions(
///   ("foo_extension" = json!("foo")),
///   ("bar_extension" = json!("bar")),
/// )
/// ```
impl_feature! {
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
    let extensions 
      = Punctuated::<Extension, Token![,]>::parse_terminated(&params)
        .map(|punctuated| {
          punctuated.into_iter().collect::<Vec<Extension>>()
        })?;
    Ok(Extensions { 
      extensions, 
    })
  }
}

impl ToTokens for Extensions {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let extensions = &self.extensions;
    tokens.extend(quote! {
      utoipa::openapi::extensions::ExtensionsBuilder::new() #(#extensions)* .build()
    });
  }
}

impl crate::ToTokensDiagnostics for Extensions {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), Diagnostics> {
    tokens.extend(quote! {
      .extensions(Some( #self ))
    });
    Ok(())
  }
}

impl From<Extensions> for Feature {
  fn from(value: Extensions) -> Self {
    Feature::Extensions(value)
  }
}

/// Parse the following into an extension:
/// ```
/// ("foo_extension" = json!("value"))
/// ```
#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Extension {
  name: parse_utils::LitStrOrExpr,
  value: crate::TokenStream2,
}

impl syn::parse::Parse for Extension {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let inner;
    syn::parenthesized!(inner in input);
    let name = inner.parse::<parse_utils::LitStrOrExpr>()?;
    
    inner.parse::<Token![=]>()?;

    let value = parse_utils::parse_json_token_stream(&inner)?;
    Ok(Extension { name, value })
  }
}

impl ToTokens for Extension {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let name = &self.name; let value = &self.value;
    tokens.extend(quote! {
      .add(#name, serde_json::json!(#value))
    });
  }
}

/*
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Extension {
  Tuple(ExtensionTuple),
}

impl Parse for Extension {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let response;
    syn::parenthesized!(response in input);
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
  name: String,
  value: TokenStream2,
}

impl syn::parse::Parse for ExtensionTuple {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let mut name = None; let mut value = None;
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
          name = Some(parse_utils::parse_next_literal_str(input)?);
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

    if let (Some(name), Some(value)) = (name, value) {
      Ok(ExtensionTuple { name, value })
    } else {
      Err(syn::Error::new(span, "Property and/or is not set"))
    }
  }
}

impl ToTokensDiagnostics for ExtensionTuple {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), Diagnostics> {
    let k = self.name.as_str(); let json = &self.value;
    tokens.extend(quote! {
      .add(#k, serde_json::json!(#json))
    });
    Ok(())
  }
}
*/
