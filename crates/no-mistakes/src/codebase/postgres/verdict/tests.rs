use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    branches: Vec<String>,
    exhaustive: bool,
    positive_survives_unknown: bool,
    expected: String,
}

fn parse(value: &str) -> SqlVerdictStatus {
    match value {
        "violation" => SqlVerdictStatus::Violation,
        "proven-clear" => SqlVerdictStatus::ProvenClear,
        "abstain" => SqlVerdictStatus::Abstain,
        _ => panic!("invalid saved verdict case: {value}"),
    }
}

#[test]
fn saved_branch_proofs_preserve_implicit_unknown() {
    let cases: Vec<Case> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-verdicts/fixture/branch-proofs.json"
    )))
    .unwrap();
    for case in cases {
        let branches = case
            .branches
            .iter()
            .map(|value| parse(value))
            .collect::<Vec<_>>();
        assert_eq!(
            combine_sql_verdicts(&branches, case.exhaustive, case.positive_survives_unknown,),
            parse(&case.expected),
            "branches: {:?}, exhaustive: {}",
            case.branches,
            case.exhaustive
        );
    }
}
