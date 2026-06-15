use super::*;

fn fixture() -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/swift-test-plan/fixture"),
    )
}

fn report() -> SwiftReport {
    analyze_project(&fixture(), None).expect("swift fixture should analyze")
}

const ENDPOINT: &str = "swift-clients/core/Sources/VouchaAPI/Endpoint.swift";

#[test]
fn importers_lists_files_that_import_the_target() {
    let report = report();
    let rows = report.importers(ENDPOINT);
    let files: Vec<&str> = rows.iter().map(|row| row.file.as_str()).collect();
    assert!(files
        .iter()
        .any(|f| f.ends_with("VouchaCore/APIClient.swift")));
    assert!(files
        .iter()
        .any(|f| f.ends_with("VouchaCoreTests/APIClientTests.swift")));
    assert!(rows.iter().all(|row| row.depth >= 1));
}

#[test]
fn importers_unknown_file_is_empty() {
    assert!(report()
        .importers("swift-clients/core/Sources/None.swift")
        .is_empty());
}

#[test]
fn test_targets_reports_covering_test_target_and_command() {
    let report = report();
    let rows = report.test_targets(ENDPOINT);
    let row = rows
        .iter()
        .find(|row| row.target == "VouchaCoreTests")
        .expect("VouchaCoreTests should cover Endpoint.swift");
    assert!(row.package.ends_with("swift-clients/core"));
    // The filter is an anchored, escaped regex so it cannot match other targets.
    assert!(row
        .command
        .contains("swift test --package-path swift-clients/core --filter ^VouchaCoreTests\\."));
}

#[test]
fn test_targets_includes_the_queried_test_files_own_target() {
    let report = report();
    let rows = report.test_targets("swift-clients/core/Tests/VouchaCoreTests/APIClientTests.swift");
    assert!(rows.iter().any(|row| row.target == "VouchaCoreTests"));
}

#[test]
fn analyze_project_without_packages_has_no_swift_edges() {
    // The crate manifest dir has no Swift package config.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let report = analyze_project(&root, None).expect("analyze");
    assert!(report.importers("anything.swift").is_empty());
    assert!(report.test_targets("anything.swift").is_empty());
}

#[test]
fn analyze_project_propagates_missing_explicit_config() {
    let result = analyze_project(&fixture(), Some(Path::new("/no/such/no-mistakes.yml")));
    assert!(result.is_err());
}

#[test]
fn test_command_handles_root_and_named_packages() {
    assert_eq!(test_command("", "T"), "swift test --filter ^T\\.");
    assert_eq!(test_command(".", "T"), "swift test --filter ^T\\.");
    assert_eq!(
        test_command("pkg/core", "T"),
        "swift test --package-path pkg/core --filter ^T\\."
    );
}

#[test]
fn test_command_escapes_regex_metacharacters() {
    assert_eq!(
        test_command("pkg", "App.Tests"),
        "swift test --package-path pkg --filter ^App\\.Tests\\."
    );
}
