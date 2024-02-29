use marine_rs_sdk::marine;
pub use curl_effector_types::*;

#[marine]
#[module_import("curl_effector")]
extern "C" {
    // Make an HTTP POST request with the request's body taken from the `data_vault_path` file
    // The reponse is written to the `request.output_data_path` file.
    // Note that the provided path should be a full path in the Particle Vault.
    pub fn curl_post(request: CurlRequest, data_vault_path: String) -> CurlResult;

    // Make an HTTP GET request.
    // The reponse is written to the `request.output_data_path` file.
    pub fn curl_get(request: CurlRequest) -> CurlResult;
}
