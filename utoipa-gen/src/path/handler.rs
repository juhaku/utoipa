use quote::quote;
use syn::ItemFn;

use crate::{as_tokens_or_diagnostics, ToTokensDiagnostics};

use super::Path;

pub struct Handler<'p> {
    pub path: Path<'p>,
    pub handler_fn: &'p ItemFn,
}

impl<'p> ToTokensDiagnostics for Handler<'p> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), crate::Diagnostics> {
        let ast_fn = &self.handler_fn;
        let path = as_tokens_or_diagnostics!(&self.path);
        tokens.extend(quote! {
            #path
            #ast_fn
        });

        Ok(())
    }
}
