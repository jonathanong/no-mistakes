use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

fn check(root: &Path, config: &str) -> (bool, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(root)
        .args(["--config", config])
        .output()
        .unwrap();
    let report = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stderr)));
    (output.status.success(), report)
}

fn assert_configured_rule_set(root: &Path, config: &str, actual: Vec<String>, context: &str) {
    let expected = no_mistakes::config::v2::load_v2_config(root, Some(&root.join(config)))
        .unwrap()
        .rules
        .into_iter()
        .filter(|rule| rule.enabled)
        .map(|rule| rule.rule)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        !expected.is_empty(),
        "{config}: expected schema rules must not be empty"
    );
    assert!(
        expected.contains("postgres-constraint-validate"),
        "{config}: schema rule inventory lost its constraint-validation sentinel"
    );
    assert_eq!(actual.len(), expected.len(), "{context}");
    assert_eq!(
        actual
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>(),
        expected,
        "{context}"
    );
}

fn assert_scoped_lines(root: &Path, config: &str, rows: &Value, ddl: &[u64], bodies: &[u64]) {
    let rules = no_mistakes::config::v2::load_v2_config(root, Some(&root.join(config)))
        .unwrap()
        .rules;
    let rows = rows.as_array().unwrap();
    let mut expected_count = 0;
    for rule in rules.into_iter().filter(|rule| rule.enabled) {
        let inspect = matches!(
            rule.rule.as_str(),
            "postgres-generated-column-predicates"
                | "postgres-idempotent-insert"
                | "postgres-no-generated-column-writes"
                | "postgres-sql-statement-policy"
        );
        let expected = if inspect { bodies } else { ddl };
        expected_count += expected.len();
        let mut actual: Vec<_> = rows
            .iter()
            .filter(|row| row["rule"] == rule.rule)
            .map(|row| row["line"].as_u64().unwrap())
            .collect();
        actual.sort_unstable();
        assert_eq!(actual, expected, "{config}: {}: {rows:?}", rule.rule);
    }
    assert_eq!(rows.len(), expected_count, "{config}: {rows:?}");
}

#[test]
fn schema_body_defaults_preserve_top_level_errors_and_explicit_policies() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-ddl-unanalyzable/fixture/body-policy");
    let top = &[2, 33, 35][..];
    let ddl_body = &[5, 6, 8, 21, 24, 28][..];
    let full_body = &[5, 6, 8, 13, 18, 21, 24, 28][..];
    let ddl_all = &[2, 5, 6, 8, 21, 24, 28, 33, 35][..];
    let full_all = &[2, 5, 6, 8, 13, 18, 21, 24, 28, 33, 35][..];
    for (config, ddl_findings, body_findings, ddl_notes, body_notes) in [
        ("default.yml", top, top, ddl_body, full_body),
        ("report.yml", ddl_all, full_all, &[], &[]),
        ("advise.yml", &[], &[], ddl_all, full_all),
        ("ignore.yml", &[], &[], &[], &[]),
        ("override.yml", ddl_all, full_all, &[], &[]),
        ("global-alias-report.yml", ddl_all, full_all, &[], &[]),
        ("rule-advise.yml", &[], &[], ddl_all, full_all),
        ("rule-ignore.yml", &[], &[], &[], &[]),
    ] {
        let (success, report) = check(&root, config);
        assert_eq!(
            success,
            ddl_findings.is_empty() && body_findings.is_empty(),
            "{config}: {report}"
        );
        assert_scoped_lines(&root, config, &report["rules"], ddl_findings, body_findings);
        assert_scoped_lines(&root, config, &report["advisories"], ddl_notes, body_notes);
        assert!(
            report["rules"]
                .as_array()
                .unwrap()
                .iter()
                .chain(report["advisories"].as_array().unwrap())
                .all(|row| row["file"] == "migrations/001.sql"),
            "{config}: {report}"
        );
    }
}

#[test]
fn valid_body_queries_and_comment_literals_have_no_schema_uncertainty() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/source/ddl-compat/body-recovery");
    let (success, report) = check(&root, "report.yml");
    assert!(success, "{report}");
    assert_eq!(report["rules"], serde_json::json!([]));
    assert_eq!(report["advisories"], serde_json::json!([]));
}

#[test]
fn opaque_body_advisories_do_not_fail_the_cli() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-ddl-unanalyzable/fixture/body-only");
    let (success, report) = check(&root, "default.yml");
    assert!(success, "{report}");
    assert_eq!(report["rules"], serde_json::json!([]));
    let advisories = report["advisories"].as_array().unwrap();
    assert_configured_rule_set(
        &root,
        "default.yml",
        advisories
            .iter()
            .map(|item| item["rule"].as_str().unwrap().to_owned())
            .collect(),
        &report.to_string(),
    );
    assert!(advisories.iter().all(|item| item["line"] == 4));
}

#[test]
fn locked_query_loops_keep_malformed_sql_visible_under_both_policies() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-ddl-unanalyzable/fixture/query-loops");
    for (config, findings, advisories) in [
        ("valid.yml", 0, 0),
        ("valid-report.yml", 0, 0),
        ("default.yml", 0, 4),
        ("report.yml", 4, 0),
    ] {
        let (success, report) = check(&root, config);
        assert_eq!(success, findings == 0, "{config}: {report}");
        let rows = report["rules"].as_array().unwrap();
        let notes = report["advisories"].as_array().unwrap();
        assert_eq!(rows.len(), findings, "{config}: {report}");
        assert_eq!(notes.len(), advisories, "{config}: {report}");
        assert_scoped_lines(
            &root,
            config,
            &report["rules"],
            &[],
            if findings == 0 { &[] } else { &[6] },
        );
        assert_scoped_lines(
            &root,
            config,
            &report["advisories"],
            &[],
            if advisories == 0 { &[] } else { &[6] },
        );
        assert!(
            rows.iter()
                .chain(notes)
                .all(|item| { item["file"] == "malformed.sql" && item["line"] == 6 }),
            "{config}: {report}"
        );
    }
}

#[test]
fn recovered_locked_loop_preserves_a_definite_sibling_write_finding() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-ddl-unanalyzable/fixture/query-loops");
    let (success, report) = check(&root, "generated-write.yml");
    assert!(!success, "{report}");
    let rows = report["rules"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "{report}");
    assert_eq!(rows[0]["rule"], "postgres-no-generated-column-writes");
    assert!(rows[0]["message"]
        .as_str()
        .unwrap()
        .contains("do not write generated column"));
    assert_eq!(report["advisories"], serde_json::json!([]));
}

#[test]
fn stored_routine_labels_and_conditional_boundaries_are_opaque_to_ddl() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-ddl-unanalyzable/fixture/query-loops");
    let (success, report) = check(&root, "boundaries.yml");
    assert!(success, "{report}");
    assert_eq!(report["rules"], serde_json::json!([]));
    assert_eq!(report["advisories"], serde_json::json!([]));
}

#[test]
fn uncertain_loop_headers_keep_definite_actions_under_both_policies() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-ddl-unanalyzable/fixture/query-loops");
    for (config, findings, advisories) in [
        ("uncertain-header-default.yml", 3, 8),
        ("uncertain-header-report.yml", 11, 0),
    ] {
        let (success, report) = check(&root, config);
        assert!(!success, "{config}: {report}");
        let rows = report["rules"].as_array().unwrap();
        let notes = report["advisories"].as_array().unwrap();
        assert_eq!(rows.len(), findings, "{config}: {report}");
        assert_eq!(notes.len(), advisories, "{config}: {report}");
        let mut definite = rows
            .iter()
            .filter_map(|row| {
                let line = row["line"].as_u64().unwrap();
                [6, 13, 20].contains(&line).then_some(line)
            })
            .collect::<Vec<_>>();
        definite.sort_unstable();
        assert_eq!(definite, [6, 13, 20], "{config}: {report}");
        let mut headers = std::collections::BTreeMap::new();
        for row in rows.iter().chain(notes) {
            assert_eq!(row["file"], "uncertain-header-actions.sql");
            let line = row["line"].as_u64().unwrap();
            if [5, 12, 19, 26].contains(&line) {
                *headers.entry(line).or_insert(0) += 1;
            }
        }
        assert_eq!(headers, [(5, 2), (12, 2), (19, 2), (26, 2)].into());
    }
}
