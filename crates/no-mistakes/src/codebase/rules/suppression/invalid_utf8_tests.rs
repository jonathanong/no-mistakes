use std::path::PathBuf;

#[test]
fn invalid_utf8_file_directive_still_suppresses() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-conflict-ordering/invalid-utf8");
    let path = root.join("caller.ts");
    let sources = crate::codebase::rules::source_store_for_files(&[path]);
    let mut findings = vec![crate::codebase::rules::RuleFinding {
        rule: "postgres-conflict-ordering".to_string(),
        file: "caller.ts".to_string(),
        line: 2,
        message: "caller.ts:2: conflict".to_string(),
        import: None,
        target: None,
        causes: Vec::new(),
    }];
    super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:?}");
}
