use super::*;

#[test]
fn missing_changed_file_is_classified_without_appending_a_deleted_path() {
    let mut missing = args(&["src/missing.ts"]);
    missing.root = generic_only_fixture();
    missing.generic_only = true;
    let report = generate_impacted_checks(&missing).unwrap();
    assert_eq!(report.changed_files, ["src/missing.ts"]);
    assert!(report.checks.is_empty());
    assert_eq!(
        report.empty_result.as_ref().unwrap().code,
        "no-impacted-checks"
    );
}

#[test]
fn run_diagnoses_requested_empty_result() {
    let mut a = args(&[]);
    a.json = true;
    a.diagnose_empty = true;
    run(a).unwrap();
}

#[test]
fn run_returns_generation_errors_without_empty_diagnostics() {
    let mut a = args(&[]);
    a.root = multi_framework_fixture();
    a.config = Some(a.root.join("invalid.no-mistakes.yml"));
    a.diagnose_empty = true;
    assert!(run(a).is_err());
}
