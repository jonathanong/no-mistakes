use super::run_all_with_sql_verdicts;
use no_mistakes::codebase::postgres::SqlVerdictStatus;
use std::path::PathBuf;

#[test]
fn imported_sql_advice_names_helper_lines_when_catalog_ownership_is_unknown() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/edge-cases/helper-lines");
    let report = run_all_with_sql_verdicts(
        root.clone(),
        Some(root.join(".no-mistakes.yml")),
        None,
        false,
        false,
        true,
    )
    .unwrap();
    let row = report
        .sql_verdicts
        .iter()
        .find(|row| {
            row.file == "src/query.mts" && row.rule == "postgres-no-generated-column-writes"
        })
        .expect("prepared imported-fragment verdict");
    assert_eq!(row.status, SqlVerdictStatus::Abstain);
    assert_eq!(row.helper_lines, [4]);
    assert!(report.rules.is_empty(), "{:?}", report.rules);
    assert!(
        report.advisories.iter().any(|finding| {
            finding.file == "src/query.mts" && finding.message.contains("; helper lines [4]")
        }),
        "{:?}",
        report.advisories
    );
}
