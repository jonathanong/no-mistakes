use super::*;
use serde::Deserialize;
mod production;

#[derive(Deserialize)]
struct Case {
    prefix: String,
    status: String,
}

#[test]
fn saved_prefix_proofs_keep_incomplete_leaders_unproved() {
    let cases: Vec<Case> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-require-query-annotation/fixture/provenance-advice/prefix-proofs.json"
    )))
    .unwrap();
    for case in cases {
        let expected = match case.status.as_str() {
            "violation" => SqlVerdictStatus::Violation,
            "proven-clear" => SqlVerdictStatus::ProvenClear,
            "abstain" => SqlVerdictStatus::Abstain,
            status => panic!("invalid saved status: {status}"),
        };
        assert_eq!(prefix_status(&case.prefix), expected, "{:?}", case.prefix);
    }
}
