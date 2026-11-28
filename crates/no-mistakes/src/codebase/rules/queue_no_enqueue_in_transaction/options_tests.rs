use super::*;
use std::collections::BTreeMap;

#[test]
fn queue_transaction_options_reject_incomplete_or_conflicting_contracts() {
    let fixture = crate::test_support::fixture_root("queue-transaction-options");
    let cases: BTreeMap<String, Options> =
        serde_yaml::from_str(&std::fs::read_to_string(fixture.join("cases.yml")).unwrap()).unwrap();
    for (name, reason) in [
        ("missing", "must both name at least one API"),
        ("duplicate", "more than once"),
        ("duplicate-begin", "more than once"),
        ("overlapping", "more than once"),
        ("blank", "non-empty module and export"),
        ("blank-callback", "non-empty module and export"),
        ("blank-begin", "non-empty module and export"),
    ] {
        let error = cases[name]
            .targets()
            .err()
            .expect("configuration is invalid");
        assert!(error.to_string().contains(reason), "{name}: {error}");
    }
}
