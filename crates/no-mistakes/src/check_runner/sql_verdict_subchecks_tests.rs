use super::*;
use std::path::PathBuf;

#[test]
fn enabled_sql_subchecks_have_independent_exact_call_verdicts() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-sql-verdicts/fixture/subchecks");
    let cases: Vec<String> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-verdicts/fixture/subchecks/cases.json"
    )))
    .unwrap();
    for case in cases {
        let root = fixtures.join(&case);
        let report = run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(".no-mistakes.yml")),
            None,
            false,
            false,
            true,
        )
        .unwrap_or_else(|error| panic!("{case}: {error}"));
        let mut actual = report
            .sql_verdicts
            .iter()
            .filter(|row| row.file == "src/query.ts")
            .map(|row| {
                assert!(row.helper_lines.is_empty(), "{case}: {row:?}");
                assert!(row.known_prefix.is_none(), "{case}: {row:?}");
                serde_json::json!({
                    "span": row.span,
                    "subcheck": row.subcheck,
                    "status": row.status,
                    "evidence": row.evidence,
                    "residualReasons": row.residual_reasons,
                })
            })
            .collect::<Vec<_>>();
        actual.sort_by_key(|row| {
            (
                row["span"][0].as_u64().unwrap(),
                row["span"][1].as_u64().unwrap(),
                row["subcheck"].as_str().unwrap().to_string(),
            )
        });
        let expected: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(root.join("expected.json")).unwrap())
                .unwrap();
        // Full equality rejects omitted siblings, duplicate calls, and accidental
        // Known verdicts when a relation or column owner is unresolved.
        assert_eq!(serde_json::json!(actual), expected, "{case}");
    }
}
