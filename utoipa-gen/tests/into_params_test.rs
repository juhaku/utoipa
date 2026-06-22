#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use utoipa::IntoParams;
use utoipa::openapi::{ObjectBuilder, Type};
use utoipa_gen::ToSchema;



#[derive(Deserialize, Serialize, ToSchema,IntoParams)]
struct TestB {
    id2: u64,
    name2: String,
    age2: Option<i32>,
}

#[derive(Deserialize, Serialize, ToSchema,IntoParams)]
#[into_params(parameter_in = Query)]
struct Pet {
    id1: u64,
    name1: String,
    age1: Option<i32>,
    #[serde(flatten)]
    test2:TestB
}
#[derive(Deserialize, Serialize, ToSchema,IntoParams)]
#[into_params(parameter_in = Query)]
struct Pet2 {
    id1: u64,
    name1: String,
    age1: Option<i32>,
    #[serde(flatten)]
    #[param(schema_with = schema_with_test1)]
    test2:TestB
}


#[derive(Deserialize, Serialize, ToSchema)]
struct Pet3{
    id1: u64,
    name1: String,
    age1: Option<i32>,
    #[serde(flatten)]
    test2: TestB
}




pub fn schema_with_test1(parameter_in_provider: impl Fn() -> Option<utoipa::openapi::path::ParameterIn>) -> Vec<utoipa::openapi::path::Parameter>{
    vec![
        utoipa::openapi::path::ParameterBuilder::new()
            .name("bbb")
            .schema(ObjectBuilder::new().schema_type(Type::String).into())
            .parameter_in(parameter_in_provider().unwrap_or_default())
            .build()
    ]
}




impl utoipa::IntoParams for Pet3 {
    fn into_params(
        parameter_in_provider: impl Fn() -> Option<utoipa::openapi::path::ParameterIn>,
    ) -> Vec<utoipa::openapi::path::Parameter> {
        let mut params: Vec<utoipa::openapi::path::Parameter> = [
            Some(
                utoipa::openapi::path::ParameterBuilder::new()
                    .name("id1")
                    .parameter_in(parameter_in_provider().unwrap_or_default())
                    .required(utoipa::openapi::Required::True)
                    .schema(
                        Some(
                            utoipa::openapi::ObjectBuilder::new()
                                .schema_type(
                                    utoipa::openapi::schema::SchemaType::new(
                                        utoipa::openapi::schema::Type::Integer,
                                    ),
                                )
                                .format(
                                    Some(
                                        utoipa::openapi::schema::SchemaFormat::KnownFormat(
                                            utoipa::openapi::schema::KnownFormat::Int64,
                                        ),
                                    ),
                                )
                                .minimum(Some(0f64)),
                        ),
                    )
                    .build(),
            ),
            Some(
                utoipa::openapi::path::ParameterBuilder::new()
                    .name("name1")
                    .parameter_in(parameter_in_provider().unwrap_or_default())
                    .required(utoipa::openapi::Required::True)
                    .schema(
                        Some(
                            utoipa::openapi::ObjectBuilder::new()
                                .schema_type(
                                    utoipa::openapi::schema::SchemaType::new(
                                        utoipa::openapi::schema::Type::String,
                                    ),
                                ),
                        ),
                    )
                    .build(),
            ),
            Some(
                utoipa::openapi::path::ParameterBuilder::new()
                    .name("age1")
                    .parameter_in(parameter_in_provider().unwrap_or_default())
                    .required(utoipa::openapi::Required::False)
                    .schema(
                        Some(
                            utoipa::openapi::ObjectBuilder::new()
                                .schema_type({
                                    use std::iter::FromIterator;
                                    utoipa::openapi::schema::SchemaType::from_iter([
                                        utoipa::openapi::schema::Type::Integer,
                                        utoipa::openapi::schema::Type::Null,
                                    ])
                                })
                                .format(
                                    Some(
                                        utoipa::openapi::schema::SchemaFormat::KnownFormat(
                                            utoipa::openapi::schema::KnownFormat::Int32,
                                        ),
                                    ),
                                ),
                        ),
                    )
                    .build(),
            ),
        ]
            .into_iter()
            .filter(Option::is_some)
            .flatten()
            .collect();
        params.extend(schema_with_test1(|| parameter_in_provider()));
        params
    }
}
#[test]
pub fn test(){
    let kk = Pet::into_params(||{None});
    let kk2 = Pet2::into_params(||{None});

    assert_eq!(kk.len(), 6_usize);
    assert_eq!(kk2.len(), 4_usize);


}
