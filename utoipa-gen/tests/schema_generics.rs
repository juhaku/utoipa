use std::borrow::Cow;
use std::marker::PhantomData;

use serde::Serialize;
use utoipa::openapi::{Info, RefOr, Schema};
use utoipa::{schema, OpenApi, PartialSchema, ToSchema};

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
fn schema_with_non_generic_root() {
    #![allow(unused)]

    #[derive(ToSchema)]
    struct Foo<T> {
        bar: Bar<T>,
    }

    #[derive(ToSchema)]
    struct Bar<T> {
        value: T,
    }

    #[derive(ToSchema)]
    struct Top {
        foo1: Foo<String>,
        foo2: Foo<i32>,
    }

    #[derive(OpenApi)]
    #[openapi(components(schemas(Top)))]
    struct ApiDoc;
    let mut api = ApiDoc::openapi();
    api.info = Info::new("title", "version");

    let actual = api.to_pretty_json().expect("schema is JSON serializable");
    println!("{actual}");
    let expected = include_str!("./testdata/schema_non_generic_root_generic_references");

    assert_eq!(actual.trim(), expected.trim())
}

#[test]
fn derive_generic_schema_enum_variants() {
    #![allow(unused)]

    #[derive(ToSchema)]
    pub struct FooStruct<B> {
        pub foo: B,
    }

    #[derive(ToSchema)]
    enum FoosEnum {
        ThingNoAliasOption(FooStruct<Option<i32>>),
        FooEnumThing(#[schema(inline)] FooStruct<Vec<i32>>),
        FooThingOptionVec(#[schema(inline)] FooStruct<Option<Vec<i32>>>),
        FooThingLinkedList(#[schema(inline)] FooStruct<std::collections::LinkedList<i32>>),
        FooThingBTreeMap(#[schema(inline)] FooStruct<std::collections::BTreeMap<String, String>>),
        FooThingHashMap(#[schema(inline)] FooStruct<std::collections::HashMap<i32, String>>),
        FooThingHashSet(#[schema(inline)] FooStruct<std::collections::HashSet<i32>>),
        FooThingBTreeSet(#[schema(inline)] FooStruct<std::collections::BTreeSet<i32>>),
    }

    let schema = FoosEnum::schema();
    let json = serde_json::to_string_pretty(&schema).expect("Schema is JSON serializable");
    let value = json.trim();

    #[derive(OpenApi)]
    #[openapi(components(schemas(FoosEnum)))]
    struct Api;

    let mut api = Api::openapi();
    api.info = Info::new("title", "version");
    let api_json = api.to_pretty_json().expect("OpenAPI is JSON serializable");
    println!("{api_json}");
    let expected = include_str!("./testdata/schema_generic_enum_variant_with_generic_type");
    assert_eq!(expected.trim(), api_json.trim());
}

#[test]
fn derive_generic_schema_collect_recursive_schema_not_inlined() {
    #![allow(unused)]

    #[derive(ToSchema)]
    pub struct FooStruct<B> {
        pub foo: B,
    }

    #[derive(ToSchema)]
    pub struct Value(String);

    #[derive(ToSchema)]
    pub struct Person<T> {
        name: String,
        account: Account,
        t: T,
    }

    #[derive(ToSchema)]
    pub struct Account {
        name: String,
    }

    #[derive(ToSchema, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Ty<T> {
        t: T,
    }

    #[derive(ToSchema, PartialEq, Eq, PartialOrd, Ord, Hash)]
    enum Ky {
        One,
        Two,
    }

    #[derive(ToSchema)]
    enum FoosEnum {
        LinkedList(std::collections::LinkedList<Person<Value>>),
        BTreeMap(FooStruct<std::collections::BTreeMap<String, Person<Value>>>),
        HashMap(FooStruct<std::collections::HashMap<i32, Person<i64>>>),
        HashSet(FooStruct<std::collections::HashSet<i32>>),
        Btre(FooStruct<std::collections::BTreeMap<Ty<Ky>, Person<Value>>>),
    }
    let schema = FoosEnum::schema();
    let json = serde_json::to_string_pretty(&schema).expect("Schema is JSON serializable");
    let value = json.trim();

    #[derive(OpenApi)]
    #[openapi(components(schemas(FoosEnum)))]
    struct Api;

    let mut api = Api::openapi();
    api.info = Info::new("title", "version");
    let api_json = api.to_pretty_json().expect("OpenAPI is JSON serializable");
    println!("{api_json}");
    let expected = include_str!("./testdata/schema_generic_collect_non_inlined_schema");
    assert_eq!(expected.trim(), api_json.trim());
}

#[test]
fn high_order_types() {
    #![allow(unused)]

    #[derive(ToSchema)]
    pub struct High<T> {
        high: T,
    }

    #[derive(ToSchema)]
    pub struct HighBox {
        value: High<Box<i32>>,
    }

    #[derive(ToSchema)]
    pub struct HighCow(High<Cow<'static, i32>>);

    #[derive(ToSchema)]
    pub struct HighRefCell(High<std::cell::RefCell<i32>>);

    #[derive(OpenApi)]
    #[openapi(components(schemas(HighBox, HighCow, HighRefCell)))]
    struct Api;

    let mut api = Api::openapi();
    api.info = Info::new("title", "version");
    let api_json = api.to_pretty_json().expect("OpenAPI is JSON serializable");
    println!("{api_json}");
    let expected = include_str!("./testdata/schema_high_order_types");
    assert_eq!(expected.trim(), api_json.trim());
}

#[test]
#[cfg(feature = "rc_schema")]
fn rc_schema_high_order_types() {
    #![allow(unused)]

    #[derive(ToSchema)]
    pub struct High<T> {
        high: T,
    }

    #[derive(ToSchema)]
    pub struct HighArc(High<std::sync::Arc<i32>>);

    #[derive(ToSchema)]
    pub struct HighRc(High<std::rc::Rc<i32>>);

    #[derive(OpenApi)]
    #[openapi(components(schemas(HighArc, HighRc)))]
    struct Api;

    let mut api = Api::openapi();
    api.info = Info::new("title", "version");
    let api_json = api.to_pretty_json().expect("OpenAPI is JSON serializable");
    println!("{api_json}");

    let expected = include_str!("./testdata/rc_schema_high_order_types");
    assert_eq!(expected.trim(), api_json.trim());
}

#[test]
#[cfg(feature = "uuid")]
fn uuid_type_generic_argument() {
    #![allow(unused)]

    #[derive(ToSchema)]
    pub struct High<T> {
        high: T,
    }

    #[derive(ToSchema)]
    pub struct HighUuid(High<Option<uuid::Uuid>>);

    #[derive(OpenApi)]
    #[openapi(components(schemas(HighUuid)))]
    struct Api;

    let mut api = Api::openapi();
    api.info = Info::new("title", "version");
    let api_json = api.to_pretty_json().expect("OpenAPI is JSON serializable");
    println!("{api_json}");

    let expected = include_str!("./testdata/uuid_type_generic_argument");
    assert_eq!(expected.trim(), api_json.trim());
}

#[test]
#[ignore = "arrays, slices, tuples as generic argument is not supported at the moment"]
fn slice_generic_args() {
    #![allow(unused)]

    #[derive(ToSchema)]
    pub struct High<T> {
        high: T,
    }

    // // #[derive(ToSchema)]
    // pub struct HighSlice(High<&'static [i32]>);
    //
    // #[derive(OpenApi)]
    // // #[openapi(components(schemas(HighSlice)))]
    // struct Api;
    //
    // let mut api = Api::openapi();
    // api.info = Info::new("title", "version");
    // let api_json = api.to_pretty_json().expect("OpenAPI is JSON serializable");
    // println!("{api_json}");
    //
    // let expected = include_str!("./testdata/rc_schema_high_order_types");
    // assert_eq!(expected.trim(), api_json.trim());
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
