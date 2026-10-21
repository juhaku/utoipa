use std::borrow::Cow;
use std::marker::PhantomData;

use insta::assert_json_snapshot;
use serde::Serialize;
use serde_json::Value;
use utoipa::openapi::{Info, RefOr, Schema};
use utoipa::{schema, OpenApi, PartialSchema, ToSchema};

#[test]
fn generic_schema_custom_bound() {
    #![allow(unused)]

    mod foreign {
        pub trait ToSchema {}
        impl ToSchema for () {}
    }

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

    #[derive(ToSchema)]
    #[schema(bound = "T: foreign::ToSchema")]
    struct Marker<T> {
        #[serde(skip)]
        value: PhantomData<T>,
    }

    impl foreign::ToSchema for NoToSchema {}

    assert_is_to_schema::<Marker<NoToSchema>>();
}

#[test]
fn generic_request_body_schema() {
    #![allow(unused)]

    #[derive(ToSchema)]
    #[schema(as = path::MyType<T>)]
    struct Type<T> {
        #[schema(inline)]
        t: T,
    }

    #[derive(ToSchema)]
    struct Person<T: Sized, P> {
        field: T,
        #[schema(inline)]
        t: P,
    }

    #[utoipa::path(
        get,
        path = "/handler",
        request_body = inline(Person<String, Type<i32>>),
    )]
    async fn handler() {}

    #[derive(OpenApi)]
    #[openapi(
        components(
            schemas(
                Person::<String, Type<i32>>,
            )
        ),
        paths(
            handler
        )
    )]
    struct ApiDoc;

    let mut doc = ApiDoc::openapi();
    doc.info = Info::new("title", "version");

    assert_json_snapshot!(doc);
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

    assert_json_snapshot!(doc);
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
        #[schema(inline)]
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

    assert_json_snapshot!(api);
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

    assert_json_snapshot!(api);
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

    assert_json_snapshot!(api);
}

#[test]
fn high_order_types() {
    #![allow(unused)]

    #[derive(ToSchema)]
    pub struct High<T> {
        #[schema(inline)]
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

    assert_json_snapshot!(api);
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

    assert_json_snapshot!(api);
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

    assert_json_snapshot!(api);
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

#[test]
fn generic_recursive_path_components_integration() {
    #![allow(unused)]

    #[derive(ToSchema)]
    struct Category {
        name: String,
    }

    #[derive(ToSchema)]
    struct Product {
        sku: String,
    }

    #[derive(ToSchema)]
    struct Metadata {
        source: String,
    }

    #[derive(ToSchema)]
    struct PlainTreeNode<T> {
        value: T,
        metadata: Metadata,
        #[schema(no_recursion)]
        children: Vec<PlainTreeNode<T>>,
    }

    #[derive(ToSchema)]
    struct AppResponse<T> {
        data: T,
    }

    #[derive(ToSchema)]
    struct Wrapper<T> {
        inner: T,
    }

    #[derive(ToSchema)]
    #[schema(bound = "T: utoipa::ToSchema")]
    struct BoundNode<T> {
        value: T,
        #[schema(no_recursion)]
        children: Vec<BoundNode<T>>,
    }

    #[derive(ToSchema)]
    struct SoloNode<T> {
        value: T,
        #[schema(no_recursion)]
        children: Vec<SoloNode<T>>,
    }

    #[derive(ToSchema)]
    struct Envelope<T> {
        inner: T,
    }

    #[derive(ToSchema)]
    struct TwinNode<T> {
        value: T,
        #[schema(no_recursion)]
        children: Vec<TwinNode<T>>,
    }

    struct Manual<T> {
        value: T,
    }

    impl<T: ToSchema> utoipa::__dev::ComposeSchema for Manual<T> {
        fn compose(mut generics: Vec<RefOr<Schema>>) -> RefOr<Schema> {
            let value = generics.pop().expect("Manual has one generic argument");
            utoipa::openapi::schema::ObjectBuilder::new()
                .property("value", value)
                .into()
        }
    }

    impl<T: ToSchema> ToSchema for Manual<T> {
        fn name() -> Cow<'static, str> {
            Cow::Borrowed("Manual")
        }

        fn composed_name() -> Cow<'static, str> {
            Cow::Owned(format!("Manual_{}", <T as ToSchema>::composed_name()))
        }

        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            <T as ToSchema>::schemas(schemas);
        }
    }
    #[derive(Serialize, ToSchema)]
    struct Ignored<T> {
        #[serde(skip)]
        marker: PhantomData<T>,
    }

    #[utoipa::path(
        get,
        path = "/categories",
        responses((status = 200, body = AppResponse<PlainTreeNode<Category>>)),
    )]
    fn categories() {}

    #[utoipa::path(
        get,
        path = "/products",
        responses((status = 200, body = AppResponse<PlainTreeNode<Product>>)),
    )]
    fn products() {}

    #[utoipa::path(
        get,
        path = "/wrapped",
        responses(
            (status = 200, body = AppResponse<Vec<PlainTreeNode<Category>>>),
            (status = 201, body = AppResponse<Option<PlainTreeNode<Category>>>),
            (status = 202, body = AppResponse<Box<PlainTreeNode<Category>>>)
        )
    )]
    fn wrapped() {}

    #[utoipa::path(
        get,
        path = "/ignored",
        responses((status = 200, body = Ignored<PlainTreeNode<Category>>)),
    )]
    fn ignored() {}

    #[utoipa::path(
        get,
        path = "/argdepth",
        responses((status = 200, body = PlainTreeNode<Wrapper<Category>>)),
    )]
    fn argdepth() {}

    #[utoipa::path(
        get,
        path = "/vecnode",
        responses((status = 200, body = PlainTreeNode<Vec<Category>>)),
    )]
    fn vecnode() {}

    #[utoipa::path(
        get,
        path = "/boundnode",
        responses((status = 200, body = PlainTreeNode<BoundNode<Category>>)),
    )]
    fn boundnode() {}

    #[utoipa::path(
        get,
        path = "/inline",
        responses((status = 200, body = inline(Envelope<TwinNode<Category>>))),
    )]
    fn inline_node() {}

    #[utoipa::path(
        get,
        path = "/manual",
        responses((status = 200, body = PlainTreeNode<Manual<Category>>)),
    )]
    fn manual_node() {}

    #[derive(OpenApi)]
    #[openapi(
        paths(
            categories,
            products,
            wrapped,
            ignored,
            argdepth,
            vecnode,
            boundnode,
            inline_node,
            manual_node
        ),
        components(schemas(Envelope<SoloNode<Category>>))
    )]
    struct ApiDoc;

    let doc = serde_json::to_value(ApiDoc::openapi()).expect("OpenAPI is JSON serializable");
    let schemas = doc["components"]["schemas"]
        .as_object()
        .expect("schemas object");

    fn assert_local_refs(value: &Value, schemas: &serde_json::Map<String, Value>) {
        match value {
            Value::Object(map) => {
                for (key, item) in map {
                    if key == "$ref" {
                        let name = item.as_str().expect("$ref is a string");
                        let name = name
                            .strip_prefix("#/components/schemas/")
                            .expect("local $ref");
                        assert!(schemas.contains_key(name), "unresolved {name}");
                    } else {
                        assert_local_refs(item, schemas);
                    }
                }
            }
            Value::Array(items) => items.iter().for_each(|v| assert_local_refs(v, schemas)),
            _ => (),
        }
    }

    // (1) every local $ref in the whole document resolves, recursively
    assert_local_refs(&doc, schemas);

    // (2) both user generic specializations exist and recurse under their own name
    assert!(schemas.contains_key("PlainTreeNode_Category"));
    assert!(schemas.contains_key("PlainTreeNode_Product"));
    let cat_node = &schemas["PlainTreeNode_Category"];
    let prod_node = &schemas["PlainTreeNode_Product"];
    let cat_children = &cat_node["properties"]["children"]["items"]["$ref"];
    assert_eq!(cat_children, "#/components/schemas/PlainTreeNode_Category");
    let prod_children = &prod_node["properties"]["children"]["items"]["$ref"];
    assert_eq!(prod_children, "#/components/schemas/PlainTreeNode_Product");

    // (3) the two specializations are isolated and carry the Metadata dependency
    let cat_value = cat_node["properties"]["value"]["properties"]
        .as_object()
        .expect("value");
    assert!(cat_value.contains_key("name") && !cat_value.contains_key("sku"));
    let prod_value = prod_node["properties"]["value"]["properties"]
        .as_object()
        .expect("value");
    assert!(prod_value.contains_key("sku") && !prod_value.contains_key("name"));
    assert_eq!(
        cat_node["properties"]["metadata"]["$ref"],
        "#/components/schemas/Metadata"
    );

    // (4) Vec/Option/Box wrapper responses resolve to the canonical node
    let vec_data = &schemas["AppResponse_Vec_PlainTreeNode_Category"]["properties"]["data"];
    assert_eq!(vec_data["type"], "array");
    assert_eq!(&vec_data["items"], cat_node);
    let opt_data = &schemas["AppResponse_Option_PlainTreeNode_Category"]["properties"]["data"];
    let one_of = opt_data["oneOf"].as_array().expect("Option data is oneOf");
    assert!(one_of
        .iter()
        .any(|item| item["type"].as_str() == Some("null")));
    assert!(one_of.iter().any(|item| item == cat_node));
    let box_data = &schemas["AppResponse_Box_PlainTreeNode_Category"]["properties"]["data"];
    assert_eq!(box_data, cat_node);

    // (5) path responses link to components and the skipped-outer case still registers
    let cat_resp = &doc["paths"]["/categories"]["get"]["responses"]["200"]["content"];
    let cat_schema = &cat_resp["application/json"]["schema"];
    assert_eq!(
        cat_schema["$ref"],
        "#/components/schemas/AppResponse_PlainTreeNode_Category"
    );
    let ign_resp = &doc["paths"]["/ignored"]["get"]["responses"]["200"]["content"];
    let ign_schema = &ign_resp["application/json"]["schema"];
    assert_eq!(
        ign_schema["$ref"],
        "#/components/schemas/Ignored_PlainTreeNode_Category"
    );
    assert!(schemas.contains_key("PlainTreeNode_Category") && schemas.contains_key("Metadata"));

    // (6) a node whose type argument is itself generic composes its full self-reference name
    let wrapper_node = &schemas["PlainTreeNode_Wrapper_Category"];
    assert_eq!(
        wrapper_node["properties"]["children"]["items"]["$ref"],
        "#/components/schemas/PlainTreeNode_Wrapper_Category"
    );

    // (7) container and custom-bound generic arguments compose full names in recursion
    let vec_children =
        &schemas["PlainTreeNode_Vec_Category"]["properties"]["children"]["items"]["$ref"];
    assert_eq!(
        vec_children,
        "#/components/schemas/PlainTreeNode_Vec_Category"
    );
    let bound_children =
        &schemas["PlainTreeNode_BoundNode_Category"]["properties"]["children"]["items"]["$ref"];
    assert_eq!(
        bound_children,
        "#/components/schemas/PlainTreeNode_BoundNode_Category"
    );

    // (8) components(schemas(...)) registration also collects intermediate generic schemas
    assert!(schemas.contains_key("Envelope_SoloNode_Category"));
    assert!(schemas.contains_key("SoloNode_Category"));
    let solo_children = &schemas["SoloNode_Category"]["properties"]["children"]["items"]["$ref"];
    assert_eq!(solo_children, "#/components/schemas/SoloNode_Category");

    // (9) an inline body still registers the non-inline schemas it references
    assert!(schemas.contains_key("TwinNode_Category"));
    let twin_children = &schemas["TwinNode_Category"]["properties"]["children"]["items"]["$ref"];
    assert_eq!(twin_children, "#/components/schemas/TwinNode_Category");
    let inline_schema = &doc["paths"]["/inline"]["get"]["responses"]["200"]["content"]
        ["application/json"]["schema"];
    assert!(inline_schema.get("$ref").is_none());

    // (10) a manually implemented generic schema composes names via its own override
    assert!(schemas.contains_key("PlainTreeNode_Manual_Category"));
    let manual_children =
        &schemas["PlainTreeNode_Manual_Category"]["properties"]["children"]["items"]["$ref"];
    assert_eq!(
        manual_children,
        "#/components/schemas/PlainTreeNode_Manual_Category"
    );
}
