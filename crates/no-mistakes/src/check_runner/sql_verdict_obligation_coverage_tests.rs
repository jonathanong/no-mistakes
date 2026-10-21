use super::*;
use std::path::PathBuf;

#[test]
fn obligation_verdicts_preserve_lock_width_and_whole_row_evidence() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/coverage-obligations");
    for case in ["locks", "generated", "explicit"] {
        let root = fixtures.join(case);
        let report = run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(".no-mistakes.yml")),
            None,
            false,
            false,
            true,
        )
        .unwrap_or_else(|error| panic!("{case}: {error}"));
        let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
        let mut actual = report
            .sql_verdicts
            .iter()
            .filter(|row| row.file == "src/query.ts")
            .map(|row| {
                let line = source[..row.span.0 as usize]
                    .bytes()
                    .filter(|byte| *byte == b'\n')
                    .count()
                    + 1;
                serde_json::json!({
                    "line": line,
                    "subcheck": row.subcheck,
                    "status": row.status,
                    "residualReasons": row.residual_reasons,
                })
            })
            .collect::<Vec<_>>();
        let mut expected: Vec<serde_json::Value> =
            serde_json::from_str(&std::fs::read_to_string(root.join("expected.json")).unwrap())
                .unwrap();
        for rows in [&mut actual, &mut expected] {
            rows.sort_by_key(|row| {
                (
                    row["line"].as_u64().unwrap(),
                    row["subcheck"].as_str().unwrap().to_string(),
                )
            });
        }
        // Whole equality rejects absent calls and sibling subchecks, and keeps
        // incomplete catalog/width evidence from becoming a false Known.
        assert_eq!(actual, expected, "{case}");
    }
}
