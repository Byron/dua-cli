#!/bin/sh

set -eu

tag=${1:?release tag required}
install_dir=$(mktemp -d)
trap 'rm -rf "$install_dir"' EXIT

sh "${0%/*}/install.sh" \
    --git Byron/dua-cli \
    --crate dua \
    --tag "$tag" \
    --target aarch64-apple-darwin \
    --to "$install_dir"
test -x "$install_dir/dua"
