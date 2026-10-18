use super::*;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Case {
    fixture: String,
    subcheck: String,
    status: String,
}

#[test]
fn saved_rule_adapter_matrix_uses_actual_prepared_policy_facts() {
    let cases: Vec<Case> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-verdicts/fixture/adapter-matrix.json"
    )))
    .unwrap();
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/rules");
    for case in cases {
        let root = fixtures.join(&case.fixture);
        let report = run_all_with_sql_verdicts(
            root.clone(),
            Some(root.join(".no-mistakes.yml")),
            None,
            false,
            false,
            true,
        )
        .unwrap_or_else(|error| panic!("{}: {error}", case.fixture));
        let rows = report
            .sql_verdicts
            .iter()
            .filter(|row| row.subcheck == case.subcheck)
            .collect::<Vec<_>>();
        assert!(
            !rows.is_empty(),
            "{}: no {} rows",
            case.fixture,
            case.subcheck
        );
        assert!(
            rows.iter().any(|row| {
                serde_json::to_value(row.status).unwrap().as_str() == Some(case.status.as_str())
            }),
            "{}: expected {} {} in {:?}",
            case.fixture,
            case.subcheck,
            case.status,
            rows
        );
    }
}
