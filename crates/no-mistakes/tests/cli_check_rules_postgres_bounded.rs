use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements/fixture"),
    )
}

fn check(config: &str) -> Output {
    let root = fixture();
    Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(config))
        .output()
        .unwrap()
}

/// (file, line, target) for every finding of the rule.
fn findings(config: &str) -> Vec<(String, u64, String)> {
    let output = check(config);
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == "postgres-bounded-statements")
        .map(|finding| {
            (
                finding["file"].as_str().unwrap().to_string(),
                finding["line"].as_u64().unwrap(),
                finding["target"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

fn expect(config: &str, file: &str, expected: &[(u64, &str)]) {
    let actual: Vec<(u64, String)> = findings(config)
        .into_iter()
        .map(|(found, line, target)| {
            assert_eq!(found, file, "{config}");
            (line, target)
        })
        .collect();
    let expected: Vec<(u64, String)> = expected
        .iter()
        .map(|(line, target)| (*line, target.to_string()))
        .collect();
    assert_eq!(actual, expected, "{config}");
}

#[test]
fn statements_that_can_match_many_rows_are_reported() {
    expect(
        "invalid.yml",
        "sql/invalid.sql",
        &[
            (1, "table:invoices"),
            (2, "table:exports"),
            (3, "table:sessions"),
        ],
    );
    let output = check("invalid.yml");
    assert!(!output.status.success());
    let body = String::from_utf8_lossy(&output.stdout);
    assert!(
        body.contains("SELECT can return every row of invoices"),
        "{body}"
    );
    assert!(
        body.contains("UPDATE can change every row of exports"),
        "{body}"
    );
    assert!(
        body.contains("DELETE can remove every row of sessions"),
        "{body}"
    );
    assert_eq!(body, String::from_utf8_lossy(&check("invalid.yml").stdout));
}

#[test]
fn limits_keys_aggregates_and_limited_target_subqueries_are_bounded() {
    assert_eq!(findings("valid.yml"), []);
    assert!(check("valid.yml").status.success());
}

#[test]
fn an_allow_entry_that_suppresses_nothing_is_stale() {
    let found = findings("stale-allow.yml");
    assert_eq!(
        found,
        [("schema.json".to_string(), 1, "table:accounts".to_string())]
    );
    let body = String::from_utf8_lossy(&check("stale-allow.yml").stdout).into_owned();
    assert!(body.contains("stale postgres-bounded-statements allow entry: table:accounts"));
}

#[test]
fn only_valid_ready_immediate_whole_key_indexes_prove_one_row() {
    expect(
        "keys.yml",
        "sql/keys.sql",
        &[
            (6, "table:order_lines"),
            (8, "table:partial_keys"),
            (9, "table:invalid_keys"),
            (10, "table:deferred_keys"),
            (11, "table:expr_keys"),
            (13, "table:orders"),
            (14, "table:orders"),
        ],
    );
}

#[test]
fn joins_and_limited_subqueries_bound_by_key_and_unknown_relations_are_not_judged() {
    expect(
        "graph.yml",
        "sql/graph.sql",
        &[(7, "table:orders"), (9, "table:orders")],
    );
}

#[test]
fn only_the_configured_statement_kinds_are_judged() {
    expect(
        "delete-only.yml",
        "sql/invalid.sql",
        &[(3, "table:sessions")],
    );
}

#[test]
fn executor_sql_is_judged_at_its_source_line() {
    expect("embedded.yml", "src/jobs.ts", &[(4, "table:invoices")]);
}

#[test]
fn standard_suppression_directives_apply() {
    expect("suppress.yml", "sql/suppress.sql", &[(4, "table:invoices")]);
}

#[test]
fn an_ordering_only_catalog_and_a_missing_catalog_path_are_errors() {
    let ordering = check("ordering.yml");
    assert!(!ordering.status.success());
    assert!(String::from_utf8_lossy(&ordering.stderr).contains("ordering-only coverage"));
    let missing = check("no-catalog.yml");
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("schemaCatalogPath: required"));
}
