#![allow(improper_ctypes)]
#![allow(non_snake_case)]

use eyre::{eyre, Result};
use ipfs_effector_types::{IpfsAddResult, IpfsResult};
use marine_rs_sdk::marine;
use marine_rs_sdk::module_manifest;
use marine_rs_sdk::MountedBinaryResult;
use marine_rs_sdk::ParticleParameters;
use marine_rs_sdk::WasmLoggerBuilder;
use std::path::Path;

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

    let input_vault_path = match inject_vault(&input_vault_path) {
        Ok(path) => path,
        Err(err) => {
            return IpfsAddResult {
                success: false,
                error: err.to_string(),
                hash: "".to_string(),
            }
        }
    };

    let args = vec![
        String::from("add"),
        String::from("-Q"),
        input_vault_path,
        String::from("--cid-version=1"),
        format!("--chunker=size-{}", CHUCK_SIZE),
    ];
    let cmd = make_cmd_args(args, api_multiaddr);
    run_ipfs(cmd).map(|res| res.trim().to_string()).into()
}

/// Get file by provided hash from IPFS, save it to a `file_path`, and return that path
#[marine]
pub fn get(api_multiaddr: String, cid: String, output_vault_path: &str) -> IpfsResult {
    let output_vault_path = match inject_vault(output_vault_path) {
        Ok(path) => path,
        Err(err) => {
            return IpfsResult {
                success: false,
                error: err.to_string(),
            }
        }
    };

    let args = vec![
        String::from("get"),
        String::from("-o"),
        output_vault_path,
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

fn inject_vault(virtual_path: &str) -> Result<String> {
    let cp = marine_rs_sdk::get_call_parameters();
    let real_vault_prefix = get_host_vault_path("/tmp/vault")?;
    inject_vault_host_path(&cp.particle, &real_vault_prefix, virtual_path)
}

/// Map the virtual particle vault path to the real path
/// In effectors, we now accept two kinds of paths:
/// 1. A full virtual path to a file in the particle vault that follows the pattern `/tmp/vault/{particle}/{filename}`
/// 2. A file name that is relative to the particle vault and is interpreted as `/tmp/vault/{particle}/{filename}`
/// All other paths are rejected as invalid.
/// This is done because we don't have a reliable way to check that the paths leads to
/// the particle vault and not to some other (potentially dangerous) location.
fn inject_vault_host_path(
    particle: &ParticleParameters,
    real_vault_prefix: &str,
    virtual_path: &str,
) -> Result<String> {
    let particle_virtual_vault_prefix = Path::new("/tmp/vault").join(format_particle_dir(particle));

    let path = Path::new(&virtual_path);
    // Get the filename from the path by cutting off the `/tmp/vault/{particle}` prefix if the path starts with / or return it
    // as it supposedly already a filename.
    let file_inside_vault = if path.has_root() {
        path.strip_prefix(particle_virtual_vault_prefix).map_err(|_| eyre!("invalid path provided, expected the full path to the particle vault for the current particle"))?
    } else {
        path
    };
    // Check that the remaining part of the path (or the original one) is actually a file name
    let filename = file_inside_vault.file_name().ok_or(eyre!(
        "invalid path provided, expected a path to a file, not a directory"
    ))?;
    if filename != file_inside_vault.as_os_str() {
        return Err(eyre!("invalid path provided, expected the full path to the particle vault for the current particle or a filename"))?;
    }
    // At this point we are sure that the filename is a filename without any path components
    //let host_vault_path = std::env::var(common_virtual_vault_prefix).expect("vault must be mapped to /tmp/vault");
    Ok(format!(
        "{real_vault_prefix}/{}/{}",
        format_particle_dir(particle),
        filename.to_string_lossy()
    ))
}

fn get_host_vault_path(vault_prefix: &str) -> Result<String> {
    std::env::var(vault_prefix)
        .map_err(|e| eyre!("vault must be mapped to {}: {:?}", vault_prefix, e))
}

fn format_particle_dir(particle: &ParticleParameters) -> String {
    format!("{}-{}", particle.id, particle.token)
}
