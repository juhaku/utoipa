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
            .partition(non_primitive_arg);

        // TODO what about primitive args and tuple args?
        // if let Some(resolved_args) = resolved_path_args {
        //     let primitive = Self::to_value_args(resolved_args, primitive_args);

        //     Some(
        //         primitive
        //             .chain(Self::to_token_stream_args(non_primitive_args))
        //             .collect(),
        //     )
        // } else {
        //     Some(Self::to_token_stream_args(non_primitive_args).collect())
        // }

        // Some(fn_arg::to_into_params_types(args.into_iter().filter(non_primitive_arg)).collect())
        // Some(fn_arg::to_into_params_types(non_primitive_args).collect())
        (
            None,
            Some(fn_arg::to_into_params_types(non_primitive_args).collect()),
        )
    }
}

fn non_primitive_arg(fn_arg: &FnArg) -> bool {
    let is_primitive = |type_path| {
        fn_arg::get_last_ident(type_path)
            .map(|ident| ComponentType(ident).is_primitive())
            .unwrap_or(false)
    };

    match fn_arg {
        FnArg::Path(path) => !is_primitive(path),
        FnArg::Query(query) => !is_primitive(query),
    }
}
