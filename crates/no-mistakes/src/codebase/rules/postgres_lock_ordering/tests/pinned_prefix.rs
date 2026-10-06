use super::super::catalog_check::orders_by_catalog_key;
use crate::codebase::postgres::{CanonicalOrderKey, LockingSelectMetadata, SchemaCatalog};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn catalog() -> SchemaCatalog {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/lock-ordering/pass-pinned-key-prefix/schema.json");
    SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn key(expression: &str, ascending: bool) -> CanonicalOrderKey {
    CanonicalOrderKey {
        expression: expression.to_owned(),
        ascending,
        nulls_first: !ascending,
    }
}

fn lock(order: Vec<CanonicalOrderKey>, pinned: Option<Vec<&str>>) -> LockingSelectMetadata {
    LockingSelectMetadata {
        has_multi_row_predicate: true,
        has_order_by: true,
        skips_locked_rows: false,
        tables: Some(vec!["feed_items".to_owned()]),
        table_qualifiers: Some(BTreeMap::from([(
            "feed_items".to_owned(),
            vec!["feed_items".to_owned()],
        )])),
        order: Some(order),
        pinned_columns: pinned.map(|columns| {
            BTreeMap::from([(
                "feed_items".to_owned(),
                columns.into_iter().map(str::to_owned).collect(),
            )])
        }),
        join_equalities: Vec::new(),
    }
}

#[test]
fn pinned_leading_columns_satisfy_catalog_key_order() {
    let catalog = catalog();
    let guid = vec![key("guid", true)];
    assert!(orders_by_catalog_key(
        &lock(guid.clone(), Some(vec!["host_id"])),
        &catalog,
    ));
    // Keeping the pinned column is still (host_id, guid) order.
    assert!(orders_by_catalog_key(
        &lock(
            vec![key("host_id", true), key("guid", true)],
            Some(vec!["host_id"]),
        ),
        &catalog,
    ));
    assert!(orders_by_catalog_key(
        &lock(
            vec![key("guid", true), key("id", true)],
            Some(vec!["host_id"]),
        ),
        &catalog,
    ));
    assert!(orders_by_catalog_key(
        &lock(vec![key("feed_items.guid", true)], Some(vec!["host_id"])),
        &catalog,
    ));
    // The partial guid index does not count, and an unpinned host_id stays required.
    assert!(!orders_by_catalog_key(
        &lock(guid.clone(), Some(vec![])),
        &catalog,
    ));
    assert!(!orders_by_catalog_key(&lock(guid.clone(), None), &catalog));
    let mut other_table = lock(guid, Some(vec!["host_id"]));
    other_table.pinned_columns = Some(BTreeMap::from([(
        "hosts".to_owned(),
        vec!["host_id".to_owned()],
    )]));
    assert!(!orders_by_catalog_key(&other_table, &catalog));
    assert!(!orders_by_catalog_key(
        &lock(vec![key("guid", false)], Some(vec!["host_id"])),
        &catalog,
    ));
    assert!(!orders_by_catalog_key(
        &lock(vec![key("other.guid", true)], Some(vec!["host_id"])),
        &catalog,
    ));
    assert!(!orders_by_catalog_key(
        &lock(vec![key("host_id", true)], Some(vec!["guid"])),
        &catalog,
    ));
    assert!(orders_by_catalog_key(
        &lock(vec![key("other", true)], Some(vec!["host_id", "guid"])),
        &catalog,
    ));
    assert!(orders_by_catalog_key(
        &lock(Vec::new(), Some(vec!["host_id", "guid"])),
        &catalog,
    ));
}
