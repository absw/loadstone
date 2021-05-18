#!/bin/bash

# exit on any failure
set -e

mkdir -p artifacts/

# Package self-booting artifacts
cargo gen_loadstone
cargo gen_demo_app
mv loadstone.bin artifacts/loadstone.bin
mv demo_app.bin artifacts/demo_app_self_booting.bin

# Package bootable artifacts
LOADSTONE_USE_ALT_MEMORY=1 cargo gen_demo_app
cp demo_app.bin artifacts/demo_app_unsigned.bin
cp demo_app.bin artifacts/demo_app_golden.bin
cp demo_app.bin artifacts/demo_app_regular.bin
LOADSTONE_USE_ALT_MEMORY=1 cargo gen_variant_demo_app
cp demo_app_variant.bin artifacts/

pushd tools/signing_tool/ >/dev/null
cargo run --release -- ../../artifacts/demo_app_golden.bin ../../src/devices/assets/test_key -g
cargo run --release -- ../../artifacts/demo_app_regular.bin ../../src/devices/assets/test_key
cargo run --release -- ../../artifacts/demo_app_variant.bin ../../src/devices/assets/test_key
popd >/dev/null

zip -r artifacts.zip artifacts
