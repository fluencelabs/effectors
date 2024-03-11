pub use build_info::PKG_VERSION as VERSION;

pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub const EFFECTOR_CID: &'static str = include_str!("../output/cidv1");

#[cfg(test)]
mod tests {
    // TODO: check that it's correct CIDv1
    #[test]
    fn test_effectors_cid_empty() {
        assert!(!crate::EFFECTOR_CID.is_empty());
    }
}
