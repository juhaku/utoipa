use std::borrow::Cow;
use std::marker::PhantomData;

use utoipa::openapi::{Info, RefOr, Schema};
use utoipa::openapi::{RefOr, Schema};
use serde::Serialize;
use utoipa::openapi::{RefOr, Schema};
use utoipa::{schema, OpenApi, ToSchema};

#[test]
fn generic_schema_custom_bound() {
    #![allow(unused)]

    #[derive(Serialize, ToSchema)]
    #[schema(bound = "T: Clone + Sized, T: Sized")]
    struct Type<T> {
        #[serde(skip)]
        t: PhantomData<T>,
    }

    #[derive(Clone)]
    struct NoToSchema;
    fn assert_is_to_schema<T: ToSchema>() {}

    assert_is_to_schema::<Type<NoToSchema>>();
}

#[test]
fn generic_schema_full_api() {
    #![allow(unused)]

    #[derive(ToSchema)]
    #[schema(as = path::MyType<T>)]
    struct Type<T> {
        t: T,
    }

    #[derive(ToSchema)]
    struct Person<'p, T: Sized, P> {
        id: usize,
        name: Option<Cow<'p, str>>,
        field: T,
        t: P,
    }

    #[derive(ToSchema)]
    #[schema(as = path::to::PageList)]
    struct Page<T> {
        total: usize,
        page: usize,
        pages: usize,
        items: Vec<T>,
    }

    #[derive(ToSchema)]
    #[schema(as = path::to::Element<T>)]
    enum E<T> {
        One(T),
        Many(Vec<T>),
    }

    struct NoToSchema;
    fn assert_no_need_to_schema_outside_api(_: Type<NoToSchema>) {}

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

    let mut doc = ApiDoc::openapi();
    doc.info = Info::new("title", "version");

    let actual = doc.to_pretty_json().expect("OpenApi is JSON serializable");
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
    struct Type<T> {
        t: T,
    }

    #[derive(ToSchema)]
    struct Person<'p, T: Sized, P> {
        id: usize,
        name: Option<Cow<'p, str>>,
        field: T,
        t: P,
    }

    #[derive(ToSchema)]
    #[schema(as = path::to::PageList)]
    struct Page<T> {
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
