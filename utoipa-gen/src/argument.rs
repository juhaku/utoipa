use proc_macro2::Ident;
use proc_macro_error::abort_call_site;
use syn::{
    punctuated::Punctuated, token::Comma, FnArg, GenericArgument, Pat, PatType, PathArguments,
    PathSegment, Type, TypePath,
};

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Argument<'a> {
    pub name: String,
    pub argument_in: ArgumentIn,
    pub ident: &'a Ident,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ArgumentIn {
    Path,
    Query,
}

pub fn get_type_path(ty: &Type) -> &TypePath {
    match ty {
        Type::Path(path) => path,
        _ => abort_call_site!("unexpected type, expected Type::Path"), // should not get here by any means with current types
    }
}

pub fn resolve_path_arguments(fn_args: &Punctuated<FnArg, Comma>) -> Option<Vec<Argument>> {
    if let Some((pat_type, path_segment)) = find_path_pat_type_and_segment(fn_args) {
        let names = get_argument_names(pat_type);
        let types = get_argument_types(path_segment);

        Some(
            names
                .into_iter()
                .zip(types.into_iter())
                .map(|(name, ty)| Argument {
                    argument_in: ArgumentIn::Path,
                    ident: ty,
                    name: name.to_string(),
                })
                .collect::<Vec<_>>(),
        )
    } else {
        None
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
            .map(|pat| match pat {
                Pat::Ident(pat_ident) => &pat_ident.ident,
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
                    Type::Tuple(tuple) => tuple.elems.iter().map(get_type_path).collect(),
                    _ => {
                        abort_call_site!("unexpected type, expected Type::Path or Type::Tuple")
                    } // should not get here by any means with current types
                },
                _ => {
                    abort_call_site!("unexpected generic argument, expected GenericArgument::Type")
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
                let segment = get_type_path(pat_type.ty.as_ref())
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
