use marine_rs_sdk::marine;

#[marine]
#[derive(Clone, Debug)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

#[marine]
#[derive(Clone, Debug)]
pub struct CurlRequest {
    pub url: String,
    pub headers: Vec<HttpHeader>,
    pub output_vault_path: String,
}

#[marine]
#[derive(Clone, Debug)]
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
