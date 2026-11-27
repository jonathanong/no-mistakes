use super::facts;

#[test]
fn index_identity_feature_modes_keep_the_legacy_default_contract() {
    // Expected strings were captured from the default-feature main addon, not
    // rebuilt from the implementation under test or sorted during comparison.
    let parsed = facts("index-identity-feature-parity.sql");
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let expected: Vec<String> = serde_json::from_str(&super::fixture(
        "index-identity-feature-parity.expected.json",
    ))
    .unwrap();
    let identities: Vec<_> = parsed
        .statements
        .iter()
        .map(|statement| {
            let super::super::PostgresSqlStatementKind::CreateIndex { index } = &statement.facts
            else {
                panic!("saved fixture only contains indexes");
            };
            index.structural_identity.as_str()
        })
        .collect();
    assert_eq!(identities, expected);
}
