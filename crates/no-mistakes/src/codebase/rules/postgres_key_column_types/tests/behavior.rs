use super::super::{compile::compile, scan::scan, Options};
use crate::codebase::postgres::SchemaCatalog;
use std::path::PathBuf;

#[test]
fn refuses_to_guess_primary_key_constraint_identity_without_primary_index_metadata() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/key-column-types/catalogs/missing-primary-index.json");
    let source = std::fs::read_to_string(path).unwrap();
    let catalog = SchemaCatalog::from_json(&source).unwrap();
    let options: Options =
        serde_yaml::from_str("schemaCatalogPath: schema.json\nallowedTypes: [uuid]\n").unwrap();
    let compiled = compile(&options, None).unwrap();
    let error = scan(&catalog, &compiled, "schema.json").unwrap_err();
    assert!(
        error
            .to_string()
            .contains("has a primary key but no primary index name"),
        "{error:#}"
    );
    assert!(
        error.to_string().contains("regenerate the schema catalog"),
        "{error:#}"
    );
}

#[test]
fn refuses_empty_primary_and_foreign_key_column_lists() {
    for (fixture, expected) in [
        (
            "fail-empty-primary",
            "primary key orders.orders_pkey has no columns",
        ),
        (
            "fail-empty-foreign",
            "foreign key orders.invalid_fkey has no columns",
        ),
    ] {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-key-column-types/fixture")
            .join(fixture);
        let source = std::fs::read_to_string(root.join("schema.json")).unwrap();
        let catalog = SchemaCatalog::from_json(&source).unwrap();
        let options: Options =
            serde_yaml::from_str("schemaCatalogPath: schema.json\nallowedTypes: [uuid]\n").unwrap();
        let compiled = compile(&options, None).unwrap();
        let error = scan(&catalog, &compiled, "schema.json").unwrap_err();
        assert!(error.to_string().contains(expected), "{error:#}");
        assert!(
            error.to_string().contains("regenerate the schema catalog"),
            "{error:#}"
        );
    }
}
