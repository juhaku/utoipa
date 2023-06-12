#![cfg(all(
    feature = "auto_types",
    feature = "actix_auto_responses",
    feature = "actix_extras"
))]

use std::fmt::Display;

use actix_web::web::Json;
use actix_web::{get, ResponseError};
use assert_json_diff::assert_json_eq;
use utoipa::OpenApi;
use utoipa_gen::ToSchema;

#[test]
fn path_operation_auto_types_responses() {
    /// Test item to to return
    #[derive(serde::Serialize, serde::Deserialize, ToSchema)]
    struct Item<'s> {
        value: &'s str,
    }

    /// Error
    #[derive(Debug, ToSchema)]
    struct Error;

    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Error")
        }
    }

    impl ResponseError for Error {}

    #[utoipa::path]
    #[get("/item")]
    async fn get_item() -> Result<Json<Item<'static>>, Error> {
        Ok(Json(Item { value: "super" }))
    }

    #[derive(OpenApi)]
    #[openapi(paths(get_item))]
    struct ApiDoc;

    let doc = ApiDoc::openapi();
    let value = serde_json::to_value(&doc).unwrap();
    let path = value.pointer("/paths/~1item/get").unwrap();

    assert_json_eq!(
        &path.pointer("/responses").unwrap(),
        serde_json::json!({
            "200": {
                "content": {
                    "application/json": {
                        "schema": {
                            "$ref": "#/components/schemas/Item"
                        }
                    }
                },
                "description": "",
            },
            "default": {
                "content": {
                    "application/json": {
                        "schema": {
                            "$ref": "#/components/schemas/Error"
                        }
                    },
                },
                "description": ""
            }
        })
    )
}

#[test]
fn path_derive_auto_types_override_responses() {
    /// Test item to to return
    #[derive(serde::Serialize, serde::Deserialize, ToSchema)]
    struct Item<'s> {
        value: &'s str,
    }

    /// Error
    #[derive(Debug, ToSchema)]
    struct Error;

    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Error")
        }
    }

    impl ResponseError for Error {}

    #[utoipa::path(
        responses(
            (status = 201, body = Item, description = "Item Created"),
            (status = NOT_FOUND, body = Error, description = "Not Found"),
            (status = 500, body = Error, description = "Server Error"),
        )
    )]
    #[get("/item")]
    async fn get_item() -> Result<Json<Item<'static>>, Error> {
        Ok(Json(Item { value: "super" }))
    }

    #[derive(OpenApi)]
    #[openapi(paths(get_item))]
    struct ApiDoc;

    let doc = ApiDoc::openapi();
    let value = serde_json::to_value(&doc).unwrap();
    let path = value.pointer("/paths/~1item/get").unwrap();

    let responses = path.pointer("/responses").unwrap();
    assert_json_eq!(
        responses,
        serde_json::json!({
            "200": {
                "content": {
                    "application/json": {
                        "schema": {
                            "$ref": "#/components/schemas/Item"
                        }
                    }
                },
                "description": "",
            },
            "201": {
                "content": {
                    "application/json": {
                        "schema": {
                            "$ref": "#/components/schemas/Item"
                        }
                    }
                },
                "description": "Item Created",
            },
            "404": {
                "content": {
                    "application/json": {
                        "schema": {
                            "$ref": "#/components/schemas/Error"
                        }
                    },
                },
                "description": "Not Found"
            },
            "500": {
                "content": {
                    "application/json": {
                        "schema": {
                            "$ref": "#/components/schemas/Error"
                        }
                    },
                },
                "description": "Server Error"
            },
            "default": {
                "content": {
                    "application/json": {
                        "schema": {
                            "$ref": "#/components/schemas/Error"
                        }
                    },
                },
                "description": ""
            }
        })
    )
}
