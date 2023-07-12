use std::borrow::Cow;

use proc_macro2::Ident;
use proc_macro_error::abort;
use regex::{Captures, Regex};
use syn::{parse::Parse, punctuated::Punctuated, token::Comma, ItemFn, LitStr};

use crate::{
    component::{TypeTree, ValueType},
    ext::ArgValue,
    path::PathOperation,
};

use super::{
    fn_arg::{self, FnArg},
    ArgumentIn, ArgumentResolver, MacroArg, MacroPath, PathOperationResolver, PathOperations,
    PathResolver, ResolvedOperation, ValueArgument,
};

impl ArgumentResolver for PathOperations {
    fn resolve_arguments(
        fn_args: &Punctuated<syn::FnArg, Comma>,
        macro_args: Option<Vec<MacroArg>>,
        _: String,
    ) -> (
        Option<Vec<super::ValueArgument<'_>>>,
        Option<Vec<super::IntoParamsType<'_>>>,
        Option<super::RequestBody<'_>>,
    ) {
        let (into_params_args, value_args): (Vec<FnArg>, Vec<FnArg>) =
            fn_arg::get_fn_args(fn_args).partition(fn_arg::is_into_params);

        if let Some(macro_args) = macro_args {
            let (primitive_args, body) = split_path_args_and_request(value_args);

            (
                Some(
                    macro_args
                        .into_iter()
                        .zip(primitive_args)
                        .map(into_value_argument)
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
            )
        } else {
            let (_, body) = split_path_args_and_request(value_args);
            (
                None,
                Some(
                    into_params_args
                        .into_iter()
                        .flat_map(fn_arg::with_parameter_in)
                        .map(Into::into)
                        .collect(),
                ),
                body.into_iter().next().map(Into::into),
            )
        }
    }
}

fn split_path_args_and_request(
    value_args: Vec<FnArg>,
) -> (
    impl Iterator<Item = TypeTree>,
    impl Iterator<Item = TypeTree>,
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
            .flat_map(|path_arg| {
                path_arg
                    .ty
                    .children
                    .expect("Path argument must have children")
            })
            .flat_map(|path_arg| match path_arg.value_type {
                ValueType::Primitive => vec![path_arg],
                ValueType::Tuple => path_arg
                    .children
                    .expect("ValueType::Tuple will always have children"),
                ValueType::Object | ValueType::Value => {
                    unreachable!("Value arguments does not have ValueType::Object arguments")
                }
            }),
        body_types.into_iter().map(|json| json.ty),
    )
}

fn into_value_argument((macro_arg, primitive_arg): (MacroArg, TypeTree)) -> ValueArgument {
    ValueArgument {
        name: match macro_arg {
            MacroArg::Path(path) => Some(Cow::Owned(path.name)),
        },
        type_tree: Some(primitive_arg),
        argument_in: ArgumentIn::Path,
    }
}

impl PathOperationResolver for PathOperations {
    fn resolve_operation(item_fn: &ItemFn) -> Option<ResolvedOperation> {
        item_fn.attrs.iter().find_map(|attribute| {
            if is_valid_request_type(attribute.path().get_ident()) {
                match attribute.parse_args::<Path>() {
                    Ok(path) => Some(ResolvedOperation {
                        path: path.0,
                        path_operation: PathOperation::from_ident(
                            attribute.path().get_ident().unwrap(),
                        ),
                        body: String::new(),
                    }),
                    Err(error) => abort!(
                        error.span(),
                        "parse path of path operation attribute: {}",
                        error
                    ),
                }
            } else {
                None
            }
        })
    }
}

struct Path(String);

impl Parse for Path {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path = input.parse::<LitStr>()?.value();

        // ignore rest of the tokens from actix-web path attribute macro
        input.step(|cursor| {
            let mut rest = *cursor;
            while let Some((_, next)) = rest.token_tree() {
                rest = next;
            }
            Ok(((), rest))
        })?;

        Ok(Self(path))
    }
}

impl PathResolver for PathOperations {
    fn resolve_path(path: &Option<String>) -> Option<MacroPath> {
        path.as_ref().map(|path| {
            let regex = Regex::new(r"\{[a-zA-Z0-9_][^{}]*}").unwrap();

            let mut args = Vec::<MacroArg>::with_capacity(regex.find_iter(path).count());
            MacroPath {
                path: regex
                    .replace_all(path, |captures: &Captures| {
                        let mut capture = &captures[0];
                        let original_name = String::from(capture);

                        if capture.contains("_:") {
                            // replace unnamed capture with generic 'arg0' name
                            args.push(MacroArg::Path(ArgValue {
                                name: String::from("arg0"),
                                original_name,
                            }));
                            "{arg0}".to_string()
                        } else if let Some(colon) = capture.find(':') {
                            //  replace colon (:) separated regexp with empty string
                            capture = &capture[1..colon];

                            args.push(MacroArg::Path(ArgValue {
                                name: String::from(capture),
                                original_name,
                            }));

                            format!("{{{capture}}}")
                        } else {
                            args.push(MacroArg::Path(ArgValue {
                                name: String::from(&capture[1..capture.len() - 1]),
                                original_name,
                            }));
                            // otherwise return the capture itself
                            capture.to_string()
                        }
                    })
                    .to_string(),
                args,
            }
        })
    }
}

#[inline]
fn is_valid_request_type(ident: Option<&Ident>) -> bool {
    matches!(ident, Some(operation) if ["get", "post", "put", "delete", "head", "connect", "options", "trace", "patch"]
        .iter().any(|expected_operation| operation == expected_operation))
}
