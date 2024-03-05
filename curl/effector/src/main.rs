#![feature(try_blocks)]
#![feature(assert_matches)]
#![allow(improper_ctypes)]
#![allow(non_snake_case)]

use std::path::Path;
use eyre::{eyre, Result};
use marine_rs_sdk::{marine, ParticleParameters};
use marine_rs_sdk::module_manifest;
use marine_rs_sdk::MountedBinaryResult;
use marine_rs_sdk::WasmLoggerBuilder;
use curl_effector_types::*;

use itertools::Itertools;
use url::Url;

module_manifest!();

const CONNECT_TIMEOUT: usize = 5;

pub fn main() {
    WasmLoggerBuilder::new()
        .with_log_level(log::LevelFilter::Info)
        .build()
        .unwrap();
}

fn run_curl(mut cmd: Vec<String>) -> Result<String> {
    let mut default_arguments = vec![format!("--connect-timeout"),
    format!("{}", CONNECT_TIMEOUT),
    String::from("--no-progress-meter"),
    String::from("--retry"),
    String::from("0")];
    cmd.append(&mut default_arguments);

    log::debug!("curl arguments: {:?}", cmd);
    let result = curl(cmd.clone());
    log::debug!("curl result: {:?}", result.stringify());

    result
        .into_std()
        .ok_or(eyre::eyre!(
            "stdout or stderr contains non valid UTF8 string"
        ))?
        .map_err(|e| eyre::eyre!("curl cli call failed \n{:?}: {}", cmd.iter().join(" "), e))
}

fn format_header_args(headers: &[HttpHeader]) -> Vec<String> {
    let mut result = Vec::new();
    for header in headers {
        result.push("-H".to_string());
        result.push(format!("{}: {}", header.name, header.value))
    }
    result
}

//
// curl <url> -X POST
//      --data @<data_vault_path>
//      -H <headers[0]> -H <headers[1]> -H ...
//      -o <output_vault_path>
//      --connect-timeout CONNECT_TIMEOUT
//      --no-progress-meter
//      --retry 0
#[marine]
pub fn curl_post(request: CurlRequest, data_vault_path: &str, output_vault_path: &str) -> CurlResult {
    let result: Result<String> = try {
        let url = check_url(request.url)?;
        let data_vault_path = inject_vault(data_vault_path)?;
        let output_vault_path = inject_vault(output_vault_path)?;
        let mut args = vec![
            String::from(url),
            String::from("-X"),
            String::from("POST"),
            String::from("--data"),
            format!("@{}", data_vault_path),
            String::from("-o"),
            output_vault_path,
        ];
        let mut headers = format_header_args(&request.headers);
        args.append(&mut headers);
        run_curl(args).map(|res| res.trim().to_string())?
    };
    result.into()
}

// curl <url> -X GET
//      -H <headers[0]> -H <headers[1]> -H ...
//      -o <output_vault_path>
//      --connect-timeout <connect-timeout>
//      --no-progress-meter
//      --retry 0
#[marine]
pub fn curl_get(request: CurlRequest, output_vault_path: &str) -> CurlResult {
    let result: Result<String> = try {
        let url = check_url(request.url)?;
        let output_vault_path = inject_vault(output_vault_path)?;
        let mut args = vec![
            String::from(url),
            String::from("-X"),
            String::from("GET"),
            String::from("-o"),
            output_vault_path,
        ];
        let mut headers = format_header_args(&request.headers);
        args.append(&mut headers);
        run_curl(args).map(|res| res.trim().to_string())?
    };
    result.into()
}

#[marine]
#[host_import]
extern "C" {
    /// Execute provided cmd as a parameters of curl, return result.
    pub fn curl(cmd: Vec<String>) -> MountedBinaryResult;
}

fn check_url(url: String) -> Result<String> {
    let url = Url::parse(&url).map_err(|e| eyre!("invalid url provided: {}", e))?;
    if url.scheme() == "file" {
       return Err(eyre!("file:// scheme is forbidden"));
    }
    Ok(url.to_string())
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
pub(crate) fn inject_vault_host_path(particle: &ParticleParameters, real_vault_prefix: &str, virtual_path: &str) -> Result<String> {
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
    let filename = file_inside_vault.file_name().ok_or(eyre!("invalid path provided, expected a path to a file, not a directory"))?;
    if filename != file_inside_vault.as_os_str() {
        return Err(eyre!("invalid path provided, expected the full path to the particle vault for the current particle or a filename"))?;
    }
    // At this point we are sure that the filename is a filename without any path components
    //let host_vault_path = std::env::var(common_virtual_vault_prefix).expect("vault must be mapped to /tmp/vault");
    Ok(format!("{real_vault_prefix}/{}/{}", format_particle_dir(particle), filename.to_string_lossy()))
}

fn get_host_vault_path(vault_prefix: &str) -> Result<String> {
    std::env::var(vault_prefix).map_err(|e| eyre!("vault must be mapped to {}: {:?}", vault_prefix, e))
}

fn format_particle_dir(particle: &ParticleParameters) -> String {
    format!("{}-{}", particle.id, particle.token)
}

#[cfg(test)]
mod unit_tests {
    use std::assert_matches::assert_matches;
    use marine_rs_sdk::ParticleParameters;
    use crate::inject_vault_host_path;

    #[test]
    fn test_inject() {
        let mut particle = ParticleParameters::default();
        particle.id = "test_id".to_string();
        particle.token = "token".to_string();

        let real_vault_prefix = "/real/storage";

        let result = inject_vault_host_path(&particle, real_vault_prefix, "/tmp/vault/test_id-token/input.json");
        assert_matches!(result, Ok(_));
        assert_eq!(result.unwrap(), "/real/storage/test_id-token/input.json");

        let result = inject_vault_host_path(&particle, real_vault_prefix, "input.json");
        assert_matches!(result, Ok(_));
        assert_eq!(result.unwrap(), "/real/storage/test_id-token/input.json");

        let result = inject_vault_host_path(&particle, real_vault_prefix, "/etc/passwd");
        assert_matches!(result, Err(_), "non-vault paths are forbidden");

        let result = inject_vault_host_path(&particle, real_vault_prefix, "/tmp/vault/test_id2-token2/input.json");
        assert_matches!(result, Err(_), "paths in vaults of other particles are also forbidden");

        let result = inject_vault_host_path(&particle, real_vault_prefix, "vault_dir/input.json");
        assert_matches!(result, Err(_), "only filenames in the vault are allowed");

   }
}

#[test_env_helpers::before_each]
#[test_env_helpers::after_each]
#[test_env_helpers::after_all]
#[cfg(test)]
mod tests {
    use marine_rs_sdk_test::{CallParameters, marine_test};
    use std::fs::{File, read_to_string};
    use std::io::Write;
    use std::path::Path;

    const VAULT_TEMP: &str = "./test_artifacts/temp";
    const PARTICLE_ID: &str = "test_id";
    const TOKEN: &str = "token";

    const PARTICLE_VAULT: &str = "./test_artifacts/temp/test_id-token";
    const VIRTUAL_VAULT: &str = "/tmp/vault/test_id-token";

    fn before_each() {
        std::fs::create_dir_all(PARTICLE_VAULT).expect(&format!("create {PARTICLE_VAULT} failed"));
    }

    fn after_each() {
        std::fs::remove_dir_all(PARTICLE_VAULT).expect(&format!("remove {PARTICLE_VAULT} failed"));
    }
    fn after_all() {
        std::fs::remove_dir_all(VAULT_TEMP).expect(&format!("remove {VAULT_TEMP} failed"));
    }

    #[marine_test(config_path = "../test_artifacts/Config.toml")]
    fn test_curl_get_file_url(curl: marine_test_env::curl_effector::ModuleInterface) {
        let _ = ::env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .filter_module("mockito", log::LevelFilter::Debug)
            .filter_module("curl_effector", log::LevelFilter::Debug)
            .filter_module("wasmer_interface_types_fl", log::LevelFilter::Off)
            .is_test(true)
            .try_init();
        let mut cp = CallParameters::default();
        cp.particle.id = PARTICLE_ID.to_string();
        cp.particle.token = TOKEN.to_string();

        let target_secrets = format!("{VAULT_TEMP}/secrets.json");
        let mut input_file = File::create(&target_secrets).unwrap();
        writeln!(input_file, "secret").unwrap();
        let full_target_secrets_path = Path::new(&target_secrets).canonicalize().unwrap();

        let input_request = marine_test_env::curl_effector::CurlRequest {
            url: format!("file://{}", full_target_secrets_path.display()),
            headers: vec![marine_test_env::curl_effector::HttpHeader {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            }],
        };
        let result = curl.curl_get_cp(input_request.clone(), "output.json".to_string(), cp.clone());
        assert!(!result.success, "forbidden url request must fail");

        let output_real_file = format!("./{PARTICLE_VAULT}/output.json");
        let output_real_file = Path::new(&output_real_file);
        assert!(!output_real_file.exists(), "output file must NOT be even created");
    }

    #[marine_test(config_path = "../test_artifacts/Config.toml")]
    fn test_curl_post(curl: marine_test_env::curl_effector::ModuleInterface) {
        let _ = ::env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .filter_module("mockito", log::LevelFilter::Debug)
            .filter_module("curl_effector", log::LevelFilter::Debug)
            .filter_module("wasmer_interface_types_fl", log::LevelFilter::Off)
            .is_test(true)
            .try_init();

        let mut server = mockito::Server::new();
        let url = server.url();
        let expected_input = "{\"a\": \"c\"}";
        let expected_output = "{\"a\": \"b\"}";
        let mock = server
            .mock("POST", "/")
            .match_body(expected_input)
            .expect(2)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(expected_output)
            .create();

        let mut cp = CallParameters::default();
        cp.particle.id = PARTICLE_ID.to_string();
        cp.particle.token = TOKEN.to_string();

        let input_real_file = format!("./{PARTICLE_VAULT}/input.json");
        let output_real_file = format!("./{PARTICLE_VAULT}/output.json");

        let mut input_file = File::create(input_real_file).unwrap();
        writeln!(input_file, "{}", expected_input).unwrap();

        let input_request = marine_test_env::curl_effector::CurlRequest {
            url: url.clone(),
            headers: vec![marine_test_env::curl_effector::HttpHeader {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            }],
        };
        let result = curl.curl_post_cp(input_request.clone(), "input.json".to_string(), "output.json".to_string(), cp.clone());
        assert!(result.success, "error: {}", result.error);

        let actual_output = read_to_string(Path::new(&output_real_file)).unwrap();
        assert_eq!(actual_output, expected_output);

        // Also check full paths
        let input_real_file2= format!("./{PARTICLE_VAULT}/input2.json");
        let output_real_file2 = format!("./{PARTICLE_VAULT}/output2.json");

        let mut input_file = File::create(input_real_file2).unwrap();
        writeln!(input_file, "{}", expected_input).unwrap();

        let result = curl.curl_post_cp(input_request, format!("{VIRTUAL_VAULT}/input2.json"), format!("{VIRTUAL_VAULT}/output2.json"), cp);
        assert!(result.success, "error: {}", result.error);

        let actual_output = read_to_string(Path::new(&output_real_file2)).unwrap();
        assert_eq!(actual_output, expected_output);


        mock.assert();
    }

    #[marine_test(config_path = "../test_artifacts/Config.toml")]
    fn test_curl_get(curl: marine_test_env::curl_effector::ModuleInterface) {
        let _ = ::env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .filter_module("mockito", log::LevelFilter::Debug)
            .filter_module("curl_effector", log::LevelFilter::Debug)
            .filter_module("wasmer_interface_types_fl", log::LevelFilter::Off)
            .is_test(true)
            .try_init();

        let mut server = mockito::Server::new();
        let url = server.url();

        let extected_output = "{\"a\": \"b\"}";
        let mock = server
            .mock("GET", "/")
            .expect(2)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(extected_output)
            .create();

        let mut cp = CallParameters::default();
        cp.particle.id = PARTICLE_ID.to_string();
        cp.particle.token = TOKEN.to_string();
        let input_request = marine_test_env::curl_effector::CurlRequest {
            url: url.clone(),
            headers: vec![marine_test_env::curl_effector::HttpHeader {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            }],
        };
        let result = curl.curl_get_cp(input_request.clone(), "output.json".to_string(), cp.clone());
        assert!(result.success, "error: {}", result.error);

        let output_real_path = format!("./{PARTICLE_VAULT}/output.json");
        let actual_output = read_to_string(Path::new(&output_real_path)).unwrap();
        assert_eq!(actual_output, extected_output);

        // Also check full paths
        let result = curl.curl_get_cp(input_request, format!("{VIRTUAL_VAULT}/output2.json"), cp);
        assert!(result.success, "error: {}", result.error);

        let output_real_path = format!("./{PARTICLE_VAULT}/output2.json");
        let actual_output = read_to_string(Path::new(&output_real_path)).unwrap();
        assert_eq!(actual_output, extected_output);

        mock.assert();
    }
}
