#!/bin/bash
# Build and test with configurable schema URL

cd /source/markup_fmt

echo "Building with configurable schema URL template..."
cargo build --release --target wasm32-unknown-unknown

echo ""
echo "Copying to /tmp..."
cp target/wasm32-unknown-unknown/release/dprint_plugin_markup.wasm /tmp/plugin.wasm

echo ""
echo "Testing with dprint editor-info..."
dprint editor-info --config-discovery=false --plugins /tmp/plugin.wasm
