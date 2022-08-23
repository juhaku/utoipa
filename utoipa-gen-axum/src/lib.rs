use syn::{punctuated::Punctuated, token::Comma};
use utoipa_corelib::fn_arg::FnArg;
use utoipa_corelib::{fn_arg, IntoParamsType, MacroArg, PathOperations, ValueArgument};

pub trait PathOperationsExt {
    fn resolve_arguments(
        fn_args: &Punctuated<syn::FnArg, Comma>,
        macro_args: Option<Vec<MacroArg>>,
    ) -> (
        Option<Vec<ValueArgument<'_>>>,
        Option<Vec<IntoParamsType<'_>>>,
    );
}

// axum framework is only able to resolve handler function arguments.
// `PathResolver` and `PathOperationResolver` is not supported in axum.
impl PathOperationsExt for PathOperations {
    fn resolve_arguments(
        args: &'_ Punctuated<syn::FnArg, Comma>,
        _: Option<Vec<utoipa_corelib::MacroArg>>, // ignored, cannot be provided
    ) -> (
        Option<Vec<utoipa_corelib::ValueArgument<'_>>>,
        Option<Vec<utoipa_corelib::IntoParamsType<'_>>>,
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
