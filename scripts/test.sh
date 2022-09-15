#!/bin/bash

set -e

crate="$1"

echo "Testing crate: $crate..."

if [[ "$crate" == "utoipa" ]]; then
    cargo test -p utoipa --features openapi_extensions
elif [[ "$crate" == "utoipa-gen" ]]; then
    cargo test -p utoipa-gen --features utoipa/actix_extras,chrono,decimal,json,utoipa/uuid,utoipa/json,utoipa/time,time,utoipa/repr

    cargo test -p utoipa-gen --test path_response_derive_test_no_serde_json --no-default-features
    cargo test -p utoipa-gen --test schema_derive_no_serde_json --no-default-features

    cargo test -p utoipa-gen --test path_derive_actix --test path_parameter_derive_actix --features actix_extras,json,utoipa/json
    cargo test -p utoipa-gen --test path_derive_rocket --features rocket_extras,json,utoipa/json
    cargo test -p utoipa-gen --test path_derive_axum_test --features axum_extras,json,utoipa/json
elif [[ "$crate" == "utoipa-swagger-ui" ]]; then
    cargo test -p utoipa-swagger-ui --features actix-web,rocket,axum
fi
