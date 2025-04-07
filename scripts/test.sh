#!/bin/bash
# shellcheck disable=SC2086

set -e

: "${CARGO:=cargo}"
: "${CARGO_COMMAND:=test}"

crates="${1:-utoipa utoipa-gen utoipa-swagger-ui utoipa-redoc utoipa-rapidoc utoipa-scalar utoipa-axum utoipa-config utoipa-actix-web}"

for crate in $crates; do
    echo "Testing crate: $crate..."

    # Always test the crate itself first, without any features added.
    if [[ "$crate" != "utoipa-gen" ]]; then
        $CARGO ${CARGO_COMMAND} -p $crate
    fi

    if [[ "$crate" == "utoipa" ]]; then
        $CARGO ${CARGO_COMMAND} -p utoipa --features openapi_extensions,preserve_order,preserve_path_order,debug,macros
    elif [[ "$crate" == "utoipa-gen" ]]; then
        $CARGO ${CARGO_COMMAND} -p utoipa-gen --features utoipa/actix_extras,chrono,decimal,utoipa/uuid,uuid,utoipa/ulid,ulid,utoipa/url,url,utoipa/time,time,jiff_0_2,utoipa/repr,utoipa/smallvec,smallvec,rc_schema,utoipa/rc_schema,utoipa/macros
        $CARGO ${CARGO_COMMAND} -p utoipa-gen --test schema_derive_test --features decimal_float,utoipa/macros

        $CARGO ${CARGO_COMMAND} -p utoipa-gen --test path_derive_auto_into_responses --features auto_into_responses,utoipa/uuid,uuid,utoipa/macros
        $CARGO ${CARGO_COMMAND} -p utoipa-gen --test path_derive_actix --test path_parameter_derive_actix --features actix_extras,utoipa/uuid,uuid,utoipa/chrono,chrono,utoipa/time,time,utoipa/macros
        $CARGO ${CARGO_COMMAND} -p utoipa-gen --test path_derive_auto_into_responses_actix --features actix_extras,auto_into_responses,utoipa/uuid,uuid,utoipa/macros

        $CARGO ${CARGO_COMMAND} -p utoipa-gen --test path_derive_rocket --features rocket_extras,utoipa/macros

        $CARGO ${CARGO_COMMAND} -p utoipa-gen --test path_derive_axum_test --features axum_extras,utoipa/macros
        $CARGO ${CARGO_COMMAND} -p utoipa-gen --test path_derive_auto_into_responses_axum --features axum_extras,auto_into_responses,utoipa/macros
    elif [[ "$crate" == "utoipa-swagger-ui" ]]; then
        $CARGO ${CARGO_COMMAND} -p utoipa-swagger-ui --features actix-web,rocket,axum,utoipa/macros
    elif [[ "$crate" == "utoipa-redoc" ]]; then
        $CARGO ${CARGO_COMMAND} -p utoipa-redoc --features actix-web,rocket,axum,utoipa/macros
    elif [[ "$crate" == "utoipa-rapidoc" ]]; then
        $CARGO ${CARGO_COMMAND} -p utoipa-rapidoc --features actix-web,rocket,axum,utoipa/macros
    elif [[ "$crate" == "utoipa-scalar" ]]; then
        $CARGO ${CARGO_COMMAND} -p utoipa-scalar --features actix-web,rocket,axum,utoipa/macros
    elif [[ "$crate" == "utoipa-axum" ]]; then
        $CARGO ${CARGO_COMMAND} -p utoipa-axum --features debug,utoipa/debug,utoipa/macros
    elif [[ "$crate" == "utoipa-config" ]]; then
        pushd utoipa-config/config-test-crate/
        $CARGO ${CARGO_COMMAND}
        popd
    elif [[ "$crate" == "utoipa-actix-web" ]]; then
        $CARGO ${CARGO_COMMAND} -p utoipa-actix-web
    fi
done
