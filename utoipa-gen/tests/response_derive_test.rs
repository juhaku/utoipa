use insta::assert_json_snapshot;
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
    assert_eq!("Person", name);
    assert_json_snapshot!(v);
}

#[test]
fn derive_unnamed_struct_response() {
    #[derive(ToResponse)]
    #[allow(unused)]
    struct Person(Vec<String>);

    let (name, v) = <Person as utoipa::ToResponse>::response();
    assert_eq!("Person", name);
    assert_json_snapshot!(v);
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
    assert_eq!("PersonType", name);
    assert_json_snapshot!(v);
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
    assert_eq!("Person", name);
    assert_json_snapshot!(v);
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
    assert_eq!("Person", name);
    assert_json_snapshot!(v);
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
    assert_eq!("Person", name);
    assert_json_snapshot!(v);
}

#[test]
fn derive_response_with_enum_contents() {
    #[derive(utoipa::ToSchema)]
    #[allow(unused)]
    struct Admin {
        name: String,
    }
    #[allow(unused)]
    #[derive(utoipa::ToSchema)]
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
    assert_eq!("Person", name);
    assert_json_snapshot!(v);
}

#[test]
fn derive_response_with_enum_contents_inlined() {
    #[allow(unused)]
    #[derive(ToSchema)]
    struct Admin {
        name: String,
    }

    #[derive(ToSchema)]
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
        Admin(
            #[content("application/json/1")]
            #[to_schema]
            Admin,
        ),
        #[response(example = json!({"name": "name3"}))]
        Moderator(
            #[content("application/json/2")]
            #[to_schema]
            Moderator,
        ),
    }
    let (name, v) = <Person as utoipa::ToResponse>::response();
    assert_eq!("Person", name);
    assert_json_snapshot!(v);
}

#[test]
fn derive_response_with_unit_type() {
    #[derive(ToSchema, ToResponse)]
    #[allow(unused)]
    struct PersonSuccessResponse;

    let (name, v) = <PersonSuccessResponse as utoipa::ToResponse>::response();
    assert_eq!("PersonSuccessResponse", name);
    assert_json_snapshot!(v);
}

#[test]
fn derive_response_with_inline_unnamed_schema() {
    #[allow(unused)]
    #[derive(ToSchema)]
    struct Person {
        name: String,
    }
    #[derive(ToResponse)]
    #[allow(unused)]
    struct PersonSuccessResponse(#[to_schema] Vec<Person>);

    let (name, v) = <PersonSuccessResponse as utoipa::ToResponse>::response();
    assert_eq!("PersonSuccessResponse", name);
    assert_json_snapshot!(v);
}

macro_rules! into_responses {
    ( $(#[$meta:meta])* $key:ident $ident:ident $($tt:tt)* ) => {
        {
            #[derive(utoipa::IntoResponses)]
            $(#[$meta])*
            #[allow(unused)]
            $key $ident $( $tt )*

            let responses = <$ident as utoipa::IntoResponses>::responses();
            serde_json::to_value(responses).unwrap()
        }
    };
}

#[test]
fn derive_into_responses_inline_named_struct_response() {
    let responses = into_responses! {
        /// This is success response
        #[response(status = 200)]
        struct SuccessResponse {
            value: String,
        }
    };

    assert_json_snapshot!(responses);
}

#[test]
fn derive_into_responses_unit_struct() {
    let responses = into_responses! {
        /// Not found response
        #[response(status = NOT_FOUND)]
        struct NotFound;
    };

    assert_json_snapshot!(responses);
}

#[test]
fn derive_into_responses_unnamed_struct_inline_schema() {
    #[derive(utoipa::ToSchema)]
    #[allow(unused)]
    struct Foo {
        bar: String,
    }

    let responses = into_responses! {
        #[response(status = 201)]
        struct CreatedResponse(#[to_schema] Foo);
    };

    assert_json_snapshot!(responses);
}

#[test]
fn derive_into_responses_unnamed_struct_with_primitive_schema() {
    let responses = into_responses! {
        #[response(status = 201)]
        struct CreatedResponse(String);
    };

    assert_json_snapshot!(responses);
}

#[test]
fn derive_into_responses_unnamed_struct_ref_schema() {
    #[derive(utoipa::ToSchema)]
    #[allow(unused)]
    struct Foo {
        bar: String,
    }

    let responses = into_responses! {
        #[response(status = 201)]
        struct CreatedResponse(Foo);
    };

    assert_json_snapshot!(responses);
}

#[test]
fn derive_into_responses_unnamed_struct_ref_response() {
    #[derive(utoipa::ToResponse)]
    #[allow(unused)]
    struct Foo {
        bar: String,
    }

    let responses = into_responses! {
        #[response(status = 201)]
        struct CreatedResponse(#[ref_response] Foo);
    };

    assert_json_snapshot!(responses);
}

#[test]
fn derive_into_responses_unnamed_struct_to_response() {
    #[derive(utoipa::ToResponse)]
    #[allow(unused)]
    struct Foo {
        bar: String,
    }

    let responses = into_responses! {
        #[response(status = 201)]
        struct CreatedResponse(#[to_response] Foo);
    };

    assert_json_snapshot!(responses);
}

#[test]
fn derive_into_responses_enum_with_multiple_responses() {
    #[derive(utoipa::ToSchema)]
    #[allow(unused)]
    struct BadRequest {
        value: String,
    }

    #[derive(utoipa::ToResponse)]
    #[allow(unused)]
    struct Response {
        message: String,
    }

    let responses = into_responses! {
        enum UserResponses {
            /// Success response
            #[response(status = 200)]
            Success { value: String },

            #[response(status = 404)]
            NotFound,

            #[response(status = 400)]
            BadRequest(BadRequest),

            #[response(status = 500)]
            ServerError(#[ref_response] Response),

            #[response(status = 418)]
            TeaPot(#[to_response] Response),
        }
    };

    assert_json_snapshot!(responses);
}
