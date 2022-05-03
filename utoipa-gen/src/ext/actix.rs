use std::borrow::Cow;

use lazy_static::lazy_static;
use proc_macro::TokenTree;
use proc_macro2::{Ident, Literal, Punct};
use proc_macro_error::{abort, abort_call_site};
use quote::{format_ident, quote, quote_spanned};
use regex::{Captures, Regex};
use syn::{
    parse::Parse, punctuated::Punctuated, token::Comma, Attribute, DeriveInput, FnArg,
    GenericArgument, ItemFn, LitStr, Pat, PatType, PathArguments, PathSegment, Type, TypePath,
};

use crate::{
    component_type::ComponentType,
    ext::{ArgValue, ArgumentValue},
    path::{self, PathOperation},
};

use super::{
    Argument, ArgumentIn, ArgumentResolver, PathOperationResolver, PathOperations, PathResolver,
    ResolvedArg, ResolvedOperation, ResolvedPath,
};

#[cfg_attr(feature = "debug", derive(Debug))]
enum Arg<'a> {
    Query(&'a Ident),
    Path(&'a Ident),
}

impl ArgumentResolver for PathOperations {
    fn resolve_path_arguments(
        fn_args: &Punctuated<FnArg, Comma>,
        resolved_path_args: Option<Vec<ResolvedArg>>,
    ) -> Option<Vec<Argument<'_>>> {
        let (primitive_args, non_primitive_args): (Vec<Arg>, Vec<Arg>) = Self::get_fn_args(fn_args)
            .partition(|arg| matches!(arg, Arg::Path(ty) if ComponentType(ty).is_primitive()));

        if let Some(resolved_args) = resolved_path_args {
            let primitive_args = Self::to_value_args(resolved_args, primitive_args);

            Some(
                primitive_args
                    .chain(Self::to_token_stream_args(non_primitive_args))
                    .collect(),
            )
        } else {
            Some(Self::to_token_stream_args(non_primitive_args).collect())
        }
    }
}

impl PathOperations {
    fn to_token_stream_args<'a, I: IntoIterator<Item = Arg<'a>>>(
        arguments: I,
    ) -> impl Iterator<Item = Argument<'a>> {
        arguments.into_iter().map(|path_arg| {
            let (ty, parameter_in) = match path_arg {
                Arg::Path(arg) => (arg, quote! { utoipa::openapi::path::ParameterIn::Path }),
                Arg::Query(arg) => (arg, quote! { utoipa::openapi::path::ParameterIn::Query }),
            };

            let assert_ty = format_ident!("_Assert{}", &ty);
            Argument::TokenStream(quote_spanned! {ty.span()=>
                {
                    struct #assert_ty where #ty : utoipa::IntoParams;

                    impl utoipa::ParameterIn for #ty {
                        fn parameter_in() -> Option<utoipa::openapi::path::ParameterIn> {
                            Some(#parameter_in)
                        }
                    }

                    <#ty>::into_params()
                }
            })
        })
    }

    fn to_value_args<'a, R: IntoIterator<Item = ResolvedArg>, P: IntoIterator<Item = Arg<'a>>>(
        resolved_args: R,
        primitive_args: P,
    ) -> impl Iterator<Item = Argument<'a>> {
        resolved_args
            .into_iter()
            .zip(primitive_args)
            .map(|(resolved_arg, primitive_arg)| {
                Argument::Value(ArgumentValue {
                    name: match resolved_arg {
                        ResolvedArg::Path(path) => Some(Cow::Owned(path.name)),
                        _ => unreachable!(
                            "ResolvedArg::Query is not reachable with primitive path type"
                        ),
                    },
                    ident: match primitive_arg {
                        Arg::Path(value) => Some(value),
                        _ => {
                            unreachable!("Arg::Query is not reachable with primitive type")
                        }
                    },
                    is_array: false,
                    is_option: false,
                    argument_in: ArgumentIn::Path,
                })
            })
    }

    fn get_type_path(ty: &Type) -> &TypePath {
        match ty {
            Type::Path(path) => path,
            _ => abort_call_site!("unexpected type in actix path operations, expected Type::Path"), // should not get here by any means with current types
        }
    }

    fn get_argument_types(path_segment: &PathSegment) -> impl Iterator<Item = &Ident> {
        match &path_segment.arguments {
            PathArguments::AngleBracketed(angle_bracketed) => angle_bracketed
                .args
                .iter()
                .flat_map(|arg| match arg {
                    GenericArgument::Type(ty) => match ty {
                        Type::Path(path) => vec![path],
                        Type::Tuple(tuple) => tuple.elems.iter().map(Self::get_type_path).collect(),
                        _ => {
                            abort_call_site!("unexpected type, expected Type::Path or Type::Tuple")
                        } // should not get here by any means with current types
                    },
                    _ => {
                        abort_call_site!(
                            "unexpected generic argument, expected GenericArgument::Type"
                        )
                    }
                })
                .flat_map(|type_path| type_path.path.get_ident()),
            _ => {
                abort_call_site!("unexpected argument type, expected Path<...> with angle brakets")
            }
        }
    }

    fn get_fn_args(fn_args: &Punctuated<FnArg, Comma>) -> impl Iterator<Item = Arg> {
        fn_args
            .iter()
            .filter_map(Self::get_fn_arg_segment)
            .flat_map(|path_segment| {
                let op = if path_segment.ident == "Path" {
                    Arg::Path
                } else {
                    Arg::Query
                };
                Self::get_argument_types(path_segment).map(op)
            })
    }

    fn get_fn_arg_segment(fn_arg: &FnArg) -> Option<&PathSegment> {
        let pat_type = Self::get_fn_arg_pat_type(fn_arg);
        let type_path = Self::get_type_path(pat_type.ty.as_ref());

        type_path
            .path
            .segments
            .iter()
            .find(|segment| segment.ident == "Path" || segment.ident == "Query")
    }

    fn get_fn_arg_pat_type(fn_arg: &FnArg) -> &PatType {
        match fn_arg {
            FnArg::Typed(value) => value,
            _ => abort_call_site!("unexpected fn argument type, expected FnArg::Typed"),
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
    fn resolve_path(path: &Option<String>) -> Option<ResolvedPath> {
        path.as_ref().map(|path| {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"\{[a-zA-Z0-9_][^{}]*}").unwrap();
            }

            let mut args = Vec::<ResolvedArg>::with_capacity(RE.find_iter(path).count());
            ResolvedPath {
                path: RE
                    .replace_all(path, |captures: &Captures| {
                        let mut capture = &captures[0];
                        let original_name = String::from(capture);

                        if capture.contains("_:") {
                            // replace unnamed capture with generic 'arg0' name
                            args.push(ResolvedArg::Path(ArgValue {
                                name: String::from("arg0"),
                                original_name,
                            }));
                            "{arg0}".to_string()
                        } else if let Some(colon) = capture.find(':') {
                            //  replace colon (:) separated regexp with empty string
                            capture = &capture[1..colon];

                            args.push(ResolvedArg::Path(ArgValue {
                                name: String::from(capture),
                                original_name,
                            }));

                            format!("{{{capture}}}")
                        } else {
                            args.push(ResolvedArg::Path(ArgValue {
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
