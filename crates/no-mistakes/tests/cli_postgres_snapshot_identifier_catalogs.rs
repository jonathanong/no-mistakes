use super::check_json_clean::assert_saved_fixture_clean;
use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_identifier_catalog_repros_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3073-catalogs",
    );
    for shape in ["producer", "record", "join"] {
        assert_saved_fixture_clean(
            &format!("postgres_identifier_catalog_{shape}"),
            &[(
                shape,
                root.clone(),
                root.join(format!(".no-mistakes-{shape}.yml")),
            )],
        );
    }
    for shape in ["producer-off", "producer-negative", "negative"] {
        assert_saved_fixture(
            &format!("postgres_identifier_catalog_{shape}"),
            &[(
                shape,
                root.clone(),
                root.join(format!(".no-mistakes-{shape}.yml")),
            )],
        );
    }
}
