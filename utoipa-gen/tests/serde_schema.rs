#![cfg(all(feature = "yaml", feature = "json"))]

use std::assert_eq;
use serde_json;
use serde_yaml;

use utoipa::openapi::{*, Schema, ObjectBuilder};
use utoipa::openapi::schema::RefOr;

#[test]
pub fn serialize_deserialize_schema() {

	let ref_or_schema = RefOr::T(
		Schema::Object(
			ObjectBuilder::new()
				.property("test", RefOr::T(Schema::Array(
				ArrayBuilder::new()
					.items(RefOr::T(
						Schema::Object(
							ObjectBuilder::new()
								.property("element", RefOr::Ref(Ref::new("#/test")))
								.build()
						)
					))
					.build()
			)))

				.build()
		));

	let yaml_str = serde_yaml::to_string(&ref_or_schema).expect("");
	println!("----------------------------");
	println!("{yaml_str}");

	let deserialized: RefOr<Schema> = serde_yaml::from_str(&yaml_str).expect("");

	let yaml_de_str = serde_yaml::to_string(&deserialized).expect("");
	println!("----------------------------");
	println!("{yaml_de_str}");

	assert_eq!(yaml_str, yaml_de_str);
}
