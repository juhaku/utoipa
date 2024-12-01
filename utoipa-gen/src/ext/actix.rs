use std::borrow::Cow;

use proc_macro2::{Ident, TokenTree};
use regex::{Captures, Regex};
use syn::{parse::Parse, punctuated::Punctuated, token::Comma, ItemFn, LitStr};

use crate::{
    component::{TypeTree, ValueType},
    ext::ArgValue,
    path::HttpMethod,
    Diagnostics,
};

use super::{
    fn_arg::{self, FnArg},
    ArgumentIn, ArgumentResolver, Arguments, MacroArg, MacroPath, PathOperationResolver,
    PathOperations, PathResolver, ResolvedOperation, ValueArgument,
};

impl ArgumentResolver for PathOperations {
    fn resolve_arguments(
        fn_args: &Punctuated<syn::FnArg, Comma>,
        macro_args: Option<Vec<MacroArg>>,
        _: String,
    ) -> Result<Arguments, Diagnostics> {
        let (into_params_args, value_args): (Vec<FnArg>, Vec<FnArg>) =
            fn_arg::get_fn_args(fn_args)?.partition(fn_arg::is_into_params);

        if let Some(macro_args) = macro_args {
            let (primitive_args, body) = split_path_args_and_request(value_args);

            Ok((
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
            ))
        } else {
            let (_, body) = split_path_args_and_request(value_args);
            Ok((
                None,
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
            #[cfg(feature = "rocket_extras")]
            MacroArg::Query(_) => None,
        },
        type_tree: Some(primitive_arg),
        argument_in: ArgumentIn::Path,
    }
}

impl PathOperationResolver for PathOperations {
    fn resolve_operation(item_fn: &ItemFn) -> Result<Option<ResolvedOperation>, Diagnostics> {
        item_fn
            .attrs
            .iter()
            .find_map(|attribute| {
                if is_valid_actix_route_attribute(attribute.path().get_ident()) {
                    match attribute.parse_args::<Route>() {
                        Ok(route) => {
                            let attribute_path = attribute.path().get_ident()
                                .expect("actix-web route macro must have ident");
                            let methods: Vec<HttpMethod> = if *attribute_path == "route" {
                                route.methods.into_iter().map(|method| {
                                    method.to_lowercase().parse::<HttpMethod>()
                                        .expect("Should never fail, validity of HTTP method is checked before parsing")
                                }).collect()
                            } else {
                                // if user used #[connect(...)] macro, return error
                                match HttpMethod::from_ident(attribute_path) {
                                    Ok(http_method) => { vec![http_method]},
                                    Err(error) => return Some(
                                        Err(
                                            error.help(
                                                format!(r#"If you want operation to be documented and executed on `{method}` try using `#[route(...)]` e.g. `#[route("/path", method = "GET", method = "{method}")]`"#, method = attribute_path.to_string().to_uppercase())
                                            )
                                        )
                                    )
                                }
                            };
                            Some(Ok(ResolvedOperation {
                                path: route.path,
                                methods,
                                body: String::new(),
                            }))
                        }
                        Err(error) => Some(Err(Into::<Diagnostics>::into(error))),
                    }
                } else {
                    None
                }
            })
            .transpose()
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct Route {
    path: String,
    methods: Vec<String>,
}

impl Parse for Route {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        const ALLOWED_METHODS: [&str; 8] = [
            "GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "TRACE", "PATCH",
        ];
        // OpenAPI spec does not support CONNECT thus we do not resolve it

        enum PrevToken {
            Method,
            Equals,
        }

        let path = input.parse::<LitStr>()?.value();
        let mut parsed_methods: Vec<String> = Vec::new();

        input.step(|cursor| {
            let mut rest = *cursor;

            let mut prev_token: Option<PrevToken> = None;
            while let Some((tt, next)) = rest.token_tree() {
                match &tt {
                    TokenTree::Ident(ident) if *ident == "method" => {
                        prev_token = Some(PrevToken::Method);
                        rest = next
                    }
                    TokenTree::Punct(punct)
                        if punct.as_char() == '='
                            && matches!(prev_token, Some(PrevToken::Method)) =>
                    {
                        prev_token = Some(PrevToken::Equals);
                        rest = next
                    }
                    TokenTree::Literal(literal)
                        if matches!(prev_token, Some(PrevToken::Equals)) =>
                    {
                        let value = literal.to_string();
                        let method = &value[1..value.len() - 1];

                        if ALLOWED_METHODS.contains(&method) {
                            parsed_methods.push(String::from(method));
                        }

                        prev_token = None;
                        rest = next;
                    }
                    _ => rest = next,
                }
            }
            Ok(((), rest))
        })?;

        Ok(Route {
            path,
            methods: parsed_methods,
        })
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
fn is_valid_actix_route_attribute(ident: Option<&Ident>) -> bool {
    matches!(ident, Some(operation) if ["get", "post", "put", "delete", "head", "connect", "options", "trace", "patch", "route"]
        .iter().any(|expected_operation| operation == expected_operation))
}
