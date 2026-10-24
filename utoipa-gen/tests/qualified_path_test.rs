//! Field types with a qualified self path (`<Ty as Trait>::Assoc`), see juhaku/utoipa#1558.
#![allow(unused)]

use insta::assert_json_snapshot;
use serde_json::{json, Value};
use utoipa::{IntoParams, IntoResponses, OpenApi, PartialSchema, ToSchema};

trait Resource {
    type Content: ToSchema;
}

trait Keyed {
    type Value: ToSchema;
}

/// A body.
#[derive(ToSchema)]
struct Body {
    a: u32,
}

#[derive(ToSchema)]
struct Res;

impl Resource for Res {
    type Content = Body;
}

impl Keyed for Res {
    type Value = Body;
}

/// Returns the schema of `T` and the names `T::schemas` registers.
fn schema_and_references<T: ToSchema>() -> Value {
    let mut schemas = Vec::new();
    T::schemas(&mut schemas);
    let names = schemas
        .into_iter()
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    json!({ "schema": T::schema(), "references": names })
}

#[test]
fn self_qualified_named_field() {
    trait Dummy {
        type Zing;
    }

    #[derive(ToSchema)]
    struct Foo {
        bar: <Self as Dummy>::Zing,
    }

    impl Dummy for Foo {
        type Zing = String;
    }

    assert_json_snapshot!(schema_and_references::<Foo>());
}

#[test]
fn qualified_newtype_reference() {
    #[derive(ToSchema)]
    struct Qualified(<Res as Resource>::Content);

    let value = schema_and_references::<Qualified>();
    assert_eq!(value["references"], json!(["Body"]));
    assert_json_snapshot!(value);
}

#[test]
fn qualified_newtype_inline() {
    #[derive(ToSchema)]
    struct Qualified(#[schema(inline)] <Res as Resource>::Content);

    let value = schema_and_references::<Qualified>();
    assert_eq!(value["schema"], json!(Body::schema()));
    assert_json_snapshot!(value);
}

#[test]
fn qualified_named_fields() {
    #[derive(ToSchema)]
    struct Named {
        by_ref: <Res as Resource>::Content,
        #[schema(inline)]
        inline: <Res as Resource>::Content,
        list: Vec<<Res as Resource>::Content>,
        optional: Option<<Res as Resource>::Content>,
    }

    let value = schema_and_references::<Named>();
    assert_eq!(
        value["schema"]["properties"]["by_ref"]["$ref"],
        json!("#/components/schemas/Body")
    );
    assert_json_snapshot!(value);
}

#[test]
fn qualified_generic_container_fields() {
    #[derive(ToSchema)]
    struct Generic<T: Resource> {
        list: Vec<<T as Resource>::Content>,
        optional: Option<<T as Resource>::Content>,
    }

    let value = schema_and_references::<Generic<Res>>();
    assert_eq!(
        value["schema"]["properties"]["list"]["items"]["$ref"],
        json!("#/components/schemas/Body")
    );
    assert_json_snapshot!(value);
}

/// A qualified generic argument produces the same schema as the type it resolves to.
#[test]
fn qualified_generic_argument() {
    #[derive(ToSchema)]
    struct Page<T: ToSchema> {
        items: Vec<T>,
    }

    #[derive(ToSchema)]
    struct Paged {
        page: Page<<Res as Resource>::Content>,
        #[schema(inline)]
        inline_page: Page<<Res as Resource>::Content>,
        plain_page: Page<Body>,
        #[schema(inline)]
        plain_inline_page: Page<Body>,
    }

    let value = schema_and_references::<Paged>();
    let properties = &value["schema"]["properties"];
    assert_eq!(
        properties["page"]["$ref"],
        json!("#/components/schemas/Page_Body")
    );
    assert_eq!(properties["page"], properties["plain_page"]);
    assert_eq!(properties["inline_page"], properties["plain_inline_page"]);
    assert_eq!(
        value["references"],
        json!(["Page_Body", "Body", "Page_Body", "Body"])
    );
}

/// An associated type named like a utoipa "virtual" type (`Value`, `Object`) must not be read
/// as that virtual type.
#[test]
fn qualified_associated_type_named_value() {
    #[derive(ToSchema)]
    struct Named {
        value: <Res as Keyed>::Value,
    }

    let value = schema_and_references::<Named>();
    assert_eq!(
        value["schema"]["properties"]["value"]["$ref"],
        json!("#/components/schemas/Body")
    );
}

/// A container generic named like the associated type must not capture the qualified type.
#[test]
fn qualified_associated_type_named_like_generic_param() {
    #[derive(ToSchema)]
    struct Generic<Content: ToSchema> {
        own: Content,
        assoc: <Res as Resource>::Content,
    }

    let value = schema_and_references::<Generic<String>>();
    assert_eq!(
        value["schema"]["properties"]["assoc"]["$ref"],
        json!("#/components/schemas/Body")
    );
}

#[test]
fn qualified_enum_variants() {
    #[derive(ToSchema)]
    enum Qualified {
        Unnamed(<Res as Resource>::Content),
        Named { content: <Res as Resource>::Content },
    }

    let value = schema_and_references::<Qualified>();
    assert_eq!(value["references"], json!(["Body", "Body"]));
    assert_json_snapshot!(value);
}

#[test]
fn qualified_value_type() {
    struct Opaque;

    #[derive(ToSchema)]
    struct Overridden {
        #[schema(value_type = <Res as Resource>::Content)]
        content: Opaque,
        #[schema(value_type = Vec<<Res as Resource>::Content>)]
        list: Vec<Opaque>,
    }

    let value = schema_and_references::<Overridden>();
    assert_eq!(
        value["schema"]["properties"]["content"]["$ref"],
        json!("#/components/schemas/Body")
    );
    assert_eq!(
        value["schema"]["properties"]["list"]["items"]["$ref"],
        json!("#/components/schemas/Body")
    );
}

trait Query {
    type Id: ToSchema;
}

impl Query for Res {
    type Id = i64;
}

type PlainId = i64;

/// A qualified type that resolves to a primitive is opaque to the macro, the same as a plain
/// type alias of that primitive, and so produces the same schema.
#[test]
fn qualified_primitive_matches_plain_alias() {
    #[derive(ToSchema)]
    struct Qualified {
        id: <Res as Query>::Id,
    }

    #[derive(ToSchema)]
    struct Aliased {
        id: PlainId,
    }

    assert_eq!(
        schema_and_references::<Qualified>(),
        schema_and_references::<Aliased>()
    );

    #[derive(IntoParams)]
    struct QualifiedParams {
        id: <Res as Query>::Id,
    }

    #[derive(IntoParams)]
    struct AliasedParams {
        id: PlainId,
    }

    assert_eq!(
        json!(<QualifiedParams as IntoParams>::into_params(|| None)),
        json!(<AliasedParams as IntoParams>::into_params(|| None))
    );
}

#[test]
fn qualified_into_params_field() {
    #[derive(IntoParams)]
    struct Params {
        id: <Res as Query>::Id,
    }

    let parameters = <Params as IntoParams>::into_params(|| None);
    assert_json_snapshot!(parameters);
}

#[test]
fn qualified_into_responses_variant() {
    #[derive(IntoResponses)]
    enum Responses {
        /// Ok.
        #[response(status = 200)]
        Ok(<Res as Resource>::Content),
    }

    assert_json_snapshot!(<Responses as IntoResponses>::responses());
}

#[test]
fn qualified_openapi_components_and_path() {
    #[derive(IntoParams)]
    struct Params {
        id: i64,
    }

    trait HasParams {
        type Params: IntoParams;
    }

    impl HasParams for Res {
        type Params = Params;
    }

    #[utoipa::path(
        post,
        path = "/foo",
        params(<Res as HasParams>::Params),
        request_body = <Res as Resource>::Content,
        responses((status = 200, body = <Res as Resource>::Content)),
    )]
    fn post_foo() {}

    #[derive(OpenApi)]
    #[openapi(paths(post_foo), components(schemas(<Res as Resource>::Content)))]
    struct ApiDoc;

    let value = serde_json::to_value(ApiDoc::openapi()).unwrap();
    assert_json_snapshot!(json!({ "paths": value["paths"], "components": value["components"] }));
}
