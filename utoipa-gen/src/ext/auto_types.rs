use syn::{ItemFn, TypePath};

pub fn parse_fn_operation_responses(fn_op: &ItemFn) -> Option<&TypePath> {
    match &fn_op.sig.output {
        syn::ReturnType::Type(_, item) => get_type_path(item.as_ref()),
        syn::ReturnType::Default => None, // default return type () should result no responses
    }
}

fn get_type_path(ty: &syn::Type) -> Option<&TypePath> {
    match ty {
        syn::Type::Path(ty_path) => Some(ty_path),
        _ => None,
    }
}
