use super::*;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join("test-plan")
        .join(path)
}

mod diagnostics;
mod reachability;
mod resolution;
