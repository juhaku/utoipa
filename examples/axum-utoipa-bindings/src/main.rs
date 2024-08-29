use std::io;
use std::net::Ipv4Addr;

use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

const CUSTOMER_TAG: &str = "customer";
const ORDER_TAG: &str = "order";

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = CUSTOMER_TAG, description = "Customer API endpoints"),
        (name = ORDER_TAG, description = "Order API endpoints")
    )
)]
struct ApiDoc;

/// Get health of the API.
#[utoipa::path(
    method(get, head),
    path = "/api/health",
    responses(
        (status = OK, description = "Success", body = str, content_type = "text/plain")
    )
)]
async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        .nest("/api/customer", customer::router())
        .nest("/api/order", order::router())
        .split_for_parts();

    let router = router.merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api));

    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 8080)).await?;
    axum::serve(listener, router).await
}

mod customer {
    use axum::Json;
    use serde::Serialize;
    use utoipa::{OpenApi, ToSchema};
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;

    #[derive(OpenApi)]
    #[openapi(components(schemas(Customer)))]
    struct CustomerApi;

    /// This is the customer
    #[derive(ToSchema, Serialize)]
    struct Customer {
        name: String,
    }

    /// expose the Customer OpenAPI to parent module
    pub fn router() -> OpenApiRouter {
        OpenApiRouter::with_openapi(CustomerApi::openapi()).routes(routes!(get_customer))
    }

    /// Get customer
    ///
    /// Just return a static Customer object
    #[utoipa::path(get, path = "", responses((status = OK, body = Customer)), tag = super::CUSTOMER_TAG)]
    async fn get_customer() -> Json<Customer> {
        Json(Customer {
            name: String::from("Bill Book"),
        })
    }
}

mod order {
    use axum::Json;
    use serde::{Deserialize, Serialize};
    use utoipa::{OpenApi, ToSchema};
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;

    #[derive(OpenApi)]
    #[openapi(components(schemas(Order, OrderRequest)))]
    struct OrderApi;

    /// This is the order
    #[derive(ToSchema, Serialize)]
    struct Order {
        id: i32,
        name: String,
    }

    #[derive(ToSchema, Deserialize, Serialize)]
    struct OrderRequest {
        name: String,
    }

    /// expose the Order OpenAPI to parent module
    pub fn router() -> OpenApiRouter {
        OpenApiRouter::with_openapi(OrderApi::openapi()).routes(routes!(get_order, create_order))
    }

    /// Get static order object
    #[utoipa::path(get, path = "", responses((status = OK, body = Order)), tag = super::ORDER_TAG)]
    async fn get_order() -> Json<Order> {
        Json(Order {
            id: 100,
            name: String::from("Bill Book"),
        })
    }

    /// Create an order.
    ///
    /// Create an order by basically passing through the name of the request with static id.
    #[utoipa::path(post, path = "", responses((status = OK, body = OrderRequest)), tag = super::ORDER_TAG)]
    async fn create_order(Json(order): Json<OrderRequest>) -> Json<Order> {
        Json(Order {
            id: 120,
            name: order.name,
        })
    }
}
