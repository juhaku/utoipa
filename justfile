# List available recipes
default:
    @just --list

# Run tests for all crates or a specific subset.
# Usage: just test
#        just test utoipa utoipa-gen
# Env:   CARGO (default: cargo), CARGO_COMMAND (default: test)
# Run tests for all crates or a specific subset Usage: `just test`
test *crates='utoipa utoipa-gen utoipa-swagger-ui utoipa-redoc utoipa-rapidoc utoipa-scalar utoipa-axum utoipa-config utoipa-actix-web':
    #!/usr/bin/env bash
    set -e
    cargo="${CARGO:-cargo}"
    cargo_command="${CARGO_COMMAND:-test}"
    for crate in {{crates}}; do
        echo "Testing crate: $crate..."

        if [[ "$crate" != "utoipa-gen" ]]; then
            $cargo $cargo_command -p $crate
        fi

        if [[ "$crate" == "utoipa" ]]; then
            $cargo $cargo_command -p utoipa --features openapi_extensions,preserve_order,preserve_path_order,debug,macros
        elif [[ "$crate" == "utoipa-gen" ]]; then
            $cargo $cargo_command -p utoipa-gen --features utoipa/actix_extras,chrono,decimal,utoipa/uuid,uuid,utoipa/ulid,ulid,utoipa/url,url,utoipa/time,time,jiff_0_2,utoipa/repr,utoipa/smallvec,smallvec,rc_schema,utoipa/rc_schema,utoipa/macros
            $cargo $cargo_command -p utoipa-gen --test schema_derive_test --features decimal_float,utoipa/macros

            $cargo $cargo_command -p utoipa-gen --test path_derive_auto_into_responses --features auto_into_responses,utoipa/uuid,uuid,utoipa/macros
            $cargo $cargo_command -p utoipa-gen --test path_derive_actix --test path_parameter_derive_actix --features actix_extras,utoipa/uuid,uuid,utoipa/chrono,chrono,utoipa/time,time,utoipa/macros
            $cargo $cargo_command -p utoipa-gen --test path_derive_auto_into_responses_actix --features actix_extras,auto_into_responses,utoipa/uuid,uuid,utoipa/macros

            $cargo $cargo_command -p utoipa-gen --test path_derive_rocket --features rocket_extras,utoipa/macros

            $cargo $cargo_command -p utoipa-gen --test path_derive_axum_test --features axum_extras,utoipa/macros
            $cargo $cargo_command -p utoipa-gen --test path_derive_auto_into_responses_axum --features axum_extras,auto_into_responses,utoipa/macros
        elif [[ "$crate" == "utoipa-swagger-ui" ]]; then
            $cargo $cargo_command -p utoipa-swagger-ui --features actix-web,rocket,axum,utoipa/macros
        elif [[ "$crate" == "utoipa-redoc" ]]; then
            $cargo $cargo_command -p utoipa-redoc --features actix-web,rocket,axum,utoipa/macros
        elif [[ "$crate" == "utoipa-rapidoc" ]]; then
            $cargo $cargo_command -p utoipa-rapidoc --features actix-web,rocket,axum,utoipa/macros
        elif [[ "$crate" == "utoipa-scalar" ]]; then
            $cargo $cargo_command -p utoipa-scalar --features actix-web,rocket,axum,utoipa/macros
        elif [[ "$crate" == "utoipa-axum" ]]; then
            $cargo $cargo_command -p utoipa-axum --features debug,utoipa/debug,utoipa/macros
        elif [[ "$crate" == "utoipa-config" ]]; then
            pushd utoipa-config/config-test-crate/
            $cargo $cargo_command
            popd
        elif [[ "$crate" == "utoipa-actix-web" ]]; then
            $cargo $cargo_command -p utoipa-actix-web
        fi
    done

# Run code coverage using grcov across all crates and feature sets (requires: cargo install grcov, nightly toolchain). Output is written to target/cov/
coverage:
    #!/usr/bin/env bash
    if ! which grcov > /dev/null 2>&1; then
        echo "Error: grcov not found. Try |cargo install grcov|"
        exit 1
    fi

    export RUSTFLAGS="-Zprofile -Clink-dead-code -Ccodegen-units=1 -Cinline-threshold=0 -Copt-level=0 -Coverflow-checks=off ${RUSTFLAGS:-}"
    export CARGO_INCREMENTAL=0
    export RUST_BACKTRACE=1
    export RUST_MIN_STACK=8388608

    rm -rf target/cov

    # Run the full per-crate, per-feature-set test suite under coverage instrumentation.
    # CARGO is set to `cargo +nightly` so every invocation uses the nightly toolchain
    # required by the -Zprofile RUSTFLAG.
    CARGO="cargo +nightly" just test

    grcov . -s . --binary-path target/debug/ -t html --branch --ignore-not-existing -o target/cov/

# Generate workspace documentation (requires nightly toolchain). Output is written to target/doc/
doc:
    cargo +nightly doc -Z unstable-options --workspace --no-deps \
        --features actix_extras,openapi_extensions,yaml,uuid,ulid,url,non_strict_integers,actix-web,axum,rocket,macros,config \
        --config 'build.rustdocflags = ["--cfg", "doc_cfg"]'

# Update vendored Swagger UI to the given version. Usage: `just update-swagger-ui 5.18.2`
update-swagger-ui version:
    #!/usr/bin/env bash
    set -eu -o pipefail
    zip_name="v{{version}}.zip"

    curl -sSL -o "$zip_name" "https://github.com/swagger-api/swagger-ui/archive/refs/tags/v{{version}}.zip"

    sed_flags=(-i)
    if [[ "$(uname)" == "Darwin" ]]; then
        sed_flags+=('')
    fi

    echo "Update vendored Swagger UI"
    mv "$zip_name" ./utoipa-swagger-ui-vendored/res/
    sed "${sed_flags[@]}" "s|version: \`.*\`|version: \`{{version}}\`|" ./utoipa-swagger-ui-vendored/README.md
    sed "${sed_flags[@]}" "s|version: \`.*\`|version: \`{{version}}\`|" ./utoipa-swagger-ui-vendored/src/lib.rs
    sed "${sed_flags[@]}" "s|res/v.*\.zip|res/v{{version}}.zip|" ./utoipa-swagger-ui-vendored/src/lib.rs

    echo "Update utoipa-swagger-ui Swagger UI version"
    sed "${sed_flags[@]}" "s|tags/v.*>|tags/v{{version}}.zip>|" ./utoipa-swagger-ui/README.md
    sed "${sed_flags[@]}" "s|tags/v.*>|tags/v{{version}}.zip>|" ./utoipa-swagger-ui/src/lib.rs
    sed "${sed_flags[@]}" "s|tags/v.*\.zip|tags/v{{version}}.zip|" ./utoipa-swagger-ui/build.rs

# Validate all examples: check formatting and run clippy.
validate-examples:
    #!/usr/bin/env bash
    set -e
    cargo="${CARGO:-cargo}"

    EXAMPLES=$(find ./examples/ -maxdepth 1 -mindepth 1 -type d -exec test -e "{}/Cargo.toml" ";" -print)

    for example in $EXAMPLES; do
        echo "Checking example: $example..."

        pushd $example

        $cargo fmt --check
        echo "  -> example is properly formatted (passes cargo fmt)"
        $cargo clippy --all-features --all-targets --workspace --quiet
        echo "  -> example compiles (passes cargo clippy)"

        popd
    done

    echo "All examples are valid!"
