#![allow(improper_ctypes)]
#![allow(non_snake_case)]

use eyre::Result;
use marine_rs_sdk::marine;
use marine_rs_sdk::module_manifest;
use marine_rs_sdk::MountedBinaryResult;
use marine_rs_sdk::WasmLoggerBuilder;
use curl_effector_types::*;

use itertools::Itertools;

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
pub fn curl_post(request: CurlRequest, data_vault_path: String) -> CurlResult {
    let mut args = vec![
        String::from(request.url),
        String::from("-X"),
        String::from("POST"),
        String::from("--data"),
        format!("@{}", inject_vault_host_path(data_vault_path)),
        String::from("-o"),
        inject_vault_host_path(request.output_vault_path),
    ];
    let mut headers = format_header_args(&request.headers);
    args.append(&mut headers);
    run_curl(args).map(|res| res.trim().to_string()).into()
}

// curl <url> -X GET
//      -H <headers[0]> -H <headers[1]> -H ...
//      -o <output_vault_path>
//      --connect-timeout <connect-timeout>
//      --no-progress-meter
//      --retry 0
#[marine]
pub fn curl_get(request: CurlRequest) -> CurlResult {
    let mut args = vec![
        String::from(request.url),
        String::from("-X"),
        String::from("GET"),
        String::from("-o"),
        inject_vault_host_path(request.output_vault_path),
    ];
    let mut headers = format_header_args(&request.headers);
    args.append(&mut headers);
    run_curl(args).map(|res| res.trim().to_string()).into()
}

#[marine]
#[host_import]
extern "C" {
    /// Execute provided cmd as a parameters of curl, return result.
    pub fn curl(cmd: Vec<String>) -> MountedBinaryResult;
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

#[cfg(test)]
mod tests {
    use marine_rs_sdk_test::marine_test;
    use std::fs::{File, read_to_string};
    use std::io::Write;
    use tempdir::TempDir;

    #[marine_test(config_path = "../../Config.toml")]
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
            .expect(1)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(expected_output)
            .create();

        let tmp_dir = TempDir::new("tmp").unwrap();
        let file_path_input = tmp_dir.path().join("input.json");
        let mut tmp_file_input = File::create(file_path_input.clone()).unwrap();
        writeln!(tmp_file_input, "{}", expected_input).unwrap();

        let file_path_output = tmp_dir.path().join("output.json");

        let input_request = marine_test_env::curl_effector::CurlRequest {
            url: url.clone(),
            headers: vec![marine_test_env::curl_effector::HttpHeader {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            }],
            output_vault_path: format!("{}", file_path_output.display()),
        };
        let result = curl.curl_post(input_request, format!("{}", file_path_input.display()));
        assert!(result.success, "error: {}", result.error);

        let actual_output = read_to_string(file_path_output).unwrap();
        assert_eq!(actual_output, expected_output);

        mock.assert();
    }

    #[marine_test(config_path = "../../Config.toml")]
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
            .expect(1)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(extected_output)
            .create();

        let tmp_dir = TempDir::new("tmp").unwrap();
        let file_path_output = tmp_dir.path().join("output.json");

        let input_request = marine_test_env::curl_effector::CurlRequest {
            url: url.clone(),
            headers: vec![marine_test_env::curl_effector::HttpHeader {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            }],
            output_vault_path: format!("{}", file_path_output.display()),
        };
        let result = curl.curl_get(input_request);
        assert!(result.success, "error: {}", result.error);

        let actual_output = read_to_string(file_path_output).unwrap();
        assert_eq!(actual_output, extected_output);

        mock.assert();
    }
}
