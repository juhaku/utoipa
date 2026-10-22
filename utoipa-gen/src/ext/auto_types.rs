use syn::{ItemFn, TypePath};

/// Get the types of the handler return type whose `IntoResponses` implementations document the
/// responses of the operation.
pub fn parse_fn_operation_responses(fn_op: &ItemFn) -> Vec<&TypePath> {
    match &fn_op.sig.output {
        syn::ReturnType::Type(_, item) => get_type_path(item.as_ref())
            .map(get_responses_types)
            .unwrap_or_default(),
        syn::ReturnType::Default => Vec::new(), // default return type () should result no responses
    }
}

fn get_type_path(ty: &syn::Type) -> Option<&TypePath> {
    match ty {
        syn::Type::Path(ty_path) => Some(ty_path),
        _ => None,
    }
}

/// Resolve `Result<T, E>` into `T` and `E` so that the responses of one side are documented even
/// when the other side, e.g. `Json<T>` of a web framework, does not implement `IntoResponses`.
/// Documenting `T` and then `E` gives the same responses as the `IntoResponses` implementation of
/// `Result<T, E>`, which appends the responses of `E` to those of `T`.
fn get_responses_types(ty: &TypePath) -> Vec<&TypePath> {
    let last_segment = ty
        .path
        .segments
        .last()
        .expect("TypePath must have at least one segment");

    if last_segment.ident != "Result" {
        return vec![ty];
    }

    match get_generic_types(last_segment).as_slice() {
        [ok, err] => [ok, err]
            .into_iter()
            .filter_map(|ty| get_type_path(ty))
            .flat_map(get_responses_types)
            .collect(),
        // The error type of an alias such as `Result<T>` is unknown. Document `T` first and then
        // the alias itself, which adds the error responses when the alias implements `IntoResponses`.
        [ok] => get_type_path(ok)
            .map(get_responses_types)
            .unwrap_or_default()
            .into_iter()
            .chain([ty])
            .collect(),
        _ => vec![ty],
    }
}

fn get_generic_types(segment: &syn::PathSegment) -> Vec<&syn::Type> {
    match &segment.arguments {
        syn::PathArguments::AngleBracketed(arguments) => arguments
            .args
            .iter()
            .filter_map(|argument| match argument {
                syn::GenericArgument::Type(ty) => Some(ty),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}
