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
