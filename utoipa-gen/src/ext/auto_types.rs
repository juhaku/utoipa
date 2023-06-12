use std::borrow::Cow;

use syn::{ItemFn, TypePath};

pub fn parse_fn_operation_responses(fn_op: &ItemFn) -> Option<&TypePath> {
    get_response_type(fn_op).and_then(get_type_path)
}

#[inline]
fn get_type_path(ty: &syn::Type) -> Option<&TypePath> {
    match ty {
        syn::Type::Path(ty_path) => Some(ty_path),
        _ => None,
    }
}

#[inline]
fn get_response_type(fn_op: &ItemFn) -> Option<&syn::Type> {
    match &fn_op.sig.output {
        syn::ReturnType::Type(_, item) => Some(item.as_ref()),
        syn::ReturnType::Default => None, // default return type () should result no responses
    }
}

#[cfg(all(feature = "actix_extras", feature = "actix_auto_responses"))]
fn to_response(
    type_tree: crate::component::TypeTree<'_>,
    status: crate::path::response::ResponseStatus,
) -> crate::path::response::Response {
    use crate::ext::TypeTreeExt;
    use crate::path::response::{Response, ResponseTuple, ResponseValue};

    dbg!(&type_tree);
    let type_path = TypePath {
        path: type_tree
            .path
            .as_deref()
            .expect("Response should have a type")
            .clone(),
        qself: None,
    };
    let content_type = type_tree.get_default_content_type();
    let path = syn::Type::Path(type_path);
    let response_value = ResponseValue::from((Cow::Owned(path), content_type));
    let response: ResponseTuple = (status, response_value).into();

    dbg!(&response);

    Response::Tuple(response)
}

#[cfg(all(feature = "actix_extras", feature = "actix_auto_responses"))]
pub fn parse_actix_web_response(fn_op: &ItemFn) -> Vec<crate::path::response::Response<'_>> {
    get_response_type(fn_op)
        .map(crate::component::TypeTree::from_type)
        .and_then(super::get_actual_type)
        .map(|(first, second)| {
            let mut responses = Vec::<crate::path::response::Response>::with_capacity(2);
            if let Some(first) = first {
                responses.push(to_response(first, syn::parse_quote!(200)));
            };
            if let Some(second) = second {
                responses.push(to_response(second, syn::parse_quote!("default")));
            };
            responses
        })
        .unwrap_or_else(Vec::new)
}
