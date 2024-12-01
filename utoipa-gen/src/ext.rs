use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::Generics;
use syn::{punctuated::Punctuated, token::Comma, ItemFn};

use crate::component::{ComponentSchema, ComponentSchemaProps, Container, TypeTree};
use crate::path::media_type::MediaTypePathExt;
use crate::path::{HttpMethod, PathTypeTree};
use crate::{Diagnostics, ToTokensDiagnostics};

#[cfg(feature = "auto_into_responses")]
pub mod auto_types;

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

#[cfg(feature = "actix_extras")]
impl<'v> From<(MacroArg, TypeTree<'v>)> for ValueArgument<'v> {
    fn from((macro_arg, primitive_arg): (MacroArg, TypeTree<'v>)) -> Self {
        Self {
            name: match macro_arg {
                MacroArg::Path(path) => Some(Cow::Owned(path.name)),
                #[cfg(feature = "rocket_extras")]
                MacroArg::Query(_) => None,
            },
            type_tree: Some(primitive_arg),
            argument_in: ArgumentIn::Path,
        }
    }
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

impl<'i> From<(Option<Cow<'i, syn::Path>>, TokenStream)> for IntoParamsType<'i> {
    fn from((type_path, parameter_in_provider): (Option<Cow<'i, syn::Path>>, TokenStream)) -> Self {
        IntoParamsType {
            parameter_in_provider,
            type_path,
        }
    }
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
pub struct ExtSchema<'a>(TypeTree<'a>);

impl<'t> From<TypeTree<'t>> for ExtSchema<'t> {
    fn from(value: TypeTree<'t>) -> ExtSchema<'t> {
        Self(value)
    }
}

impl ExtSchema<'_> {
    fn get_actual_body(&self) -> Cow<'_, TypeTree<'_>> {
        let actual_body_type = get_actual_body_type(&self.0);

        actual_body_type.map(|actual_body| {
            if let Some(option_type) = find_option_type_tree(actual_body) {
                let path = option_type.path.clone();
                Cow::Owned(TypeTree {
                    children: Some(vec![actual_body.clone()]),
                    generic_type: Some(crate::component::GenericType::Option),
                    value_type: crate::component::ValueType::Object,
                    span: Some(path.span()),
                    path,
                })
            } else {
                Cow::Borrowed(actual_body)
            }
        }).expect("ExtSchema must have actual request body resoved from TypeTree of handler fn argument")
    }

    pub fn get_type_tree(&self) -> Result<Option<Cow<'_, TypeTree<'_>>>, Diagnostics> {
        Ok(Some(Cow::Borrowed(&self.0)))
    }

    pub fn get_default_content_type(&self) -> Result<Cow<'static, str>, Diagnostics> {
        let type_tree = &self.0;

        let content_type = if type_tree.is("Bytes") {
            Cow::Borrowed("application/octet-stream")
        } else if type_tree.is("Form") {
            Cow::Borrowed("application/x-www-form-urlencoded")
        } else {
            let get_actual_body = self.get_actual_body();
            let actual_body = get_actual_body.as_ref();

            actual_body.get_default_content_type()
        };

        Ok(content_type)
    }

    pub fn get_component_schema(&self) -> Result<Option<ComponentSchema>, Diagnostics> {
        use crate::OptionExt;

        let type_tree = &self.0;
        let actual_body_type = get_actual_body_type(type_tree);

        actual_body_type.and_then_try(|body_type| body_type.get_component_schema())
    }
}

impl ToTokensDiagnostics for ExtSchema<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) -> Result<(), Diagnostics> {
        let get_actual_body = self.get_actual_body();
        let type_tree = get_actual_body.as_ref();

        let component_tokens = ComponentSchema::new(ComponentSchemaProps {
            type_tree,
            features: Vec::new(),
            description: None,
            container: &Container {
                generics: &Generics::default(),
            },
        })?;
        component_tokens.to_tokens(tokens);

        Ok(())
    }
}

fn get_actual_body_type<'t>(ty: &'t TypeTree<'t>) -> Option<&'t TypeTree<'t>> {
    ty.path
        .as_deref()
        .expect("RequestBody TypeTree must have syn::Path")
        .segments
        .iter()
        .find_map(|segment| match &*segment.ident.to_string() {
            "Json" => Some(
                ty.children
                    .as_deref()
                    .expect("Json must have children")
                    .first()
                    .expect("Json must have one child"),
            ),
            "Form" => Some(
                ty.children
                    .as_deref()
                    .expect("Form must have children")
                    .first()
                    .expect("Form must have one child"),
            ),
            "Option" => get_actual_body_type(
                ty.children
                    .as_deref()
                    .expect("Option must have children")
                    .first()
                    .expect("Option must have one child"),
            ),
            "Bytes" => Some(ty),
            _ => match ty.children {
                Some(ref children) => get_actual_body_type(children.first().expect(
                    "Must have first child when children has been defined in get_actual_body_type",
                )),
                None => None,
            },
        })
}

fn find_option_type_tree<'t>(ty: &'t TypeTree) -> Option<&'t TypeTree<'t>> {
    let eq = ty.generic_type == Some(crate::component::GenericType::Option);

    if !eq {
        ty.children
            .as_ref()
            .and_then(|children| children.iter().find_map(find_option_type_tree))
    } else {
        Some(ty)
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct MacroPath {
    pub path: String,
    #[allow(unused)] // this is needed only if axum, actix or rocket
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
    fn by_name(a: &MacroArg, b: &MacroArg) -> std::cmp::Ordering {
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
    pub methods: Vec<HttpMethod>,
    pub path: String,
    #[allow(unused)] // this is needed only if axum, actix or rocket
    pub body: String,
}

#[allow(unused)]
pub type Arguments<'a> = (
    Option<Vec<ValueArgument<'a>>>,
    Option<Vec<IntoParamsType<'a>>>,
    Option<ExtSchema<'a>>,
);

#[allow(unused)]
pub trait ArgumentResolver {
    fn resolve_arguments(
        _: &'_ Punctuated<syn::FnArg, Comma>,
        _: Option<Vec<MacroArg>>,
        _: String,
    ) -> Result<Arguments, Diagnostics> {
        Ok((None, None, None))
    }
}

pub trait PathResolver {
    fn resolve_path(_: &Option<String>) -> Option<MacroPath> {
        None
    }
}

pub trait PathOperationResolver {
    fn resolve_operation(_: &ItemFn) -> Result<Option<ResolvedOperation>, Diagnostics> {
        Ok(None)
    }
}

pub struct PathOperations;

#[cfg(not(any(
    feature = "actix_extras",
    feature = "rocket_extras",
    feature = "axum_extras"
)))]
impl ArgumentResolver for PathOperations {}

#[cfg(not(any(
    feature = "actix_extras",
    feature = "rocket_extras",
    feature = "axum_extras"
)))]
impl PathResolver for PathOperations {}

#[cfg(not(any(feature = "actix_extras", feature = "rocket_extras")))]
impl PathOperationResolver for PathOperations {}

#[cfg(any(
    feature = "actix_extras",
    feature = "axum_extras",
    feature = "rocket_extras"
))]
pub mod fn_arg {

    use proc_macro2::Ident;
    #[cfg(any(feature = "actix_extras", feature = "axum_extras"))]
    use quote::quote;
    use syn::spanned::Spanned;
    use syn::PatStruct;
    use syn::{punctuated::Punctuated, token::Comma, Pat, PatType};

    use crate::component::TypeTree;
    #[cfg(any(feature = "actix_extras", feature = "axum_extras"))]
    use crate::component::ValueType;
    use crate::Diagnostics;

    /// Http operation handler functions fn argument.
    #[cfg_attr(feature = "debug", derive(Debug))]
    pub struct FnArg<'a> {
        pub ty: TypeTree<'a>,
        pub arg_type: FnArgType<'a>,
    }

    #[cfg_attr(feature = "debug", derive(Debug))]
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    pub enum FnArgType<'t> {
        Single(&'t Ident),
        Destructed(Vec<&'t Ident>),
    }

    #[cfg(feature = "rocket_extras")]
    impl FnArgType<'_> {
        /// Get best effort name `Ident` for the type. For `FnArgType::Tuple` types it will take the first one
        /// from `Vec`.
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
            Some(self.arg_type.cmp(&other.arg_type))
        }
    }

    impl<'a> PartialEq for FnArg<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.ty == other.ty && self.arg_type == other.arg_type
        }
    }

    impl<'a> Eq for FnArg<'a> {}

    pub fn get_fn_args(
        fn_args: &Punctuated<syn::FnArg, Comma>,
    ) -> Result<impl Iterator<Item = FnArg>, Diagnostics> {
        fn_args
            .iter()
            .filter_map(|arg| {
                let pat_type = match get_fn_arg_pat_type(arg) {
                    Ok(pat_type) => pat_type,
                    Err(diagnostics) => return Some(Err(diagnostics)),
                };

                match pat_type.pat.as_ref() {
                    syn::Pat::Wild(_) => None,
                    _ => {
                        let arg_name = match get_pat_fn_arg_type(pat_type.pat.as_ref()) {
                            Ok(arg_type) => arg_type,
                            Err(diagnostics) => return Some(Err(diagnostics)),
                        };
                        match TypeTree::from_type(&pat_type.ty) {
                            Ok(type_tree) => Some(Ok((type_tree, arg_name))),
                            Err(diagnostics) => Some(Err(diagnostics)),
                        }
                    }
                }
            })
            .map(|value| value.map(FnArg::from))
            .collect::<Result<Vec<FnArg>, Diagnostics>>()
            .map(IntoIterator::into_iter)
    }

    #[inline]
    fn get_pat_fn_arg_type(pat: &Pat) -> Result<FnArgType<'_>, Diagnostics> {
        let arg_name = match pat {
            syn::Pat::Ident(ident) => Ok(FnArgType::Single(&ident.ident)),
            syn::Pat::Tuple(tuple) => {
                tuple.elems.iter().map(|item| {
                    match item {
                        syn::Pat::Ident(ident) => Ok(&ident.ident),
                        _ => Err(Diagnostics::with_span(item.span(), "expected syn::Ident in get_pat_fn_arg_type Pat::Tuple"))
                    }
                }).collect::<Result<Vec<_>, Diagnostics>>().map(FnArgType::Destructed)
            },
            syn::Pat::TupleStruct(tuple_struct) => {
                get_pat_fn_arg_type(tuple_struct.elems.first().as_ref().expect(
                    "PatTuple expected to have at least one element, cannot get fn argument",
                ))
            },
            syn::Pat::Struct(PatStruct { fields, ..}) => {
                let idents = fields.iter()
                    .flat_map(|field| Ok(match get_pat_fn_arg_type(&field.pat) {
                        Ok(field_type) => field_type,
                        Err(diagnostics) => return Err(diagnostics),
                    }))
                    .fold(Vec::<&'_ Ident>::new(), |mut idents, field_type| {
                        if let FnArgType::Single(ident) = field_type {
                            idents.push(ident)
                        }
                        idents
                    });

                Ok(FnArgType::Destructed(idents))
            }
            _ => Err(Diagnostics::with_span(pat.span(), "unexpected syn::Pat, expected syn::Pat::Ident,in get_fn_args, cannot get fn argument name")),
        };
        arg_name
    }

    #[inline]
    fn get_fn_arg_pat_type(fn_arg: &syn::FnArg) -> Result<&PatType, Diagnostics> {
        match fn_arg {
            syn::FnArg::Typed(value) => Ok(value),
            _ => Err(Diagnostics::with_span(
                fn_arg.span(),
                "unexpected fn argument type, expected FnArg::Typed",
            )),
        }
    }

    #[cfg(any(feature = "actix_extras", feature = "axum_extras"))]
    pub(super) fn with_parameter_in(
        arg: FnArg<'_>,
    ) -> Option<(
        Option<std::borrow::Cow<'_, syn::Path>>,
        proc_macro2::TokenStream,
    )> {
        let parameter_in_provider = if arg.ty.is("Path") {
            quote! { || Some (utoipa::openapi::path::ParameterIn::Path) }
        } else if arg.ty.is("Query") {
            quote! { || Some(utoipa::openapi::path::ParameterIn::Query) }
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

    // if type is either Path or Query with direct children as Object types without generics
    #[cfg(any(feature = "actix_extras", feature = "axum_extras"))]
    pub(super) fn is_into_params(fn_arg: &FnArg) -> bool {
        use crate::component::GenericType;
        let mut ty = &fn_arg.ty;

        if fn_arg.ty.generic_type == Some(GenericType::Option) {
            ty = fn_arg
                .ty
                .children
                .as_ref()
                .expect("FnArg Option must have children")
                .first()
                .expect("FnArg Option must have 1 child");
        }

        (ty.is("Path") || ty.is("Query"))
            && ty
                .children
                .as_ref()
                .map(|children| {
                    children.iter().all(|child| {
                        matches!(child.value_type, ValueType::Object)
                            && child.generic_type.is_none()
                    })
                })
                .unwrap_or(false)
    }
}
