use crate::{
    component_type::ComponentType,
    ext::fn_arg::{self, FnArg},
};

use super::{ArgumentResolver, PathOperations};

// axum framework is only able to resolve handler function arguments.
// `PathResolver` and `PathOperationResolver` is not supported in axum.
impl ArgumentResolver for PathOperations {
    fn resolve_path_arguments(
        args: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
        _: Option<Vec<super::ResolvedArg>>, // ignored, cannot be provided
    ) -> Option<Vec<super::Argument<'_>>> {
        let (primitive_args, non_primitive_args): (Vec<FnArg>, Vec<FnArg>) = fn_arg::get_fn_args(args)
            .partition(|arg| matches!(arg, FnArg::Path(ty) if ComponentType(fn_arg::get_last_ident(ty).unwrap()).is_primitive()));

        // TODO what about primitive args and tuple args?

        Some(fn_arg::to_into_params_arguments(non_primitive_args).collect())
    }
}
