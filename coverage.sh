#!/usr/bin/sh

if ! which grcov; then
  echo "Error: grcov not found. Try |cargo install grcov|"
  exit 1
fi

export RUSTFLAGS="-Zprofile -Clink-dead-code -Ccodegen-units=1 -Cinline-threshold=0 -Copt-level=0 -Coverflow-checks=off $RUSTFLAGS"
export CARGO_INCREMENTAL=0
export RUST_BACKTRACE=1
export RUST_MIN_STACK=8388608

rm -rf target/cov

cargo +nightly test

grcov . -s . --binary-path target/debug/ -t html --branch --ignore-not-existing -o target/cov/
