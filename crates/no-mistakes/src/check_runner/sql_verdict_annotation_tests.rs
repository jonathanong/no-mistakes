use super::run_all_with_sql_verdicts;
use no_mistakes::codebase::postgres::{SqlEvidenceKind, SqlVerdictStatus};
use std::path::PathBuf;

#[test]
fn a_verified_leading_annotation_has_no_residual_advisory() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture/provenance-advice");
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
    let start = source
        .find("query(`/* known */ SELECT ${opaque}`)")
        .unwrap();
    let rows = report
        .sql_verdicts
        .iter()
        .filter(|row| row.file == "src/query.ts" && row.span.0 as usize == start)
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 1, "{rows:?}");
    let row = rows[0];
    assert_eq!(row.status, SqlVerdictStatus::ProvenClear);
    assert_eq!(row.evidence, Some(SqlEvidenceKind::VerifiedPrefix));
    assert!(row.residual_reasons.is_empty(), "{row:?}");
    assert_eq!(report.advisories.len(), 1, "{:?}", report.advisories);
    assert_eq!(report.advisories[0].line, 6);
}
