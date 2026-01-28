#!/usr/bin/env bash
set -e

echo "== building kernel =="
cargo build -p kernel --release --target wasm32-unknown-unknown

echo "== building hello app =="
cargo build -p hello --release --target wasm32-unknown-unknown

echo "== preparing static dir =="
rm -rf server/static
mkdir -p server/static/apps

cp web/index.html server/static/index.html
cp web/bootstrap.js server/static/bootstrap.js

cp target/wasm32-unknown-unknown/release/kernel.wasm \
  server/static/kernel.wasm

cp target/wasm32-unknown-unknown/release/hello.wasm \
  server/static/apps/hello.wasm

echo "== done =="
