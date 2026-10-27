#[test]
fn a_qualified_reference_to_the_same_catalog_relation_is_local() {
    let catalog = super::catalog();
    assert!(catalog.same_relation("accounts", "accounts"));
    assert!(catalog.same_relation("public.accounts", "accounts"));
    assert!(!catalog.same_relation("audit.accounts", "accounts"));
}
