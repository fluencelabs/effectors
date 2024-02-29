use marine_rs_sdk::marine;

pub use ipfs_effector_types::*;

#[marine]
#[module_import("ipfs_effector")]
extern "C" {
    pub fn add(api_multiaddr: String, input_vault_path: String) -> IpfsAddResult;
    pub fn get(api_multiaddr: String, cid: String, output_vault_path: &str) -> IpfsResult;
}
