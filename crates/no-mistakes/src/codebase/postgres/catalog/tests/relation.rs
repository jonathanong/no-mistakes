use super::SchemaCatalog;

#[test]
fn relation_uses_exact_names_then_a_unique_tail() {
    let catalog = SchemaCatalog::from_json(
        r#"{
            "formatVersion": 2,
            "tables": {
                "public.orders": { "columns": { "id": { "dataType": "integer" } } },
                "public.events": { "columns": { "id": { "dataType": "integer" } } },
                "other.events": { "columns": { "id": { "dataType": "integer" } } }
            }
        }"#,
    )
    .unwrap();
    assert_eq!(
        catalog
            .relation("public.orders")
            .map(|table| table.name.as_str()),
        Some("public.orders")
    );
    assert_eq!(
        catalog.relation("orders").map(|table| table.name.as_str()),
        Some("public.orders")
    );
    assert!(catalog.relation("schema.events").is_none());
    assert!(catalog.relation("missing").is_none());
}
