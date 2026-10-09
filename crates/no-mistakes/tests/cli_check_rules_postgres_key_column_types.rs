use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-key-column-types/fixture")
            .join(scenario),
    )
}

fn check(root: &PathBuf, json: bool) -> Output {
    let mut command = Command::new(bin());
    command
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"));
    if json {
        command.args(["--format", "json"]);
    }
    command.output().unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

const RULE: &str = "postgres-key-column-types";

#[test]
fn postgres_key_column_types_passes_allowed_keys_and_partition_leaf() {
    let root = fixture("pass");
    let output = check(&root, false);
    assert!(output.status.success(), "{}", stdout(&output));
}

#[test]
fn postgres_key_column_types_reports_composite_primary_and_foreign_keys() {
    let root = fixture("fail");
    let output = check(&root, false);
    let body = stdout(&output);
    assert!(!output.status.success(), "expected failure: {body}");
    assert!(body.contains(RULE), "{body}");
    assert!(
        body.contains("constraint:composite_orders.composite_orders_pkey"),
        "{body}"
    );
    assert!(
        body.contains("text column tag, character varying column locale"),
        "{body}"
    );
    assert!(
        body.contains("constraint:legacy_order.legacy_order_fkey"),
        "{body}"
    );
    assert!(
        body.contains("partitioned_parent.partitioned_parent_pkey"),
        "{body}"
    );
    assert!(
        body.contains("partitioned_subparent.partitioned_subparent_pkey"),
        "nested partitioned parents are checked: {body}"
    );
    assert!(
        !body.contains("partitioned_child"),
        "partition leaf duplicated a parent finding: {body}"
    );
    assert!(
        !body.contains("partitioned_grandchild"),
        "nested partition leaf duplicated a parent finding: {body}"
    );
}

#[test]
fn postgres_key_column_types_checks_local_foreign_keys_on_partition_leaves() {
    let root = fixture("fail-local-partition-fk");
    let output = check(&root, false);
    let body = stdout(&output);
    assert!(!output.status.success(), "expected failure: {body}");
    assert!(
        body.contains("constraint:partitioned_leaf.partitioned_leaf_external_fkey"),
        "{body}"
    );
    assert!(body.contains("text column external_id"), "{body}");
}

#[test]
fn postgres_key_column_types_supports_constraint_allow_entries() {
    let root = fixture("pass-suppressed");
    let output = check(&root, false);
    assert!(output.status.success(), "{}", stdout(&output));
}

#[test]
fn postgres_key_column_types_supports_jsonc_file_and_line_suppressions() {
    for scenario in ["pass-file-disabled", "pass-line-disabled"] {
        let root = fixture(scenario);
        let output = check(&root, false);
        assert!(output.status.success(), "{scenario}: {}", stdout(&output));
    }
}

#[test]
fn postgres_key_column_types_reports_stale_allow_entries() {
    let root = fixture("fail-stale-allow");
    let output = check(&root, false);
    let body = stdout(&output);
    assert!(!output.status.success(), "expected failure: {body}");
    assert!(
        body.contains("stale postgres-key-column-types allow entry"),
        "{body}"
    );
}

#[test]
fn postgres_key_column_types_json_is_deterministic() {
    let root = fixture("fail");
    let first = check(&root, true);
    let second = check(&root, true);
    let body = stdout(&first);
    assert!(!first.status.success(), "expected failure: {body}");
    assert!(body.contains(RULE), "{body}");
    assert_eq!(body, stdout(&second));
}
