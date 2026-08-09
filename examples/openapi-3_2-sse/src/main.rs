//! Server-Sent Events (SSE) example documented with OpenAPI 3.2.
//!
//! OpenAPI 3.2 adds support for streaming/sequential media types such as `text/event-stream`,
//! `application/jsonl` and `application/json-seq`.
//! Refer to [Announcing OpenAPI v3.2](https://www.openapis.org/blog/2025/09/23/announcing-openapi-v3-2)
//!
//! This example shows how to:
//!
//! - opt in to `utoipa`'s OpenAPI 3.2 output via the `#[openapi(version = "3.2.0")]` derive attribute
//! - describe a `text/event-stream` SSE endpoint using the new `itemSchema` media type keyword
//!   introduced in OpenAPI 3.2, so consumers know the shape of each event pushed down the stream
//! - serve the endpoint for real with axum's [`axum::response::sse::Sse`] type

use std::convert::Infallible;
use std::time::Duration;

use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::Stream;
use rand::Rng;
use serde::Serialize;
use tokio_stream::StreamExt;
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

/// A single pet status update pushed to subscribers of the `/pets/events` stream.
#[derive(Serialize, ToSchema)]
struct PetEvent {
    /// Identifier of the pet the event relates to.
    pet_id: Uuid,
    /// New status of the pet, e.g. `"adopted"`, `"vaccinated"` or `"groomed"`.
    status: String,
}

/// Stream status updates for pets as Server-Sent Events.
///
/// The `text/event-stream` response describes the shape of each streamed event with the
/// `item_schema` attribute, which maps to OpenAPI 3.2's `itemSchema` media type keyword. Because
/// `item_schema = PetEvent` produces a `$ref`, `PetEvent` is registered as a reusable component via
/// `#[openapi(components(schemas(PetEvent)))]` on [`ApiDoc`].
#[utoipa::path(
    get,
    path = "/pets/events",
    responses(
        (
            status = 200,
            description = "Stream of pet status updates",
            content_type = "text/event-stream",
            item_schema = PetEvent,
        )
    )
)]
async fn pet_events() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let uuids = [Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    let statuses = ["vaccinated", "groomed", "adopted"];
    // Emit a finite number of events (one per second) so clients that buffer the whole
    // response body
    let stream = tokio_stream::iter(0u64..10)
        .map(move |i| {
            let mut rng = rand::thread_rng();
            let uuid_idx = rng.gen_range(0..3);
            let event = PetEvent {
                pet_id: uuids[uuid_idx],
                status: statuses[(i % statuses.len() as u64) as usize].to_string(),
            };
            Ok(Event::default().json_data(&event).unwrap())
        })
        .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

/// Opt the whole document in to OpenAPI 3.2 output via `ApiDoc::openapi().openapi(...)`
#[derive(OpenApi)]
#[openapi(components(schemas(PetEvent)))]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let mut api = ApiDoc::openapi();
    api.openapi = utoipa::openapi::OpenApiVersion::Version32;

    let (router, api) = OpenApiRouter::with_openapi(api)
        .routes(routes!(pet_events))
        .split_for_parts();

    let router = router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_response_documents_item_schema() {
        let mut api = ApiDoc::openapi();
        api.openapi = utoipa::openapi::OpenApiVersion::Version32;

        let (_, api) = OpenApiRouter::<()>::with_openapi(api)
            .routes(routes!(pet_events))
            .split_for_parts();

        let json = serde_json::to_value(&api).unwrap();

        assert_eq!(
            json["openapi"],
            serde_json::json!("3.2.0"),
            "expected document to opt in to OpenAPI 3.2, got: {}",
            json["openapi"]
        );

        let content = &json["paths"]["/pets/events"]["get"]["responses"]["200"]["content"]
            ["text/event-stream"];

        assert_eq!(
            content["itemSchema"]["$ref"],
            serde_json::json!("#/components/schemas/PetEvent"),
            "expected text/event-stream response to carry an itemSchema $ref to PetEvent, got: {content:#}"
        );
    }
}
