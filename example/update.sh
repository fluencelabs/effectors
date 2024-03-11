#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail

# set current working directory to script directory to run script from everywhere
cd "$(dirname "$0")"

echo "Updating cURL effector.."
../curl/build.sh
cp ../curl/target/wasm32-wasi/release/curl_effector.wasm src/services/myRPC/modules/curl_effector/

echo "Updating IPFS effector.."
../ipfs/build.sh
cp ../ipfs/target/wasm32-wasi/release/ipfs_effector.wasm src/services/myRPC/modules/ipfs_effector/
