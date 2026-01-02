#!/bin/bash
set -e

case "$TARGETPLATFORM" in
    "linux/amd64")
        export RUST_TARGET=x86_64-unknown-linux-gnu
        export CC_x86_64_unknown_linux_gnu=gcc
        export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=gcc
        ;;
    "linux/arm64")
        export RUST_TARGET=aarch64-unknown-linux-gnu
        if [ ! -d "/opt/aarch64-linux-gnu-cross" ]; then
            apt-get install -y gcc-aarch64-linux-gnu
        fi
        export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
        ;;
    *)
        echo "Unsupported platform: $TARGETPLATFORM"
        exit 1
        ;;
esac

cd web
rustup target add $RUST_TARGET
cargo build --release --locked --target=$RUST_TARGET
mkdir -p /source/web/target/release
cp /source/web/target/$RUST_TARGET/release/web /source/web/target/release/web
