#![allow(improper_ctypes)]
#![allow(non_snake_case)]

use eyre::Result;
use marine_rs_sdk::marine;
use marine_rs_sdk::module_manifest;
use marine_rs_sdk::MountedBinaryResult;
use marine_rs_sdk::WasmLoggerBuilder;
use ipfs_effector_types::{IpfsAddResult, IpfsResult};

use itertools::Itertools;

module_manifest!();

/// Default chunk size for `ipfs add` command to produce stable CIDs.
const CHUCK_SIZE: usize = 262144;
const CONNECT_TIMEOUT: usize = 5;

pub fn main() {
    WasmLoggerBuilder::new()
        .with_log_level(log::LevelFilter::Info)
        .build()
        .unwrap();
}

/// Run `ipfs` mounted binary with the specified arguments
fn run_ipfs(cmd: Vec<String>) -> Result<String> {
    let result = ipfs(cmd.clone());

    result
        .into_std()
        .ok_or(eyre::eyre!(
            "stdout or stderr contains non valid UTF8 string"
        ))?
        .map_err(|e| eyre::eyre!("ipfs cli call failed \n{:?}: {}", cmd.iter().join("  "), e))
}

fn make_cmd_args(args: Vec<String>, api_multiaddr: String) -> Vec<String> {
    args.into_iter()
        .chain(vec![
            String::from("--timeout"),
            format!("{}s", CONNECT_TIMEOUT),
            String::from("--api"),
            api_multiaddr,
        ])
        .collect()
}

/// Put file from specified path to IPFS and return its hash.
// ipfs add --cid-version 1 --hash sha2-256 --chunker=size-262144 # to produce CIDv1
//   --api <api>
//   -Q   # to get hash as the output
//   <data_vault_path>
#[marine]
pub fn add(api_multiaddr: String, input_vault_path: String) -> IpfsAddResult {
    if !std::path::Path::new(&input_vault_path).exists() {
        return IpfsAddResult {
            success: false,
            error: format!("path {} doesn't exist", input_vault_path),
            hash: "".to_string(),
        };
    }

    let args = vec![
        String::from("add"),
        String::from("-Q"),
        inject_vault_host_path(input_vault_path),
        String::from("--cid-version=1"),
        format!("--chunker=size-{}", CHUCK_SIZE),
    ];
    let cmd = make_cmd_args(args, api_multiaddr);
    run_ipfs(cmd).map(|res| res.trim().to_string()).into()
}

/// Get file by provided hash from IPFS, save it to a `file_path`, and return that path
#[marine]
pub fn get(api_multiaddr: String, cid: String, output_vault_path: &str) -> IpfsResult {
    let args = vec![
        String::from("get"),
        String::from("-o"),
        inject_vault_host_path(output_vault_path.to_string()),
        cid,
    ];
    let cmd = make_cmd_args(args, api_multiaddr);

    run_ipfs(cmd).map(drop).into()
}

#[marine]
#[host_import]
extern "C" {
    /// Execute provided cmd as a parameters of ipfs cli, return result.
    pub fn ipfs(cmd: Vec<String>) -> MountedBinaryResult;
}

// to map the virtual particle vault path to the real path 
fn inject_vault_host_path(path: String) -> String {
    let vault = "/tmp/vault";
    if let Some(stripped) = path.strip_prefix(&vault) {
        let host_vault_path = std::env::var(vault).expect("vault must be mapped to /tmp/vault");
        format!("/{}/{}", host_vault_path, stripped)
    } else {
        path
    }
}