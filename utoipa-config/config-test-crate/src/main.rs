use utoipa::ToSchema;

#[allow(unused)]
#[derive(ToSchema)]
struct AliasValues {
    name: String,

    #[schema(value_type = MyType)]
    my_type: String,

    #[schema(value_type = MyInt)]
    my_int: String,

    #[schema(value_type = MyValue)]
    my_value: bool,

    date: MyDateTime,
}

#[allow(unused)]
struct MyDateTime {
    millis: usize,
}

fn main() {
    let schema = utoipa::schema!(
        #[inline]
        AliasValues
    );

    println!(
        "{}",
        serde_json::to_string_pretty(&schema).expect("schema must be JSON serializable")
    );
}
