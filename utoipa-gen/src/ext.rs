#![allow(unused)]
use std::{borrow::Cow, cmp::Ordering};

use proc_macro2::{Ident, TokenStream};
use syn::{punctuated::Punctuated, token::Comma, Attribute, FnArg, ItemFn};

use crate::path::PathOperation;

#[cfg(feature = "actix_extras")]
pub mod actix;
#[cfg(feature = "rocket_extras")]
pub mod rocket;

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Argument<'a> {
    Value(ArgumentValue<'a>),
    TokenStream(TokenStream),
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ArgumentValue<'a> {
    pub name: Option<Cow<'a, str>>,
    pub argument_in: ArgumentIn,
    pub ident: Option<&'a Ident>,
    pub is_array: bool,
    pub is_option: bool,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq)]
pub enum ArgumentIn {
    Path,
    Query,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ResolvedPath {
    pub path: String,
    pub args: Vec<ResolvedArg>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ResolvedArg {
    Path(ArgValue),
    Query(ArgValue),
}

impl ResolvedArg {
    fn by_name(a: &ResolvedArg, b: &ResolvedArg) -> Ordering {
        a.get_value().name.cmp(&b.get_value().name)
    }

    fn get_value(&self) -> &ArgValue {
        match self {
            ResolvedArg::Path(path) => path,
            ResolvedArg::Query(query) => query,
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
    fn resolve_path_arguments(
        _: &Punctuated<FnArg, Comma>,
        _: Option<Vec<ResolvedArg>>,
    ) -> Option<Vec<Argument<'_>>> {
        None
    }
}

pub trait PathResolver {
    fn resolve_path(_: &Option<String>) -> Option<ResolvedPath> {
        None
    }
}

pub trait PathOperationResolver {
    fn resolve_operation(_: &ItemFn) -> Option<ResolvedOperation> {
        None
    }
}

pub struct PathOperations;

// #[cfg(not(feature = "actix_extras"))]
#[cfg(not(any(feature = "actix_extras", feature = "rocket_extras")))]
impl ArgumentResolver for PathOperations {}
// #[cfg(not(feature = "actix_extras"))]
#[cfg(not(any(feature = "actix_extras", feature = "rocket_extras")))]
impl PathResolver for PathOperations {}
// #[cfg(all(not(feature = "actix_extras"), not(feature = "rocket_extras")))]
#[cfg(not(any(feature = "actix_extras", feature = "rocket_extras")))]
impl PathOperationResolver for PathOperations {}
