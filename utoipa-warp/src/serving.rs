//! Convenience warp filters for serving Swagger UI.

use std::sync::Arc;

use utoipa_swagger_ui::Config;
use warp::filters::BoxedFilter;
use warp::http::Uri;
use warp::hyper::{Response, StatusCode};
use warp::path::{FullPath, Tail};
use warp::{Filter, Rejection, Reply};

/// Create a warp filter that serves the Swagger UI.
///
/// This filter serves the Swagger UI HTML page and all associated static assets.
/// It includes a redirect from the base path (without trailing slash) to the path
/// with a trailing slash.
///
/// # Arguments
///
/// * `base_path` - The base path to serve Swagger UI at (e.g., `"swagger-ui"`)
/// * `config` - Swagger UI configuration (typically created from the spec URL)
///
/// # Examples
///
/// ```rust,no_run
/// # use utoipa_warp::serving::swagger_ui_filter;
/// # use utoipa_swagger_ui::Config;
/// let config = Config::from("/api-doc.json");
/// let filter = swagger_ui_filter("swagger-ui", config);
/// ```
pub fn swagger_ui_filter(
    base_path: &str,
    config: Config<'static>,
) -> BoxedFilter<(Box<dyn Reply>,)> {
    let config = Arc::new(config);
    let base_path_owned = base_path.trim_matches('/').to_string();
    let redirect_target = format!("/{base_path_owned}/");

    warp::path(base_path_owned)
        .and(warp::get())
        .and(warp::path::full())
        .and(warp::path::tail())
        .and(warp::any().map(move || config.clone()))
        .and(warp::any().map(move || redirect_target.clone()))
        .and_then(serve_swagger)
        .boxed()
}

async fn serve_swagger(
    full_path: FullPath,
    tail: Tail,
    config: Arc<Config<'static>>,
    redirect_target: String,
) -> Result<Box<dyn Reply>, Rejection> {
    if !full_path.as_str().ends_with('/') && tail.as_str().is_empty() {
        return Ok(Box::new(warp::redirect::found(
            redirect_target.parse::<Uri>().unwrap(),
        )));
    }

    let path = tail.as_str();
    match utoipa_swagger_ui::serve(path, config) {
        Ok(file) => {
            if let Some(file) = file {
                Ok(Box::new(
                    Response::builder()
                        .header("Content-Type", file.content_type)
                        .body(file.bytes)
                        .unwrap(),
                ))
            } else {
                Ok(Box::new(StatusCode::NOT_FOUND))
            }
        }
        Err(error) => Ok(Box::new(
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string())
                .unwrap(),
        )),
    }
}
