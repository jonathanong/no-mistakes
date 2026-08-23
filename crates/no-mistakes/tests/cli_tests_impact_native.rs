use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
}

fn assert_self_selected(root: &Path, test: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args([
            "tests",
            "impact",
            "--root",
            root.to_str().unwrap(),
            test,
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["selected_tests"][0]["test_file"], test, "{plan:#}");
    assert_eq!(plan["selected_tests"][0]["reasons"][0]["via"][0], "self");
}

#[test]
fn tests_impact_cli_preserves_configured_dotnet_test_projects() {
    assert_self_selected(
        &fixture("dotnet-test-plan"),
        "dotnet-clients/tests/App.Tests/FeedServiceTests.cs",
    );
}

#[test]
fn tests_impact_cli_preserves_configured_swift_test_projects() {
    assert_self_selected(
        &fixture("swift-test-plan"),
        "swift-clients/core/Tests/VouchaCoreTests/APIClientTests.swift",
    );
}
