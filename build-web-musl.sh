#!/bin/bash
set -e

case "$TARGETPLATFORM" in
    "linux/amd64")
        export RUST_TARGET=x86_64-unknown-linux-musl
        ;;
    "linux/arm64")
        export RUST_TARGET=aarch64-unknown-linux-musl
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
