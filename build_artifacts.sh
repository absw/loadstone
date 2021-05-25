#!/bin/bash

# exit on any failure
set -e

# Setup
mkdir -p artifacts/

# Argument handling
if [[ $# == 0 ]]; then
    echo "Missing target argument."
    exit 1
fi

function generate_artifact {
    cargo objcopy --bin $1 --features $2 --release --target thumbv7em-none-eabihf -- -O binary $3
}

# Package self-booting artifacts
generate_artifact loadstone $1 loadstone.bin
generate_artifact demo_app $1 demo_app.bin
mv loadstone.bin artifacts/loadstone.bin
mv demo_app.bin artifacts/demo_app_self_booting.bin

# Package bootable artifacts
LOADSTONE_USE_ALT_MEMORY=1 generate_artifact demo_app $1 demo_app.bin
cp demo_app.bin artifacts/demo_app_unsigned.bin
cp demo_app.bin artifacts/demo_app_golden.bin
cp demo_app.bin artifacts/demo_app_regular.bin
LOADSTONE_USE_ALT_MEMORY=1 generate_artifact demo_app_variant $1 demo_app_variant.bin
cp demo_app_variant.bin artifacts/

pushd tools/signing_tool/ >/dev/null
cargo run --release -- ../../artifacts/demo_app_golden.bin ../../src/devices/assets/test_key -g
cargo run --release -- ../../artifacts/demo_app_regular.bin ../../src/devices/assets/test_key
cargo run --release -- ../../artifacts/demo_app_variant.bin ../../src/devices/assets/test_key
popd >/dev/null

zip -r artifacts.zip artifacts
