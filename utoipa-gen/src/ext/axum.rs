use syn::{punctuated::Punctuated, token::Comma};

use super::{
    fn_arg::{self, FnArg},
    ArgumentResolver, PathOperations,
};

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
        let (into_params_args, _): (Vec<FnArg>, Vec<FnArg>) = fn_arg::get_fn_args(args)
            .into_iter()
            .partition(fn_arg::is_into_params);

        (
            None,
            Some(
                into_params_args
                    .into_iter()
                    .flat_map(fn_arg::with_parameter_in)
                    .map(fn_arg::into_into_params_type)
                    .collect(),
            ),
        )
    }
}
