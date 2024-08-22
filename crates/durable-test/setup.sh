#!/usr/bin/bash

target="$(cargo metadata --format-version 1 | jq -r .target_directory)"
echo "DURABLE_TEST_BIN_DIR=$target/wasm32-wasip1/wasm" >> "$NEXTEST_ENV"

cargo component build --profile wasm -p durable-test-workflows --bins
