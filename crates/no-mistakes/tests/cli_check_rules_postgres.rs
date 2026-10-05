use std::path::PathBuf;
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

fn check(root: &PathBuf, yaml: &str) -> Output {
    let config = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
    std::fs::write(config.path(), yaml).unwrap();
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config.path())
        .output()
        .unwrap()
}

fn check_fixture_config(root: &PathBuf, name: &str) -> Output {
    let yaml = std::fs::read_to_string(root.join(name)).unwrap();
    check(root, &yaml)
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

const RULE: &str = "postgres-lock-ordering";

#[test]
fn postgres_executor_selection_fails_loudly_unless_explicitly_disabled() {
    let root = fixture("executor-selection");
    let missing = check_fixture_config(&root, "missing.yml");
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
    let opted_out = check_fixture_config(&root, "opt-out.yml");
    assert!(opted_out.status.success(), "{}", stdout(&opted_out));
}

#[test]
fn postgres_lock_ordering_fails_for_multi_row_for_update() {
    let root = fixture("fail");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(body.contains("lock.ts"), "{body}");
    assert!(body.contains("ABBA"), "{body}");
}

#[test]
fn postgres_lock_ordering_passes_with_order_by() {
    let root = fixture("pass-order");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_lock_ordering_passes_with_skip_locked() {
    let root = fixture("pass-skip");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_lock_ordering_passes_with_safe_directive() {
    let root = fixture("pass-directive");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_lock_ordering_unparseable_has_distinct_diagnostic() {
    let root = fixture("unparseable");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
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
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert_eq!(body.matches("ABBA").count(), 2, "{body}");
}

#[test]
fn postgres_lock_ordering_scans_scoped_executors_from_import_subpaths() {
    let root = fixture("fail-scoped-subpath-executors");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&out);
    assert!(!out.status.success(), "expected exit 1: {body}");
    assert!(body.contains("src/lock.ts"), "{body}");
    assert!(body.contains("src/open.ts"), "{body}");
}

#[test]
fn postgres_lock_ordering_ignores_scoped_executors_by_default() {
    // Same source as `fail-scoped-executors`, scoped options absent.
    let root = fixture("pass-scoped-defaults");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_lock_ordering_checks_no_key_update() {
    let pass = check_fixture_config(&fixture("pass-key-strengths"), ".no-mistakes.yml");
    assert!(pass.status.success(), "exit non-zero: {}", stdout(&pass));
    let fail = check_fixture_config(&fixture("fail-key-strengths"), ".no-mistakes.yml");
    let body = stdout(&fail);
    assert!(!fail.status.success(), "expected exit 1: {body}");
    assert!(body.contains("ABBA"), "{body}");
    assert!(!body.contains("parseable"), "{body}");
}

#[test]
fn postgres_lock_ordering_checks_multi_target_of_lists() {
    let pass = check_fixture_config(&fixture("pass-multi-target"), ".no-mistakes.yml");
    assert!(pass.status.success(), "exit non-zero: {}", stdout(&pass));
    let fail = check_fixture_config(&fixture("fail-multi-target"), ".no-mistakes.yml");
    let body = stdout(&fail);
    assert!(!fail.status.success(), "expected exit 1: {body}");
    assert!(body.contains("ABBA"), "{body}");
    assert!(!body.contains("parseable"), "{body}");
}

#[test]
fn postgres_lock_ordering_accepts_pinned_unique_keys_with_in_filters() {
    let pass = check_fixture_config(&fixture("pass-unique-key-filter"), ".no-mistakes.yml");
    assert!(pass.status.success(), "exit non-zero: {}", stdout(&pass));
    // Partial pins, OR pins, other-table pins, and a missing catalog still fail closed.
    for scenario in ["fail-unique-key-partial", "fail-unique-key-no-catalog"] {
        let fail = check_fixture_config(&fixture(scenario), ".no-mistakes.yml");
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
    let pass = check_fixture_config(&fixture("pass-catalog-lateral"), ".no-mistakes.yml");
    assert!(pass.status.success(), "exit non-zero: {}", stdout(&pass));
    let fail = check_fixture_config(&fixture("fail-catalog-lateral"), ".no-mistakes.yml");
    let body = stdout(&fail);
    assert!(!fail.status.success(), "expected exit 1: {body}");
    assert!(body.contains("schema-catalog unique-key order"), "{body}");
}

#[test]
fn postgres_lock_ordering_pins_lock_targets_through_join_keys_and_interpolations() {
    let pass = check_fixture_config(&fixture("pass-join-pinned"), ".no-mistakes.yml");
    assert!(pass.status.success(), "exit non-zero: {}", stdout(&pass));
    let fail = check_fixture_config(&fixture("fail-join-pinned"), ".no-mistakes.yml");
    let body = stdout(&fail);
    assert!(!fail.status.success(), "expected exit 1: {body}");
    assert!(body.contains("ABBA"), "{body}");
}

#[test]
fn postgres_lock_ordering_reports_interpolated_relations_distinctly() {
    let fail = check_fixture_config(&fixture("fail-catalog-interpolated"), ".no-mistakes.yml");
    let body = stdout(&fail);
    assert!(!fail.status.success(), "expected exit 1: {body}");
    assert!(body.contains("name is interpolated"), "{body}");
    assert!(!body.contains("unique-key order"), "{body}");
}
