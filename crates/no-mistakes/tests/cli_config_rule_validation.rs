use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};
use no_mistakes::codebase::rules::rule_ids::configurable_rule_ids;
use serde_yaml::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn invalid_rule_configuration_aborts_before_analysis() {
    let root = rules_path("options-audit", "fixture", "issue-4110");
    let cases = [
        ("unknown", root.clone(), root.join("unknown.yml")),
        ("unknown-map", root.clone(), root.join("unknown-map.yml")),
        (
            "unknown-override",
            root.clone(),
            root.join("unknown-override.yml"),
        ),
        (
            "unknown-project",
            root.clone(),
            root.join("unknown-project.yml"),
        ),
        ("unknown-jsonc", root.clone(), root.join("unknown.jsonc")),
        (
            "invalid-options",
            root.clone(),
            root.join("invalid-options.yml"),
        ),
        (
            "invalid-values",
            root.clone(),
            root.join("invalid-values.yml"),
        ),
        (
            "semantic-errors",
            root.clone(),
            root.join("semantic-errors.yml"),
        ),
        ("fragment", root.join("fragment"), PathBuf::new()),
    ];
    let result = assert_saved_fixture_report("invalid_rule_configuration", &cases, &[], &[]);
    for (label, _, _) in cases {
        assert_eq!(result[label]["exitCode"], 2, "{label}: {result}");
        assert!(result[label]["report"].is_null(), "{label}: {result}");
    }
}

#[test]
fn every_registered_rule_rejects_an_unknown_option() {
    let root = rules_path("options-audit", "fixture", "issue-4110");
    let path = root.join("all-rules.yml");
    let yaml: Value = serde_yaml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let rules = yaml["rules"].as_sequence().unwrap();
    let declared: BTreeSet<_> = rules
        .iter()
        .map(|rule| {
            assert_eq!(rule["options"]["bogusOption"].as_bool(), Some(true));
            rule["rule"].as_str().unwrap()
        })
        .collect();
    let registered: BTreeSet<_> = configurable_rule_ids().collect();
    assert!(!registered.is_empty());
    assert_eq!(
        declared, registered,
        "update the saved fixture when registering a rule"
    );
    assert_eq!(rules.len(), registered.len());

    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["--lock-timeout", "0", "--jobs", "1", "check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(&path)
        .args(["--format", "json"])
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(output.stdout.is_empty(), "{stderr}");
    for id in registered {
        let diagnostic = stderr
            .lines()
            .find(|line| line.contains(&format!("`{id}`")));
        assert!(
            diagnostic.is_some_and(|line| line.contains("bogusOption") && line.contains("all-rules.yml:")),
            "missing bogusOption diagnostic for {id}: {stderr}"
        );
    }
}

#[test]
fn declared_option_values_reject_invalid_and_run_valid() {
    let root = rules_path("options-audit", "fixture", "declared-invalid-options");
    let cases = [
        (
            "invalid",
            root.clone(),
            root.join("invalid-declared-options.yml"),
        ),
        (
            "valid",
            root.clone(),
            root.join("valid-declared-options.yml"),
        ),
    ];
    let result = assert_saved_fixture_report("declared_option_values", &cases, &[], &[]);
    assert_eq!(result["invalid"]["exitCode"], 2, "{}", result["invalid"]);
    assert!(
        result["invalid"]["report"].is_null(),
        "{}",
        result["invalid"]
    );
    let stderr = result["invalid"]["stderr"].as_str().unwrap();
    for (rule, option) in [
        ("nextjs-redirect-destinations", "includeRewrites"),
        ("rust-max-lines-per-file", "srcMax"),
        ("rust-no-inline-allows", "testMax"),
        ("rust-no-inline-tests", "excludes"),
        ("postgres-array-columns", "allowElementTypes"),
        ("postgres-bounded-statements", "statements"),
        ("postgres-conflict-ordering", "include"),
        ("postgres-constraint-validate", "sqlInclude"),
        ("postgres-duplicate-function-body", "minClusterSize"),
        ("postgres-explain-write-rollback", "sqlInclude"),
        ("postgres-explicit-columns", "maxColumns"),
        ("postgres-finite-text-columns", "columnTypes"),
        ("postgres-fk-index", "allowedColumns"),
        ("postgres-generated-column-predicates", "functions"),
        ("postgres-idempotent-insert", "include"),
        ("postgres-identifier-length", "maxBytes"),
        ("postgres-json-relation-ids", "idKeyPattern"),
        ("postgres-key-column-types", "allowedTypes"),
        ("postgres-lock-ordering", "include"),
        ("postgres-lock-written-rows-only", "not-a-check"),
        ("postgres-materialized-view-shape", "windowFunctions"),
        ("postgres-no-transformed-uuid-predicates", "sqlInclude"),
        ("postgres-redundant-index", "allowedIndexes"),
        ("postgres-require-fk-on-delete", "sqlInclude"),
        ("postgres-require-named-constraints", "sqlInclude"),
        ("postgres-require-query-annotation", "sqlLiteralHelpers"),
        ("postgres-required-comments", "objects"),
        ("postgres-sql-statement-policy", "bannedStatements"),
        ("postgres-status-with-lifecycle-timestamps", "statusColumns"),
    ] {
        let diagnostic = stderr.lines().find(|line| line.contains(rule));
        assert!(
            diagnostic.is_some_and(|line| line.contains(option)),
            "{rule} options.{option} was accepted:\n{stderr}"
        );
    }
    assert_eq!(result["valid"]["exitCode"], 0, "{}", result["valid"]);
    assert!(
        result["valid"]["report"]["rules"]
            .as_array()
            .is_some_and(Vec::is_empty),
        "{}",
        result["valid"]
    );
}
