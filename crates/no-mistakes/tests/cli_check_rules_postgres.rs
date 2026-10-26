use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-lock-ordering/fixture")
            .join(scenario),
    )
}

fn config(rule: &str, name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(rule)
        .join("config")
        .join(name)
}

fn check(root: &Path, config: &Path) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config)
        .output()
        .unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

const RULE: &str = "postgres-lock-ordering";
const MISSING: &str =
    "postgres_executor_selection_fails_loudly_unless_explicitly_disabled-missing.yml";
const OPT_OUT: &str =
    "postgres_executor_selection_fails_loudly_unless_explicitly_disabled-opt-out.yml";
const COMMON: &str = "postgres_lock_ordering_fails_for_multi_row_for_update.yml";
const SCOPED: &str = "postgres_lock_ordering_scans_factory_and_typed_executors.yml";
const SUBPATH: &str = "postgres_lock_ordering_scans_scoped_executors_from_import_subpaths.yml";
const OUTSIDE_ONLY: &str =
    "postgres_lock_ordering_ignores_relative_imports_outside_the_package.yml";
const UNMATCHED_INSIDE: &str =
    "postgres_unmatched_executor_names_count_relative_imports_inside_the_package-inside.yml";
const UNMATCHED_OUTSIDE: &str =
    "postgres_unmatched_executor_names_count_relative_imports_inside_the_package-outside.yml";
const CATALOG: &str = "postgres_lock_ordering_accepts_pinned_unique_keys_with_in_filters.yml";

#[test]
fn postgres_executor_selection_fails_loudly_unless_explicitly_disabled() {
    let root = fixture("executor-selection");
    let missing = check(&root, &config(RULE, MISSING));
    let diagnostic = format!(
        "{}{}",
        stdout(&missing),
        String::from_utf8_lossy(&missing.stderr)
    );
    assert!(!missing.status.success(), "{diagnostic}");
    assert!(
        diagnostic.contains("set importSpecifier (or executorNames)"),
        "{diagnostic}"
    );
    let opted_out = check(&root, &config(RULE, OPT_OUT));
    assert!(opted_out.status.success(), "{}", stdout(&opted_out));
}

#[test]
fn postgres_lock_ordering_fails_for_multi_row_for_update() {
    let root = fixture("fail");
    let out = check(&root, &config(RULE, COMMON));
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(body.contains("lock.ts"), "{body}");
    assert!(body.contains("ABBA"), "{body}");
}

#[test]
fn postgres_lock_ordering_passes_with_order_by() {
    let root = fixture("pass-order");
    let out = check(&root, &config(RULE, COMMON));
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_lock_ordering_passes_with_skip_locked() {
    let root = fixture("pass-skip");
    let out = check(&root, &config(RULE, COMMON));
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_lock_ordering_passes_with_safe_directive() {
    let root = fixture("pass-directive");
    let out = check(&root, &config(RULE, COMMON));
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_lock_ordering_unparseable_has_distinct_diagnostic() {
    let root = fixture("unparseable");
    let out = check(&root, &config(RULE, COMMON));
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(body.contains("parseable"), "{body}");
    assert!(!body.contains("ABBA"), "{body}");
}

#[test]
fn postgres_lock_ordering_json_has_rule_id() {
    let root = fixture("fail");
    let out = Command::new(bin())
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let body = stdout(&out);
    assert!(body.contains(RULE), "{body}");
    assert!(!out.status.success());
}

#[test]
fn postgres_lock_ordering_filesystem_runner_discovers_files() {
    let root = fixture("fail");
    let findings = no_mistakes::codebase::rules::run_filesystem_rules(&root, None).unwrap();
    let body = format!("{findings:?}");
    assert!(!findings.is_empty(), "expected findings");
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule == no_mistakes::codebase::rules::POSTGRES_LOCK_ORDERING),
        "{body}"
    );
}

#[test]
fn postgres_lock_ordering_scans_factory_and_typed_executors() {
    let root = fixture("fail-scoped-executors");
    let out = check(&root, &config(RULE, SCOPED));
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert_eq!(body.matches("ABBA").count(), 2, "{body}");
}

#[test]
fn postgres_lock_ordering_scans_scoped_executors_from_import_subpaths() {
    let root = fixture("fail-scoped-subpath-executors");
    let out = check(&root, &config(RULE, SUBPATH));
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains("src/lock.ts"), "{body}");
    assert!(body.contains("src/open.ts"), "{body}");
}

#[test]
fn postgres_lock_ordering_ignores_scoped_executors_by_default() {
    // Same source as `fail-scoped-executors`, scoped options absent.
    let root = fixture("pass-scoped-defaults");
    let out = check(&root, &config(RULE, COMMON));
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_lock_ordering_scans_relative_imports_inside_the_package() {
    let root = fixture("fail-scoped-relative-executors");
    let out = check(&root, &config(RULE, SCOPED));
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert_eq!(body.matches("ABBA").count(), 2, "{body}");
    assert!(body.contains("packages/db/src/orders/lock.ts"), "{body}");
    assert!(!body.contains("packages/other"), "{body}");
}

#[test]
fn postgres_lock_ordering_ignores_relative_imports_outside_the_package() {
    let root = fixture("fail-scoped-relative-executors");
    let out = check(&root, &config(RULE, OUTSIDE_ONLY));
    let body = stdout(&out);
    assert!(out.status.success(), "exit non-zero: {body}");
    assert!(!body.contains("ABBA"), "{body}");
}

#[test]
fn postgres_unmatched_executor_names_count_relative_imports_inside_the_package() {
    let root = fixture("fail-scoped-relative-executors");
    let inside = check(&root, &config(RULE, UNMATCHED_INSIDE));
    let inside_body = stdout(&inside);
    assert!(!inside.status.success(), "expected exit 1: {inside_body}");
    assert!(inside_body.contains("ABBA"), "{inside_body}");
    assert!(!inside_body.contains("was not imported"), "{inside_body}");
    let outside = check(&root, &config(RULE, UNMATCHED_OUTSIDE));
    let outside_body = stdout(&outside);
    assert!(!outside.status.success(), "expected exit 1: {outside_body}");
    assert!(outside_body.contains("was not imported"), "{outside_body}");
    assert!(!outside_body.contains("0 file(s)"), "{outside_body}");
    assert!(!outside_body.contains("ABBA"), "{outside_body}");
}

#[test]
fn postgres_lock_ordering_checks_no_key_update() {
    let pass = check(&fixture("pass-key-strengths"), &config(RULE, COMMON));
    assert!(pass.status.success(), "exit non-zero: {}", stdout(&pass));
    let fail = check(&fixture("fail-key-strengths"), &config(RULE, COMMON));
    let body = stdout(&fail);
    assert!(!fail.status.success(), "expected exit 1: {body}");
    assert!(body.contains("ABBA"), "{body}");
    assert!(!body.contains("parseable"), "{body}");
}

#[test]
fn postgres_lock_ordering_checks_multi_target_of_lists() {
    let pass = check(&fixture("pass-multi-target"), &config(RULE, COMMON));
    assert!(pass.status.success(), "exit non-zero: {}", stdout(&pass));
    let fail = check(&fixture("fail-multi-target"), &config(RULE, COMMON));
    let body = stdout(&fail);
    assert!(!fail.status.success(), "expected exit 1: {body}");
    assert!(body.contains("ABBA"), "{body}");
    assert!(!body.contains("parseable"), "{body}");
}

#[test]
fn postgres_lock_ordering_accepts_pinned_unique_keys_with_in_filters() {
    let pass = check(&fixture("pass-unique-key-filter"), &config(RULE, CATALOG));
    assert!(pass.status.success(), "exit non-zero: {}", stdout(&pass));
    // Partial pins, OR pins, other-table pins, and a missing catalog still fail closed.
    for (scenario, saved) in [
        ("fail-unique-key-partial", CATALOG),
        ("fail-unique-key-no-catalog", COMMON),
    ] {
        let fail = check(&fixture(scenario), &config(RULE, saved));
        let body = stdout(&fail);
        assert!(
            !fail.status.success(),
            "{scenario}: expected exit 1: {body}"
        );
        assert!(body.contains("ABBA"), "{scenario}: {body}");
    }
}

#[test]
fn postgres_lock_ordering_checks_the_locked_table_beside_a_lateral_join() {
    let pass = check(&fixture("pass-catalog-lateral"), &config(RULE, CATALOG));
    assert!(pass.status.success(), "exit non-zero: {}", stdout(&pass));
    let fail = check(&fixture("fail-catalog-lateral"), &config(RULE, CATALOG));
    let body = stdout(&fail);
    assert!(!fail.status.success(), "expected exit 1: {body}");
    assert!(body.contains("schema-catalog unique-key order"), "{body}");
}

#[test]
fn postgres_lock_ordering_pins_lock_targets_through_join_keys_and_interpolations() {
    let pass = check(&fixture("pass-join-pinned"), &config(RULE, CATALOG));
    assert!(pass.status.success(), "exit non-zero: {}", stdout(&pass));
    let fail = check(&fixture("fail-join-pinned"), &config(RULE, CATALOG));
    let body = stdout(&fail);
    assert!(!fail.status.success(), "expected exit 1: {body}");
    assert!(body.contains("ABBA"), "{body}");
}

#[test]
fn postgres_lock_ordering_reports_interpolated_relations_distinctly() {
    let fail = check(
        &fixture("fail-catalog-interpolated"),
        &config(RULE, CATALOG),
    );
    let body = stdout(&fail);
    assert!(!fail.status.success(), "expected exit 1: {body}");
    assert!(body.contains("name is interpolated"), "{body}");
    assert!(!body.contains("unique-key order"), "{body}");
}
