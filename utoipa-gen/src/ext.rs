use syn::{punctuated::Punctuated, token::Comma, ItemFn};

pub struct PathOperations;

impl PathOperations {
    pub fn resolve_arguments(
        fn_args: &'_ Punctuated<syn::FnArg, Comma>,
        macro_args: Option<Vec<utoipa_corelib::MacroArg>>,
    ) -> (
        Option<Vec<utoipa_corelib::ValueArgument<'_>>>,
        Option<Vec<utoipa_corelib::IntoParamsType<'_>>>,
    ) {
        #[cfg(feature = "actix_extras")]
        {
            <utoipa_corelib::PathOperations as utoipa_gen_actix_web::PathOperationExt>
                ::resolve_arguments(fn_args, macro_args)
        }
        #[cfg(feature = "rocket_extras")]
        {
            <utoipa_corelib::PathOperations as utoipa_gen_rocket::PathOperationsExt>
                ::resolve_arguments(fn_args, macro_args)
        }
        #[cfg(feature = "axum_extras")]
        {
            <utoipa_corelib::PathOperations as utoipa_gen_axum::PathOperationsExt>
                ::resolve_arguments(fn_args, macro_args)
        }

        #[cfg(not(any(
            feature = "actix_extras",
            feature = "rocket_extras",
            feature = "axum_extras"
        )))]
        (None, None)
    }

    #[allow(unused_variables)]
    pub fn resolve_path(path: &Option<String>) -> Option<utoipa_corelib::MacroPath> {
        #[cfg(feature = "actix_extras")]
        {
            <utoipa_corelib::PathOperations as utoipa_gen_actix_web::PathOperationExt>::resolve_path(
                path,
            )
        }

        #[cfg(feature = "rocket_extras")]
        {
            <utoipa_corelib::PathOperations as utoipa_gen_rocket::PathOperationsExt>::resolve_path(
                path,
            )
        }

        #[cfg(not(any(feature = "actix_extras", feature = "rocket_extras",)))]
        None
    }

    #[allow(unused_variables)]
    pub fn resolve_operation(item_fn: &ItemFn) -> Option<utoipa_corelib::ResolvedOperation> {
        #[cfg(feature = "actix_extras")]
        {
            <utoipa_corelib::PathOperations as utoipa_gen_actix_web::PathOperationExt>::resolve_operation(item_fn)
        }

        #[cfg(feature = "rocket_extras")]
        {
            <utoipa_corelib::PathOperations as utoipa_gen_rocket::PathOperationsExt>::resolve_operation(item_fn)
        }

        #[cfg(not(any(feature = "actix_extras", feature = "rocket_extras",)))]
        None
    }
}
