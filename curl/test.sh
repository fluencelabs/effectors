#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail

# set current working directory to script directory to run script from everywhere
cd "$(dirname "$0")"

# This script builds all subprojects and puts all created Wasm modules in one dir
cd effector
marine build --release
rm test_artifacts/*.wasm
cp ../target/wasm32-wasi/release/curl_effector.wasm test_artifacts/

# Not that these test must be run one by one like spell tests because they
# use the same temp directory (marine test restriction)
cargo nextest run --release --no-fail-fast --nocapture
