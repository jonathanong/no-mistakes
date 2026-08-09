use super::*;

#[test]
fn run_all_keeps_forbidden_graph_files_outside_filesystem_skips() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/check-runner/forbidden-deps-ignores-filesystem-skip/fixture");
    let config = root.join(".no-mistakes.yml");
    let results = run_all(root, Some(config), None).unwrap();

    assert!(
        results
            .rules
            .iter()
            .any(|f| f.rule == no_mistakes::codebase::rules::FORBIDDEN_DEPENDENCIES),
        "expected forbidden-dependencies finding for file under filesystem skip"
    );
}

#[test]
fn run_all_keeps_dynamic_import_graph_within_filesystem_skips() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/check-runner/dynamic-import-respects-filesystem-skip/fixture");
    let config = root.join(".no-mistakes.yml");
    let results = run_all(root, Some(config), None).unwrap();
    let finding = results
        .rules
        .iter()
        .find(|finding| finding.file == "tests/scoped.test.ts")
        .expect("skipped dynamic import remains reportable as unresolved");

    assert_eq!(finding.import.as_deref(), Some("../skipped/target"));
    assert_eq!(finding.target, None);
}

#[test]
fn run_all_keeps_reachability_sources_outside_filesystem_skips() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/check-runner/required-reachability-ignores-filesystem-skip/fixture",
    );
    let config = root.join(".no-mistakes.yml");
    let results = run_all(root, Some(config), None).unwrap();

    assert!(results.rules.iter().any(|finding| {
        finding.rule == no_mistakes::codebase::rules::REQUIRED_ENTRYPOINT_REACHABILITY
            && finding.file == "sources/unreachable.ts"
            && finding.message.contains("not runtime-reachable")
    }));
}

#[test]
fn run_all_keeps_playwright_graph_files_outside_filesystem_skips() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/check-runner/playwright-graph-ignores-filesystem-skip/fixture");
    let config = root.join(".no-mistakes.yml");
    let results = run_all(root, Some(config), None).unwrap();

    assert!(!results.rules.iter().any(|finding| {
        finding.rule == no_mistakes::playwright::rules::PLAYWRIGHT_COVERAGE
            && finding.target.as_deref() == Some("data-testid=save")
    }));
    assert!(results.rules.iter().any(|finding| {
        finding.rule == no_mistakes::playwright::rules::PLAYWRIGHT_COVERAGE
            && finding.target.as_deref() == Some("data-testid=delete")
    }));
}
