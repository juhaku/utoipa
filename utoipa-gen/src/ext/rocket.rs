use std::{borrow::Cow, str::FromStr};

use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use regex::{Captures, Regex};
use syn::{parse::Parse, LitStr, Token};

use crate::{
    component::ValueType,
    ext::{ArgValue, ArgumentIn, IntoParamsType, MacroArg, ValueArgument},
    path::PathOperation,
    ResultExt,
};

use super::{
    fn_arg::{self, FnArg},
    ArgumentResolver, MacroPath, PathOperationResolver, PathOperations, PathResolver, RequestBody,
};

const ANONYMOUS_ARG: &str = "<_>";

impl<'a> ArgumentResolver<'a> for PathOperations {
    type Item = Body;
    fn resolve_arguments(
        fn_args: &'a syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
        macro_path: Option<MacroPath<Self::Item>>,
    ) -> (
        Vec<ValueArgument<'a>>,
        Vec<IntoParamsType<'a>>,
        Option<RequestBody<'a>>,
    ) {
        let mut args = fn_arg::get_fn_args(fn_args).collect::<Vec<_>>();
        args.sort_unstable();
        let (into_params_args, value_args): (Vec<FnArg>, Vec<FnArg>) =
            args.into_iter().partition(is_into_params);
        // TODO resolve request body from value_args

        dbg!(&value_args, &into_params_args);

        macro_path
            .map(|path| {
                // let body = path.path; // resolve body??
                let (anonymous_args, named_args): (Vec<MacroArg>, Vec<MacroArg>) =
                    path.args.into_iter().partition(is_anonymous_arg);

                (
                    value_args
                        .into_iter()
                        .flat_map(with_argument_in(&named_args))
                        .map(to_value_arg)
                        .chain(anonymous_args.into_iter().map(to_anonymous_value_arg))
                        .collect(),
                    into_params_args
                        .into_iter()
                        .flat_map(with_parameter_in(&named_args))
                        .map(Into::into)
                        .collect(),
                    None,
                )
            })
            .unwrap_or_else(|| (Vec::new(), Vec::new(), None))
    }
}

fn to_value_arg((arg, argument_in): (FnArg, ArgumentIn)) -> ValueArgument {
    ValueArgument {
        type_tree: Some(arg.ty),
        argument_in,
        name: Some(Cow::Owned(arg.arg_type.get_name().to_string())),
    }
}

fn to_anonymous_value_arg<'a>(macro_arg: MacroArg) -> ValueArgument<'a> {
    let (name, argument_in) = match macro_arg {
        MacroArg::Path(arg_value) => (arg_value.name, ArgumentIn::Path),
        MacroArg::Query(arg_value) => (arg_value.name, ArgumentIn::Query),
    };

    ValueArgument {
        type_tree: None,
        argument_in,
        name: Some(Cow::Owned(name)),
    }
}

fn with_parameter_in(
    named_args: &[MacroArg],
) -> impl Fn(FnArg) -> Option<(Option<Cow<'_, syn::Path>>, TokenStream)> + '_ {
    move |arg: FnArg| {
        let parameter_in = named_args.iter().find_map(|macro_arg| match macro_arg {
            MacroArg::Path(path) => {
                if arg.arg_type.get_name() == &*path.name {
                    Some(quote! { || Some(utoipa::openapi::path::ParameterIn::Path) })
                } else {
                    None
                }
            }
            MacroArg::Query(query) => {
                if arg.arg_type.get_name() == &*query.name {
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
                if arg.arg_type.get_name() == &*path.name {
                    Some(ArgumentIn::Path)
                } else {
                    None
                }
            }
            MacroArg::Query(query) => {
                if arg.arg_type.get_name() == &*query.name {
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
fn is_into_params(fn_arg: &FnArg) -> bool {
    matches!(fn_arg.ty.value_type, ValueType::Object) && matches!(fn_arg.ty.generic_type, None)
}

#[inline]
fn is_anonymous_arg(arg: &MacroArg) -> bool {
    matches!(arg, MacroArg::Path(path) if path.original_name == ANONYMOUS_ARG)
        || matches!(arg, MacroArg::Query(query) if query.original_name == ANONYMOUS_ARG)
}

impl PathOperationResolver for PathOperations {
    type Item = Path;
    fn resolve_operation(ast_fn: &syn::ItemFn) -> Option<super::Operation<Self::Item>> {
        ast_fn.attrs.iter().find_map(|attribute| {
            if is_valid_route_type(attribute.path().get_ident()) {
                let Path(path, operation, body) =
                    attribute.parse_args::<Path>().unwrap_or_else(|error| {
                        abort!(
                            error.span(),
                            "parse path of path operation attribute: {}",
                            error
                        )
                    });

                Some(super::Operation {
                    path_operation: operation
                        .as_ref()
                        .map_or_else(
                            || PathOperation::try_from(attribute.path().get_ident().unwrap()),
                            |operation| {
                                PathOperation::from_str(&operation.to_string()).map_err(|error| {
                                    syn::Error::new(operation.0.span(), error.to_string())
                                })
                            },
                        )
                        .unwrap_or_abort(),
                    path: Path(path, operation, body),
                })
            } else {
                None
            }
        })
    }
}

struct Operation(Ident);

impl ToString for Operation {
    fn to_string(&self) -> String {
        self.0.to_string().to_lowercase()
    }
}

type Body = Option<String>;

#[derive(Default)]
pub struct Path(String, Option<Operation>, Body);

impl From<Path> for String {
    fn from(value: Path) -> Self {
        value.0
    }
}

impl From<String> for Path {
    fn from(value: String) -> Self {
        Self(value, None, None)
    }
}

impl From<MacroPath<Path>> for MacroPath<Option<String>> {
    fn from(value: MacroPath<Path>) -> Self {
        MacroPath {
            path: value.path.2,
            args: value.args,
        }
    }
}

impl Parse for Path {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let (path, operation, body) = if input.peek(syn::Ident) {
            // expect format (GET, uri = "url...")
            let ident = input.parse::<Ident>()?;
            input.parse::<Token![,]>()?;
            input.parse::<Ident>()?; // explicitly 'uri'
            input.parse::<Token![=]>()?;

            let uri_value = input.parse::<LitStr>()?.value();

            // parse , data = <...> if found
            let body_value = if input.peek(Token![,]) && input.peek2(syn::Ident) {
                input.parse::<Token![,]>()?;
                input.parse::<Ident>()?; // explisitly 'data'
                input.parse::<Token![=]>()?;
                Some(input.parse::<LitStr>()?.value())
            } else {
                None
            };

            (uri_value, Some(Operation(ident)), body_value)
        } else {
            // expect format ("url...")

            (input.parse::<LitStr>()?.value(), None, None)
        };

        // ignore rest of the tokens from rocket path attribute macro
        input.step(|cursor| {
            let mut rest = *cursor;
            while let Some((_, next)) = rest.token_tree() {
                rest = next;
            }
            Ok(((), rest))
        })?;

        Ok(Self(path, operation, body))
    }
}

#[inline]
fn is_valid_route_type(ident: Option<&Ident>) -> bool {
    matches!(ident, Some(operation) if ["get", "post", "put", "delete", "head", "options", "patch", "route"]
        .iter().any(|expected_operation| operation == expected_operation))
}

impl PathResolver for PathOperations {
    type Item = Path;
    fn resolve_macro_path<P: Into<Path>>(path: Option<P>) -> Option<MacroPath<Self::Item>> {
        path.map(|whole_path| {
            let Path(ref whole_path, operation, body) = whole_path.into();

            let regex = Regex::new(r"<[a-zA-Z0-9_][^<>]*>").unwrap();

            whole_path
                .split_once('?')
                .or(Some((whole_path, "")))
                .map(|(path, query)| {
                    let mut names =
                        Vec::<MacroArg>::with_capacity(regex.find_iter(whole_path).count());
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

                    let path = regex.replace_all(path, |captures: &Captures| {
                        format_arg(captures, MacroArg::Path)
                    });

                    if !query.is_empty() {
                        regex.replace_all(query, |captures: &Captures| {
                            format_arg(captures, MacroArg::Query)
                        });
                    }

                    names.sort_unstable_by(MacroArg::by_name);

                    MacroPath {
                        args: names,
                        path: Path(
                            path.to_string(),
                            operation,
                            body.map(|body| body.replace(['<', '>'], "")),
                        ),
                    }
                })
                .unwrap()
        })
    }
}
