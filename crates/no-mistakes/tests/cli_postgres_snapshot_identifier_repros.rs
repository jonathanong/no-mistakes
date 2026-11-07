use super::check_json_clean::assert_saved_fixture_clean;
use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_imported_identifier_guard_repro_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3073-imported-guards",
    );
    assert_saved_fixture_clean(
        "postgres_imported_identifier_guard_repro",
        &[("clean", root.clone(), root.join(".no-mistakes.yml"))],
    );
    assert_saved_fixture_clean(
        "postgres_imported_identifier_guard_alias_readonly",
        &[(
            "clean",
            root.clone(),
            root.join(".no-mistakes-alias-readonly.yml"),
        )],
    );
    assert_saved_fixture(
        "postgres_imported_identifier_guard_alias_controls",
        &[(
            "reported",
            root.clone(),
            root.join(".no-mistakes-alias-controls.yml"),
        )],
    );
    assert_saved_fixture(
        "postgres_imported_identifier_guard_controls",
        &[(
            "reported",
            root.clone(),
            root.join(".no-mistakes-negative.yml"),
        )],
    );
}

#[test]
fn postgres_identifier_validator_return_repro_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3073-validator-returns",
    );
    for (label, config) in [
        ("origin", ".no-mistakes.yml"),
        ("callsite", ".no-mistakes-callsite.yml"),
        ("unconfigured", ".no-mistakes-disabled.yml"),
        ("controls", ".no-mistakes-negative.yml"),
    ] {
        let cases = [(label, root.clone(), root.join(config))];
        let name = format!("postgres_identifier_validator_return_{label}");
        if matches!(label, "origin" | "callsite") {
            assert_saved_fixture_clean(&name, &cases);
        } else {
            assert_saved_fixture(&name, &cases);
        }
    }
}
