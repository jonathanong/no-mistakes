use super::tests::run;

#[test]
fn expression_unique_keys_reuse_catalog_columns_without_matching_names_or_literals() {
    let findings = run(
        "schemaCatalogPath: schema-expression-index.json\nfingerprintNamePattern: '_sha256$'",
        &["schema-expression-index.json"],
    );
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let finding = &findings[0];
    assert_eq!(
        finding.target.as_deref(),
        Some("column:invoices.document_sha256")
    );
    assert_eq!(finding.message, format!(
        "schema-expression-index.json:{}: column invoices.document_sha256 matches fingerprintNamePattern and is part of UNIQUE key invoices_lower_hash_key; key on foreign key columns instead",
        finding.line
    ));
}
