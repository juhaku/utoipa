use std::{borrow::Cow, str::FromStr};

use lazy_static::lazy_static;
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use regex::{Captures, Regex};
use syn::{parse::Parse, LitStr, Token};

use crate::{
    component::{GenericType, TypeTree, ValueType},
    ext::{ArgValue, ArgumentIn, IntoParamsType, MacroArg, ValueArgument},
    path::PathOperation,
};

use super::{
    fn_arg::{self, FnArg},
    ArgumentResolver, MacroPath, PathOperationResolver, PathOperations, PathResolver,
    ResolvedOperation,
};

const ANONYMOUS_ARG: &str = "<_>";

impl ArgumentResolver for PathOperations {
    fn resolve_arguments(
        fn_args: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
        macro_args: Option<Vec<MacroArg>>,
    ) -> (
        Option<Vec<ValueArgument<'_>>>,
        Option<Vec<IntoParamsType<'_>>>,
    ) {
        let mut args = fn_arg::get_fn_args(fn_args).collect::<Vec<_>>();
        args.sort_unstable();
        let (into_params_args, value_args): (Vec<FnArg>, Vec<FnArg>) =
            args.into_iter().partition(is_into_params);

        macro_args
            .map(|args| {
                let (anonymous_args, named_args): (Vec<MacroArg>, Vec<MacroArg>) =
                    args.into_iter().partition(is_anonymous_arg);

                (
                    Some(
                        value_args
                            .into_iter()
                            .flat_map(with_argument_in(&named_args))
                            .map(to_value_arg)
                            .chain(anonymous_args.into_iter().map(to_anonymous_value_arg))
                            .collect(),
                    ),
                    Some(
                        into_params_args
                            .into_iter()
                            .flat_map(with_parameter_in(&named_args))
                            .map(fn_arg::into_into_params_type)
                            .collect(),
                    ),
                )
            })
            .unwrap_or_else(|| (None, None))
    }
}

fn to_value_arg((arg, argument_in): (FnArg, ArgumentIn)) -> ValueArgument {
    let (is_option, is_vec) = is_option_or_vec(&arg.ty);

    ValueArgument {
        type_path: get_value_type(arg.ty),
        argument_in,
        name: Some(Cow::Owned(arg.name.to_string())),
        is_array: is_vec,
        is_option,
    }
}

fn to_anonymous_value_arg<'a>(macro_arg: MacroArg) -> ValueArgument<'a> {
    let (name, argument_in) = match macro_arg {
        MacroArg::Path(arg_value) => (arg_value.name, ArgumentIn::Path),
        MacroArg::Query(arg_value) => (arg_value.name, ArgumentIn::Query),
    };

    ValueArgument {
        type_path: None,
        argument_in,
        name: Some(Cow::Owned(name)),
        is_array: false,
        is_option: false,
    }
}

fn with_parameter_in(
    named_args: &[MacroArg],
) -> impl Fn(FnArg) -> Option<(Option<Cow<'_, syn::Path>>, TokenStream)> + '_ {
    move |arg: FnArg| {
        let parameter_in = named_args.iter().find_map(|macro_arg| match macro_arg {
            MacroArg::Path(path) => {
                if arg.name == &*path.name {
                    Some(quote! { || Some(utoipa::openapi::path::ParameterIn::Path) })
                } else {
                    None
                }
            }
            MacroArg::Query(query) => {
                if arg.name == &*query.name {
                    Some(quote! { || Some(utoipa::openapi::path::ParameterIn::Query) })
                } else {
                    None
                }
            }
        });

        Some(arg.ty.path).zip(parameter_in)
    }
}

fn with_argument_in(named_args: &[MacroArg]) -> impl Fn(FnArg) -> Option<(FnArg, ArgumentIn)> + '_ {
    move |arg: FnArg| {
        let argument_in = named_args.iter().find_map(|macro_arg| match macro_arg {
            MacroArg::Path(path) => {
                if arg.name == &*path.name {
                    Some(ArgumentIn::Path)
                } else {
                    None
                }
            }
            MacroArg::Query(query) => {
                if arg.name == &*query.name {
                    Some(ArgumentIn::Query)
                } else {
                    None
                }
            }
        });

        Some(arg).zip(argument_in)
    }
}

#[inline]
fn get_value_type(ty: TypeTree<'_>) -> Option<Cow<syn::Path>> {
    // TODO abort if map
    match ty.generic_type {
        Some(GenericType::Vec)
        | Some(GenericType::Box)
        | Some(GenericType::Cow)
        | Some(GenericType::Map)
        | Some(GenericType::Option)
        | Some(GenericType::RefCell) => {
            get_value_type(ty.children.unwrap().into_iter().next().unwrap())
        }
        None => ty.path,
    }
}

#[inline]
fn is_into_params(fn_arg: &FnArg) -> bool {
    matches!(fn_arg.ty.value_type, ValueType::Object) && matches!(fn_arg.ty.generic_type, None)
}

#[inline]
fn is_anonymous_arg(arg: &MacroArg) -> bool {
    matches!(arg, MacroArg::Path(path) if path.original_name == ANONYMOUS_ARG)
        || matches!(arg, MacroArg::Query(query) if query.original_name == ANONYMOUS_ARG)
}

type OptionOrVec = (bool, bool);

#[inline]
fn is_option_or_vec(ty: &TypeTree<'_>) -> OptionOrVec {
    // TODO abort if map
    let mut is_vec = matches!(ty.generic_type, Some(GenericType::Vec));
    let mut is_option = matches!(ty.generic_type, Some(GenericType::Option));

    if let Some(ref child) = ty.children {
        let (child_option, child_vec) = is_option_or_vec(child.first().unwrap());

        is_option = is_option || child_option;
        is_vec = is_vec || child_vec;
    }

    (is_option, is_vec)
}

impl PathOperationResolver for PathOperations {
    fn resolve_operation(ast_fn: &syn::ItemFn) -> Option<super::ResolvedOperation> {
        ast_fn.attrs.iter().find_map(|attribute| {
            if is_valid_route_type(attribute.path.get_ident()) {
                let Path(path, operation) = match attribute.parse_args::<Path>() {
                    Ok(path) => path,
                    Err(error) => abort!(
                        error.span(),
                        "parse path of path operation attribute: {}",
                        error
                    ),
                };

                if let Some(operation) = operation {
                    Some(ResolvedOperation {
                        path_operation: PathOperation::from_str(&operation).unwrap(),
                        path,
                    })
                } else {
                    Some(ResolvedOperation {
                        path_operation: PathOperation::from_ident(
                            attribute.path.get_ident().unwrap(),
                        ),
                        path,
                    })
                }
            } else {
                None
            }
        })
    }
}

struct Path(String, Option<String>);

impl Parse for Path {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let (path, operation) = if input.peek(syn::Ident) {
            // expect format (GET, uri = "url...")
            let ident = input.parse::<Ident>()?;
            input.parse::<Token![,]>()?;
            input.parse::<Ident>()?; // explisitly 'uri'
            input.parse::<Token![=]>()?;

            (
                input.parse::<LitStr>()?.value(),
                Some(ident.to_string().to_lowercase()),
            )
        } else {
            // expect format ("url...")

            (input.parse::<LitStr>()?.value(), None)
        };

        // ignore rest of the tokens from rocket path attribute macro
        input.step(|cursor| {
            let mut rest = *cursor;
            while let Some((_, next)) = rest.token_tree() {
                rest = next;
            }
            Ok(((), rest))
        })?;

        Ok(Self(path, operation))
    }
}

#[inline]
fn is_valid_route_type(ident: Option<&Ident>) -> bool {
    matches!(ident, Some(operation) if ["get", "post", "put", "delete", "head", "options", "patch", "route"]
        .iter().any(|expected_operation| operation == expected_operation))
}

impl PathResolver for PathOperations {
    fn resolve_path(path: &Option<String>) -> Option<MacroPath> {
        path.as_ref().map(|whole_path| {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"<[a-zA-Z0-9_][^<>]*>").unwrap();
            }

            whole_path
                .split_once('?')
                .or(Some((whole_path, "")))
                .map(|(path, query)| {
                    let mut names =
                        Vec::<MacroArg>::with_capacity(RE.find_iter(whole_path).count());
                    let mut underscore_count = 0;

                    let mut format_arg =
                        |captures: &Captures, resolved_arg_op: fn(ArgValue) -> MacroArg| {
                            let capture = &captures[0];
                            let original_name = String::from(capture);

                            let mut arg = capture
                                .replace("..", "")
                                .replace('<', "{")
                                .replace('>', "}");

                            if arg == "{_}" {
                                arg = format!("{{arg{underscore_count}}}");
                                names.push(resolved_arg_op(ArgValue {
                                    name: String::from(&arg[1..arg.len() - 1]),
                                    original_name,
                                }));
                                underscore_count += 1;
                            } else {
                                names.push(resolved_arg_op(ArgValue {
                                    name: String::from(&arg[1..arg.len() - 1]),
                                    original_name,
                                }))
                            }

                            arg
                        };

                    let path = RE.replace_all(path, |captures: &Captures| {
                        format_arg(captures, MacroArg::Path)
                    });

                    if !query.is_empty() {
                        RE.replace_all(query, |captures: &Captures| {
                            format_arg(captures, MacroArg::Query)
                        });
                    }

                    names.sort_unstable_by(MacroArg::by_name);

                    MacroPath {
                        args: names,
                        path: path.to_string(),
                    }
                })
                .unwrap()
        })
    }
}
