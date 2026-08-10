# openapi-3_2-sse

This example demonstrates documenting a Server-Sent Events endpoint using `utoipa`'s
OpenAPI 3.2 support: opting in to `openapi: "3.2.0"` output and describing the
`text/event-stream` response with the new `itemSchema` media type keyword.

Run with:
```bash
cargo run
```

Then either:

- Browse to `http://localhost:8080/swagger-ui/` to inspect the generated OpenAPI 3.2 document, or
- Stream events directly with `curl -N http://localhost:8080/pets/events`
