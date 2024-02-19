#!/bin/bash

set -e

: "${CARGO:=cargo}"

# Finds examples in the `./examples` directory. This query will also automatically 
# ignore directories that are not Rust projects (i.e. those that don't contain Cargo.toml).
EXAMPLES=$(find ./examples/ -maxdepth 1 -mindepth 1 -type d -exec test -e "{}/Cargo.toml" ";" -print)


for example in $EXAMPLES
do
    echo "Checking example: $example..."
    
    pushd $example

    $CARGO fmt --check
    echo "  -> example is properly formatted (passes cargo fmt)"
    $CARGO clippy --all-features --all-targets --workspace --quiet
    echo "  -> example compiles (passes cargo clippy)"

    popd
done

echo "All examples are valid!"
