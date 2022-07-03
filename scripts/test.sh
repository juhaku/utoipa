#!/bin/bash

crate="$1"

echo "Testing crate: $crate..."

if [[ "$crate" == "utoipa" ]]; then
  cargo test --features uuid
  cargo test --test path_response_derive_test_no_serde_json --no-default-features
  cargo test --test component_derive_no_serde_json --no-default-features
  cargo test --test path_derive_actix --test path_parameter_derive_actix --features actix_extras
  cargo test --test component_derive_test --features chrono,decimal,uuid
  cargo test --test component_derive_test --features chrono_with_format
  cargo test --test path_derive_rocket --features rocket_extras,json
  elif [[ "$crate" == "utoipa-gen" ]]; then
  cargo test -p utoipa-gen --features utoipa/actix_extras
  elif [[ "$crate" == "utoipa-swagger-ui" ]]; then
  cargo test -p utoipa-swagger-ui --features actix-web,rocket,axum
fi
