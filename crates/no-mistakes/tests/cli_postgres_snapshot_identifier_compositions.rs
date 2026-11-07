use super::check_json_clean::assert_saved_fixture_clean;
use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_guarded_identifier_compositions_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3073-guard-composition",
    );
    for name in [
        "conditional-fragment",
        "local-template",
        "alias-template",
        "early-return",
        "or-throw",
        "direct",
        "lookup",
        "multi-hole",
        "helper",
        "property",
        "captured",
    ] {
        assert_saved_fixture_clean(
            &format!("postgres_guarded_identifier_composition_{name}"),
            &[(
                name,
                root.clone(),
                root.join(format!(".no-mistakes-{name}.yml")),
            )],
        );
    }
}

#[test]
fn postgres_open_identifier_compositions_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3073-guard-composition",
    );
    assert_saved_fixture(
        "postgres_open_identifier_compositions",
        &[(
            "negative",
            root.clone(),
            root.join(".no-mistakes-negative.yml"),
        )],
    );
}

#[test]
fn postgres_open_conditional_fragments_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3073-guard-composition",
    );
    assert_saved_fixture(
        "postgres_open_conditional_fragments",
        &[(
            "open-conditional-fragment",
            root.clone(),
            root.join(".no-mistakes-open-conditional-fragment.yml"),
        )],
    );
}
