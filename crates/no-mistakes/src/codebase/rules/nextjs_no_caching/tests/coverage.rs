use super::*;

#[test]
fn fact_runner_ignores_missing_facts_outside_target_roots() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture());
    let outside = root.join("other/app/bad.ts");
    let facts = CheckFactMap {
        files: vec![outside.clone()],
        ts: [(outside, std::sync::Arc::new(CheckFileFacts::default()))]
            .into_iter()
            .collect(),
        ..Default::default()
    };
    let findings = check_with_facts(&root, &config(), &facts).unwrap();

    assert!(findings.is_empty());
}

#[test]
fn fact_runner_requires_source_and_cache_facts_for_target_files() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture());
    let inside = root.join("web/app/bad.ts");
    let missing_source = CheckFactMap {
        files: vec![inside.clone()],
        ts: [(
            inside.clone(),
            std::sync::Arc::new(CheckFileFacts::default()),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    };
    let err = check_with_facts(&root, &config(), &missing_source).unwrap_err();
    assert!(err.to_string().contains("requires source facts"), "{err:?}");

    let missing_cache = CheckFactMap {
        files: vec![inside.clone()],
        ts: [(
            inside,
            CheckFileFacts {
                source: Some("export const value = 1".into()),
                ..Default::default()
            }
            .into(),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    };
    let err = check_with_facts(&root, &config(), &missing_cache).unwrap_err();
    assert!(
        err.to_string().contains("requires Next.js caching facts"),
        "{err:?}"
    );
}

#[test]
fn fact_runner_reports_invalid_rule_include_globs() {
    let root = fixture();
    let mut config = config();
    config.rules[0].include = vec!["[".to_string()];

    let error = check_with_facts(&root, &config, &CheckFactMap::default()).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("rule `nextjs-no-caching` include contains invalid glob"),
        "{error:#}"
    );
}
