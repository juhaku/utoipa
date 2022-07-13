use std::borrow::Cow;

use lazy_static::lazy_static;
use proc_macro::TokenTree;
use proc_macro2::{Ident, Literal, Punct};
use proc_macro_error::{abort, abort_call_site};
use quote::{format_ident, quote, quote_spanned};
use regex::{Captures, Regex};
use syn::{
    parse::Parse, punctuated::Punctuated, token::Comma, Attribute, DeriveInput, ExprPath,
    GenericArgument, ItemFn, LitStr, Pat, PatType, PathArguments, PathSegment, Type, TypeInfer,
    TypePath,
};

use crate::{
    component_type::ComponentType,
    ext::{ArgValue, ValueArgument},
    path::{self, PathOperation},
};

use super::{
    fn_arg::{self, FnArg},
    ArgumentIn, ArgumentResolver, MacroArg, MacroPath, PathOperationResolver, PathOperations,
    PathResolver, ResolvedOperation,
};

impl ArgumentResolver for PathOperations {
    fn resolve_arguments(
        fn_args: &Punctuated<syn::FnArg, Comma>,
        macro_args: Option<Vec<MacroArg>>,
    ) -> (
        Option<Vec<super::ValueArgument<'_>>>,
        Option<Vec<super::IntoParamsType<'_>>>,
    ) {
        let (non_primitive_args, primitive_args): (Vec<FnArg>, Vec<FnArg>) =
            fn_arg::get_fn_args(fn_args)
                .into_iter()
                .partition(fn_arg::non_primitive_arg);

        if let Some(macro_args) = macro_args {
            (
                Some(fn_arg::to_value_args(macro_args, primitive_args).collect()),
                Some(fn_arg::to_into_params_types(non_primitive_args).collect()),
            )
        } else {
            (
                None,
                Some(fn_arg::to_into_params_types(non_primitive_args).collect()),
            )
        }
    }
}

impl PathOperationResolver for PathOperations {
    fn resolve_operation(item_fn: &ItemFn) -> Option<ResolvedOperation> {
        item_fn.attrs.iter().find_map(|attribute| {
            if is_valid_request_type(attribute.path.get_ident()) {
                match attribute.parse_args::<Path>() {
                    Ok(path) => Some(ResolvedOperation {
                        path: path.0,
                        path_operation: PathOperation::from_ident(
                            attribute.path.get_ident().unwrap(),
                        ),
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
            while let Some((tt, next)) = rest.token_tree() {
                rest = next;
            }
            Ok(((), rest))
        });

        Ok(Self(path))
    }
}

impl PathResolver for PathOperations {
    fn resolve_path(path: &Option<String>) -> Option<MacroPath> {
        path.as_ref().map(|path| {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"\{[a-zA-Z0-9_][^{}]*}").unwrap();
            }

            let mut args = Vec::<MacroArg>::with_capacity(RE.find_iter(path).count());
            MacroPath {
                path: RE
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

#[inline]
fn get_last_ident(type_path: &TypePath) -> Option<&Ident> {
    type_path.path.segments.last().map(|segment| &segment.ident)
}
