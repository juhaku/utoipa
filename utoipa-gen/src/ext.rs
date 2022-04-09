#![allow(unused)]
use proc_macro2::Ident;
use syn::{punctuated::Punctuated, token::Comma, Attribute, FnArg, ItemFn};

use crate::path::PathOperation;

#[cfg(feature = "actix_extras")]
pub mod actix;

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Argument<'a> {
    pub name: Option<&'a str>,
    pub argument_in: ArgumentIn,
    pub ident: &'a Ident,
}

impl Argument<'_> {
    pub fn has_name(&self) -> bool {
        self.name.is_some()
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq)]
pub enum ArgumentIn {
    Path,
}

pub struct ResolvedPath {
    pub path: String,
    pub args: Vec<String>,
}

pub struct ResolvedOperation {
    pub path_operation: PathOperation,
    pub path: String,
}

pub trait ArgumentResolver {
    fn resolve_path_arguments<'a>(
        _: &'a Punctuated<FnArg, Comma>,
        _: &'a Option<ResolvedPath>,
    ) -> Option<Vec<Argument<'a>>> {
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

#[cfg(not(feature = "actix_extras"))]
impl ArgumentResolver for PathOperations {}
#[cfg(not(feature = "actix_extras"))]
impl PathResolver for PathOperations {}
#[cfg(not(feature = "actix_extras"))]
impl PathOperationResolver for PathOperations {}
