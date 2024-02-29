use marine_rs_sdk::marine;
pub use curl_effector_types::*;

#[marine]
#[module_import("curl_effector")]
extern "C" {
    pub fn curl_post(request: CurlRequest, data_vault_path: String) -> CurlResult;

    pub fn curl_get(request: CurlRequest) -> CurlResult;
}
