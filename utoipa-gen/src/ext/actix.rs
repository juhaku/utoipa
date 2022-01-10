use proc_macro2::Ident;
use proc_macro_error::abort_call_site;
use syn::{
    punctuated::Punctuated, token::Comma, Attribute, FnArg, GenericArgument, ItemFn, LitStr, Pat,
    PatType, PathArguments, PathSegment, Type, TypePath,
};

use super::{
    Argument, ArgumentIn, ArgumentResolver, PathOperationResolver, PathOperations, PathResolver,
};
use crate::path::{Parameter, ParameterIn};

#[cfg(feature = "actix_extras")]
impl ArgumentResolver for PathOperations {
    fn resolve_path_arguments(fn_args: &Punctuated<FnArg, Comma>) -> Option<Vec<Argument<'_>>> {
        if let Some((pat_type, path_segment)) = Self::find_path_pat_type_and_segment(fn_args) {
            let names = Self::get_argument_names(pat_type);
            let types = Self::get_argument_types(path_segment);

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
            lit.value() // TODO format path according OpenAPI specs
        })
    }
}

fn format_path(path: &str) -> String {
    // TODO

    // for c in path.chars() {
    //     // char

    //     c.
    // }
    "".to_string()
}

fn is_valid_request_type(s: &str) -> bool {
    match s {
        "get" | "post" | "put" | "delete" | "head" | "connect" | "options" | "trace" | "patch" => {
            true
        }
        _ => false,
    }
}

#[inline]
#[cfg(feature = "actix_extras")]
pub fn update_parameters_from_arguments(
    arguments: Option<Vec<Argument>>,
    parameters: &mut Option<Vec<Parameter>>,
) {
    if let Some(arguments) = arguments {
        let new_parameter = |argument: &Argument| {
            Parameter::new(
                &argument.name,
                argument.ident,
                if argument.argument_in == ArgumentIn::Path {
                    ParameterIn::Path
                } else {
                    ParameterIn::Query
                },
            )
        };

        if let Some(ref mut parameters) = parameters {
            parameters.iter_mut().for_each(|parameter| {
                if let Some(argument) = arguments
                    .iter()
                    .find(|argument| argument.name == parameter.name)
                {
                    parameter.update_parameter_type(argument.ident)
                }
            });

            arguments.iter().for_each(|argument| {
                // cannot use filter() for mutli borrow situation. :(
                if !parameters
                    .iter()
                    .any(|parameter| parameter.name == argument.name)
                {
                    // if parameters does not contain argument
                    parameters.push(new_parameter(argument))
                }
            });
        } else {
            // no parameters at all, add arguments to the parameters
            let mut params = Vec::with_capacity(arguments.len());
            arguments
                .iter()
                .map(new_parameter)
                .for_each(|arg| params.push(arg));
            *parameters = Some(params);
        }
    }
}
