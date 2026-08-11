use super::*;

#[test]
fn standalone_entrypoint_returns_configuration_errors() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/filesystem-dispatch/invalid-config");
    let error = run_filesystem_rules(&root, Some(&root.join(".no-mistakes.yml"))).unwrap_err();
    assert!(error.to_string().contains("parse"), "{error:#}");
}

#[test]
fn dispatch_uses_fallback_for_an_unknown_rule() {
    fn fallback(
        _root: &std::path::Path,
        _config: &crate::config::v2::NoMistakesConfig,
        _files: &[std::path::PathBuf],
    ) -> anyhow::Result<Vec<RuleFinding>> {
        Ok(vec![RuleFinding {
            rule: "fallback".to_string(),
            file: "fixture.txt".to_string(),
            line: 1,
            message: "fallback rule ran".to_string(),
            import: None,
            target: None,
        }])
    }

    let root = std::path::Path::new("/fixture");
    let config = crate::config::v2::NoMistakesConfig::default();
    let files = Vec::new();
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let findings = super::run_rule::run_rule_with_sources(super::run_rule::RunRuleRequest {
        rule_id: "future-filesystem-rule",
        fallback,
        root,
        config: &config,
        files: &files,
        sources: &sources,
        facts: None,
        defer_suppression: false,
    })
    .unwrap();

    assert_eq!(findings[0].rule, "fallback");
}
