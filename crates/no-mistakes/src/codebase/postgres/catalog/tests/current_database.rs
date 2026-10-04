use super::load_fixture;

#[test]
fn database_identity_is_explicit_and_preserves_quoted_case_and_dots() {
    assert_eq!(
        load_fixture("current-database.json")
            .unwrap()
            .current_database(),
        Some("Audit.Database")
    );
    assert_eq!(load_fixture("full.json").unwrap().current_database(), None);
}
