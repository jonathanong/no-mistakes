use super::*;
use std::sync::Arc;

#[test]
fn catalog_view_queries_share_parsed_definitions_and_cache_failures_lazily() {
    let path = crate::test_support::rule_fixture_root("postgres-materialized-view-shape")
        .join("fail/schema.json");
    let catalog = SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert!(catalog.projections.get().is_none());
    let first = catalog.try_view_query("good_range").unwrap();
    let qualified = catalog.try_view_query("public.good_range").unwrap();
    assert!(Arc::ptr_eq(&first, &qualified));
    assert!(Arc::ptr_eq(
        &first,
        &catalog.view_query("good_range").unwrap()
    ));
    let cloned = catalog.clone();
    assert!(Arc::ptr_eq(
        &first,
        &cloned.try_view_query("good_range").unwrap()
    ));
    let _ = catalog.view_column_names("good_range");
    assert_eq!(
        catalog.projections().view_projections.get().unwrap().len(),
        1
    );
    for _ in 0..10 {
        assert!(Arc::ptr_eq(
            &first,
            &catalog.try_view_query("good_range").unwrap()
        ));
        assert!(catalog.try_view_query("bad_parse").is_err());
        assert!(catalog.view_query("bad_parse").is_none());
        assert!(catalog.view_column_names("bad_parse").is_none());
    }
    assert_eq!(
        catalog.projections().view_projections.get().unwrap().len(),
        2
    );
    assert!(catalog.try_view_query("missing_view").is_err());
    assert_eq!(
        catalog.projections().view_projections.get().unwrap().len(),
        2
    );
}
