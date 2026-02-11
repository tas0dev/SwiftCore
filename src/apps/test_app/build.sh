#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building user application"

cargo build --release --target=x86_64-swiftcore.json

# Copy to initfs directory
mkdir -p ../../../initfs
cp target/x86_64-swiftcore/release/test_app ../../../initfs/test_app.elf

echo "Built successfully"
ls -lh ../../../initfs/test_app.elf
