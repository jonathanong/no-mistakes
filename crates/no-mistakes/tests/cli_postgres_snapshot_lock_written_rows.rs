use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "postgres-lock-written-rows-only";

fn snapshot(name: &str, cases: &[(&str, &str, &str)]) {
    let rows = cases
        .iter()
        .map(|(label, fixture, config)| {
            (
                *label,
                rules_path(RULE, "fixture", fixture),
                rules_path(RULE, "configs", config),
            )
        })
        .collect::<Vec<_>>();
    assert_saved_fixture(name, &rows);
}

#[test]
fn postgres_lock_written_rows_relation_check_json_snapshot() {
    snapshot(
        "postgres_lock_written_rows_relation",
        &[
            ("unwritten", "row-unwritten", "fail.yml"),
            ("written", "row-written", "fail.yml"),
            ("different_key", "row-key-mismatch", "fail.yml"),
            ("separate_transactions", "isolated-transactions", "fail.yml"),
            ("all_strengths", "lock-strengths", "fail.yml"),
        ],
    );
}

#[test]
fn postgres_lock_written_rows_table_check_json_snapshot() {
    snapshot(
        "postgres_lock_written_rows_table",
        &[("table_lock", "table-lock", "fail.yml")],
    );
}

#[test]
fn postgres_lock_written_rows_advisory_check_json_snapshot() {
    snapshot(
        "postgres_lock_written_rows_advisory",
        &[
            ("constant", "constant-advisory", "fail.yml"),
            ("keyed", "keyed-advisory", "fail.yml"),
        ],
    );
}

#[test]
fn postgres_lock_written_rows_transaction_check_json_snapshot() {
    snapshot(
        "postgres_lock_written_rows_transaction",
        &[
            ("autocommit", "outside-transaction", "fail.yml"),
            ("transaction", "row-written", "fail.yml"),
            ("unresolved_fail", "unresolved", "fail.yml"),
            ("unresolved_advise", "unresolved", "advise.yml"),
            ("unresolved_ignore", "unresolved", "ignore.yml"),
            ("line_suppressions", "suppressed", "fail.yml"),
            ("file_suppression", "suppressed-file", "fail.yml"),
            ("line_suppressions_fail", "suppressed", "checks-fail.yml"),
            (
                "file_suppression_fail",
                "suppressed-file",
                "checks-fail.yml",
            ),
        ],
    );
}

#[test]
fn postgres_lock_written_rows_options_check_json_snapshot() {
    snapshot(
        "postgres_lock_written_rows_options",
        &[
            ("checks_fail", "row-unwritten", "checks-fail.yml"),
            ("checks_ignore", "row-unwritten", "checks-ignore.yml"),
            ("relation_allow", "row-unwritten", "allow.yml"),
            ("site_allow", "row-unwritten", "allow-call.yml"),
            ("key_allow", "constant-advisory", "allow-key.yml"),
            ("row_key_allow", "row-key-mismatch", "allow-row-key.yml"),
            ("write_sets", "row-unwritten", "write-sets.yml"),
            ("ignored_rows", "typed-membership", "ignore-row-checks.yml"),
            (
                "ignored_written",
                "unknown-write-predicate",
                "ignore-written-check.yml",
            ),
            (
                "ignored_outside",
                "outside-transaction",
                "ignore-outside-check.yml",
            ),
            ("all_ignored_dynamic", "unresolved", "checks-ignore.yml"),
            (
                "ignored_allow_is_stale",
                "row-unwritten",
                "ignored-allow.yml",
            ),
        ],
    );
}

#[test]
fn postgres_lock_written_rows_proofs_check_json_snapshot() {
    snapshot(
        "postgres_lock_written_rows_proofs",
        &[
            ("uncalled_write", "nested-write", "checks-fail.yml"),
            ("prepared_write", "prepared-write", "checks-fail.yml"),
            ("opaque_write", "unknown-write", "checks-fail.yml"),
            (
                "unknown_write_predicate",
                "unknown-write-predicate",
                "checks-fail.yml",
            ),
            ("cte_relation", "cte-relation", "checks-fail.yml"),
            ("committed", "committed", "checks-fail.yml"),
            ("changed_search_path", "search-path", "checks-fail.yml"),
            ("temporary_shadow", "temp-shadow", "checks-fail.yml"),
            (
                "placeholder_provenance_fail",
                "placeholder-provenance",
                "placeholder-provenance-fail.yml",
            ),
            (
                "placeholder_provenance_advise",
                "placeholder-provenance",
                "placeholder-provenance-advise.yml",
            ),
            (
                "placeholder_provenance_ignore",
                "placeholder-provenance",
                "placeholder-provenance-ignore.yml",
            ),
            (
                "placeholder_table_lock",
                "placeholder-table-lock",
                "placeholder-table-lock.yml",
            ),
            ("sql_lock_forms", "sql-lock-forms", "checks-fail.yml"),
            ("destructured_keys", "destructured-keys", "checks-fail.yml"),
            ("early_exit", "early-exit", "checks-fail.yml"),
            ("key_proofs", "key-proof", "checks-fail.yml"),
            (
                "key_proofs_unknown_ignored",
                "key-proof",
                "key-proof-ignore.yml",
            ),
        ],
    );
}

#[test]
fn postgres_lock_written_rows_executors_check_json_snapshot() {
    snapshot(
        "postgres_lock_written_rows_executors",
        &[
            ("defaults", "executor-selection", "executor-defaults.yml"),
            ("extras", "executor-selection", "executor-extras.yml"),
            (
                "replacement_warnings",
                "executor-selection",
                "executor-replacement.yml",
            ),
            (
                "excluded_default",
                "executor-selection",
                "executor-excluded-default.yml",
            ),
            (
                "all_defaults_excluded_scoped_retained",
                "executor-selection",
                "executor-all-excluded-scoped.yml",
            ),
            (
                "unused_exclusion_warning",
                "executor-unused-exclusion",
                "executor-unused-exclusion.yml",
            ),
            (
                "project_inherited_extras_and_exclusion",
                "executor-selection",
                "executor-project-inherited.yml",
            ),
        ],
    );
}

#[test]
fn lock_written_rows_executor_replacement_and_extension_cannot_both_be_configured() {
    let root = rules_path(RULE, "fixture", "executor-selection");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(rules_path(RULE, "configs", "executor-conflict.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let diagnostic = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    for expected in [
        RULE,
        "executorNames",
        "extraExecutorNames",
        "cannot both be set",
    ] {
        assert!(diagnostic.contains(expected), "{diagnostic}");
    }
}

#[test]
fn postgres_lock_written_rows_hashed_keys_check_json_snapshot() {
    snapshot(
        "postgres_lock_written_rows_hashed_keys",
        &[("hashes", "hashed-advisory", "checks-fail.yml")],
    );
}

#[test]
fn postgres_lock_written_rows_atomic_cte_check_json_snapshot() {
    snapshot(
        "postgres_lock_written_rows_atomic_cte",
        &[("atomic", "atomic-cte", "checks-fail.yml")],
    );
}

#[test]
fn postgres_lock_written_rows_resource_options_check_json_snapshot() {
    snapshot(
        "postgres_lock_written_rows_resource_options",
        &[(
            "resource_options",
            "resource-options",
            "resource-options.yml",
        )],
    );
}

#[test]
fn postgres_lock_written_rows_helpers_check_json_snapshot() {
    snapshot(
        "postgres_lock_written_rows_helpers",
        &[
            ("unwritten", "helper-unwritten", "checks-fail.yml"),
            ("completion", "helper-completion", "checks-fail.yml"),
            (
                "imported_completion",
                "helper-imported-completion",
                "checks-fail.yml",
            ),
            ("reexports", "helper-reexport", "checks-fail.yml"),
            (
                "unsupported",
                "helper-unsupported-shapes",
                "checks-fail.yml",
            ),
            ("different_key", "helper-key-mismatch", "checks-fail.yml"),
            ("written", "helper-written", "checks-fail.yml"),
            ("dead_write", "helper-dead-write", "checks-fail.yml"),
            ("caller_exited", "helper-caller-exited", "checks-fail.yml"),
            (
                "imported_caller_exited",
                "helper-imported-caller-exited",
                "checks-fail.yml",
            ),
            (
                "option_override",
                "helper-option-override",
                "helper-option-override.yml",
            ),
            (
                "imported_unwritten",
                "helper-imported-unwritten",
                "checks-fail.yml",
            ),
            (
                "imported_written",
                "helper-imported-written",
                "checks-fail.yml",
            ),
        ],
    );
}

#[test]
fn postgres_lock_written_rows_resources_check_json_snapshot() {
    snapshot(
        "postgres_lock_written_rows_resources",
        &[("resources", "resource-transactions", "checks-fail.yml")],
    );
}
