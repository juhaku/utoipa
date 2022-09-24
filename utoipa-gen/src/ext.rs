use std::borrow::Cow;

#[cfg(feature = "rocket_extras")]
use std::cmp::Ordering;

use proc_macro2::TokenStream;
use syn::{punctuated::Punctuated, token::Comma, ItemFn, Path};

use crate::path::PathOperation;

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
    pub type_path: Option<Cow<'a, Path>>,
    pub is_array: bool,
    pub is_option: bool,
}

/// Represents Identifier with `parameter_in` provider function which is used to
/// update the `parameter_in` to [`Parameter::Struct`].
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct IntoParamsType<'a> {
    pub parameter_in_provider: TokenStream,
    pub type_path: Option<Cow<'a, syn::Path>>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq, Eq)]
pub enum ArgumentIn {
    Path,
    #[cfg(feature = "rocket_extras")]
    Query,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct MacroPath {
    pub path: String,
    pub args: Vec<MacroArg>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum MacroArg {
    #[cfg_attr(feature = "axum_extras", allow(dead_code))]
    Path(ArgValue),
    #[cfg(feature = "rocket_extras")]
    Query(ArgValue),
}

impl MacroArg {
    /// Get ordering by name
    #[cfg(feature = "rocket_extras")]
    fn by_name(a: &MacroArg, b: &MacroArg) -> Ordering {
        a.get_value().name.cmp(&b.get_value().name)
    }

    #[cfg(feature = "rocket_extras")]
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

    use std::borrow::Cow;

    use proc_macro2::{Ident, TokenStream};
    use proc_macro_error::abort;
    #[cfg(any(feature = "actix_extras", feature = "axum_extras"))]
    use quote::quote;
    use syn::{punctuated::Punctuated, token::Comma, Pat, PatType};

    use crate::component::TypeTree;
    #[cfg(any(feature = "actix_extras", feature = "axum_extras"))]
    use crate::component::ValueType;

    use super::IntoParamsType;

    /// Http operation handler functions fn argument.
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct FnArg<'a> {
        pub(super) ty: TypeTree<'a>,
        pub(super) name: &'a Ident,
    }

    impl<'a> From<(TypeTree<'a>, &'a Ident)> for FnArg<'a> {
        fn from((ty, name): (TypeTree<'a>, &'a Ident)) -> Self {
            Self { ty, name }
        }
    }

    impl<'a> Ord for FnArg<'a> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.name.cmp(other.name)
        }
    }

    impl<'a> PartialOrd for FnArg<'a> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.name.partial_cmp(other.name)
        }
    }

    impl<'a> PartialEq for FnArg<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.ty == other.ty && self.name == other.name
        }
    }

    impl<'a> Eq for FnArg<'a> {}

    pub fn get_fn_args(fn_args: &Punctuated<syn::FnArg, Comma>) -> impl Iterator<Item = FnArg<'_>> {
        fn_args
            .iter()
            .map(|arg| {
                let pat_type = get_fn_arg_pat_type(arg);

                let arg_name = get_pat_ident(pat_type.pat.as_ref());
                (TypeTree::from_type(&pat_type.ty), arg_name)
            })
            .map(FnArg::from)
    }

    #[inline]
    fn get_pat_ident(pat: &Pat) -> &Ident {
        let arg_name = match pat {
            syn::Pat::Ident(ident) => &ident.ident,
            syn::Pat::TupleStruct(tuple_struct) => {
                get_pat_ident(tuple_struct.pat.elems.first().as_ref().expect(
                    "PatTuple expected to have at least one element, cannot get fn argument",
                ))
            }
            _ => abort!(pat,
                "unexpected syn::Pat, expected syn::Pat::Ident,in get_fn_args, cannot get fn argument name"
            ),
        };
        arg_name
    }

    #[inline]
    fn get_fn_arg_pat_type(fn_arg: &syn::FnArg) -> &PatType {
        match fn_arg {
            syn::FnArg::Typed(value) => value,
            _ => abort!(fn_arg, "unexpected fn argument type, expected FnArg::Typed"),
        }
    }

    #[cfg(any(feature = "actix_extras", feature = "axum_extras"))]
    pub(super) fn with_parameter_in(
        arg: FnArg<'_>,
    ) -> Option<(Option<std::borrow::Cow<'_, syn::Path>>, TokenStream)> {
        let parameter_in_provider = if arg.ty.is("Path") {
            quote! { || Some (utoipa::openapi::path::ParameterIn::Path) }
        } else if arg.ty.is("Query") {
            quote! { || Some( utoipa::openapi::path::ParameterIn::Query) }
        } else {
            quote! { || None }
        };

        let type_path = arg
            .ty
            .children
            .expect("FnArg TypeTree generic type Path must have children")
            .into_iter()
            .next()
            .unwrap()
            .path;

        Some((type_path, parameter_in_provider))
    }

    pub(super) fn into_into_params_type(
        (type_path, parameter_in_provider): (Option<Cow<'_, syn::Path>>, TokenStream),
    ) -> IntoParamsType<'_> {
        IntoParamsType {
            parameter_in_provider,
            type_path,
        }
    }

    // if type is either Path or Query with direct children as Object types without generics
    #[cfg(any(feature = "actix_extras", feature = "axum_extras"))]
    pub(super) fn is_into_params(fn_arg: &FnArg) -> bool {
        (fn_arg.ty.is("Path") || fn_arg.ty.is("Query"))
            && fn_arg
                .ty
                .children
                .as_ref()
                .map(|children| {
                    children.iter().all(|child| {
                        matches!(child.value_type, ValueType::Object)
                            && matches!(child.generic_type, None)
                    })
                })
                .unwrap_or(false)
    }
}
