use syn::{punctuated::Punctuated, token::Comma};

use crate::{
    component_type::ComponentType,
    ext::fn_arg::{self, FnArg},
};

use super::{ArgumentResolver, PathOperations, ValueArgument};

// axum framework is only able to resolve handler function arguments.
// `PathResolver` and `PathOperationResolver` is not supported in axum.
impl ArgumentResolver for PathOperations {
    fn resolve_arguments(
        args: &'_ Punctuated<syn::FnArg, Comma>,
        _: Option<Vec<super::MacroArg>>, // ignored, cannot be provided
    ) -> (
        Option<Vec<super::ValueArgument<'_>>>,
        Option<Vec<super::IntoParamsType<'_>>>,
    ) {
        let (non_primitive_args, _): (Vec<FnArg>, Vec<FnArg>) = fn_arg::get_fn_args(args)
            .into_iter()
            .partition(fn_arg::non_primitive_arg);

        // TODO what about primitive args and tuple args?
        (
            None,
            Some(fn_arg::to_into_params_types(non_primitive_args).collect()),
        )
    }
}
