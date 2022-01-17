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
};

#[cfg(feature = "actix_extras")]
impl ArgumentResolver for PathOperations {
    fn resolve_path_arguments(fn_args: &Punctuated<FnArg, Comma>) -> Option<Vec<Argument<'_>>> {
        if let Some((pat_type, path_segment)) = Self::find_path_pat_type_and_segment(fn_args) {
            let names = Self::get_argument_names(pat_type);
            let types = Self::get_argument_types(path_segment);

            if names.len() != types.len() {
                Some(
                    types
                        .into_iter()
                        .map(|ty| Argument {
                            argument_in: ArgumentIn::Path,
                            ident: ty,
                            name: None,
                        })
                        .collect::<Vec<_>>(),
                )
            } else {
                Some(
                    names
                        .into_iter()
                        .zip(types.into_iter())
                        .map(|(name, ty)| Argument {
                            argument_in: ArgumentIn::Path,
                            ident: ty,
                            name: Some(name.to_string()),
                        })
                        .collect::<Vec<_>>(),
                )
            }
        } else {
            None
        }
    }
}

#[cfg(feature = "actix_extras")]
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

#[cfg(feature = "actix_extras")]
impl PathOperationResolver for PathOperations {
    fn resolve_attribute(item_fn: &ItemFn) -> Option<&Attribute> {
        item_fn.attrs.iter().find_map(|attribute| {
            if is_valid_request_type(
                &attribute
                    .path
                    .get_ident()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
            ) {
                Some(attribute)
            } else {
                None
            }
        })
    }
}

#[cfg(feature = "actix_extras")]
impl PathResolver for PathOperations {
    fn resolve_path(operation_attribute: &Option<&Attribute>) -> Option<String> {
        operation_attribute.map(|attribute| {
            let lit = attribute.parse_args::<LitStr>().unwrap();
            format_path(&lit.value())
        })
    }
}

fn format_path(path: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\{[a-zA-Z0-9_]*:[^{}]*}").unwrap();
    }

    RE.replace_all(path, |captures: &Captures| {
        let mut capture = captures.get(0).unwrap().as_str().to_string();

        if capture.contains("_:") {
            // replace unnamed capture with generic 'arg0' name
            "{arg0}".to_string()
        } else if capture.contains(':') {
            //  replace colon (:) separated regexp with empty string
            let colon = capture.find(':').unwrap();
            let end = capture.len() - 1;
            capture.replace_range(colon..end, "");

            capture
        } else {
            // otherwise return the capture itself
            capture
        }
    })
    .to_string()
}

fn is_valid_request_type(s: &str) -> bool {
    match s {
        "get" | "post" | "put" | "delete" | "head" | "connect" | "options" | "trace" | "patch" => {
            true
        }
        _ => false,
    }
}
