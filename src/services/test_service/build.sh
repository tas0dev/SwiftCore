#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building user application"

cargo build --release --target=x86_64-swiftcore.json

mkdir -p ../../initfs
cp target/x86_64-swiftcore/release/test_service ../../initfs/test_service.service

echo "Built successfully"
ls -lh ../../initfs/test_service.service
