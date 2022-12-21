use assert_json_diff::assert_json_eq;
use serde_json::json;
use utoipa::ToSchema;
use utoipa_gen::ToResponse;

#[test]
fn derive_name_struct_response() {
    #[derive(ToResponse)]
    #[allow(unused)]
    struct Person {
        name: String,
    }
    let (name, v) = <Person as utoipa::ToResponse>::response();
    let value = serde_json::to_value(v).unwrap();

    assert_eq!("Person", name);
    assert_json_eq!(
        value,
        json!({
            "content": {
                "application/json": {
                    "schema": {
                        "$ref": "#/components/schemas/Person"
                    }
                }
            },
            "description": ""
        })
    )
}

#[test]
fn derive_unnamed_struct_response() {
    #[derive(ToResponse)]
    #[allow(unused)]
    struct Person(Vec<String>);

    let (name, v) = <Person as utoipa::ToResponse>::response();
    let value = serde_json::to_value(v).unwrap();

    assert_eq!("Person", name);
    assert_json_eq!(
        value,
        json!({
            "content": {
                "application/json": {
                    "schema": {
                        "items": {
                            "type": "string"
                        },
                        "type": "array"
                    }
                }
            },
            "description": ""
        })
    )
}

#[test]
fn derive_enum_response() {
    #[derive(ToResponse)]
    #[allow(unused)]
    enum PersonType {
        Value(String),
        Foobar,
    }
    let (name, v) = <PersonType as utoipa::ToResponse>::response();
    let value = serde_json::to_value(v).unwrap();

    assert_eq!("PersonType", name);
    assert_json_eq!(
        value,
        json!({
            "content": {
                "application/json": {
                    "schema": {
                        "$ref": "#/components/schemas/PersonType"
                    }
                }
            },
            "description": ""
        })
    )
}

#[test]
fn derive_struct_response_with_description() {
    /// This is description
    ///
    /// It will also be used in `ToSchema` if present
    #[derive(ToResponse)]
    #[allow(unused)]
    struct Person {
        name: String,
    }
    let (name, v) = <Person as utoipa::ToResponse>::response();
    let value = serde_json::to_value(v).unwrap();

    assert_eq!("Person", name);
    assert_json_eq!(
        value,
        json!({
            "content": {
                "application/json": {
                    "schema": {
                        "$ref": "#/components/schemas/Person"
                    }
                }
            },
            "description": "This is description\n\nIt will also be used in `ToSchema` if present"
        })
    )
}

#[test]
fn derive_response_with_attributes() {
    /// This is description
    ///
    /// It will also be used in `ToSchema` if present
    #[derive(ToSchema, ToResponse)]
    #[response(
        description = "Override description for response",
        content_type = "text/xml"
    )]
    #[response(
        example = json!({"name": "the name"}),
        headers(
            ("csrf-token", description = "response csrf token"),
            ("random-id" = i32)
        )
    )]
    #[allow(unused)]
    struct Person {
        name: String,
    }
    let (name, v) = <Person as utoipa::ToResponse>::response();
    let value = serde_json::to_value(v).unwrap();

    assert_eq!("Person", name);
    assert_json_eq!(
        value,
        json!({
            "content": {
                "text/xml": {
                    "example": {
                        "name": "the name"
                    },
                    "schema": {
                        "$ref": "#/components/schemas/Person"
                    }
                }
            },
            "description": "Override description for response",
            "headers": {
                "csrf-token": {
                    "description": "response csrf token",
                    "schema": {
                        "type": "string"
                    }
                },
                "random-id": {
                    "schema": {
                        "type": "integer",
                        "format": "int32"
                    }
                }
            }
        })
    )
}

#[test]
fn derive_response_with_mutliple_content_types() {
    #[derive(ToSchema, ToResponse)]
    #[response(content_type = ["application/json", "text/xml"] )]
    #[allow(unused)]
    struct Person {
        name: String,
    }
    let (name, v) = <Person as utoipa::ToResponse>::response();
    let value = serde_json::to_value(v).unwrap();

    assert_eq!("Person", name);
    assert_json_eq!(
        value,
        json!({
            "content": {
                "application/json": {
                    "schema": {
                        "$ref": "#/components/schemas/Person"
                    }
                },
                "text/xml": {
                    "schema": {
                        "$ref": "#/components/schemas/Person"
                    }
                }
            },
            "description": ""
        })
    )
}

#[test]
fn derive_response_multiple_examples() {
    #[derive(ToSchema, ToResponse)]
    #[response(examples(
            ("Person1" = (value = json!({"name": "name1"}))),
            ("Person2" = (value = json!({"name": "name2"})))
    ))]
    #[allow(unused)]
    struct Person {
        name: String,
    }
    let (name, v) = <Person as utoipa::ToResponse>::response();
    let value = serde_json::to_value(v).unwrap();

    assert_eq!("Person", name);
    assert_json_eq!(
        value,
        json!({
            "content": {
                "application/json": {
                    "examples": {
                        "Person1": {
                            "value": {
                                "name": "name1"
                            }
                        },
                        "Person2": {
                            "value": {
                                "name": "name2"
                            }
                        }
                    },
                    "schema": {
                        "$ref": "#/components/schemas/Person"
                    }
                },
            },
            "description": ""
        })
    )
}

#[test]
fn derive_response_with_enum_contents() {
    #[allow(unused)]
    struct Admin {
        name: String,
    }
    #[allow(unused)]
    struct Moderator {
        name: String,
    }
    #[derive(ToSchema, ToResponse)]
    #[allow(unused)]
    enum Person {
        #[response(examples(
                ("Person1" = (value = json!({"name": "name1"}))),
                ("Person2" = (value = json!({"name": "name2"})))
        ))]
        Admin(#[content("application/json/1")] Admin),
        #[response(example = json!({"name": "name3"}))]
        Moderator(#[content("application/json/2")] Moderator),
    }
    let (name, v) = <Person as utoipa::ToResponse>::response();
    let value = serde_json::to_value(v).unwrap();

    assert_eq!("Person", name);
    assert_json_eq!(
        value,
        json!({
            "content": {
                "application/json/1": {
                    "examples": {
                        "Person1": {
                            "value": {
                                "name": "name1"
                            }
                        },
                        "Person2": {
                            "value": {
                                "name": "name2"
                            }
                        }
                    },
                    "schema": {
                        "$ref": "#/components/schemas/Admin"
                    }
                },
                "application/json/2": {
                    "example": {
                        "name": "name3"
                    },
                    "schema": {
                        "$ref": "#/components/schemas/Moderator"
                    }
                }
            },
            "description": ""
        })
    )
}
