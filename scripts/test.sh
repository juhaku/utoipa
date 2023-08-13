#!/bin/bash

set -e

crate="$1"

echo "Testing crate: $crate..."

if [[ "$crate" == "utoipa" ]]; then
    cargo test -p utoipa --features openapi_extensions,preserve_order,preserve_path_order,debug
elif [[ "$crate" == "utoipa-gen" ]]; then
    cargo test -p utoipa-gen --features utoipa/actix_extras,chrono,decimal,utoipa/uuid,uuid,utoipa/ulid,ulid,utoipa/time,time,utoipa/repr,utoipa/smallvec,smallvec,rc_schema,utoipa/rc_schema

    cargo test -p utoipa-gen --test path_derive_auto_into_responses --features auto_into_responses,utoipa/uuid,uuid
    cargo test -p utoipa-gen --test path_derive_actix --test path_parameter_derive_actix --features actix_extras,utoipa/uuid,uuid
    cargo test -p utoipa-gen --test path_derive_auto_into_responses_actix --features actix_extras,utoipa/auto_into_responses,utoipa/uuid,uuid

    cargo test -p utoipa-gen --test path_derive_rocket --features rocket_extras

    cargo test -p utoipa-gen --test path_derive_axum_test --features axum_extras
    cargo test -p utoipa-gen --test path_derive_auto_into_responses_axum --features axum_extras,utoipa/auto_into_responses
elif [[ "$crate" == "utoipa-swagger-ui" ]]; then
    cargo test -p utoipa-swagger-ui --features actix-web,rocket,axum
elif [[ "$crate" == "utoipa-redoc" ]]; then
    cargo test -p utoipa-redoc --features actix-web,rocket,axum
elif [[ "$crate" == "utoipa-rapidoc" ]]; then
    cargo test -p utoipa-rapidoc --features actix-web,rocket,axum
fi
