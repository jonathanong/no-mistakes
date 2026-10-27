// The auxiliary path is named in the inventory but is not on disk, so CLI
// discovery cannot produce the "not a JSON object" finding.

#[test]
fn missing_auxiliary_source_is_not_a_json_object() {
    let root = fixture("missing-auxiliary");
    let files = vec![
        root.join("tsconfig.json"),
        root.join("index.ts"),
        root.join("tsconfig.dependency-cruiser.json"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
auxiliaryConfigs:
  - path: tsconfig.dependency-cruiser.json
    reason: missing from disk
"#,
        ),
        &files,
    )
    .unwrap();
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("not a JSON object")),
        "{findings:?}"
    );
}
