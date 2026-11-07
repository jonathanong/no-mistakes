use super::SchemaCatalog;
use std::sync::Arc;

fn catalog() -> SchemaCatalog {
    let path = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("view-projection-columns/schema.json");
    SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn scalar_names_do_not_initialize_view_projections() {
    let catalog = catalog();
    assert!(catalog.projections.get().is_none());
    let names = catalog.scalar_function_names();
    let cache = catalog.projections.get().unwrap();
    assert!(cache.view_projections.get().is_none());
    assert!(Arc::ptr_eq(
        &names,
        cache.scalar_function_names.get().unwrap()
    ));
    let cloned = catalog.clone();
    assert!(Arc::ptr_eq(cache, cloned.projections.get().unwrap()));
    assert!(Arc::ptr_eq(&names, &cloned.scalar_function_names()));
    let query = cloned.view_query("simple_view").unwrap();
    assert!(Arc::ptr_eq(
        &query,
        &catalog.view_query("simple_view").unwrap()
    ));
    assert!(Arc::ptr_eq(&names, &catalog.scalar_function_names()));
}

#[test]
fn view_projections_do_not_initialize_scalar_names() {
    let catalog = catalog();
    assert!(catalog.projections.get().is_none());
    let query = catalog.view_query("simple_view").unwrap();
    let cache = catalog.projections.get().unwrap();
    assert!(cache.scalar_function_names.get().is_none());
    assert_eq!(cache.view_projections.get().unwrap().len(), 1);
    let names = catalog.scalar_function_names();
    assert!(Arc::ptr_eq(
        &names,
        cache.scalar_function_names.get().unwrap()
    ));
    assert!(Arc::ptr_eq(
        &query,
        &catalog.view_query("simple_view").unwrap()
    ));
    assert_eq!(cache.view_projections.get().unwrap().len(), 1);
}
