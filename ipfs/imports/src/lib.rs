use marine_rs_sdk::marine;

pub use ipfs_effector_types::*;

#[marine]
#[module_import("ipfs_effector")]
extern "C" {
    // Upload a file `input_vault_path` to IPFS node with the `api_multiaddr` multiaddress
    pub fn add(api_multiaddr: String, input_vault_path: String) -> IpfsAddResult;

    // Downloads a file by `cid` to the `output_vault_path` file from IPFS node with the `api_multiaddr` multiaddress
    pub fn get(api_multiaddr: String, cid: String, output_vault_path: &str) -> IpfsResult;
}
