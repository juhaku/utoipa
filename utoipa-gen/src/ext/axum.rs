use std::borrow::Cow;

use syn::{punctuated::Punctuated, token::Comma};

use crate::component::TypeTree;

use super::{
    fn_arg::{self, FnArg, FnArgType},
    ArgumentResolver, PathOperations, ValueArgument,
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
        let (into_params_args, value_args): (Vec<FnArg>, Vec<FnArg>) = fn_arg::get_fn_args(args)
            .into_iter()
            .partition(fn_arg::is_into_params);

        (
            Some(get_value_arguments(value_args).collect()),
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

fn get_value_arguments(value_args: Vec<FnArg>) -> impl Iterator<Item = super::ValueArgument<'_>> {
    value_args
        .into_iter()
        .filter(|arg| arg.ty.is("Path"))
        .flat_map(|path_arg| match path_arg.arg_type {
            FnArgType::Single(name) => path_arg
                .ty
                .children
                .expect("Path argument must have children")
                .into_iter()
                .map(|ty| to_value_argument(Some(Cow::Owned(name.to_string())), ty))
                .collect::<Vec<_>>(),
            FnArgType::Tuple(tuple) => tuple
                .iter()
                .zip(
                    path_arg
                        .ty
                        .children
                        .expect("Path argument must have children")
                        .into_iter()
                        .flat_map(|child| {
                            child
                                .children
                                .expect("ValueType::Tuple will always have children")
                        }),
                )
                .map(|(name, ty)| to_value_argument(Some(Cow::Owned(name.to_string())), ty))
                .collect::<Vec<_>>(),
        })
}

fn to_value_argument<'a>(name: Option<Cow<'a, str>>, ty: TypeTree<'a>) -> ValueArgument<'a> {
    ValueArgument {
        name,
        type_tree: Some(ty),
        argument_in: super::ArgumentIn::Path,
    }
}
