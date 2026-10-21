use super::*;
use serde_json::json;
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/core-analysis")
        .canonicalize()
        .expect("performance fixture should exist")
}

mod graph;
mod postgres;
mod surfaces;
