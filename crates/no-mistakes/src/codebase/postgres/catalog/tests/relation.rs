use super::SchemaCatalog;

#[test]
fn relation_uses_exact_names_then_a_unique_tail() {
    let catalog = SchemaCatalog::from_json(
        r#"{
            "formatVersion": 2, "coverage": "complete",
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
    // A name that spells another schema is not the bare name of a qualified key.
    assert!(catalog.relation("schema.orders").is_none());
    assert!(catalog.relation("missing").is_none());
    assert_eq!(catalog.column_line("public.orders", "id"), 1);
}

#[test]
fn an_explicit_foreign_schema_is_never_the_catalogs_bare_table() {
    let json = |schema: &str| {
        format!(
            r#"{{"formatVersion": 2, "coverage": "complete"{schema},
                "tables": {{"accounts": {{"columns": {{"id": {{"dataType": "integer"}}}}}}}}}}"#
        )
    };
    let own = SchemaCatalog::from_json(&json(r#", "schema": "public""#)).unwrap();
    for spelling in [
        "accounts",
        "public.accounts",
        "\"public\".accounts",
        "PUBLIC.accounts",
    ] {
        assert!(own.relation(spelling).is_some(), "{spelling}");
    }
    assert!(own.relation("audit.accounts").is_none());
    assert!(own.relation("\"Public\".accounts").is_none());
    // A catalog that names no schema cannot tell, and keeps matching the bare key.
    let anonymous = SchemaCatalog::from_json(&json("")).unwrap();
    assert!(anonymous.relation("audit.accounts").is_some());
}
