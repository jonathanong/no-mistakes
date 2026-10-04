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
    // Names are split quote-aware: a quoted schema or table that contains a dot is one part.
    let quoted = SchemaCatalog::from_json(
        r#"{"formatVersion": 2, "coverage": "complete", "schema": "Catalog.Test",
            "tables": {"\"audit.log\"": {"columns": {"id": {"dataType": "integer"}}}}}"#,
    )
    .unwrap();
    for spelling in [
        "\"audit.log\"",
        "\"Catalog.Test\".\"audit.log\"",
        "\"Catalog.Test\".\"AUDIT.LOG\"",
    ] {
        assert_eq!(
            quoted.relation(spelling).is_some(),
            !spelling.contains("AUDIT"),
            "{spelling}"
        );
    }
    assert!(quoted.relation("public.\"audit.log\"").is_none());
    // A catalog that names no schema cannot tell, and keeps matching the bare key.
    let anonymous = SchemaCatalog::from_json(&json("")).unwrap();
    assert!(anonymous.relation("audit.accounts").is_some());
}

#[test]
fn relation_identity_needs_explicit_schema_evidence_for_bare_names() {
    let json = |schema: &str| {
        format!(
            r#"{{"formatVersion": 2, "coverage": "complete"{schema},
                "tables": {{"accounts": {{"columns": {{"id": {{"dataType": "integer"}}}}}}}}}}"#
        )
    };
    let explicit = SchemaCatalog::from_json(&json(r#", "schema": "public""#)).unwrap();
    assert!(explicit.same_relation("public.accounts", "accounts"));
    assert!(!explicit.same_relation("audit.accounts", "accounts"));

    let anonymous = SchemaCatalog::from_json(&json("")).unwrap();
    assert!(anonymous.same_relation("accounts", "accounts"));
    assert!(!anonymous.same_relation("public.accounts", "accounts"));
}
