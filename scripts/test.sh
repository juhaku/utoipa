#!/bin/bash

set -e

crate="$1"

echo "Testing crate: $crate..."

if [[ "$crate" == "utoipa" ]]; then
    cargo test -p utoipa --features openapi_extensions
elif [[ "$crate" == "utoipa-gen" ]]; then
    cargo test -p utoipa-gen --features utoipa/actix_extras,chrono,decimal,utoipa/uuid,utoipa/time,time,utoipa/repr

    cargo test -p utoipa-gen --test path_derive_actix --test path_parameter_derive_actix --features actix_extras
    cargo test -p utoipa-gen --test path_derive_rocket --features rocket_extras
    cargo test -p utoipa-gen --test path_derive_axum_test --features axum_extras
elif [[ "$crate" == "utoipa-swagger-ui" ]]; then
    cargo test -p utoipa-swagger-ui --features actix-web,rocket,axum
fi
