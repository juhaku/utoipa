use std::borrow::Cow;

use utoipa::openapi::{RefOr, Schema};
use utoipa::{schema, OpenApi, ToSchema};

#[test]
fn generic_schema_full_api() {
    #![allow(unused)]

    #[derive(ToSchema)]
    #[schema(as = path::MyType<T>)]
    struct Type<T: ToSchema> {
        t: T,
    }

    #[derive(ToSchema)]
    struct Person<'p, T: Sized + ToSchema, P: ToSchema> {
        id: usize,
        name: Option<Cow<'p, str>>,
        field: T,
        t: P,
    }

    #[derive(ToSchema)]
    #[schema(as = path::to::PageList)]
    struct Page<T: ToSchema> {
        total: usize,
        page: usize,
        pages: usize,
        items: Vec<T>,
    }

    #[derive(ToSchema)]
    #[schema(as = path::to::Element<T>)]
    enum E<T: ToSchema> {
        One(T),
        Many(Vec<T>),
    }

    #[utoipa::path(
        get,
        path = "/handler",
        request_body = inline(Person<'_, String, Type<i32>>),
        responses(
            (status = OK, body = inline(Page<Person<'_, String, Type<i32>>>)),
            (status = 400, body = Page<Person<'_, String, Type<i32>>>)
        )
    )]
    async fn handler() {}

    #[derive(OpenApi)]
    #[openapi(
        components(
            schemas(
                Person::<'_, String, Type<i32>>,
                Page::<Person<'_, String, Type<i32>>>,
                E::<String>,
            )
        ),
        paths(
            handler
        )
    )]
    struct ApiDoc;

    let actual = ApiDoc::openapi()
        .to_pretty_json()
        .expect("ApiDoc is JSON serializable");
    println!("{actual}");

    let expected = include_str!("./testdata/schema_generics_openapi");

    assert_eq!(expected.trim(), actual.trim());
}

#[test]
#[ignore = "For debugging only"]
fn schema_macro_run() {
    #![allow(unused)]

    #[derive(ToSchema)]
    #[schema(as = path::MyType<T>)]
    struct Type<T: ToSchema> {
        t: T,
    }

    #[derive(ToSchema)]
    struct Person<'p, T: Sized + ToSchema, P: ToSchema> {
        id: usize,
        name: Option<Cow<'p, str>>,
        field: T,
        t: P,
    }

    #[derive(ToSchema)]
    #[schema(as = path::to::PageList)]
    struct Page<T: ToSchema> {
        total: usize,
        page: usize,
        pages: usize,
        items: Vec<T>,
    }

    let schema: RefOr<Schema> = schema!(Page<Person<'_, String, Type<i32>>>).into();
    // let schema: RefOr<Schema> = schema!(Person<'_, String, Type<i32>>).into();
    // let schema: RefOr<Schema> = schema!(Vec<Person<'_, String, Type<i32>>>).into();
    println!(
        "{}",
        serde_json::to_string_pretty(&schema).expect("schema is JSON serializable")
    );
}
