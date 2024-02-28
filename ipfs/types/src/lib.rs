use marine_rs_sdk::marine;

#[marine]
pub struct IpfsResult {
    pub success: bool,
    pub error: String,
}

impl <A, E: ToString> From<Result<A, E>> for IpfsResult {
    fn from(result: Result<A, E>) -> Self {
            result.err().into()
    }
}

impl <E: ToString> From<Option<E>> for IpfsResult {
    fn from(res: Option<E>) -> Self {
        match res {
            None => IpfsResult {
                success: true,
                error: String::new(),
            },
            Some(err) => IpfsResult {
                success: false,
                error: err.to_string(),
            },
        }
    }
}

#[marine]
pub struct IpfsAddResult {
    pub success: bool,
    pub error: String,
    pub hash: String,
}

impl <E: ToString> From<Result<String, E>> for IpfsAddResult {
    fn from(result: Result<String, E>) -> Self {
        match result {
            Ok(hash) => Self {
                success: true,
                error: "".to_string(),
                hash,
            },
            Err(err) => Self {
                success: false,
                error: err.to_string(),
                hash: "".to_string(),
            },
        }
    }
}
