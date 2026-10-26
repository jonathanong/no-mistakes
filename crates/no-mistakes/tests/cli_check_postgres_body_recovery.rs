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

#[test]
fn schema_body_defaults_preserve_top_level_errors_and_explicit_policies() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-ddl-unanalyzable/fixture/body-policy");
    for (config, findings, advisories) in [
        ("default.yml", 33, 88),
        ("report.yml", 121, 0),
        ("advise.yml", 0, 121),
        ("ignore.yml", 0, 0),
        ("override.yml", 121, 0),
        ("global-alias-report.yml", 121, 0),
        ("rule-advise.yml", 0, 121),
        ("rule-ignore.yml", 0, 0),
    ] {
        let (success, report) = check(&root, config);
        assert_eq!(success, findings == 0, "{config}: {report}");
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            findings,
            "{config}: {report}"
        );
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            advisories,
            "{config}: {report}"
        );
        if config == "default.yml" {
            // Each of eleven rules keeps the malformed top-level siblings failing.
            assert!(report["rules"]
                .as_array()
                .unwrap()
                .iter()
                .all(|item| [2, 33, 35].contains(&item["line"].as_u64().unwrap())));
            assert!(report["advisories"]
                .as_array()
                .unwrap()
                .iter()
                .all(
                    |item| [5, 6, 8, 13, 18, 21, 24, 28].contains(&item["line"].as_u64().unwrap())
                ));
        }
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
    assert_eq!(advisories.len(), 11, "{report}");
    assert!(advisories.iter().all(|item| item["line"] == 4));
}

#[test]
fn locked_query_loops_keep_malformed_sql_visible_under_both_policies() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-ddl-unanalyzable/fixture/query-loops");
    for (config, findings, advisories) in [
        ("valid.yml", 0, 0),
        ("valid-report.yml", 0, 0),
        ("default.yml", 0, 11),
        ("report.yml", 11, 0),
    ] {
        let (success, report) = check(&root, config);
        assert_eq!(success, findings == 0, "{config}: {report}");
        let rows = report["rules"].as_array().unwrap();
        let notes = report["advisories"].as_array().unwrap();
        assert_eq!(rows.len(), findings, "{config}: {report}");
        assert_eq!(notes.len(), advisories, "{config}: {report}");
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
fn incomplete_labels_and_conditional_boundaries_remain_findings() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-ddl-unanalyzable/fixture/query-loops");
    let (success, report) = check(&root, "boundaries.yml");
    assert!(!success, "{report}");
    let rows = report["rules"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "{report}");
    assert!(rows
        .iter()
        .all(|row| row["file"] == "malformed-boundaries.sql"));
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
