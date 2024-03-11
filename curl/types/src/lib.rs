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

impl<A, E: ToString> From<Result<A, E>> for CurlResult {
    fn from(res: Result<A, E>) -> Self {
        res.err().into()
    }
}

impl<E: ToString> From<Option<E>> for CurlResult {
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
