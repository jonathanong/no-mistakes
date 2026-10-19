use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(rule: &str, scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(rule)
            .join("fixture")
            .join(scenario),
    )
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn check_json(root: &PathBuf) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", "json"])
        .output()
        .unwrap()
}

#[test]
fn postgres_required_predicates_json_has_rule_id() {
    let root = fixture("postgres-required-predicates", "fail");
    let out = check_json(&root);
    let body = stdout(&out);
    assert!(body.contains("postgres-required-predicates"), "{body}");
    assert!(!out.status.success());
}

#[test]
fn postgres_required_predicates_flags_missing_partition_key() {
    let root = fixture("postgres-required-predicates", "fail-partition");
    let out = check_json(&root);
    let body = stdout(&out);
    assert!(!out.status.success(), "{body}");
    assert!(body.contains("partition key column account_id"), "{body}");
}

#[test]
fn postgres_required_predicates_passes_constrained_partition_key() {
    let root = fixture("postgres-required-predicates", "pass-partition");
    let out = check_json(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_required_predicates_passes_suppressed_sql() {
    let root = fixture("postgres-required-predicates", "pass-suppressed");
    let out = check_json(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_required_predicates_flags_escaped_receiver_as_unanalyzable() {
    let root = fixture("postgres-required-predicates", "escaped-receiver");
    let out = check_json(&root);
    let body = stdout(&out);
    let err = stderr(&out);
    assert!(!out.status.success(), "{body}\n{err}");
    assert!(body.contains("unanalyzable"), "{body}");
    assert!(
        !body.contains("weren't proven"),
        "the warning stays off JSON stdout: {body}"
    );
    assert!(
        err.contains("warning: 1 member .query calls with SQL-looking text weren't proven to be executors; see docs/migrations/explicit-postgres-executors.md"),
        "{err}"
    );
}

#[test]
fn postgres_required_predicates_flags_stale_allow() {
    let root = fixture("postgres-required-predicates", "fail-stale-allow");
    let out = check_json(&root);
    let body = stdout(&out);
    assert!(!out.status.success(), "{body}");
    assert!(
        body.contains("stale postgres-required-predicates allow entry: table:missing"),
        "{body}"
    );
}

#[test]
fn postgres_sql_shape_policy_json_has_rule_id() {
    let root = fixture("postgres-sql-shape-policy", "fail");
    let out = check_json(&root);
    let body = stdout(&out);
    assert!(body.contains("postgres-sql-shape-policy"), "{body}");
    assert!(!out.status.success());
}

#[test]
fn postgres_sql_shape_policy_flags_opt_in_shapes() {
    let root = fixture("postgres-sql-shape-policy", "fail-shapes");
    let out = check_json(&root);
    let body = stdout(&out);
    assert!(!out.status.success(), "{body}");
    assert!(body.contains("NOT IN (SELECT"), "{body}");
    assert!(body.contains("COUNT(*) compared with 0/1"), "{body}");
}

#[test]
fn postgres_sql_shape_policy_passes_opt_in_shapes() {
    let root = fixture("postgres-sql-shape-policy", "pass-shapes");
    let out = check_json(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_sql_shape_policy_passes_suppressed_shapes() {
    let root = fixture("postgres-sql-shape-policy", "pass-suppressed-shapes");
    let out = check_json(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_explicit_columns_json_has_rule_id() {
    let root = fixture("postgres-explicit-columns", "fail");
    let out = check_json(&root);
    let body = stdout(&out);
    assert!(body.contains("postgres-explicit-columns"), "{body}");
    assert!(body.contains("40 columns of orders"), "{body}");
    assert!(!out.status.success());
}

#[test]
fn postgres_explicit_columns_passes_narrow_and_wrapped_stars() {
    let root = fixture("postgres-explicit-columns", "pass");
    let out = check_json(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_explicit_columns_passes_suppressed_stars() {
    let root = fixture("postgres-explicit-columns", "pass-suppressed");
    let out = check_json(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_explicit_columns_flags_embedded_sql() {
    let root = fixture("postgres-explicit-columns", "embedded");
    let out = check_json(&root);
    let body = stdout(&out);
    assert!(body.contains("40 columns of orders"), "{body}");
    assert!(!out.status.success());
}

#[test]
fn postgres_generated_column_predicates_json_has_rule_id() {
    let root = fixture("postgres-generated-column-predicates", "fail");
    let out = check_json(&root);
    let body = stdout(&out);
    assert!(
        body.contains("postgres-generated-column-predicates"),
        "{body}"
    );
    assert!(
        body.contains(
            "WHERE filters orders.created_at, which is generated from uuid_extract_timestamp(id)"
        ),
        "{body}"
    );
    assert!(
        body.contains("ORDER BY orders.created_at sorts by a column generated from id"),
        "{body}"
    );
    assert!(!out.status.success());
}

#[test]
fn postgres_generated_column_predicates_passes_primary_key_predicates() {
    let root = fixture("postgres-generated-column-predicates", "pass");
    let out = check_json(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_generated_column_predicates_passes_suppressed_predicates() {
    let root = fixture("postgres-generated-column-predicates", "pass-suppressed");
    let out = check_json(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_generated_column_predicates_flags_embedded_sql() {
    let root = fixture("postgres-generated-column-predicates", "embedded");
    let out = check_json(&root);
    let body = stdout(&out);
    assert!(
        body.contains(
            "WHERE filters orders.created_at, which is generated from uuid_extract_timestamp(id)"
        ),
        "{body}"
    );
    assert!(!out.status.success());
}

#[test]
fn postgres_idempotent_insert_json_has_rule_id() {
    let root = fixture("postgres-idempotent-insert", "fail");
    let out = check_json(&root);
    let body = stdout(&out);
    assert!(body.contains("postgres-idempotent-insert"), "{body}");
    assert!(!out.status.success());
}

#[test]
fn postgres_idempotent_insert_passes_do_nothing() {
    let root = fixture("postgres-idempotent-insert", "pass-do-nothing");
    let out = check_json(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_idempotent_insert_passes_partial_where_noop() {
    let root = fixture("postgres-idempotent-insert", "pass-partial-where");
    let out = check_json(&root);
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn postgres_idempotent_insert_flags_volatile_where_proof() {
    let root = fixture("postgres-idempotent-insert", "fail-volatile-proof");
    let out = check_json(&root);
    let body = stdout(&out);
    assert!(!out.status.success(), "{body}");
    assert!(body.contains("re-fire"), "{body}");
}

#[test]
fn postgres_idempotent_insert_expression_arbiters_fail_closed() {
    let root = fixture("postgres-idempotent-insert", "expression-arbiters");
    let out = check_json(&root);
    assert!(!out.status.success(), "{}", stdout(&out));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    let findings = report["rules"].as_array().unwrap();
    for file in [
        "coupled_partial.sql",
        "expression.sql",
        "mixed.sql",
        "parenthesized.sql",
        "quoted.sql",
        "plain_column_control.sql",
        "unsupported.sql",
    ] {
        assert!(
            findings.iter().any(|finding| finding["file"]
                .as_str()
                .is_some_and(|path| path.ends_with(file))),
            "missing {file}: {report}"
        );
    }
    assert!(
        findings.iter().any(|finding| finding["file"]
            .as_str()
            .is_some_and(|path| path.ends_with("unsupported.sql"))
            && finding["message"]
                .as_str()
                .is_some_and(|message| message.contains("cannot prove"))),
        "{report}"
    );
}

#[test]
fn postgres_idempotent_insert_expression_safe_updates_pass() {
    let root = fixture("postgres-idempotent-insert", "expression-arbiters-safe");
    let out = check_json(&root);
    assert!(out.status.success(), "{}", stdout(&out));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
}

#[test]
fn postgres_required_predicates_suppresses_the_unproven_member_query_warning() {
    let root = fixture("postgres-required-predicates", "unproven-suppressed");
    let out = check_json(&root);
    let body = stdout(&out);
    let err = stderr(&out);
    assert!(out.status.success(), "{body}\n{err}");
    assert!(!err.contains("weren't proven"), "{err}");
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(report.get("unprovenMemberQuerySql").is_none(), "{report}");
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
}

#[test]
fn postgres_required_predicates_counts_one_unproven_member_across_profiles() {
    let root = fixture("postgres-required-predicates", "unproven-two-rules");
    let out = check_json(&root);
    let body = stdout(&out);
    let err = stderr(&out);
    assert!(out.status.success(), "{body}\n{err}");
    assert!(
        err.contains("warning: 1 member .query calls with SQL-looking text weren't proven to be executors; see docs/migrations/explicit-postgres-executors.md"),
        "{err}"
    );
    assert!(!body.contains("weren't proven"), "{body}");
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["unprovenMemberQuerySql"], 1, "{report}");
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
}

#[test]
fn postgres_idempotent_insert_recovered_expression_still_fails() {
    let root = fixture("postgres-idempotent-insert", "expression-arbiter-recovered");
    let out = check_json(&root);
    assert!(!out.status.success(), "{}", stdout(&out));
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    let findings = report["rules"].as_array().unwrap();
    assert!(
        findings.iter().any(|finding| finding["file"]
            .as_str()
            .is_some_and(|path| path.ends_with("malformed-neighbor.sql"))
            && finding["message"]
                .as_str()
                .is_some_and(|message| message.contains("expression arbiter depends on email"))),
        "{report}"
    );
}
