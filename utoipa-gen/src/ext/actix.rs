use std::borrow::Cow;

use lazy_static::lazy_static;
use proc_macro2::Ident;
use proc_macro_error::abort_call_site;
use regex::{Captures, Regex};
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, FnArg, GenericArgument, ItemFn, LitStr, Pat,
    PatType, PathArguments, PathSegment, Type, TypePath,
};

use super::{
    Argument, ArgumentIn, ArgumentResolver, PathOperationResolver, PathOperations, PathResolver,
    ResolvedPath,
};

impl ArgumentResolver for PathOperations {
    fn resolve_path_arguments<'a>(
        fn_args: &'a Punctuated<FnArg, Comma>,
        resolved_path: &'a Option<ResolvedPath>,
    ) -> Option<Vec<Argument<'a>>> {
        resolved_path
            .as_ref()
            .zip(Self::find_path_pat_type_and_segment(fn_args))
            .map(|(path, (_, path_segment))| {
                let types = Self::get_argument_types(path_segment);

                path.args
                    .iter()
                    .zip(types.into_iter())
                    .map(|(name, ty)| Argument {
                        argument_in: ArgumentIn::Path,
                        ident: ty,
                        name: Some(name),
                    })
                    .collect::<Vec<_>>()
            })
    }
}

impl PathOperations {
    fn get_type_path(ty: &Type) -> &TypePath {
        match ty {
            Type::Path(path) => path,
            _ => abort_call_site!("unexpected type, expected Type::Path"), // should not get here by any means with current types
        }
    }

    fn get_argument_names(pat_type: &PatType) -> Vec<&Ident> {
        match pat_type.pat.as_ref() {
            Pat::Ident(pat_ident) => {
                vec![&pat_ident.ident]
            }
            Pat::TupleStruct(tuple) => tuple
                .pat
                .elems
                .iter()
                .flat_map(|pat| match pat {
                    Pat::Ident(pat_ident) => vec![&pat_ident.ident],
                    Pat::Tuple(tuple) => tuple
                        .elems
                        .iter()
                        .map(|pat| match pat {
                            Pat::Ident(pat_ident) => &pat_ident.ident,
                            _ => abort_call_site!(
                                "unexpected pat ident in Pat::Tuple expected Pat::Ident"
                            ),
                        })
                        .collect(),
                    _ => abort_call_site!("unexpected pat type expected Pat::Ident"),
                })
                .collect::<Vec<_>>(),
            _ => abort_call_site!("unexpected pat type expected Pat::Ident or Pat::Tuple"),
        }
    }

    fn get_argument_types(path_segment: &PathSegment) -> Vec<&Ident> {
        match &path_segment.arguments {
            PathArguments::AngleBracketed(angle_bracketed) => angle_bracketed
                .args
                .iter()
                .map(|arg| match arg {
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
                .flatten()
                .map(|type_path| type_path.path.get_ident())
                .flatten()
                .collect::<Vec<_>>(),
            _ => {
                abort_call_site!("unexpected argument type, expected Path<...> with angle brakets")
            }
        }
    }

    fn find_path_pat_type_and_segment(
        fn_args: &Punctuated<FnArg, Comma>,
    ) -> Option<(&PatType, &PathSegment)> {
        fn_args.iter().find_map(|arg| {
            match arg {
                FnArg::Typed(pat_type) => {
                    let segment = Self::get_type_path(pat_type.ty.as_ref())
                        .path
                        .segments
                        .iter()
                        .find_map(|segment| {
                            if &*segment.ident.to_string() == "Path" {
                                Some(segment)
                            } else {
                                None
                            }
                        });

                    segment.map(|segment| (pat_type, segment))
                }
                _ => abort_call_site!("unexpected fn argument type, expected FnArg::Typed(...)"), // should not get here
            }
        })
    }
}

impl PathOperationResolver for PathOperations {
    fn resolve_operation(item_fn: &ItemFn) -> Option<&Attribute> {
        item_fn.attrs.iter().find_map(|attribute| {
            if is_valid_request_type(attribute.path.get_ident()) {
                Some(attribute)
            } else {
                None
            }
        })
    }
}

impl PathResolver for PathOperations {
    fn resolve_path(path: &Option<String>) -> Option<ResolvedPath> {
        path.as_ref().map(|path| {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"\{[a-zA-Z0-9_][^{}]*}").unwrap();
            }

            let mut args = Vec::<String>::with_capacity(RE.find_iter(path).count());
            ResolvedPath {
                path: RE
                    .replace_all(path, |captures: &Captures| {
                        let mut capture = captures.get(0).unwrap().as_str().to_string();

                        if capture.contains("_:") {
                            // replace unnamed capture with generic 'arg0' name
                            args.push(String::from("arg0"));
                            "{arg0}".to_string()
                        } else if let Some(colon) = capture.find(':') {
                            //  replace colon (:) separated regexp with empty string
                            let end = capture.len() - 1;
                            capture.replace_range(colon..end, "");

                            args.push(String::from(&capture[1..capture.len() - 1]));

                            capture
                        } else {
                            args.push(String::from(&capture[1..capture.len() - 1]));
                            // otherwise return the capture itself
                            capture
                        }
                    })
                    .to_string(),
                args,
            }
        })
    }
}

fn is_valid_request_type(ident: Option<&Ident>) -> bool {
    matches!(ident, Some(operation) if ["get", "post", "put", "delete", "head", "connect", "options", "trace", "patch"]
        .iter().any(|expected_operation| operation == expected_operation))
}
