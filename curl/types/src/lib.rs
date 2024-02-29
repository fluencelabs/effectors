use marine_rs_sdk::marine;

// HTTP Header description
#[marine]
#[derive(Clone, Debug)]
pub struct HttpHeader {
    // Name of the header. For example: "Content-Type"
    pub name: String,
    // Value of the header. For example: "application/json"
    pub value: String,
}

// A generic cURL request
#[marine]
#[derive(Clone, Debug)]
pub struct CurlRequest {
    pub url: String,
    pub headers: Vec<HttpHeader>,
    // The path in the Particle Vault with the result of the request.
    // Note that the file is created when doesn't exist.
    pub output_vault_path: String,
}

// A generic cURL call result
#[marine]
#[derive(Clone, Debug)]
pub struct CurlResult {
    // True when cURL executed successfully.
    // Note that it's also true on non-200 responses
    pub success: bool,
    pub error: String,
}

impl <A, E: ToString> From<Result<A, E>> for CurlResult {
    fn from(res: Result<A, E>) -> Self {
        res.err().into()
    }
}

impl <E: ToString> From<Option<E>> for CurlResult {
    fn from(res: Option<E>) -> Self {
        match res {
            None => CurlResult {
                success: true,
                error: String::new(),
            },
            Some(err) => CurlResult {
                success: false,
                error: err.to_string(),
            },
        }
    }
}
