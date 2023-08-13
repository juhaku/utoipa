#!/bin/bash

# Generate utoipa workspace docs

cargo +nightly doc -Z unstable-options --workspace --no-deps \
    --features actix_extras,openapi_extensions,yaml,uuid,ulid,actix-web,axum,rocket \
    --config 'build.rustdocflags = ["--cfg", "doc_cfg"]'
