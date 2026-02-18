#!/usr/bin/env bash
set -e

TARGET_TRIPLE="x86_64-unknown-uefi"
NEWLIB_TARGET="x86_64-elf"

# scripts1つ上をプロジェクトルートにする
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

TARGET_DIR="$PROJECT_ROOT/target/$TARGET_TRIPLE"

PROFILE="${1:-debug}"

INSTALL_DIR="$TARGET_DIR/$PROFILE"
BUILD_DIR="$INSTALL_DIR/newlib_build"

SOURCE_DIR="$PROJECT_ROOT/src/lib"

echo "Project root: $PROJECT_ROOT"
echo "Install dir : $INSTALL_DIR"
echo "Build dir   : $BUILD_DIR"

mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

if [ -f "config.status" ]; then
    echo "Cleaning previous configure..."
    make distclean || true
    rm -f config.cache
fi

echo "Running configure..."

"$SOURCE_DIR/configure" \
    --target=x86_64-elf \
    --host=x86_64-elf \
    --prefix="$INSTALL_DIR" \
    --disable-newlib-supplied-syscalls \
    --disable-nls \
    --disable-werror \
    --disable-libssp \
    --disable-shared \
    CFLAGS_FOR_TARGET="-ffreestanding -nostdlib"

echo "Configure complete."
