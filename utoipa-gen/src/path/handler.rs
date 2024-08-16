use quote::{quote, ToTokens};
use syn::ItemFn;

use crate::{as_tokens_or_diagnostics, ext, ToTokensDiagnostics};

use super::Path;

pub struct Handler<
    'p,
    // F: FnOnce() -> I + Copy,
    // I: IntoIterator<Item = crate::ext::fn_arg::FnArg<'p>>,
> {
    pub path: Path<'p>,
    pub handler_fn: &'p ItemFn,
    // pub fn_args_provider: F,
}

#[cfg(not(feature = "axum_handler"))]
impl ToTokensDiagnostics for Handler {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), crate::Diagnostics> {
        let ast_fn = &self.handler_fn;
        let path = self.path.to_token_stream()?;
        tokens.extend(quote! {
            #path
            #ast_fn
        });

        Ok(())
    }
}

enum State {
    Arg(proc_macro2::TokenStream),
    Default,
}

impl State {
    fn into_state_tokens(self) -> (Option<proc_macro2::TokenStream>, proc_macro2::TokenStream) {
        match self {
            Self::Arg(tokens) => (None, tokens),
            Self::Default => (
                Some(quote! {<S: Clone + Send + Sync + 'static>}),
                quote! {S},
            ),
        }
    }
}

#[cfg(feature = "axum_handler")]
impl<'p> ToTokensDiagnostics for Handler<'p>
// where
//     F: FnOnce() -> I + Copy,
//     I: IntoIterator<Item = crate::ext::fn_arg::FnArg<'a>>,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), crate::Diagnostics> {
        let ast_fn = &self.handler_fn;
        let path = as_tokens_or_diagnostics!(&self.path);
        let fn_name = &ast_fn.sig.ident;
        // TODO refactor the extension FnArg processing, now it is done twice for axum, is there a
        // way to just do it once???
        // See lib.rs and ext/axum.rs
        let fn_args = ext::fn_arg::get_fn_args(&ast_fn.sig.inputs)?;

        let state = if let Some(arg) = fn_args
            .into_iter()
            .find(|fn_arg| fn_arg.ty.is("State"))
            .and_then(|fn_arg| fn_arg.ty.path)
        {
            let args = arg
                .segments
                .first()
                .map(|segment| &segment.arguments)
                .and_then(|path_args| match path_args {
                    syn::PathArguments::AngleBracketed(arg) => Some(&arg.args),
                    _ => None,
                });

            State::Arg(args.to_token_stream())
        } else {
            State::Default
        };
        let (generic, state) = state.into_state_tokens();

        tokens.extend(quote! {
            #path

            impl #generic axum::handler::Handler<std::convert::Infallible, #state> for #fn_name {
                type Future = std::pin::Pin<
                    std::boxed::Box<
                        (dyn std::future::Future<Output = axum::http::Response<axum::body::Body>>
                             + std::marker::Send
                             + 'static),
                    >,
                >;

                fn call(self, req: axum::extract::Request, state: #state) -> Self::Future {
                    #ast_fn
                    #fn_name.call(req, state)
                }
            }
        });

        Ok(())
    }
}
