use std::{borrow::Cow, cmp::Ordering, str::FromStr};

use lazy_static::lazy_static;
use proc_macro2::Ident;
use proc_macro_error::{abort, abort_call_site, OptionExt};
use quote::quote;
use regex::{Captures, Regex};
use syn::{
    parse::Parse, punctuated::Punctuated, token::Comma, FnArg, LitStr, PatIdent, Token, Type,
    TypePath,
};

use crate::{
    component_type::ComponentType,
    ext::{
        fn_arg::{self, FnArg2},
        ArgValue, ArgumentIn, IntoParamsType, MacroArg, ValueArgument,
    },
    path::PathOperation,
};

use super::{
    fn_arg::SegmentFinder, ArgumentResolver, MacroPath, PathOperationResolver, PathOperations,
    PathResolver, ResolvedOperation,
};

impl ArgumentResolver for PathOperations {
    fn resolve_arguments(
        fn_args: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
        macro_args: Option<Vec<MacroArg>>,
    ) -> (
        Option<Vec<super::ValueArgument<'_>>>,
        Option<Vec<super::IntoParamsType<'_>>>,
    ) {
        const ANONYMOUS_ARG: &str = "<_>";

        dbg!(&macro_args);

        let args = fn_arg::get_fn_args(fn_args);
        let mut a = args.collect::<Vec<_>>();

        a.sort_unstable();

        dbg!(&a);

        let (non_primitive_args, primitive_args): (Vec<fn_arg::FnArg2>, Vec<fn_arg::FnArg2>) =
            a.into_iter().partition(fn_arg::non_primitive_arg2);

        macro_args
            .map(|args| {
                // TODO
                // find macro arg by name and map it to value argument
                //
                // find macro arg by nam an map it to into params type

                let aa = non_primitive_args
                    .into_iter()
                    .flat_map(|arg| {
                        let parameter_in_provider =
                            args.iter().find_map(|macro_arg| match macro_arg {
                                MacroArg::Path(path) => {
                                    if arg.name == &*path.name {
                                        Some(quote! { utoipa::openapi::path::ParameterIn::Path })
                                    } else {
                                        None
                                    }
                                }
                                MacroArg::Query(query) => {
                                    if arg.name == &*query.name {
                                        Some(quote! { utoipa::openapi::path::ParameterIn::Query })
                                    } else {
                                        None
                                    }
                                }
                            });

                        Some(arg).zip(parameter_in_provider)
                    })
                    .map(|(path_arg, parameter_in)| IntoParamsType {
                        parameter_in_provider: quote! {
                            || Some(#parameter_in)
                        },
                        type_path: path_arg.ty.path,
                    });

                // fn_arg::to_into_params_types(non_primitive_args.into_iter(), |arg| {
                //     // tODO
                //     arg.ty.path
                // });

                (None, None)
            })
            .unwrap_or_else(|| (None, None))

        // if let Some(ref macro_args) = macro_args {
        //     non_primitive_args.iter().map(|arg| {
        //         // TODO
        //         let arg = macro_args
        //             .iter()
        //             .find(|macro_arg| match macro_arg {
        //                 MacroArg::Path(path) => arg.name == &*path.name,
        //                 MacroArg::Query(query) => arg.name == &*query.name,
        //             })
        //             .expect_or_abort("");
        //     });
        //     // a.into_iter().zip(macro_args).map(|(arg, marg)| {
        //     //     // tODO

        //     // });
        // }

        // [utoipa-gen/src/ext/rocket.rs:33] &macro_args = Some(
        //     [
        //         Path(
        //             ArgValue {
        //                 name: "id",
        //                 original_name: "<id>",
        //             },
        //         ),
        //         Query(
        //             ArgValue {
        //                 name: "rest",
        //                 original_name: "<rest..>",
        //             },
        //         ),
        //     ],
        // )
        // [utoipa-gen/src/ext/rocket.rs:95] &args.collect::<Vec<_>>() = [
        //     FnArg2(
        //         ComponentPart {
        //             path: TypePath {
        //                 qself: None,
        //                 path: Path {
        //                     leading_colon: None,
        //                     segments: [
        //                         PathSegment {
        //                             ident: Ident {
        //                                 ident: "i32",
        //                                 span: #0 bytes(11718..11721),
        //                             },
        //                             arguments: None,
        //                         },
        //                     ],
        //                 },
        //             },
        //             value_type: Primitive,
        //             generic_type: None,
        //             child: None,
        //         },
        //     ),
        //     FnArg2(
        //         ComponentPart {
        //             path: TypePath {
        //                 qself: None,
        //                 path: Path {
        //                     leading_colon: None,
        //                     segments: [
        //                         PathSegment {
        //                             ident: Ident {
        //                                 ident: "QueryParams",
        //                                 span: #0 bytes(11729..11740),
        //                             },
        //                             arguments: None,
        //                         },
        //                     ],
        //                 },
        //             },
        //             value_type: Object,
        //             generic_type: None,
        //             child: None,
        //         },
        //     ),
        // ]

        // let value_arguments = macro_args.map(|args| {
        //     let (anonymous_args, mut named_args): (Vec<MacroArg>, Vec<MacroArg>) =
        //         args.into_iter().partition(|arg| {
        //             matches!(arg, MacroArg::Path(path) if path.original_name == ANONYMOUS_ARG)
        //                 || matches!(arg, MacroArg::Query(query) if query.original_name == ANONYMOUS_ARG)
        //         });

        //     named_args.sort_unstable_by(MacroArg::by_name);

        //     Self::get_fn_args(fn_args)
        //         .zip(named_args)
        //         .map(|(arg, named_arg)| {
        //             let (name, argument_in) = match named_arg {
        //                 MacroArg::Path(arg_value) => (arg_value.name, ArgumentIn::Path),
        //                 MacroArg::Query(arg_value) => (arg_value.name, ArgumentIn::Query),
        //             };

        //             ValueArgument {
        //                 name: Some(Cow::Owned(name)),
        //                 argument_in,
        //                 type_path: Some(arg.ty),
        //                 is_array: arg.is_array,
        //                 is_option: arg.is_option,
        //             }
        //         })
        //         .chain(anonymous_args.into_iter().map(|anonymous_arg| {
        //             let (name, argument_in) = match anonymous_arg {
        //                 MacroArg::Path(arg_value) => (arg_value.name, ArgumentIn::Path),
        //                 MacroArg::Query(arg_value) => (arg_value.name, ArgumentIn::Query),
        //             };

        //             ValueArgument {
        //                 name: Some(Cow::Owned(name)),
        //                 argument_in,
        //                 type_path: None,
        //                 is_array: false,
        //                 is_option: false,
        //             }
        //         }))
        //         .collect()
        // });

        // (None, None)
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
struct Arg<'a> {
    name: &'a Ident,
    ty: Cow<'a, TypePath>,
    is_array: bool,
    is_option: bool,
}

impl Arg<'_> {
    fn by_name(a: &Arg, b: &Arg) -> Ordering {
        a.name.cmp(b.name)
    }
}

impl PathOperations {
    fn get_fn_args(fn_args: &Punctuated<FnArg, Comma>) -> impl Iterator<Item = Arg> + '_ {
        let mut ordered_args = fn_args
            .into_iter()
            .filter(Self::is_supported_type)
            .map(|arg| match arg {
                FnArg::Typed(pat_type) => {
                    let ident = match pat_type.pat.as_ref() {
                        syn::Pat::Ident(ref pat) => &pat.ident,
                        _ => abort_call_site!("unexpected Pat, expected Pat::Ident"),
                    };

                    let (ty, is_array, is_option) = Self::get_type_ident(pat_type.ty.as_ref());

                    Arg {
                        is_array,
                        is_option,
                        name: ident,
                        ty,
                    }
                }
                _ => abort_call_site!("unexpected FnArg, expected FnArg::Typed"),
            })
            .collect::<Vec<_>>();

        ordered_args.sort_unstable_by(Arg::by_name);

        ordered_args.into_iter()
    }

    fn get_type_ident<'t>(ty: &'t Type) -> (Cow<'t, TypePath>, bool, bool) {
        match ty {
            Type::Path(path) => {
                let first_segment: syn::PathSegment = path.path.segments.first().unwrap().clone();
                let mut path: Cow<'t, TypePath> = Cow::Borrowed(path);

                if first_segment.arguments.is_empty() {
                    return (path, false, false);
                } else {
                    let is_array = first_segment.ident == "Vec";
                    let is_option = first_segment.ident == "Option";

                    match first_segment.arguments {
                        syn::PathArguments::AngleBracketed(ref angle_bracketed) => {
                            match angle_bracketed.args.first() {
                                Some(syn::GenericArgument::Type(arg)) => {
                                    let child_type = Self::get_type_ident(arg);

                                    let is_array = is_array || child_type.1;
                                    let is_option = is_option || child_type.2;

                                    // Discard the current segment if we are one of the special
                                    // types recognised as array or option
                                    if is_array || is_option {
                                        path = Cow::Owned(child_type.0.into_owned());
                                    }

                                    (path, is_array, is_option)
                                }
                                _ => abort_call_site!(
                                    "unexpected generic type, expected GenericArgument::Type"
                                ),
                            }
                        }
                        _ => abort_call_site!(
                            "unexpected path argument, expected angle bracketed arguments"
                        ),
                    }
                }
            }
            Type::Reference(reference) => Self::get_type_ident(reference.elem.as_ref()),
            _ => abort_call_site!(
                "unexpected pat type, expected one of: Type::Path, Type::Reference"
            ),
        }
    }

    fn get_type_path(ty: &Type) -> &TypePath {
        match ty {
            Type::Path(path) => path,
            Type::Reference(reference) => Self::get_type_path(reference.elem.as_ref()),
            _ => abort_call_site!("unexpected type, expected one of: Type::Path, Type::Reference"),
        }
    }

    fn is_supported_type(arg: &&FnArg) -> bool {
        match arg {
            FnArg::Typed(pat_type) => {
                let path = Self::get_type_path(pat_type.ty.as_ref());
                let segment = &path.path.segments.first().unwrap();

                let mut is_supported = ComponentType(path).is_primitive();

                if !is_supported {
                    is_supported = matches!(&*segment.ident.to_string(), "Vec" | "Option")
                }

                is_supported
            }
            _ => abort_call_site!("unexpected FnArg, expected FnArg::Typed"),
        }
    }
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
            while let Some((tt, next)) = rest.token_tree() {
                rest = next;
            }
            Ok(((), rest))
        });

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
                .or(Some((&*whole_path, "")))
                .map(|(path, query)| {
                    let mut names =
                        Vec::<MacroArg>::with_capacity(RE.find_iter(whole_path).count());
                    let mut underscore_count = 0;

                    let mut format_arg =
                        |captures: &Captures, resolved_arg_op: fn(ArgValue) -> MacroArg| {
                            let mut capture = &captures[0];
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
