//! Crate-local fixture loader and snapshot helpers for `no-mistakes-coverage`.

use serde::Serialize;
use std::path::PathBuf;

pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/coverage")
        .join(name)
}

pub fn read_fixture_bytes(name: &str) -> Vec<u8> {
    let path = fixture_path(name);
    std::fs::read(&path)
        .unwrap_or_else(|err| panic!("failed to read fixture {}: {err}", path.display()))
}

pub fn pretty_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).expect("coverage value must serialize to JSON")
}
