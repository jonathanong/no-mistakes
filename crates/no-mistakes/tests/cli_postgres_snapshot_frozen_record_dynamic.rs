use super::check_json_snapshot::{assert_saved_fixture, assert_saved_fixture_report, rules_path};

#[test]
fn postgres_frozen_record_dynamic_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-test-state-mutation",
        "fixture",
        "frozen-record-dynamic",
    );
    assert_saved_fixture(
        "postgres_frozen_record_dynamic",
        &[(
            "dynamic_key_enumerates_imported_records",
            root,
            rules_path(
                "postgres-no-test-state-mutation",
                "configs",
                "frozen-record-dynamic.yml",
            ),
        )],
    );
}

#[test]
fn postgres_frozen_record_alias_bindings_check_json_snapshot() {
    let rule = "postgres-no-test-state-mutation";
    let reports = assert_saved_fixture_report(
        "postgres_frozen_record_alias_bindings",
        &[(
            "only_const_aliases_are_resolved",
            rules_path(rule, "fixture", "frozen-record-aliases"),
            rules_path(rule, "configs", "frozen-record-aliases.yml"),
        )],
        &[],
        &[],
    );
    let report = &reports["only_const_aliases_are_resolved"]["report"];
    assert_eq!(report["rules"], serde_json::json!([]));
    let advisories = report["advisories"].as_array().unwrap();
    let lines: Vec<_> = advisories.iter().map(|item| item["line"].as_u64().unwrap()).collect();
    assert_eq!(lines, [14, 21], "{report}");
}
