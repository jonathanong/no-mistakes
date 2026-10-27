use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::{Command, Output};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/sql-policy-effort")
}

fn check(config: &str, effort: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
    command
        .args(["check", "--root"])
        .arg(root())
        .args(["--config", config, "--format", "json"]);
    if let Some(effort) = effort {
        command.args(["--effort", effort]);
    }
    command.output().unwrap()
}

fn report(config: &str, effort: Option<&str>) -> Value {
    let output = check(config, effort);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{config}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn assert_semantics(report: &Value, policy: &str) {
    assert_eq!(report["warnings"], json!([]), "{report}");
    let definite: Vec<_> = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["target"] != "unanalyzable")
        .map(|finding| {
            json!({
                "rule": finding["rule"], "file": finding["file"],
                "line": finding["line"], "target": finding["target"]
            })
        })
        .collect();
    let expected: Vec<_> = ["postgres-explicit-columns", "postgres-required-predicates"]
        .into_iter()
        .flat_map(|rule| {
            [9, 10, 19].map(|line| {
                json!({"rule": rule, "file": "src/queries.ts", "line": line, "target": "accounts"})
            })
        })
        .collect();
    // Exact witnesses protect direct/const/helper recovery and both suppressions.
    // Good bind templates must not become residual uncertainty at low effort.
    assert_eq!(definite, expected, "{report}");
    let uncertain: Vec<_> = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["target"] == "unanalyzable")
        .collect();
    assert_eq!(uncertain.len(), usize::from(policy == "report") * 4);
    let advisories = report["advisories"].as_array().unwrap();
    assert_eq!(advisories.len(), usize::from(policy == "advise") * 4);
    for (rule, target) in [
        ("postgres-explicit-columns", "returning-star, select-star"),
        (
            "postgres-required-predicates",
            "relation:accounts:predicate:deleted_at IS NULL",
        ),
    ] {
        for (rows, expected_target, enabled) in [
            (uncertain.clone(), "unanalyzable", policy == "report"),
            (advisories.iter().collect(), target, policy == "advise"),
        ] {
            let actual: Vec<_> = rows.into_iter().filter(|row| row["rule"] == rule)
                .map(|row| json!({"file": row["file"], "line": row["line"], "target": row["target"]})).collect();
            let expected = if enabled {
                vec![
                    json!({"file": "src/queries.ts", "line": 25, "target": expected_target}),
                    json!({"file": "src/queries.ts", "line": 26, "target": expected_target}),
                ]
            } else {
                vec![]
            };
            assert_eq!(actual, expected, "{report}");
        }
    }
}

fn assert_low_rewrite(report: &Value, rule: Option<&str>) {
    let rows: Vec<_> = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .chain(report["advisories"].as_array().unwrap())
        .filter(|row| row["target"] != "accounts" && rule.is_none_or(|rule| row["rule"] == rule))
        .collect();
    assert!(!rows.is_empty(), "{report}");
    for row in rows {
        let message = row["message"].as_str().unwrap();
        for text in ["effort low", "literal string", "one local const"] {
            assert!(message.contains(text), "{row}");
        }
    }
}

#[test]
fn low_report_preserves_literal_and_helper_violations_and_reports_unknown_sql() {
    let report = report("low-report.yml", None);
    assert_semantics(&report, "report");
    assert_low_rewrite(&report, None);
}

#[test]
fn low_advise_moves_only_unknown_and_malformed_sql_to_advisories() {
    let report = report("low-advise.yml", None);
    assert_semantics(&report, "advise");
    assert_low_rewrite(&report, None);
}

#[test]
fn low_ignore_keeps_all_proven_column_and_predicate_violations() {
    assert_semantics(&report("low-ignore.yml", None), "ignore");
}

#[test]
fn default_and_explicit_max_keep_identical_json_and_exit_status() {
    let default = check("default.yml", None);
    let explicit = check("max.yml", None);
    let invocation = check("default.yml", Some("max"));
    assert_eq!(default.status.code(), Some(1));
    assert_eq!(explicit.status.code(), default.status.code());
    assert_eq!(invocation.status.code(), default.status.code());
    assert_eq!(default.stdout, explicit.stdout);
    assert_eq!(default.stdout, invocation.stdout);
    assert_eq!(default.stderr, explicit.stderr);
    assert_eq!(default.stderr, invocation.stderr);
    let report = serde_json::from_slice(&default.stdout).unwrap();
    assert_semantics(&report, "report");
}

#[test]
fn mixed_applications_use_their_own_effort_and_ordinary_sql_scope() {
    let mixed = report("mixed.yml", None);
    assert_semantics(&mixed, "advise");
    assert_low_rewrite(&mixed, Some("postgres-explicit-columns"));
    let low = report("low-advise.yml", None);
    let max = report("low-advise.yml", Some("max"));
    for field in ["rules", "advisories"] {
        for (rule, expected) in [
            ("postgres-explicit-columns", &low),
            ("postgres-required-predicates", &max),
        ] {
            let for_rule = |report: &Value| {
                report[field]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|row| row["rule"] == rule)
                    .cloned()
                    .collect::<Vec<_>>()
            };
            assert_eq!(for_rule(&mixed), for_rule(expected), "{field}: {mixed}");
        }
    }
}

#[test]
fn configured_messages_do_not_change_legacy_rule_behavior_or_suppression() {
    let low = report("custom.yml", None);
    let max = report("custom.yml", Some("max"));
    assert_semantics(&low, "ignore");
    // Existing max ignores RuleDef.message for these two rules; low preserves it.
    assert_eq!(low["rules"], max["rules"]);
    assert!(low["rules"][0]["message"]
        .as_str()
        .unwrap()
        .contains("SELECT *"));
    assert!(low["rules"][3]["message"]
        .as_str()
        .unwrap()
        .contains("deleted_at IS NULL"));
}

#[test]
fn sql_policy_effort_check_json_snapshot() {
    let cases: Vec<_> = ["low-report", "low-advise", "low-ignore", "mixed"]
        .into_iter()
        .map(|name| (name, root(), root().join(format!("{name}.yml"))))
        .collect();
    super::check_json_snapshot::assert_saved_fixture("sql_policy_effort", &cases);
}
