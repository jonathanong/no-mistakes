use super::*;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join("test-plan")
        .join(path)
}

#[path = "../edge_resources_tests/diagnostics.rs"]
mod diagnostics;
#[path = "../edge_resources_tests/reachability.rs"]
mod reachability;
#[path = "../edge_resources_tests/resolution.rs"]
mod resolution;
#[path = "../edge_resources_tests/resolution_globs.rs"]
mod resolution_globs;
