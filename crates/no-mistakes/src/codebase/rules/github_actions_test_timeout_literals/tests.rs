use super::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-test-timeout-literals/fixture")
            .join(name),
    )
}

fn scan_source(source: &str, yaml: &str) -> Vec<RuleFinding> {
    scan::check_source(
        ".github/workflows/ci.test.mts",
        source,
        &compile_options(serde_yaml::from_str(yaml).unwrap()).unwrap(),
    )
}

// Output cases moved to cli_gha_snapshot_test_timeout_literals.

#[test]
fn property_to_be_is_a_finding() {
    let findings = scan_source("expect(step?.['timeout-minutes']).toBe(10)\n", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0]
        .message
        .contains("duplicates a timeout-minutes value"));
}

#[test]
fn unquoted_bracket_property_to_be_is_a_finding() {
    assert_eq!(
        scan_source("expect(job['timeout-minutes']).toBe(10)\n", "{}").len(),
        1
    );
}

#[test]
fn double_quoted_bracket_property_to_be_is_a_finding() {
    assert_eq!(
        scan_source(r#"expect(job["timeout-minutes"]).toBe(10)"#, "{}").len(),
        1
    );
}

#[test]
fn property_to_equal_is_a_finding() {
    let findings = scan_source("expect(job.timeoutMinutes).toEqual(8)\n", "{}");
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn property_to_strict_equal_is_a_finding() {
    assert_eq!(
        scan_source("expect(job.timeoutMinutes).toStrictEqual(10)\n", "{}").len(),
        1
    );
}

#[test]
fn to_contain_with_digit_is_a_finding() {
    let findings = scan_source(
        r#"expect(job?.['timeout-minutes']).toContain("&& '45'")"#,
        "{}",
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn range_assertion_passes() {
    assert!(scan_source(
        "expect(step?.['timeout-minutes']).toBeLessThanOrEqual(job?.['timeout-minutes'])\n",
        "{}"
    )
    .is_empty());
}

#[test]
fn to_contain_without_digit_passes() {
    assert!(scan_source(
        "expect(job?.['timeout-minutes']).toContain('HOST_SUPERVISION_READY')\n",
        "{}"
    )
    .is_empty());
}

// Output cases moved to cli_gha_snapshot_test_timeout_literals.

// Output cases moved to cli_gha_snapshot_test_timeout_literals.

#[test]
fn file_disable_checks_the_shared_header_even_when_an_allow_would_match() {
    use crate::codebase::ts_source::{FileInventory, SourceStore};
    use crate::diagnostics::InvocationObserver;
    use std::sync::Arc;

    let root = fixture("file-disabled");
    let path = root.join(".github/workflows/ci.test.mts");
    let files = std::slice::from_ref(&path);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let observer = InvocationObserver::new(true);
    let inventory = Arc::new(FileInventory::from_paths(files));
    let sources = SourceStore::new_observed(inventory, Some(Arc::clone(&observer)));

    let findings = check_with_files_and_sources(&root, &config, files, &sources, false).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
    let work = observer.snapshot().work;
    assert_eq!(work.get("suppression.header_builds"), Some(&1));
    assert_eq!(work.get("suppression.line_index_builds"), None);
    assert_eq!(work.get("suppression.standalone_fallbacks"), None);
}

#[test]
fn deferred_suppression_still_emits_disabled_file() {
    let root = fixture("file-disabled");
    let path = root.join(".github/workflows/ci.test.mts");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&path));
    let opts = compile_options(Options::default()).unwrap();
    let findings = scan::check_file(&root, &path, &opts, &sources, true);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 2);
}

#[test]
fn unreadable_test_is_skipped() {
    let root = fixture("pass");
    let path = root.join(".github/workflows/missing.test.mts");
    let sources = super::super::source_store_for_files(&[]);
    let opts = compile_options(Options::default()).unwrap();
    assert!(scan::check_file(&root, &path, &opts, &sources, false).is_empty());
}

#[test]
fn yaml_without_digits_is_silent() {
    assert!(scan_source("timeout-minutes: fromJSON(vars.BUDGET)\n", "{}").is_empty());
}

#[test]
fn quoted_yaml_numeric_string_is_a_finding() {
    assert_eq!(scan_source("timeout-minutes: '15'\n", "{}").len(), 1);
    assert_eq!(scan_source("timeout-minutes: \"15\"\n", "{}").len(), 1);
}

#[test]
fn longer_yaml_key_is_silent() {
    assert!(scan_source("default-timeout-minutes: 15\n", "{}").is_empty());
}

#[test]
fn equality_without_literal_is_silent() {
    assert!(scan_source("expect(job.timeoutMinutes).toBe(expectedTimeout)\n", "{}").is_empty());
}

#[test]
fn full_line_comment_is_silent() {
    assert!(scan_source("// expect(job.timeoutMinutes).toBe(10)\n", "{}").is_empty());
    assert!(scan_source("// previous value: timeout-minutes: 15\n", "{}").is_empty());
}

#[test]
fn quoted_yaml_keys_are_findings() {
    assert_eq!(scan_source("'timeout-minutes': 15\n", "{}").len(), 1);
    assert_eq!(scan_source("\"timeout-minutes\": 15\n", "{}").len(), 1);
}

#[test]
fn same_line_independent_expects_are_silent() {
    assert!(scan_source(
        "expect(job.timeoutMinutes).toBeDefined(); expect(retries).toBe(3)\n",
        "{}"
    )
    .is_empty());
}

#[test]
fn yaml_number_must_consume_the_scalar() {
    assert!(scan_source("timeout-minutes: 15m\n", "{}").is_empty());
    assert!(scan_source("timeout-minutes: '15m'\n", "{}").is_empty());
    assert_eq!(scan_source("timeout-minutes: 15\n", "{}").len(), 1);
}

#[test]
fn negated_equality_is_a_finding() {
    assert_eq!(
        scan_source("expect(job.timeoutMinutes).not.toBe(10)\n", "{}").len(),
        1
    );
    assert_eq!(
        scan_source("expect(job.timeoutMinutes).not.toEqual(8)\n", "{}").len(),
        1
    );
    assert_eq!(
        scan_source("expect(job.timeoutMinutes).not.toStrictEqual(10)\n", "{}").len(),
        1
    );
}

#[test]
fn allow_entry_default_is_empty() {
    let allow = AllowEntry::default();
    let cloned = allow.clone();
    assert!(cloned.file.is_empty());
    assert!(cloned.text.is_empty());
    assert!(cloned.reason.is_empty());
}
