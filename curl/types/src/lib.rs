use marine_rs_sdk::marine;

#[marine]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

#[marine]
pub struct CurlRequest {
    pub url: String,
    pub headers: Vec<HttpHeader>,
    pub output_vault_path: String,
}

#[marine]
pub struct CurlResult {
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
