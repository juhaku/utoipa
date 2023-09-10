#!/bin/bash

set -e

: "${CARGO:=cargo}"

crates="${1:-utoipa utoipa-gen utoipa-swagger-ui utoipa-redoc utoipa-rapidoc}"

for crate in $crates; do
    echo "Testing crate: $crate..."

    if [[ "$crate" == "utoipa" ]]; then
        $CARGO test -p utoipa --features openapi_extensions,preserve_order,preserve_path_order,debug
    elif [[ "$crate" == "utoipa-gen" ]]; then
        $CARGO test -p utoipa-gen --features utoipa/actix_extras,chrono,decimal,utoipa/uuid,uuid,utoipa/ulid,ulid,utoipa/url,url,utoipa/time,time,utoipa/repr,utoipa/smallvec,smallvec,rc_schema,utoipa/rc_schema
        $CARGO test -p utoipa-gen --test schema_derive_test --features decimal_float

        $CARGO test -p utoipa-gen --test path_derive_auto_into_responses --features auto_into_responses,utoipa/uuid,uuid
        $CARGO test -p utoipa-gen --test path_derive_actix --test path_parameter_derive_actix --features actix_extras,utoipa/uuid,uuid,utoipa/chrono,chrono,utoipa/time,time
        $CARGO test -p utoipa-gen --test path_derive_auto_into_responses_actix --features actix_extras,utoipa/auto_into_responses,utoipa/uuid,uuid

        $CARGO test -p utoipa-gen --test path_derive_rocket --features rocket_extras

        $CARGO test -p utoipa-gen --test path_derive_axum_test --features axum_extras
        $CARGO test -p utoipa-gen --test path_derive_auto_into_responses_axum --features axum_extras,utoipa/auto_into_responses
    elif [[ "$crate" == "utoipa-swagger-ui" ]]; then
        $CARGO test -p utoipa-swagger-ui --features actix-web,rocket,axum
    elif [[ "$crate" == "utoipa-redoc" ]]; then
        $CARGO test -p utoipa-redoc --features actix-web,rocket,axum
    elif [[ "$crate" == "utoipa-rapidoc" ]]; then
        $CARGO test -p utoipa-rapidoc --features actix-web,rocket,axum
    fi
done
