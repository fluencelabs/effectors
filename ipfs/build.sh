#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail

# set current working directory to script directory to run script from everywhere
cd "$(dirname "$0")"

# This script builds all subprojects and puts all created Wasm modules in one dir
cd effector
marine build --release
cd ..

mkdir -p cid/output
rm -f cid/output/*
ipfs add --only-hash -Q --cid-version 1 --hash sha2-256 --chunker=size-262144 target/wasm32-wasi/release/ipfs_effector.wasm | tee cid/output/cidv1

cd cid
cargo build --release
