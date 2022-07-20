#![allow(unused)]
use std::{borrow::Cow, cmp::Ordering};

use proc_macro2::{Ident, TokenStream};
use syn::{punctuated::Punctuated, token::Comma, Attribute, FnArg, ItemFn, TypePath};

use crate::path::{parameter::Parameter, PathOperation};

#[cfg(feature = "actix_extras")]
pub mod actix;

#[cfg(feature = "axum_extras")]
pub mod axum;

#[cfg(feature = "rocket_extras")]
pub mod rocket;

/// Represents single argument of handler operation.
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ValueArgument<'a> {
    pub name: Option<Cow<'a, str>>,
    pub argument_in: ArgumentIn,
    pub type_path: Option<Cow<'a, TypePath>>,
    pub is_array: bool,
    pub is_option: bool,
}

/// Represents Identifier with `parameter_in` provider function which is used to
/// update the `parameter_in` to [`Parameter::Struct`].
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct IntoParamsType<'a> {
    pub parameter_in_provider: TokenStream,
    pub type_path: Cow<'a, TypePath>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq)]
pub enum ArgumentIn {
    Path,
    Query,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct MacroPath {
    pub path: String,
    pub args: Vec<MacroArg>,
}

// #[derive(PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum MacroArg {
    Path(ArgValue),
    Query(ArgValue),
}

impl MacroArg {
    /// Get ordering by name
    fn by_name(a: &MacroArg, b: &MacroArg) -> Ordering {
        a.get_value().name.cmp(&b.get_value().name)
    }

    fn get_value(&self) -> &ArgValue {
        match self {
            MacroArg::Path(path) => path,
            MacroArg::Query(query) => query,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ArgValue {
    pub name: String,
    pub original_name: String,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ResolvedOperation {
    pub path_operation: PathOperation,
    pub path: String,
}

pub trait ArgumentResolver {
    fn resolve_arguments(
        _: &'_ Punctuated<syn::FnArg, Comma>,
        _: Option<Vec<MacroArg>>,
    ) -> (
        Option<Vec<ValueArgument<'_>>>,
        Option<Vec<IntoParamsType<'_>>>,
    ) {
        (None, None)
    }
}

pub trait PathResolver {
    fn resolve_path(_: &Option<String>) -> Option<MacroPath> {
        None
    }
}

pub trait PathOperationResolver {
    fn resolve_operation(_: &ItemFn) -> Option<ResolvedOperation> {
        None
    }
}

pub struct PathOperations;

#[cfg(not(any(
    feature = "actix_extras",
    feature = "rocket_extras",
    feature = "axum_extras"
)))]
impl ArgumentResolver for PathOperations {}

#[cfg(not(any(feature = "actix_extras", feature = "rocket_extras")))]
impl PathResolver for PathOperations {}

#[cfg(not(any(feature = "actix_extras", feature = "rocket_extras")))]
impl PathOperationResolver for PathOperations {}

#[cfg(any(
    feature = "actix_extras",
    feature = "axum_extras",
    feature = "rocket_extras"
))]
pub mod fn_arg {
    use std::{borrow::Cow, cmp::Ordering};

    use proc_macro2::{Ident, TokenStream};
    use proc_macro_error::abort_call_site;
    use quote::quote;
    use syn::{
        punctuated::Punctuated, token::Comma, GenericArgument, PatType, PathArguments, PathSegment,
        Type, TypePath,
    };

    use crate::{
        component_type::ComponentType,
        schema::{ComponentPart, GenericType, ValueType},
    };

    use super::{ArgumentIn, IntoParamsType, MacroArg, ValueArgument};

    /// Http operation handler funtion's fn argument.
    ///
    /// [`FnArg`] is used to indicate the parameter type of handler function argument.
    /// E.g in following case [`FnArg::Path`] would be used. This in turn will specify the
    /// [`utoipa::openapi::path::ParameterIn`] for the parameter(s).
    /// ```text
    /// fn get_me(params: Path<i32>) {}
    /// ```
    #[cfg_attr(feature = "debug", derive(Debug, Clone))]
    pub enum FnArg<'a> {
        /// Path query parameters after the question mark (?).
        Query(&'a TypePath),
        /// Path parameters
        Path(&'a TypePath),
    }

    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct FnArg2<'a> {
        pub(super) ty: ComponentPart<'a>,
        pub(super) name: &'a Ident,
    }

    impl<'a> From<(ComponentPart<'a>, &'a Ident)> for FnArg2<'a> {
        fn from((ty, name): (ComponentPart<'a>, &'a Ident)) -> Self {
            Self { ty, name }
        }
    }

    impl<'a> Ord for FnArg2<'a> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.name.cmp(other.name)
        }
    }

    impl<'a> PartialOrd for FnArg2<'a> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.name.partial_cmp(other.name)
        }
    }

    impl<'a> PartialEq for FnArg2<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.ty == other.ty && self.name == other.name
        }
    }

    impl<'a> Eq for FnArg2<'a> {}

    /// Will
    pub trait SegmentFinder {
        fn matches(&self, path_segment: &PathSegment) -> bool;
        fn to_fn_arg<'a>(&self, type_path: &'a TypePath) -> FnArg<'a>;
    }

    fn get_type_path(ty: &Type) -> &TypePath {
        match ty {
            Type::Path(path) => path,
            Type::Reference(reference) => get_type_path(reference.elem.as_ref()),
            _ => abort_call_site!("unexpected type in path operations, expected Type::Path"), // should not get here by any means with current types
        }
    }

    fn get_argument_types(path_segment: &PathSegment) -> impl Iterator<Item = &TypePath> {
        match &path_segment.arguments {
            PathArguments::AngleBracketed(angle_bracketed) => {
                angle_bracketed.args.iter().flat_map(|arg| match arg {
                    GenericArgument::Type(ty) => match ty {
                        Type::Path(path) => vec![path],
                        Type::Tuple(tuple) => tuple.elems.iter().map(get_type_path).collect(),
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
            }
            _ => {
                abort_call_site!("unexpected argument type, expected Path<...> with angle brakets")
            }
        }
    }

    pub fn get_fn_args(
        fn_args: &Punctuated<syn::FnArg, Comma>,
    ) -> impl Iterator<Item = FnArg2<'_>> {
        let tt = fn_args
            .iter()
            .map(|arg| {
                let pat_type = get_fn_arg_pat_type(arg);

                let arg_name = match pat_type.pat.as_ref() {
                    syn::Pat::Ident(ident) => &ident.ident,
                    _ => abort_call_site!(
                        "unexpected syn::Pat, expected syn::Pat::Ident,in get_fn_args, cannot get fn argument name"
                    ),
                };

                (ComponentPart::from_type(pat_type.ty.as_ref()), arg_name)
            })
            .map(FnArg2::from);

        tt

        // let ff = fn_args
        //     .iter()
        //     .filter_map(|arg| get_fn_arg_segment(arg, finder))
        //     .flat_map(|path_segment| {
        //         // let op = if path_segment.ident == "Path" {
        //         //     FnArg::Path
        //         // } else {
        //         //     FnArg::Query
        //         // };
        //         get_argument_types(path_segment).map(|segment| finder.to_fn_arg(segment))
        //     });

        // ff
    }

    fn get_fn_arg_segment<'a>(
        fn_arg: &'a syn::FnArg,
        finder: &impl SegmentFinder,
    ) -> Option<&'a PathSegment> {
        let pat_type = get_fn_arg_pat_type(fn_arg);
        let type_path = get_type_path(pat_type.ty.as_ref());

        type_path
            .path
            .segments
            .iter()
            .find(|segment| finder.matches(segment))
        // .find(|segment| segment.ident == "Path" || segment.ident == "Query")
    }

    fn get_fn_arg_pat_type(fn_arg: &syn::FnArg) -> &PatType {
        match fn_arg {
            syn::FnArg::Typed(value) => value,
            _ => abort_call_site!("unexpected fn argument type, expected FnArg::Typed"),
        }
    }

    pub(super) fn to_into_params_types<'a, I: IntoIterator<Item = FnArg2<'a>>>(
        arguments: I,
        parameter_in_provider: impl Fn(&'_ FnArg2<'a>) -> TokenStream + 'a,
    ) -> impl Iterator<Item = IntoParamsType<'a>> {
        arguments.into_iter().map(move |path_arg| {
            // let FnArg2 { ty, name } = &path_arg;
            // let (arg, parameter_in) = match path_arg {
            //     FnArg::Path(arg) => (arg, quote! { utoipa::openapi::path::ParameterIn::Path }),
            //     FnArg::Query(arg) => (arg, quote! { utoipa::openapi::path::ParameterIn::Query }),
            // };

            let parameter_in = parameter_in_provider(&path_arg);

            IntoParamsType {
                parameter_in_provider: quote! {
                    || Some(#parameter_in)
                },
                type_path: path_arg.ty.path,
            }
        })
    }

    pub(super) fn to_value_args<
        'a,
        R: IntoIterator<Item = MacroArg>,
        P: IntoIterator<Item = FnArg<'a>>,
    >(
        macro_args: R,
        primitive_args: P,
    ) -> impl Iterator<Item = ValueArgument<'a>> {
        macro_args
            .into_iter()
            .zip(primitive_args)
            .map(|(macro_arg, primitive_arg)| ValueArgument {
                name: match macro_arg {
                    MacroArg::Path(path) => Some(Cow::Owned(path.name)),
                    _ => {
                        unreachable!("ResolvedArg::Query is not reachable with primitive path type")
                    }
                },
                type_path: match primitive_arg {
                    FnArg::Path(arg_type) => Some(Cow::Borrowed(arg_type)),
                    _ => {
                        unreachable!("FnArg::Query is not reachable with primitive type")
                    }
                },
                is_array: false,
                is_option: false,
                argument_in: ArgumentIn::Path,
            })
    }

    pub(super) fn non_primitive_arg(fn_arg: &FnArg) -> bool {
        let is_primitive = |type_path| ComponentType(type_path).is_primitive();

        match fn_arg {
            FnArg::Path(path) => !is_primitive(path),
            FnArg::Query(query) => !is_primitive(query),
        }
    }

    pub(super) fn into_params(fn_arg: &FnArg2) -> bool {
        matches!(fn_arg.ty.value_type, ValueType::Object) && matches!(fn_arg.ty.generic_type, None)
    }

    pub(super) fn into_params_actix(fn_arg: &FnArg2) -> bool {
        fn_arg.ty.is("Path") || fn_arg.ty.is("Query")
    }

    #[inline]
    pub(super) fn get_last_ident(type_path: &TypePath) -> Option<&Ident> {
        type_path.path.segments.last().map(|segment| &segment.ident)
    }
}
