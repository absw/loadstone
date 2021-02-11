#!/bin/bash

mkdir artifacts

# exit on any failure
set -e

# Package self-booting artifacts
sed -i'' 's/0x0801/0x0800/' memory.x
cargo gen_loadstone
cargo gen_demo_app
mv loadstone.bin artifacts/loadstone.bin
mv demo_app.bin artifacts/demo_app_self_booting.bin

# Package bootable artifacts
cargo clean
sed -i'' 's/0x0800/0x0801/' memory.x
cargo gen_demo_app
cp demo_app.bin artifacts/demo_app_unsigned.bin
cp demo_app.bin artifacts/demo_app_golden.bin
cp demo_app.bin artifacts/demo_app_regular.bin
cargo gen_variant_demo_app
cp demo_app_variant.bin artifacts/
cd tools/signing_tool/
cargo run -- ../../artifacts/demo_app_golden.bin ../../src/devices/assets/test_key -g
cargo run -- ../../artifacts/demo_app_regular.bin ../../src/devices/assets/test_key
cargo run -- ../../artifacts/demo_app_variant.bin ../../src/devices/assets/test_key
cd ../../
sed -i'' 's/0x0801/0x0800/' memory.x
cargo clean
