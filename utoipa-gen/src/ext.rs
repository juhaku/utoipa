use std::borrow::Cow;

#[cfg(feature = "rocket_extras")]
use std::cmp::Ordering;

use proc_macro2::TokenStream;
use syn::{punctuated::Punctuated, token::Comma, ItemFn};

use crate::component::TypeTree;
use crate::path::PathOperation;

#[cfg(feature = "actix_extras")]
pub mod actix;

#[cfg(feature = "axum_extras")]
pub mod axum;

#[cfg(feature = "rocket_extras")]
pub mod rocket;

/// Represents single argument of handler operation.
#[cfg_attr(
    not(any(
        feature = "actix_extras",
        feature = "rocket_extras",
        feature = "axum_extras"
    )),
    allow(dead_code)
)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ValueArgument<'a> {
    pub name: Option<Cow<'a, str>>,
    #[cfg(any(
        feature = "actix_extras",
        feature = "rocket_extras",
        feature = "axum_extras"
    ))]
    pub argument_in: ArgumentIn,
    pub type_tree: Option<TypeTree<'a>>,
}

#[cfg_attr(
    not(any(
        feature = "actix_extras",
        feature = "rocket_extras",
        feature = "axum_extras"
    )),
    allow(dead_code)
)]
/// Represents Identifier with `parameter_in` provider function which is used to
/// update the `parameter_in` to [`Parameter::Struct`].
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct IntoParamsType<'a> {
    pub parameter_in_provider: TokenStream,
    pub type_path: Option<Cow<'a, syn::Path>>,
}

#[cfg(any(
    feature = "actix_extras",
    feature = "rocket_extras",
    feature = "axum_extras"
))]
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
    #[cfg_attr(
        not(any(feature = "actix_extras", feature = "rocket_extras")),
        allow(dead_code)
    )]
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
    use syn::PatStruct;
    use syn::{punctuated::Punctuated, token::Comma, Pat, PatType};

    use crate::component::TypeTree;
    #[cfg(any(feature = "actix_extras", feature = "axum_extras"))]
    use crate::component::ValueType;

    use super::IntoParamsType;

    /// Http operation handler functions fn argument.
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct FnArg<'a> {
        pub(super) ty: TypeTree<'a>,
        pub(super) arg_type: FnArgType<'a>,
    }

    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    pub enum FnArgType<'t> {
        Single(&'t Ident),
        Destructed(Vec<&'t Ident>),
    }

    impl FnArgType<'_> {
        /// Get best effort name `Ident` for the type. For `FnArgType::Tuple` types it will take the first one
        /// from `Vec`.
        #[cfg(feature = "rocket_extras")]
        pub(super) fn get_name(&self) -> &Ident {
            match self {
                Self::Single(ident) => ident,
                // perform best effort name, by just taking the first one from the list
                Self::Destructed(tuple) => tuple
                    .first()
                    .expect("Expected at least one argument in FnArgType::Tuple"),
            }
        }
    }

    impl<'a> From<(TypeTree<'a>, FnArgType<'a>)> for FnArg<'a> {
        fn from((ty, arg_type): (TypeTree<'a>, FnArgType<'a>)) -> Self {
            Self { ty, arg_type }
        }
    }

    impl<'a> Ord for FnArg<'a> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.arg_type.cmp(&other.arg_type)
        }
    }

    impl<'a> PartialOrd for FnArg<'a> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.arg_type.partial_cmp(&other.arg_type)
        }
    }

    impl<'a> PartialEq for FnArg<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.ty == other.ty && self.arg_type == other.arg_type
        }
    }

    impl<'a> Eq for FnArg<'a> {}

    pub fn get_fn_args(fn_args: &Punctuated<syn::FnArg, Comma>) -> impl Iterator<Item = FnArg<'_>> {
        fn_args
            .iter()
            .filter_map(|arg| {
                let pat_type = get_fn_arg_pat_type(arg);

                match pat_type.pat.as_ref() {
                    syn::Pat::Wild(_) => None,
                    _ => {
                        let arg_name = get_pat_fn_arg_type(pat_type.pat.as_ref());
                        Some((TypeTree::from_type(&pat_type.ty), arg_name))
                    }
                }
            })
            .map(FnArg::from)
    }

    #[inline]
    fn get_pat_fn_arg_type(pat: &Pat) -> FnArgType {
        let arg_name = match pat {
            syn::Pat::Ident(ident) => FnArgType::Single(&ident.ident),
            syn::Pat::Tuple(tuple) => {
                FnArgType::Destructed(tuple.elems.iter().map(|item| {
                    match item {
                        syn::Pat::Ident(ident) => &ident.ident,
                        _ => abort!(item, "expected syn::Ident in get_pat_fn_arg_type Pat::Tuple")
                    }
                }).collect::<Vec<_>>())
            },
            syn::Pat::TupleStruct(tuple_struct) => {
                get_pat_fn_arg_type(tuple_struct.elems.first().as_ref().expect(
                    "PatTuple expected to have at least one element, cannot get fn argument",
                ))
            },
            syn::Pat::Struct(PatStruct { fields, ..}) => {
                let idents = fields.iter()
                    .map(|field| get_pat_fn_arg_type(&field.pat))
                    .fold(Vec::<&'_ Ident>::new(), |mut idents, field_type| {
                        if let FnArgType::Single(ident) = field_type {
                            idents.push(ident)
                        }
                        idents
                    });

                FnArgType::Destructed(idents)
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
