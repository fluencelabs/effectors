#![allow(non_snake_case)]

use marine_rs_sdk::marine;
use marine_rs_sdk::module_manifest;

use curl_effector_imports as curl;
use curl_effector_imports::CurlRequest;

use ipfs_effector_imports as ipfs;

module_manifest!();

pub fn main() {}

#[marine]
pub fn simple_get_http(url: String) -> String {
    let path = vault_path("some_path");
    let my_request = CurlRequest {
        url,
        headers: vec![],
    };
    let result = curl::curl_get(my_request, path.clone());
    if result.success {
        match std::fs::read_to_string(&path) {
            Ok(result) => result,
            Err(err) => err.to_string()
        }
    } else {
        result.error
    }
}

#[marine]
pub fn simple_get_ipfs(ipfs_api: String, cid: String) -> String {
    let path = vault_path("output");
    let result = ipfs::get(ipfs_api, cid, &path);
    if result.success {
        match std::fs::read_to_string(&path) {
            Ok(result) => result,
            Err(err) => err.to_string()
        }
    } else {
        result.error
    }
}


// Since all effectors are working via the Particle Vault, you need to provide a correct path to the vault.
// At the moment, we don't have any nice library for this sort of things, so you need to do it manually.
//
// Here we need to create a path to the vault which has a form of `/tmp/vault/{particle-id}-{particle-token}`.
// In this directory, you can freely write and read any files you need. Note that this directory exists only when
// a particle that called the function exsits, so you'll see here a different path each run.
fn vault_path(filename: &str) -> String {
    let cp = marine_rs_sdk::get_call_parameters();
    format!("/tmp/vault/{}-{}/{}", cp.particle.id, cp.particle.token, filename)
}
