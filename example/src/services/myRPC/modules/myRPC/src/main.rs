#![allow(non_snake_case)]

use std::path::Path;
use marine_rs_sdk::marine;
use marine_rs_sdk::module_manifest;

use curl_effector_types as curl;
use curl_effector_types::CurlRequest;
module_manifest!();

pub fn main() {}

#[marine]
pub fn greeting(name: String) -> String {
    let my_request = CurlRequest {
        url: "my_url".to_string(),
        headers: vec![],
        output_vault_path: "some path".to_string()
    };
    let result = curl::curl_get(my_request);
    format!("Hi, {}", name)
}
