use super::*;

#[test]
fn propagates_invalid_optional_catalog_instead_of_guessing_relationships() {
    let mut options = config();
    options.rules[0].options["schemaCatalogPath"] = "schema.json".into();
    let error = check(&fixture("ddl-catalog-invalid"), &options).unwrap_err();
    let message = format!("{error:#}");
    assert!(message.contains("dataType"), "{message}");
}
