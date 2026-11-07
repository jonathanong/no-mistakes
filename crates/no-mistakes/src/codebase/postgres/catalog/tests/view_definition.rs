use super::SchemaCatalog;
use std::collections::BTreeSet;
use std::sync::Arc;

fn catalog() -> SchemaCatalog {
    let path = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("view-projection-columns/schema.json");
    SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn view_output_names_are_independent_of_key_inference() {
    let catalog = catalog();
    for (view, expected) in [
        ("simple_view", vec!["active", "id", "label_text"]),
        ("grouped_view", vec!["active", "id"]),
        ("union_view", vec!["key"]),
        ("nested_view", vec!["id"]),
        ("cte_view", vec!["id"]),
        ("quoted_view", vec!["MiXeD"]),
    ] {
        let names = catalog.view_column_names(view).unwrap();
        let expected: BTreeSet<String> = expected.into_iter().map(str::to_owned).collect();
        assert_eq!(names.as_ref(), &expected, "{view}");
    }
}

#[test]
fn uncertain_view_output_names_and_colliding_identities_stay_unknown() {
    let catalog = catalog();
    for view in [
        "wildcard_view",
        "qualified_wildcard_view",
        "unnamed_view",
        "values_view",
        "non_query_view",
        "invalid_view",
        "multiple_view",
        "by_name_view",
        "pipe_view",
        "colliding_view",
        "missing_view",
    ] {
        assert!(catalog.view_column_names(view).is_none(), "{view}");
    }
}

#[test]
fn view_projection_preparation_is_lazy_shared_and_catalog_local() {
    let catalog = catalog();
    assert!(catalog.view_projections.get().is_none());
    assert!(catalog.view_query("missing_view").is_none());
    assert!(catalog.view_projections.get().is_none());
    let names = catalog.view_column_names("simple_view").unwrap();
    let query = catalog.view_query("simple_view").unwrap();
    assert_eq!(catalog.view_projections.get().unwrap().len(), 1);
    for _ in 0..10 {
        assert!(Arc::ptr_eq(
            &names,
            &catalog.view_column_names("simple_view").unwrap()
        ));
        assert!(Arc::ptr_eq(
            &query,
            &catalog.view_query("public.simple_view").unwrap()
        ));
    }
    let cloned = catalog.clone();
    assert!(Arc::ptr_eq(
        &query,
        &cloned.view_query("simple_view").unwrap()
    ));
    let independent = self::catalog();
    assert!(!Arc::ptr_eq(
        &query,
        &independent.view_query("simple_view").unwrap()
    ));
    for _ in 0..2 {
        assert!(catalog.view_query("invalid_view").is_none());
    }
    assert_eq!(catalog.view_projections.get().unwrap().len(), 2);
}

#[test]
fn concurrent_view_queries_reuse_one_prepared_ast() {
    let catalog = catalog();
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..4)
            .map(|_| scope.spawn(|| catalog.view_query("simple_view").unwrap()))
            .collect();
        let queries: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        assert!(queries.iter().all(|query| Arc::ptr_eq(&queries[0], query)));
    });
    assert_eq!(catalog.view_projections.get().unwrap().len(), 1);
}
