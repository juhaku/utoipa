use syn::{ItemFn, TypePath};

/// A response resolved from the handler return type.
pub enum AutoResponse<'a> {
    /// A type documented by its `IntoResponses` implementation, if it has one.
    IntoResponses(&'a TypePath),
    /// `Json<T>` of a web framework, documented as a `200` response with `T` as JSON body.
    Json(&'a syn::Type),
}

/// Get the responses of the operation resolved from the handler return type.
pub fn parse_fn_operation_responses(fn_op: &ItemFn) -> Vec<AutoResponse<'_>> {
    match &fn_op.sig.output {
        syn::ReturnType::Type(_, item) => get_type_path(item.as_ref())
            .map(get_responses_types)
            .unwrap_or_default()
            .into_iter()
            .map(to_auto_response)
            .collect(),
        syn::ReturnType::Default => Vec::new(), // default return type () should result no responses
    }
}

fn to_auto_response(ty: &TypePath) -> AutoResponse<'_> {
    match get_json_body_type(ty) {
        Some(body) => AutoResponse::Json(body),
        None => AutoResponse::IntoResponses(ty),
    }
}

/// Get `T` of a web framework `Json<T>`. Like `Json<T>` request bodies, a type named `Json` is
/// only recognized when one of the framework extras is enabled.
fn get_json_body_type(ty: &TypePath) -> Option<&syn::Type> {
    if !cfg!(any(
        feature = "actix_extras",
        feature = "axum_extras",
        feature = "rocket_extras"
    )) {
        return None;
    }

    let last_segment = ty.path.segments.last()?;
    match get_generic_types(last_segment).as_slice() {
        [body] if last_segment.ident == "Json" => Some(*body),
        _ => None,
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
