use super::*;
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/dotnet-dependency-diff")
            .join(name),
    )
    .unwrap()
}

#[path = "tests/diagnostics.rs"]
mod diagnostics;
#[path = "tests/semantic.rs"]
mod semantic;
#[path = "tests/xml.rs"]
mod xml;
