use std::borrow::Cow;

use regex::Captures;
use syn::{punctuated::Punctuated, token::Comma};

use crate::{
    component::{TypeTree, ValueType},
    Diagnostics,
};

use super::{
    fn_arg::{self, FnArg, FnArgType},
    ArgValue, ArgumentResolver, Arguments, MacroArg, MacroPath, PathOperations, PathResolver,
    ValueArgument,
};

// axum framework is only able to resolve handler function arguments.
// `PathResolver` and `PathOperationResolver` is not supported in axum.
impl ArgumentResolver for PathOperations {
    fn resolve_arguments(
        args: &'_ Punctuated<syn::FnArg, Comma>,
        macro_args: Option<Vec<super::MacroArg>>,
        _: String,
    ) -> Result<Arguments<'_>, Diagnostics> {
        let (into_params_args, value_args): (Vec<FnArg>, Vec<FnArg>) =
            fn_arg::get_fn_args(args)?.partition(fn_arg::is_into_params);

        let (value_args, body) = split_value_args_and_request_body(value_args);

        Ok((
            Some(
                value_args
                    .zip(macro_args.unwrap_or_default())
                    .map(|(value_arg, macro_arg)| ValueArgument {
                        name: match macro_arg {
                            MacroArg::Path(path) => Some(Cow::Owned(path.name)),
                            #[cfg(feature = "rocket_extras")]
                            MacroArg::Query(_) => None,
                        },
                        argument_in: value_arg.argument_in,
                        type_tree: value_arg.type_tree,
                    })
                    .collect(),
            ),
            Some(
                into_params_args
                    .into_iter()
                    .flat_map(fn_arg::with_parameter_in)
                    .map(Into::into)
                    .collect(),
            ),
            body.into_iter().next().map(Into::into),
        ))
    }
}

fn split_value_args_and_request_body(
    value_args: Vec<FnArg>,
) -> (
    impl Iterator<Item = super::ValueArgument<'_>>,
    impl Iterator<Item = TypeTree<'_>>,
) {
    let (path_args, body_types): (Vec<FnArg>, Vec<FnArg>) = value_args
        .into_iter()
        .filter(|arg| {
            arg.ty.is("Path") || arg.ty.is("Json") || arg.ty.is("Form") || arg.ty.is("Bytes")
        })
        .partition(|arg| arg.ty.is("Path"));

    (
        path_args
            .into_iter()
            .filter(|arg| arg.ty.is("Path"))
            .flat_map(|path_arg| {
                match (
                    path_arg.arg_type,
                    path_arg.ty.children.expect("Path must have children"),
                ) {
                    (FnArgType::Single(name), path_children) => path_children
                        .into_iter()
                        .flat_map(|ty| match ty.value_type {
                            ValueType::Tuple => ty
                                .children
                                .expect("ValueType::Tuple will always have children")
                                .into_iter()
                                .map(|ty| to_value_argument(None, ty))
                                .collect(),
                            ValueType::Primitive => {
                                vec![to_value_argument(Some(Cow::Owned(name.to_string())), ty)]
                            }
                            ValueType::Object | ValueType::Value => unreachable!("Cannot get here"),
                        })
                        .collect::<Vec<_>>(),
                    (FnArgType::Destructed(tuple), path_children) => tuple
                        .iter()
                        .zip(path_children.into_iter().flat_map(|child| {
                            child
                                .children
                                .expect("ValueType::Tuple will always have children")
                        }))
                        .map(|(name, ty)| to_value_argument(Some(Cow::Owned(name.to_string())), ty))
                        .collect::<Vec<_>>(),
                }
            }),
        body_types.into_iter().map(|body| body.ty),
    )
}

fn to_value_argument<'a>(name: Option<Cow<'a, str>>, ty: TypeTree<'a>) -> ValueArgument<'a> {
    ValueArgument {
        name,
        type_tree: Some(ty),
        argument_in: super::ArgumentIn::Path,
    }
}

impl PathResolver for PathOperations {
    fn resolve_path(path: &Option<String>) -> Option<MacroPath> {
        path.as_ref().map(|path| {
            let regex = regex::Regex::new(r"\{[a-zA-Z0-9][^{}]*}").unwrap();

            let mut args = Vec::<MacroArg>::with_capacity(regex.find_iter(path).count());
            MacroPath {
                path: regex
                    .replace_all(path, |captures: &Captures| {
                        let capture = &captures[0];
                        let original_name = String::from(capture);

                        args.push(MacroArg::Path(ArgValue {
                            name: String::from(&capture[1..capture.len() - 1]),
                            original_name,
                        }));
                        // otherwise return the capture itself
                        capture.to_string()
                    })
                    .to_string(),
                args,
            }
        })
    }
}
