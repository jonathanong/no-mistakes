use super::super::{names, SchemaCatalog};

#[test]
fn bounded_collision_lookups_reuse_catalog_name_indexes() {
    let path = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("catalog-resolution/catalog.json");
    let catalog = SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap();
    let lookup = || {
        for name in [
            "customer_accounts",
            "public.customer_accounts",
            "bounded_customer_view",
            "public.bounded_customer_view",
            "active_accounts",
            "public.table_view_collision",
        ] {
            let _ = catalog.bounded_relation(name);
            let _ = catalog.view_relation(name);
        }
    };
    lookup();
    let indexed = names::key_split_count();
    // Measure catalog-key processing, rather than only output equality: the old scans split
    // every competing catalog key on each of these statement-sized lookups.
    for _ in 0..1000 {
        lookup();
    }
    assert_eq!(names::key_split_count(), indexed);
}

#[test]
fn catalogs_without_views_skip_table_collision_indexing() {
    let path = crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("catalog-resolution/catalog-ambiguous-owner.json");
    let catalog = SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(catalog.views().count(), 0);
    let before = names::key_split_count();
    for _ in 0..1000 {
        assert!(catalog.view_relation("customer_accounts").is_none());
    }
    assert_eq!(names::key_split_count(), before);
    assert!(catalog.relation_fallback.get().is_none());
}
