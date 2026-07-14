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
fn analyze_project_reuses_one_discovery_and_swift_fact_collection() {
    assert!(!report().importers(ENDPOINT).is_empty());

    let source = include_str!("../swift_api.rs");
    let analyze_body = source
        .split("pub fn analyze_project(root: &Path, config_path: Option<&Path>)")
        .nth(1)
        .and_then(|source| source.split("impl SwiftReport").next())
        .expect("swift analyze_project body");

    assert_eq!(analyze_body.matches("VisiblePathSnapshot::new").count(), 1);
    assert_eq!(
        analyze_body.matches("discover_files_from_visible(").count(),
        1
    );
    assert_eq!(analyze_body.matches("collect_swift_facts(").count(), 1);
    assert_eq!(
        analyze_body
            .matches("build_with_plan_files_prepared_config_and_swift_facts(")
            .count(),
        1
    );
    assert!(!analyze_body.contains("build_with_plan_files_prepared_config("));
}

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
    // The package path and the anchored, escaped regex filter are both quoted.
    assert!(row
        .command
        .contains("swift test --package-path 'swift-clients/core' --filter '^VouchaCoreTests\\.'"));
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
fn analyze_project_uses_one_snapshot_and_one_swift_fact_collection() {
    assert!(!report().importers(ENDPOINT).is_empty());
    let source = include_str!("../swift_api.rs");
    let analyze = source
        .split("pub fn analyze_project(")
        .nth(1)
        .and_then(|source| source.split("impl SwiftReport").next())
        .expect("Swift analyze_project body");

    assert_eq!(analyze.matches("VisiblePathSnapshot::new(").count(), 1);
    assert_eq!(analyze.matches("discover_files_from_visible(").count(), 1);
    assert_eq!(analyze.matches("collect_swift_facts(").count(), 1);
    assert_eq!(
        analyze
            .matches("build_with_plan_files_prepared_config_and_swift_facts(")
            .count(),
        1
    );
    assert!(!analyze.contains("build_with_plan_and_config("));
}

#[test]
fn test_command_handles_root_and_named_packages() {
    assert_eq!(test_command("", "T"), "swift test --filter '^T\\.'");
    assert_eq!(test_command(".", "T"), "swift test --filter '^T\\.'");
    assert_eq!(
        test_command("pkg/core", "T"),
        "swift test --package-path 'pkg/core' --filter '^T\\.'"
    );
}

#[test]
fn test_command_escapes_regex_metacharacters() {
    assert_eq!(
        test_command("pkg", "App.Tests"),
        "swift test --package-path 'pkg' --filter '^App\\.Tests\\.'"
    );
}

#[test]
fn test_command_escapes_quotes_in_package_path() {
    assert_eq!(
        test_command("bob's app", "T"),
        "swift test --package-path 'bob'\\''s app' --filter '^T\\.'"
    );
}
