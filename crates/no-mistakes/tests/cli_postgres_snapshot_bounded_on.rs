use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use serde_json::Value;
use std::process::Command;

fn root() -> std::path::PathBuf {
    rules_path("postgres-bounded-statements", "fixture/bounded-on", "")
}

fn report(config: &str, exit: i32) -> Value {
    report_in(&root(), config, exit)
}

fn report_in(root: &std::path::Path, config: &str, exit: i32) -> Value {
    report_with_verdicts(root, config, exit, true)
}

fn report_with_verdicts(
    root: &std::path::Path,
    config: &str,
    exit: i32,
    include_sql_verdicts: bool,
) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(config))
        .args(include_sql_verdicts.then_some("--include-sql-verdicts"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(exit), "{report}");
    report
}

fn findings_at(report: &Value, file: &str) -> Vec<(u64, String)> {
    let mut findings = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| {
            finding["file"]
                .as_str()
                .is_some_and(|path| path.ends_with(file))
        })
        .map(|finding| {
            (
                finding["line"].as_u64().unwrap(),
                finding["target"].as_str().unwrap().to_string(),
            )
        })
        .collect::<Vec<_>>();
    findings.sort();
    findings
}

#[test]
fn postgres_bounded_statements_on_clause_roots_check_json_snapshot() {
    let fixture = root();
    assert_saved_fixture(
        "postgres_bounded_statements_on_clause_roots",
        &[
            ("lookup_roots", fixture.clone(), fixture.join("valid.yml")),
            ("advisory", fixture.clone(), fixture.join("advisory.yml")),
            ("embedded", fixture.clone(), fixture.join("embedded.yml")),
        ],
    );
    let report = report("valid.yml", 1);

    for file in [
        "sql/item-feed-topic.sql",
        "sql/account-authorization.sql",
        "sql/participant-user.sql",
    ] {
        assert!(findings_at(&report, file).is_empty(), "{file}: {report}");
    }
    assert_eq!(
        findings_at(&report, "sql/non-key.sql"),
        [(3, "table:authorizations".to_string())],
        "a non-key join is not bounded by its source: {report}"
    );
    assert_eq!(
        findings_at(&report, "sql/unbounded-chain.sql"),
        [(2, "table:feeds, table:items, table:topics".to_string()),],
        "an unbounded key-join chain reports every unbounded relation: {report}"
    );
    let advisory = self::report("advisory.yml", 0);
    assert!(advisory["rules"].as_array().unwrap().is_empty());
    let findings = advisory["advisories"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{advisory}");
    for table in ["items", "feeds"] {
        assert!(
            findings.iter().any(|finding| {
                finding["target"]
                    .as_str()
                    .unwrap()
                    .split(", ")
                    .any(|target| target == format!("table:{table}"))
                    && finding["message"]
                        .as_str()
                        .unwrap()
                        .contains(&format!("SELECT can return every row of {table}"))
            }),
            "{advisory}"
        );
    }
    let embedded = self::report("embedded.yml", 0);
    assert!(embedded["rules"].as_array().unwrap().is_empty());
    assert!(embedded["advisories"].as_array().unwrap().is_empty());
    let selects = embedded["sqlVerdicts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|verdict| verdict["subcheck"] == "select")
        .collect::<Vec<_>>();
    assert_eq!(selects.len(), 3, "{embedded}");
    assert!(
        selects.iter().all(|verdict| {
            verdict["status"] == "proven-clear" && verdict["evidence"] == "exact"
        }),
        "{embedded}"
    );
}

#[test]
fn postgres_bounded_statements_statement_groups_check_json_snapshot() {
    let fixture = rules_path(
        "postgres-bounded-statements",
        "statement-groups/fixture",
        "",
    );
    assert_saved_fixture(
        "postgres_bounded_statements_statement_groups",
        &[
            ("all_relations", fixture.clone(), fixture.join("valid.yml")),
            (
                "advisory_relations",
                fixture.clone(),
                fixture.join("advisory.yml"),
            ),
            (
                "relation_controls",
                fixture.clone(),
                fixture.join("controls.yml"),
            ),
            (
                "embedded_sites",
                fixture.clone(),
                fixture.join("embedded.yml"),
            ),
            (
                "embedded_sites_with_extra",
                fixture.clone(),
                fixture.join("embedded-with-extra.yml"),
            ),
            (
                "variant_prefix_union",
                fixture.clone(),
                fixture.join("variant-prefix.yml"),
            ),
            (
                "repeated_helper_occurrences",
                fixture.clone(),
                fixture.join("repeated-helper.yml"),
            ),
            ("rule_off", fixture.clone(), fixture.join("off.yml")),
        ],
    );
    let report = report_in(&fixture, "controls.yml", 1);
    assert_eq!(
        findings_at(&report, "sql/groups.sql"),
        [
            (
                2,
                "table:accounts, table:feeds, table:items, table:users".to_string()
            ),
            (9, "table:items".to_string()),
            (17, "table:feeds".to_string()),
            (19, "table:items".to_string()),
            (19, "table:items".to_string()),
            (20, "table:items, view:activity".to_string()),
        ],
        "relation directives, allow entries and column bounds apply before grouping: {report}"
    );
    let findings = report["rules"].as_array().unwrap();
    let message = findings[0]["message"].as_str().unwrap();
    for table in ["accounts", "feeds", "items", "users"] {
        assert!(
            message.contains(&format!("SELECT can return every row of {table}")),
            "{report}"
        );
    }
    let mixed = findings
        .iter()
        .find(|finding| finding["line"] == 20)
        .unwrap();
    let mixed_message = mixed["message"].as_str().unwrap();
    assert!(
        mixed_message.contains("SELECT can return every row of items"),
        "{report}"
    );
    assert!(
        mixed_message.contains("Select can read unbounded rows from view activity"),
        "{report}"
    );
    let same_line = findings
        .iter()
        .filter(|finding| finding["line"] == 19)
        .collect::<Vec<_>>();
    assert_eq!(same_line.len(), 2, "{report}");
    assert_ne!(same_line[0]["message"], same_line[1]["message"], "{report}");
    assert!(
        !findings
            .iter()
            .any(|finding| finding["message"].as_str().unwrap().contains("stale")),
        "{report}"
    );
}

#[test]
fn postgres_bounded_statements_variants_preserve_physical_statement_occurrences() {
    let fixture = rules_path(
        "postgres-bounded-statements",
        "statement-groups/fixture",
        "",
    );
    let variant = report_in(&fixture, "variant-prefix.yml", 1);
    let findings = variant["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{variant}");
    assert_eq!(
        findings[0]["target"], "table:feeds, table:items, table:topics",
        "{variant}"
    );
    let repeated = report_in(&fixture, "repeated-helper.yml", 1);
    let mut targets = repeated["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["target"].as_str().unwrap())
        .collect::<Vec<_>>();
    targets.sort();
    assert_eq!(
        targets,
        ["table:feeds, table:items", "table:items"],
        "{repeated}"
    );
}

#[test]
fn postgres_bounded_statements_site_labels_ignore_unrelated_files_and_verdicts() {
    let fixture = rules_path(
        "postgres-bounded-statements",
        "statement-groups/fixture",
        "",
    );
    let base = report_with_verdicts(&fixture, "embedded.yml", 1, false);
    assert_eq!(base["rules"].as_array().unwrap().len(), 4, "{base}");
    for (config, verdicts) in [
        ("embedded-with-extra.yml", false),
        ("embedded.yml", true),
        ("embedded-with-extra.yml", true),
    ] {
        let report = report_with_verdicts(&fixture, config, 1, verdicts);
        assert_eq!(
            report["rules"], base["rules"],
            "{config} verdicts={verdicts}"
        );
    }
}
