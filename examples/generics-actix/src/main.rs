use std::{error::Error, net::Ipv4Addr};

use actix_web::{
    get,
    middleware::Logger,
    web::{Json, Query},
    App, HttpServer, Responder, Result,
};
use serde::{Deserialize, Serialize};
use utoipa::{
    openapi::schema::{Object, ObjectBuilder},
    IntoParams, OpenApi, PartialSchema, ToSchema,
};
use utoipa_swagger_ui::SwaggerUi;

fn get_coord_schema<T: PartialSchema>() -> Object {
    ObjectBuilder::new()
        .property("x", T::schema())
        .required("x")
        .property("y", T::schema())
        .required("y")
        .description(Some("this is the coord description"))
        .build()
}

#[derive(Serialize, ToSchema)]
pub struct MyObject<T: geo_types::CoordNum + PartialSchema> {
    #[schema(schema_with=get_coord_schema::<T>)]
    at: geo_types::Coord<T>,
}

// FloatParams and IntegerParams cant be merged using generics because
// IntoParams does not support it, and does not support `schema_with` either.
#[derive(Deserialize, Debug, IntoParams)]
pub struct FloatParams {
    /// x value
    x: f64,
    /// y value
    y: f64,
}

#[utoipa::path(
    params(
        FloatParams
    ),
    responses(
        (status = 200, description = "OK", body = MyObject<f64>),
    ),
    security(
        ("api_key" = [])
    ),
)]
#[get("/coord_f64")]
pub async fn coord_f64(params: Query<FloatParams>) -> Result<impl Responder> {
    let params: FloatParams = params.into_inner();
    let coord = geo_types::Coord::<f64> {
        x: params.x,
        y: params.y,
    };
    eprintln!("response = {:?}", coord);
    Ok(Json(coord))
}

#[derive(Deserialize, Debug, IntoParams)]
pub struct IntegerParams {
    /// x value
    x: u64,
    /// y value
    y: u64,
}

#[utoipa::path(
    params(
        IntegerParams,
    ),
    responses(
        (status = 200, description = "OK", body = MyObject<u64>),
    ),
    security(
        ("api_key" = [])
    ),
)]
#[get("/coord_u64")]
pub async fn coord_u64(params: Query<IntegerParams>) -> Result<impl Responder> {
    let params: IntegerParams = params.into_inner();
    let coord = geo_types::Coord::<u64> {
        x: params.x,
        y: params.y,
    };
    eprintln!("response = {:?}", coord);
    Ok(Json(coord))
}

#[actix_web::main]
async fn main() -> Result<(), impl Error> {
    env_logger::init();

    #[derive(OpenApi)]
    #[openapi(
        paths(coord_f64, coord_u64),
        components(schemas(MyObject<f64>, MyObject<u64>))
    )]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(coord_f64)
            .service(coord_u64)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await
}
