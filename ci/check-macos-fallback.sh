#!/bin/sh
set -eu

# An Intel deployment target predating getattrlistbulk must use ordinary directory reads.
export MACOSX_DEPLOYMENT_TARGET=10.9
export CARGO_TARGET_DIR=target/macos-fallback
target=x86_64-apple-darwin

cargo test --workspace --all-targets --target "$target"
cargo build -p dua-core --target "$target"
symbols=$(nm -u "$CARGO_TARGET_DIR/$target/debug/libdua_core.rlib")
if printf '%s\n' "$symbols" | grep -E '_getattrlistbulk|_clock_gettime'; then
    echo 'The fallback must not link macOS APIs introduced after the deployment target.' >&2
    exit 1
fi

# Reusing the build directory also checks that changing the deployment target reruns build.rs.
export MACOSX_DEPLOYMENT_TARGET=10.10
cargo test -p dua-core --target "$target"
cargo build -p dua-core --target "$target"
symbols=$(nm -u "$CARGO_TARGET_DIR/$target/debug/libdua_core.rlib")
printf '%s\n' "$symbols" | grep '_getattrlistbulk' > /dev/null
if printf '%s\n' "$symbols" | grep '_clock_gettime'; then
    echo 'macOS 10.10 supports bulk reads but does not provide clock_gettime.' >&2
    exit 1
fi
