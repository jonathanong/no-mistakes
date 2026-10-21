use super::*;
use std::path::PathBuf;

#[test]
fn conflict_ordering_distinguishes_nonconflict_uncertainty_and_violation() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/coverage-decisions/conflict");
    let report = run_all_with_sql_verdicts(
        root.clone(),
        Some(root.join(".no-mistakes.yml")),
        None,
        false,
        false,
        true,
    )
    .unwrap();
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let mut actual = report
        .sql_verdicts
        .iter()
        .filter(|row| row.file == "src/query.ts" && row.rule == "postgres-conflict-ordering")
        .map(|row| {
            let line = source[..row.span.0 as usize]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count()
                + 1;
            serde_json::json!({
                "line": line,
                "status": row.status,
                "residualReasons": row.residual_reasons,
            })
        })
        .collect::<Vec<_>>();
    actual.sort_by_key(|row| row["line"].as_u64().unwrap());
    let expected: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(root.join("expected.json")).unwrap())
            .unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn generated_predicates_abstain_when_unqualified_owner_is_ambiguous() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/coverage-decisions/generated");
    let report = run_all_with_sql_verdicts(
        root.clone(),
        Some(root.join(".no-mistakes.yml")),
        None,
        false,
        false,
        true,
    )
    .unwrap();
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let mut actual = report
        .sql_verdicts
        .iter()
        .filter(|row| {
            row.file == "src/query.ts" && row.rule == "postgres-generated-column-predicates"
        })
        .map(|row| {
            let line = source[..row.span.0 as usize]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count()
                + 1;
            serde_json::json!({
                "line": line,
                "status": row.status,
                "residualReasons": row.residual_reasons,
            })
        })
        .collect::<Vec<_>>();
    actual.sort_by_key(|row| row["line"].as_u64().unwrap());
    let expected: Vec<serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(root.join("expected.json")).unwrap())
            .unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn leading_violation_and_opaque_tail_keep_finding_and_advisory_distinct() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/coverage-decisions/annotation");
    let report = run_all_with_sql_verdicts(
        root.clone(),
        Some(root.join(".no-mistakes.yml")),
        None,
        false,
        false,
        true,
    )
    .unwrap();
    assert_eq!(report.sql_verdicts.len(), 1);
    let row = &report.sql_verdicts[0];
    assert_eq!(row.file, "src/query.ts");
    assert_eq!(row.subcheck, "leading-annotation");
    assert_eq!(
        row.status,
        no_mistakes::codebase::postgres::SqlVerdictStatus::Violation
    );
    assert_eq!(row.residual_reasons, ["opaque-sql-tail".to_string()]);
    assert_eq!(report.rules.len(), 1);
    assert_eq!(report.rules[0].file, "src/query.ts");
    assert_eq!(report.rules[0].line, 3);
    assert_eq!(report.advisories.len(), 1);
    assert_eq!(report.advisories[0].file, "src/query.ts");
    assert_eq!(report.advisories[0].line, 3);
    assert!(report.advisories[0]
        .message
        .contains("proved a violation, but remaining SQL is unresolved for"));
    assert!(report.advisories[0]
        .message
        .contains("application \"leading-prefix\""));
}

#[test]
fn nested_unknown_relation_keeps_bounded_select_uncertain() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/coverage-decisions/bounded");
    let report = run_all_with_sql_verdicts(
        root.clone(),
        Some(root.join(".no-mistakes.yml")),
        None,
        false,
        false,
        true,
    )
    .unwrap();
    assert_eq!(report.sql_verdicts.len(), 1);
    let row = &report.sql_verdicts[0];
    assert_eq!(row.file, "src/query.ts");
    assert_eq!(row.subcheck, "select");
    assert_eq!(
        row.status,
        no_mistakes::codebase::postgres::SqlVerdictStatus::Abstain
    );
    assert_eq!(row.residual_reasons, ["unknown-relation".to_string()]);
}
