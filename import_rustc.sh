#!/bin/bash
# (c) Meta Platforms, Inc. and affiliates. Confidential and proprietary.

set -euo pipefail

### CONSTS
VERSION=1.67.1
# These are taken from https://github.com/rust-lang/rust/blob/master/src/stage0.json
BOOTSTRAP_RUSTC_URL="https://static.rust-lang.org/dist/2023-01-10/rustc-1.66.1-x86_64-unknown-linux-gnu.tar.xz"
BOOTSTRAP_STDLIBS_URL="https://static.rust-lang.org/dist/2023-01-10/rust-std-1.66.1-x86_64-unknown-linux-gnu.tar.xz"
# Pull in LLVM artifacts from CI
RUST_DEV_NAME="rust-dev-1.67.1-x86_64-unknown-linux-gnu"
LLVM_ARTIFACTS_URL="https://ci-artifacts.rust-lang.org/rustc-builds/d5a82bbd26e1ad8b7401f6a718a9c57c96905483/$RUST_DEV_NAME.tar.xz"
### END OF CONSTS

has_param() {
    local term="$1"
    shift
    for arg; do
        if [[ $arg == "$term" ]]; then
            return 0
        fi
    done
    return 1
}

if has_param '--download-source' "$@"; then
    DOWNLOAD_SOURCE=1
fi

THIS_DIR=$(cd -P -- "$(dirname "${BASH_SOURCE[0]}")" >/dev/null && pwd)
SCRATCH_DIR=$(mktemp -d -t import_rust_std.XXX)
trap 'rm -rf "$SCRATCH_DIR"' EXIT

OUT_DIR=$THIS_DIR
LIBRARY_DIR=$THIS_DIR/library
COMPILER_DIR=$THIS_DIR/compiler

cd "$SCRATCH_DIR"
# There are some things that need to be downloaded (bootstrap compiler, ci-llvm artifacts)
# TODO(themarwhal): Download these with http_archive
TOOLCHAINS_DOWNLOADS="$THIS_DIR/toolchains/downloads"
rm -rf "$TOOLCHAINS_DOWNLOADS"
mkdir -p "$TOOLCHAINS_DOWNLOADS"

echo "Downloading bootstrap rustc from $BOOTSTRAP_RUSTC_URL"
wget -O "$TOOLCHAINS_DOWNLOADS/bootstrap_rustc.tar.xz" "$BOOTSTRAP_RUSTC_URL"
echo "Downloading bootstrap stdlibs from $BOOTSTRAP_STDLIBS_URL"
wget -O "$TOOLCHAINS_DOWNLOADS/bootstrap_stdlib.tar.xz" "$BOOTSTRAP_STDLIBS_URL"

LLVM_DIR="$THIS_DIR/compiler/ci-llvm"
echo "Downloading llvm artifacts from $LLVM_ARTIFACTS_URL into $LLVM_DIR"
rm -rf "$THIS_DIR/compiler/ci-llvm/include"
rm -rf "$THIS_DIR/compiler/ci-llvm/lib"
wget "$LLVM_ARTIFACTS_URL"
tar -xf "$RUST_DEV_NAME.tar.xz"
cp -r "$RUST_DEV_NAME/rust-dev/lib" "$LLVM_DIR/lib"
cp -r "$RUST_DEV_NAME/rust-dev/include" "$LLVM_DIR/include"


if [[ -z "${DOWNLOAD_SOURCE:-}" ]]; then
    echo "Exiting after downloading ci-llvm and rust bootstrap toolchain. Use --download-source to re-download Rust source code"
    exit
fi

cd "$SCRATCH_DIR"
HTTPS_PROXY=fwdproxy:8080 git clone https://github.com/rust-lang/rust.git --branch $VERSION

RUST_SRC="$SCRATCH_DIR/rust"
cd "$RUST_SRC"
HTTPS_PROXY=fwdproxy:8080 git submodule update --init library/

echo "Setup library/"
# Remove library/src dir.
rm -rf "$LIBRARY_DIR/src"
# Copy sources.
cp -r "$RUST_SRC/library" "$LIBRARY_DIR/src"

cd $LIBRARY_DIR
HTTPS_PROXY=fwdproxy:8080 reindeer vendor
reindeer buckify

echo "Setup compiler/"
# Remove compiler/src dir.
rm -rf "$COMPILER_DIR/src"

# Copy sources.
cp -r "$RUST_SRC/compiler" "$COMPILER_DIR/src"
# TODO(themarwhal): These might just be workarounds
# Regenerate source files in rustc_baked_icu_data as they are somehow outdated
# https://github.com/rust-lang/rust/pull/107673
echo "Regenerate compiler/src/rustc_baked_icu_data with updated icu_datagen"
HTTPS_PROXY=fwdproxy:8080 cargo install icu_datagen --features bin
cd "$COMPILER_DIR/src/rustc_baked_icu_data"
rm -r src/data
HTTPS_PROXY=fwdproxy:8080 icu4x-datagen -W --pretty --fingerprint --use-separate-crates --format mod -l en es fr it ja pt ru tr zh zh-Hans zh-Hant -k list/and@1 fallback/likelysubtags@1 fallback/parents@1 fallback/supplement/co@1 --cldr-tag latest --icuexport-tag latest -o src/data

cd "$COMPILER_DIR"
echo "Remove 'crate-type = [\"dylib\"]' so that reindeer works"
# TODO(themarwhal): Reindeer seems to ignore if crate-type=["dylib"]
sed -i 's/crate-type/\#crate-type/g' src/rustc_driver/Cargo.toml
sed -i 's/crate-type/\#crate-type/g' src/rustc_codegen_cranelift/Cargo.toml
sed -i 's/crate-type/\#crate-type/g' src/rustc_codegen_gcc/Cargo.toml

cd "$COMPILER_DIR"
HTTPS_PROXY=fwdproxy:8080 reindeer vendor
reindeer buckify

cd $OUT_DIR
echo "Updating rust src version"
echo "$VERSION" > rust_src_version
