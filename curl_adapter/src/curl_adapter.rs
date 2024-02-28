#![allow(improper_ctypes)]
#![allow(non_snake_case)]

use eyre::{ErrReport, Result};
use marine_rs_sdk::marine;
use marine_rs_sdk::module_manifest;
use marine_rs_sdk::MountedBinaryResult;
use marine_rs_sdk::WasmLoggerBuilder;
use std::fmt;

use itertools::Itertools;

module_manifest!();

const CONNECT_TIMEOUT: usize = 5;

pub fn main() {
    WasmLoggerBuilder::new()
        .with_log_level(log::LevelFilter::Info)
        .build()
        .unwrap();
}

fn run_curl(cmd: Vec<String>) -> Result<String> {
    let result = curl(cmd.clone());

    result
        .into_std()
        .ok_or(eyre::eyre!(
            "stdout or stderr contains non valid UTF8 string"
        ))?
        .map_err(|e| eyre::eyre!("curl cli call failed \n{:?}: {}", cmd.iter().join(" "), e))
}

#[marine]
pub struct Header {
    name: String,
    value: String,
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

#[marine]
pub struct CurlRequest {
    pub url: String,
    pub headers: Vec<Header>,
    pub output_vault_path: String,
}

#[marine]
pub struct CurlResult {
    success: bool,
    // MountedBinaryResult::error
    error: String,
}

impl From<Result<String, ErrReport>> for CurlResult {
    fn from(res: Result<String, ErrReport>) -> Self {
        match res {
            Ok(_) => CurlResult {
                success: true,
                error: String::new(),
            },
            Err(err) => CurlResult {
                success: false,
                error: err.to_string(),
            },
        }
    }
}

//
// curl <url> -X POST
//      --data @<data_vault_path>
//      -H <headers[0]> -H <headers[1]> -H ...
//      -o <output_vault_path>
//      --connect-timeout 5 # todo: choose the constant
//      --no-progress-meter
//      --retry 0
#[marine]
pub fn curl_post(request: CurlRequest, data_vault_path: String) -> CurlResult {
    let mut headers = Vec::new();

    for header in &request.headers {
        //let formatted = format!("-H {}", header);
        headers.push("-H".to_string());
        headers.push(format!("{}", header))
    }

    let mut args = vec![
        String::from(request.url),
        String::from("-X"),
        String::from("POST"),
        String::from("--data"),
        format!("@{}", data_vault_path),
        // String::from(data_vault_path),
        String::from("-o"),
        inject_vault_host_path(request.output_vault_path),
        format!("--connect-timeout"),
        format!("{}", CONNECT_TIMEOUT),
        String::from("--no-progress-meter"),
        String::from("--retry"),
        String::from("0"),
    ];
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
    let mut headers = Vec::new();

    for header in &request.headers {
        //let formatted = format!("-H {}", header);
        headers.push("-H".to_string());
        headers.push(format!("{}", header))
    }

    let mut args = vec![
        String::from(request.url),
        String::from("-X"),
        String::from("GET"),
        String::from("-o"),
        inject_vault_host_path(request.output_vault_path),
        format!("--connect-timeout"),
        format!("{}", CONNECT_TIMEOUT),
        String::from("--no-progress-meter"),
        String::from("--retry"),
        String::from("0"),
    ];
    args.append(&mut headers);
    run_curl(args).map(|res| res.trim().to_string()).into()
}

#[marine]
#[link(wasm_import_module = "host")]
extern "C" {
    /// Execute provided cmd as a parameters of ipfs cli, return result.
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
    use std::fs::File;
    use std::io::{self, Write};
    use std::path::Path;
    use system_interface::io::IoExt;
    use tempdir::TempDir;
    #[marine_test(config_path = "../Config.toml")]
    fn test_curl_post(curl: marine_test_env::curl_adapter::ModuleInterface) {
        let _ = ::env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .filter_module("mockito", log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
        let mut server = mockito::Server::new();
        let url = server.url();

        let mock = server
            .mock("POST", "/")
            .expect(1)
            .with_status(200)
            .with_header("content-type", "application/json")
            .match_body("input")
            .create();

        let tmp_dir = TempDir::new("tmp").unwrap();
        let file_path_input = tmp_dir.path().join("input.json");
        let mut tmp_file_input = File::create(file_path_input.clone());
        writeln!(tmp_file_input.unwrap(), "input");

        let file_path_output = tmp_dir.path().join("output.json");
        let mut tmp_file_output = File::create(file_path_output.clone());

        let input_request = marine_test_env::curl_adapter::CurlRequest {
            url: url.clone(),
            headers: vec![marine_test_env::curl_adapter::Header {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            }],
            output_vault_path: format!("{}", file_path_output.display()),
        };
        let result = curl.curl_post(input_request, format!("{}", file_path_input.display()));

        assert!(result.success, "error: {}", result.error);
        mock.assert();
    }

    #[marine_test(config_path = "../Config.toml")]
    fn test_curl_get(curl: marine_test_env::curl_adapter::ModuleInterface) {
        let _ = ::env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .filter_module("mockito", log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
        let mut server = mockito::Server::new();
        let url = server.url();
        println!("{}", url);

        let mock = server
            .mock("GET", "/")
            .expect(1)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("input")
            .create();

        let tmp_dir = TempDir::new("tmp").unwrap();
        let file_path_input = tmp_dir.path().join("input.json");
        let mut tmp_file_input = File::create(file_path_input.clone());
        writeln!(tmp_file_input.unwrap(), "input");

        let file_path_output = tmp_dir.path().join("output.json");
        let mut tmp_file_output = File::create(file_path_output.clone());

        let input_request = marine_test_env::curl_adapter::CurlRequest {
            url: url.clone(),
            headers: vec![marine_test_env::curl_adapter::Header {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            }],
            output_vault_path: format!("{}", file_path_output.display()),
        };
        let result = curl.curl_get(input_request);

        assert!(result.success, "error: {}", result.error);
        mock.assert();
    }
}
