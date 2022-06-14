use std::{net::Ipv4Addr, sync::Arc};

use utoipa::OpenApi;
use utoipa_swagger_ui::Config;
use warp::{
    http::Uri,
    hyper::{Response, StatusCode},
    path::{FullPath, Tail},
    Filter, Rejection, Reply,
};

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = Arc::new(Config::new(["/api-doc1.json", "/api-doc2.json"]));

    #[derive(OpenApi)]
    #[openapi(handlers(api1::hello1))]
    struct ApiDoc1;

    #[derive(OpenApi)]
    #[openapi(handlers(api2::hello2))]
    struct ApiDoc2;

    let api_doc1 = warp::path("api-doc1.json")
        .and(warp::get())
        .map(|| warp::reply::json(&ApiDoc1::openapi()));

    let api_doc2 = warp::path("api-doc2.json")
        .and(warp::get())
        .map(|| warp::reply::json(&ApiDoc2::openapi()));

    let swagger_ui = warp::path("swagger-ui")
        .and(warp::get())
        .and(warp::path::full())
        .and(warp::path::tail())
        .and(warp::any().map(move || config.clone()))
        .and_then(serve_swagger);

    let hello1 = warp::path("hello1")
        .and(warp::get())
        .and(warp::path::end())
        .and_then(api1::hello1);

    let hello2 = warp::path("hello2")
        .and(warp::get())
        .and(warp::path::end())
        .and_then(api2::hello2);

    warp::serve(api_doc1.or(api_doc2).or(swagger_ui).or(hello1).or(hello2))
        .run((Ipv4Addr::UNSPECIFIED, 8080))
        .await
}

async fn serve_swagger(
    full_path: FullPath,
    tail: Tail,
    config: Arc<Config<'static>>,
) -> Result<Box<dyn Reply + 'static>, Rejection> {
    if full_path.as_str() == "/swagger-ui" {
        return Ok(Box::new(warp::redirect::found(Uri::from_static("/swagger-ui/"))));
    }

    let path = tail.as_str();
    match utoipa_swagger_ui::serve(path, config) {
        Ok(file) => {
            if let Some(file) = file {
                Ok(Box::new(
                    Response::builder()
                        .header("Content-Type", file.content_type)
                        .body(file.bytes),
                ))
            } else {
                Ok(Box::new(StatusCode::NOT_FOUND))
            }
        }
        Err(error) => Ok(Box::new(
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string()),
        )),
    }
}

mod api1 {
    use std::convert::Infallible;

    use warp::{hyper::Response, Reply};

    #[utoipa::path(
        get,
        path = "/hello1",
        responses(
            (status = 200, body = String)
        )
    )]
    pub async fn hello1() -> Result<impl Reply, Infallible> {
        Ok(Response::builder()
            .header("content-type", "text/plain")
            .body("hello 1"))
    }
}

mod api2 {
    use std::convert::Infallible;

    use warp::{hyper::Response, Reply};

    #[utoipa::path(
        get,
        path = "/hello2",
        responses(
            (status = 200, body = String)
        )
    )]
    pub async fn hello2() -> Result<impl Reply, Infallible> {
        Ok(Response::builder()
            .header("content-type", "text/plain")
            .body("hello 2"))
    }
}
