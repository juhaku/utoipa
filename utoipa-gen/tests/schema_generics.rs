use std::borrow::Cow;

use utoipa::openapi::{RefOr, Schema};
use utoipa::{schema, OpenApi, ToSchema};

#[test]
fn test_generics() {
    #![allow(unused)]

    #[derive(ToSchema)]
    struct Type<T> {
        t: T,
    }

    #[derive(ToSchema)]
    struct Person<'p, T: Sized, P> {
        id: usize,
        name: Cow<'p, str>,
        field: T,
        t: P,
    }

    #[derive(ToSchema)]
    struct Page<T> {
        total: usize,
        page: usize,
        pages: usize,
        items: Vec<T>,
    }

    #[derive(OpenApi)]
    #[openapi(
        components(
            schemas(
                Person::<'_, String, Type<i32>>,
                Page::<Person<'_, String, Type<i32>>>,
            )
        )
    )]
    struct ApiDoc;

    let schema: RefOr<Schema> = schema!(Page<Person<'_, String, Type<i32>>>);
    // let schema: RefOr<Schema> = schema!(Person<'_, String>);
    // let schema: RefOr<Schema> = schema!(Vec<Person<'_, String>>);
    println!(
        "{}",
        serde_json::to_string_pretty(&schema).expect("schema is JSON serializable")
    );
    dbg!("output schema", &schema);

    println!(
        "{}",
        ApiDoc::openapi()
            .to_pretty_json()
            .expect("ApiDoc is JSON serializable")
    );
}
