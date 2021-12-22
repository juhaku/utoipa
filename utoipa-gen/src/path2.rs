use syn::parse::Parse;

// #[api_operation(delete, responses = [
//     (200, "success", String),
//     (400, "my bad error", u64),
//     (404, "vault not found"),
//     (500, "internal server error")
// ])]

/// PathAttr is parsed #[path(...)] proc macro and its attributes.
/// Parsed attributes can be used to override or append OpenAPI Path
/// options.
pub struct PathAttr {
    path_operation: PathOperation,
    responses: Vec<String>, // TODO correct response type???
    path: Option<String>,
    operation_id: Option<String>,
}

/// Parse implementation for PathAttr will parse arguments
/// exhaustively.
impl Parse for PathAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // TODO parse in a loop the args similar fashion to the component attribute
        Ok(Self {
            operation_id: None,
            path: None,
            path_operation: PathOperation::Get,
            responses: vec![],
        })
    }
}

enum PathOperation {
    Get,
}

struct PathResponse<T: Sized> {
    status_code: i32,
    message: String,
    response_type: T,
}
