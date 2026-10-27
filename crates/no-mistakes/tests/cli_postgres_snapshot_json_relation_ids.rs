use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn repo_rel(rel: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel),
    )
}

#[test]
fn postgres_json_relation_ids_check_json_snapshot() {
    let rule = "postgres-json-relation-ids";
    // Advisory rule: asserted findings are report.advisories and exitCode stays 0.
    let mut cases = [
        ("fail_guidance", "fail", "ids.yml"),
        ("mapped_fk", "mapped-fk", "ids.yml"),
        ("wrong_fk", "wrong-fk", "ids.yml"),
        ("unmapped", "unmapped", "ids.yml"),
        ("ignored", "ignored", "ids.yml"),
        ("edges", "edges", "ids.yml"),
        ("allow_baseline", "allow", "ids.yml"),
        // Findings stay on orders.
        ("allow_table", "allow", "allow-table.yml"),
        // Targets must include orders.extra and invoice_revisions.changes.
        ("allow_column", "allow", "allow-column.yml"),
        ("syntax_pass", "syntax-pass", "ids.yml"),
        ("syntax_fail", "syntax-fail", "ids.yml"),
        ("wrappers_fail", "wrappers-fail", "ids.yml"),
        ("wrappers_pass", "wrappers-pass", "ids.yml"),
        // One target must be table:orders.
        ("wildcards", "wildcards", "ids.yml"),
        ("conflict_fail", "conflict-fail", "ids.yml"),
        ("conflict_pass", "conflict-pass", "ids.yml"),
        ("constructor_text", "constructor-text", "ids.yml"),
        ("literal_keys", "literal-keys", "ids.yml"),
        ("opaque_keys", "opaque-keys", "ids.yml"),
        ("positional", "positional", "positional.yml"),
        (
            "missing_ordinals",
            "missing-ordinals",
            "missing-ordinals.yml",
        ),
        ("fingerprint_keys", "fingerprint", "fingerprint.yml"),
        (
            "fingerprint_allow_column",
            "fingerprint",
            "fingerprint-column.yml",
        ),
        (
            "fingerprint_allow_table",
            "fingerprint",
            "fingerprint-table.yml",
        ),
        // Empty fingerprint regex is valid and matches every column name.
        ("fingerprint_empty", "fingerprint", "fingerprint-empty.yml"),
        ("expression_index", "expression-index", "expression.yml"),
        // Omitted sqlInclude still scans db/fail.sql.
        ("defaults_scan_sql", "defaults", "defaults.yml"),
        // Omitted sqlInclude scans every SQL file, so this root has no SQL.
        ("id_pattern_only", "pattern-only", "id-pattern-only.yml"),
        ("empty_map", "pattern-only", "empty-map.yml"),
    ]
    .map(|(label, fixture, config)| {
        (
            label,
            rules_path(rule, "fixture", fixture),
            rules_path(rule, "configs", config),
        )
    })
    .to_vec();
    let root = repo_rel("test-cases/rules/postgres-json-relation-ids/fixture");
    // Patterns unset: rules and advisories stay empty even though db/fail.sql is present.
    for (label, config) in [
        ("no_options", "no-options.yml"),
        ("catalog_only", "catalog-only.yml"),
        ("map_without_pattern", "map-without-pattern.yml"),
    ] {
        cases.push((label, root.clone(), rules_path(rule, "configs", config)));
    }
    // Shared-root include/sqlInclude stand in for the unit test file list.
    // sql_not_selected must not scan src/*.ts. exclude_files drops db/fail.sql and src/query.ts.
    // include_missing and opaque_tail must not scan db/*.sql.
    for (label, config) in [
        ("sql_not_selected", "sql-not-selected.yml"),
        ("exclude_files", "exclude-files.yml"),
        ("include_missing", "include-missing.yml"),
        ("opaque_tail", "opaque-tail.yml"),
    ] {
        cases.push((label, root.clone(), rules_path(rule, "configs", config)));
    }
    assert_saved_fixture("postgres_json_relation_ids", &cases);
}
