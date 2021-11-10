#!/bin/bash
TARGET="${CARGO_TARGET_DIR:-target}"
set -e

cd "$(dirname $0)"

perl -i -pe 's/\["cdylib", "rlib"\]/\["cdylib"\]/' contract/Cargo.toml

RUSTFLAGS='-C link-arg=-s' cargo build --all --target wasm32-unknown-unknown --release
cp $TARGET/wasm32-unknown-unknown/release/test_oracle.wasm ./res/
cp $TARGET/wasm32-unknown-unknown/release/contract.wasm ./res/burrowland.wasm

perl -i -pe 's/\["cdylib"\]/\["cdylib", "rlib"\]/' contract/Cargo.toml
