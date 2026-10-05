use super::support::{fixtures, Database};

#[test]
fn a_recorded_database_name_makes_catalogs_independent_of_the_generating_database() {
    let (Some(first), Some(second)) = (
        Database::create("recorded_a"),
        Database::create("recorded_b"),
    ) else {
        return;
    };
    let schema = fixtures().join("all-rules/schema.sql");
    first.load(&schema);
    second.load(&schema);
    // The throwaway databases have different names, so only the recorded name can match.
    assert_ne!(
        first.query("SELECT current_database()"),
        second.query("SELECT current_database()")
    );
    let recorded = ["--current-database", "app"];
    let bytes = first.catalog_bytes_with("catalog_demo", &recorded);
    assert_eq!(
        bytes,
        second.catalog_bytes_with("catalog_demo", &recorded),
        "one --current-database gives byte-identical catalogs from differently named databases"
    );
    let catalog: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(catalog["currentDatabase"], "app");
    // Every other fact is still what the connected database reports.
    let mut observed = first.catalog("catalog_demo", None);
    assert_ne!(observed["currentDatabase"], "app");
    observed["currentDatabase"] = "app".into();
    assert_eq!(catalog, observed);
}
