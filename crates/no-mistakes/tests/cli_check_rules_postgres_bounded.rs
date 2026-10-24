use std::path::PathBuf;
use std::process::{Command, Output};

#[path = "cli_check_rules_postgres_bounded/joined_dml.rs"]
mod joined_dml;

fn fixture() -> PathBuf {
    fixture_dir("fixture")
}

fn fixture_dir(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements")
            .join(name),
    )
}

fn check(config: &str) -> Output {
    check_in(&fixture(), config)
}

fn check_in(root: &std::path::Path, config: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(config))
        .output()
        .unwrap()
}

/// (file, line, target) for every finding of the rule.
fn findings(config: &str) -> Vec<(String, u64, String)> {
    findings_of(check(config))
}

fn findings_of(output: Output) -> Vec<(String, u64, String)> {
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
    // The unchanged fixture's lower(slug) lookup now matches its expression key.
    expect(
        "keys.yml",
        "sql/keys.sql",
        &[
            (6, "table:order_lines"),
            (8, "table:partial_keys"),
            (9, "table:invalid_keys"),
            (10, "table:deferred_keys"),
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
        &[
            (7, "table:orders"),
            (9, "table:orders"),
            (13, "table:accounts"),
            (15, "table:contacts"),
            (19, "table:orders"),
            (24, "table:orders"),
        ],
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
fn plain_partition_parent_drop_reveals_physical_namesake() {
    // The attached child disappears even though the statement omits CASCADE.
    expect(
        "temporary-partition-drop.yml",
        "sql/temporary-partition-drop.sql",
        &[(6, "table:orders")],
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
fn embedded_statement_start_directives_cover_later_relation_lines() {
    expect(
        "statement-suppressions.yml",
        "src/statement-suppressions.mts",
        &[(20, "table:invoices")],
    );
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

#[test]
fn template_interpolations_are_binds_and_key_joins_propagate_bounds() {
    // `sql-template-strings` interpolations (`${id}`, `${image.id}`, `${id}::uuid`, a list) pin a
    // key like `$1`, and a pinned order bounds its account and the account its profile. The two
    // reported queries have no key, and an account's orders are not bounded by the account.
    expect(
        "template.yml",
        "src/templates.mts",
        &[(17, "table:invoices"), (20, "table:orders")],
    );
}

#[test]
fn nested_sql_fragment_checks_each_concrete_version_instead_of_reading_a_bind() {
    let root = fixture_dir("nested-fragment");
    for config in ["fail.yml", "ignore.yml"] {
        assert_eq!(
            findings_of(check_in(&root, config)),
            [("src/nested.ts".to_string(), 7, "table:accounts".to_string())]
        );
    }
}

#[test]
fn unanalyzable_sql_fails_closed_unless_ignored() {
    let found = findings("unanalyzable.yml");
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found.iter().all(|(_, _, target)| target == "unanalyzable"));
    assert!(!check("unanalyzable.yml").status.success());
    assert_eq!(findings("ignore-unanalyzable.yml"), []);
    assert!(check("ignore-unanalyzable.yml").status.success());
}

#[test]
fn embedded_wrapped_statement_directives_keep_recovery_fixtures_isolated() {
    expect(
        "wrapped-statement-suppressions.yml",
        "src/wrapped-statement-suppression.mts",
        &[(9, "table:accounts"), (11, "table:accounts")],
    );
}

#[test]
fn unique_index_shape_guards_survive_cli_preparation() {
    let root = fixture().join("unique-index-shapes");
    let actual = findings_of(check_in(&root, ".no-mistakes.yml"));
    assert_eq!(actual.len(), 24, "{actual:?}");
    assert!(actual
        .iter()
        .any(|(file, line, _)| file == "partial-unique.sql" && *line == 3));
    assert!(!actual
        .iter()
        .any(|(file, line, _)| file == "partial-unique.sql" && *line == 2));
}

#[test]
fn catalog_key_ownership_and_view_policies_survive_cli_preparation() {
    for (directory, config, expected) in [
        ("catalog-resolution", "key-resolution.yml", 6),
        ("catalog-resolution", "key-operators.yml", 4),
        ("catalog-resolution", "key-booleans.yml", 1),
        ("catalog-resolution", "view-bounds.yml", 1),
        ("catalog-resolution", "view-allow.yml", 0),
        ("catalog-resolution", "view-suppression.yml", 0),
        ("catalog-resolution-identity", "view-identity.yml", 1),
    ] {
        let actual = findings_of(check_in(&fixture().join(directory), config));
        assert_eq!(actual.len(), expected, "{config}: {actual:?}");
    }
}

#[test]
fn schemaless_bare_catalog_keys_still_judge_qualified_tables() {
    let root = fixture().join("schemaless-key-resolution");
    let mut actual = findings_of(check_in(&root, "qualified-bare-tables.yml"));
    actual.sort();
    assert_eq!(
        actual,
        [
            (
                "sql/qualified-bare-tables.sql".to_string(),
                1,
                "table:big".to_string(),
            ),
            (
                "sql/qualified-bare-tables.sql".to_string(),
                2,
                "table:big".to_string(),
            ),
            (
                "sql/qualified-bare-tables.sql".to_string(),
                3,
                "table:a".to_string(),
            ),
            (
                "sql/qualified-bare-tables.sql".to_string(),
                3,
                "table:b".to_string(),
            ),
        ]
    );
}
