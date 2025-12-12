#!/bin/bash
set -e

case "$TARGETPLATFORM" in
    "linux/amd64")
        export RUST_TARGET=x86_64-unknown-linux-musl
        ;;
    "linux/arm64")
        export RUST_TARGET=aarch64-unknown-linux-musl
        if [ ! -d "/opt/aarch64-linux-musl-cross" ]; then
            wget https://meshorganiserdemo.suchmeme.nl/aarch64-linux-musl-cross.tgz
            tar -xzf aarch64-linux-musl-cross.tgz -C /opt
            rm aarch64-linux-musl-cross.tgz
        fi
        export PATH="/opt/aarch64-linux-musl-cross/bin:$PATH"
        export CC_aarch64_unknown_linux_musl=aarch64-linux-musl-gcc
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-musl-gcc
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
