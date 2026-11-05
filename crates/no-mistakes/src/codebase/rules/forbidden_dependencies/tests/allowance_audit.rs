use super::*;

#[test]
fn overlapping_exceptions_are_used_and_stale_targets_require_full_source_scope() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/rules/unused-allow-forbidden"),
    );
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let opts: Options = config.rules[0].try_rule_options().unwrap();
    let tsconfig = resolve_tsconfig(&root, None).unwrap();
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let findings = check_rule_application(
        &root,
        &config,
        &config.rules[0],
        &opts,
        &graph,
        application_scope(None, None, Some(&sources)),
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].rule, "unused-allow-entry");
    assert_eq!(findings[0].target.as_deref(), Some("clean.ts"));
    let mut narrowed = config.clone();
    narrowed.rules[0].include = vec!["target.ts".into()];
    let findings = check_rule_application(
        &root,
        &narrowed,
        &narrowed.rules[0],
        &opts,
        &graph,
        application_scope(None, None, Some(&sources)),
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn dynamic_resources_cannot_prove_an_allow_entry_unused() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/rules/unused-allow-forbidden"),
    );
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let mut opts: Options = config.rules[0].try_rule_options().unwrap();
    opts.relationships.clear();
    let graph = DepGraph::build_with_plan(
        &root,
        &resolve_tsconfig(&root, None).unwrap(),
        GraphBuildPlan {
            resources: true,
            ..GraphBuildPlan::imports_and_workspace()
        },
    )
    .unwrap();
    assert!(!graph.resource_diagnostics().is_empty());
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let findings = check_rule_application(
        &root,
        &config,
        &config.rules[0],
        &opts,
        &graph,
        application_scope(None, None, Some(&sources)),
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}
