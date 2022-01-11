use proc_macro2::Ident;
use syn::{punctuated::Punctuated, token::Comma, Attribute, FnArg, ItemFn};

#[cfg(feature = "actix_extras")]
pub mod actix;

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Argument<'a> {
    pub name: String,
    pub argument_in: ArgumentIn,
    pub ident: &'a Ident,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq)]
pub enum ArgumentIn {
    Path,
}

pub trait ArgumentResolver {
    fn resolve_path_arguments(_: &Punctuated<FnArg, Comma>) -> Option<Vec<Argument<'_>>> {
        None
    }
}

pub trait PathResolver {
    fn resolve_path(_: &Option<&Attribute>) -> Option<String> {
        None
    }
}

pub trait PathOperationResolver {
    fn resolve_attribute(_: &ItemFn) -> Option<&Attribute> {
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
