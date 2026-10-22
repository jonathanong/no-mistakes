use std::path::PathBuf;
use std::process::Command;

fn fixture() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/postgres-require-query-annotation/fixture/provenance-advice",
        ),
    )
}

fn report(include_verdicts: bool) -> serde_json::Value {
    fixture_report(&fixture(), ".no-mistakes.yml", include_verdicts, 0)
}

fn fixture_report(
    root: &std::path::Path,
    config: &str,
    include_verdicts: bool,
    expected_exit: i32,
) -> serde_json::Value {
    // Isolate the CLI invocation lock while keeping the saved SQL fixture intact.
    let runtime = tempfile::tempdir().unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
    command.env("XDG_RUNTIME_DIR", runtime.path());
    command
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(config))
        .args(["--format", "json", "--include-suppressed"]);
    if include_verdicts {
        command.arg("--include-sql-verdicts");
    }
    let output = command.output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(expected_exit),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn sql_verdict_flag_only_adds_confirmed_call_evidence() {
    let plain = report(false);
    let mut detailed = report(true);
    assert!(plain.get("sqlVerdicts").is_none(), "{plain}");
    let rows = detailed["sqlVerdicts"].as_array().unwrap();
    assert_eq!(rows.len(), 5, "{detailed}");
    let source = std::fs::read_to_string(fixture().join("src/query.ts")).unwrap();
    let prefix_call = source
        .find("query(`/* known */ SELECT ${opaque}`)")
        .unwrap();
    assert!(
        rows.iter().any(|row| {
            row["file"] == "src/query.ts"
                && row["span"][0].as_u64() == Some(prefix_call as u64)
                && row["status"] == "proven-clear"
                && row["evidence"] == "verified-prefix"
                && row["knownPrefix"]
                    .as_str()
                    .is_some_and(|prefix| prefix.starts_with("/* known */ SELECT "))
        }),
        "{detailed}"
    );
    detailed.as_object_mut().unwrap().remove("sqlVerdicts");
    assert_eq!(detailed, plain);
}

#[test]
fn residuals_aggregate_per_call_and_application_with_call_line_attribution() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/advisory-aggregation");
    let plain = fixture_report(&root, ".no-mistakes.yml", false, 1);
    let mut detailed = fixture_report(&root, ".no-mistakes.yml", true, 1);
    let rows = detailed["sqlVerdicts"].as_array().unwrap();
    assert_eq!(rows.len(), 84, "{detailed}");
    let advice = detailed["advisories"].as_array().unwrap();
    assert_eq!(advice.len(), 12, "{detailed}");
    for line in [4, 5, 7, 14] {
        assert_eq!(advice.iter().filter(|row| row["line"] == line).count(), 2);
    }
    assert_eq!(advice.iter().filter(|row| row["line"] == 13).count(), 4);
    for row in advice {
        let message = row["message"].as_str().unwrap();
        assert!(message.contains("statement:ALTER TABLE"), "{row}");
        assert!(message.contains("statement:DROP VIEW"), "{row}");
        assert!(message.contains("application"), "{row}");
    }
    assert!(advice
        .iter()
        .any(|row| row["line"] == 5 && row["message"].as_str().unwrap().contains("invalid-sql")));
    assert!(advice.iter().any(|row| row["line"] == 14
        && row["message"]
            .as_str()
            .unwrap()
            .contains("known SQL prefix")));
    // Finite alternatives keep their own violation origin; advice stays at the call.
    assert!(detailed["rules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["line"] == 9 && row["target"] == "CREATE TABLE"));
    detailed.as_object_mut().unwrap().remove("sqlVerdicts");
    assert_eq!(detailed, plain);
}

#[test]
fn fail_mode_keeps_finite_branch_origins_with_either_ledger_flag() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/advisory-aggregation");
    let plain = fixture_report(&root, "fail.yml", false, 1);
    let mut detailed = fixture_report(&root, "fail.yml", true, 1);
    assert!(plain["advisories"].as_array().unwrap().is_empty());
    let advice = fixture_report(&root, ".no-mistakes.yml", false, 1);
    let proven = |report: &serde_json::Value| {
        report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["target"] == "CREATE TABLE")
            .cloned()
            .collect::<Vec<_>>()
    };
    assert_eq!(proven(&plain), proven(&advice));
    assert!(proven(&plain).iter().any(|row| row["line"] == 9));
    // Fail-mode parse errors retain the invalid finite branch's origin line.
    assert!(plain["rules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["line"] == 10 && row["target"] == "unanalyzable-sql"));
    assert_eq!(detailed["sqlVerdicts"].as_array().unwrap().len(), 84);
    detailed.as_object_mut().unwrap().remove("sqlVerdicts");
    assert_eq!(detailed, plain);
}

#[test]
fn coincident_applications_preserve_identity_and_every_other_report_field() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/borrowed-applications");
    let plain = fixture_report(&root, ".no-mistakes.yml", false, 1);
    let mut detailed = fixture_report(&root, ".no-mistakes.yml", true, 1);
    assert_eq!(detailed, fixture_report(&root, ".no-mistakes.yml", true, 1));
    assert_eq!(plain["rules"].as_array().unwrap().len(), 1, "{plain}");
    assert!(
        !plain["advisories"].as_array().unwrap().is_empty(),
        "{plain}"
    );
    assert_eq!(plain["suppressed"].as_array().unwrap().len(), 1, "{plain}");
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let span = source.find("query(\"SELECT * FROM orders\")").unwrap() as u64;
    let rows = detailed["sqlVerdicts"].as_array().unwrap();
    let applications = rows
        .iter()
        .filter(|row| row["subcheck"] == "select-star" && row["span"][0] == span)
        .map(|row| (row["application"].clone(), row["status"].clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        applications,
        vec![
            (
                serde_json::json!({"index": 0, "name": "coincident"}),
                serde_json::json!("violation")
            ),
            (
                serde_json::json!({"index": 1, "name": "coincident"}),
                serde_json::json!("violation")
            ),
            (
                serde_json::json!({"index": 2}),
                serde_json::json!("proven-clear")
            ),
        ]
    );
    assert!(rows
        .iter()
        .any(|row| row["status"] == "abstain" && row["advisory"] == true));
    detailed.as_object_mut().unwrap().remove("sqlVerdicts");
    assert_eq!(detailed, plain);
}

#[test]
fn narrowed_preparation_preserves_imported_sql_and_excludes_private_executors() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/narrowed-preparation");
    let plain = fixture_report(&root, ".no-mistakes.yml", false, 1);
    let mut detailed = fixture_report(&root, ".no-mistakes.yml", true, 1);
    let rules = plain["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 3, "{plain}");
    assert!(
        rules
            .iter()
            .all(|row| row["target"].as_str().unwrap().starts_with("offset")),
        "{plain}"
    );
    assert!(
        rules.iter().any(|row| row["file"] == "selected/query.ts"),
        "{plain}"
    );
    assert!(
        rules.iter().any(|row| row["file"] == "other/query.ts"),
        "{plain}"
    );
    assert!(!rules
        .iter()
        .any(|row| row["file"].as_str().unwrap().starts_with("unselected/")));
    assert_eq!(plain["suppressed"].as_array().unwrap().len(), 1, "{plain}");
    let rows = detailed["sqlVerdicts"].as_array().unwrap();
    assert_eq!(rows.len(), 6, "{detailed}");
    assert!(
        rows.iter().all(|row| row["status"] == "violation"),
        "{detailed}"
    );
    for (index, name) in [(0, "selected"), (1, "overlap"), (2, "other-profile")] {
        assert!(
            rows.iter().any(|row| row["application"]["index"] == index
                && row["application"]["name"] == name
                && row["status"] == "violation"),
            "{detailed}"
        );
    }
    detailed.as_object_mut().unwrap().remove("sqlVerdicts");
    assert_eq!(detailed, plain);
}

#[test]
fn scoped_annotation_keeps_private_module_summaries_with_other_sql_profiles() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/narrowed-preparation");
    let plain = fixture_report(&root, "annotation.yml", false, 1);
    let mut detailed = fixture_report(&root, "annotation.yml", true, 1);
    let rules = plain["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 3, "{plain}");
    assert_eq!(
        rules
            .iter()
            .filter(|row| row["target"] == "annotation")
            .count(),
        2
    );
    let rows = detailed["sqlVerdicts"].as_array().unwrap();
    assert_eq!(rows.len(), 3, "{detailed}");
    assert!(rows
        .iter()
        .all(|row| row["status"] == "violation" && row["evidence"] == "exact"));
    detailed.as_object_mut().unwrap().remove("sqlVerdicts");
    assert_eq!(detailed, plain);
}

#[test]
fn low_annotation_heads_preserve_literal_outcomes_with_mixed_sql_scans() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/narrowed-preparation");
    for (config, count) in [("low.yml", 1), ("low-mixed.yml", 2)] {
        let plain = fixture_report(&root, config, false, 1);
        let mut detailed = fixture_report(&root, config, true, 1);
        let rules = plain["rules"].as_array().unwrap();
        assert_eq!(rules.len(), count, "{plain}");
        // The mixed file uses one tagged const at two executor profiles, but
        // only the annotation profile scans it. Both raw call spans are needed.
        assert!(
            !rules.iter().any(|row| row["file"] == "low/shared.ts"),
            "{plain}"
        );
        let low = rules
            .iter()
            .find(|row| row["file"] == "low/query.ts")
            .unwrap();
        assert_eq!(low["line"], 2);
        assert!(low["message"].as_str().unwrap().contains("const statement"));
        assert!(!rules
            .iter()
            .any(|row| row["file"] == "low/query.ts" && row["line"] == 3));
        detailed.as_object_mut().unwrap().remove("sqlVerdicts");
        assert_eq!(detailed, plain);
    }
}
