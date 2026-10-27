use super::*;

use std::path::{Path, PathBuf};

mod directory_roots;
mod shared_facts;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
}

fn resolve_tsconfig(
    root: &Path,
    explicit: Option<&Path>,
) -> Result<crate::codebase::ts_resolver::TsConfig> {
    crate::codebase::ts_resolver::resolve_tsconfig(explicit, root)
}

fn check_application(root: &Path, opts: &Options, graph: &DepGraph) -> Result<Vec<RuleFinding>> {
    let config = NoMistakesConfig {
        rules: vec![crate::config::v2::schema::RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(crate::config::v2::schema::RuleScope::Repository),
            ..Default::default()
        }],
        ..Default::default()
    };
    check_rule_application(
        root,
        &config,
        &config.rules[0],
        opts,
        graph,
        application_scope(None, None, None),
    )
}

fn check_with_facts(
    root: &Path,
    config: &NoMistakesConfig,
    config_path: Option<&Path>,
    tsconfig_path: Option<&Path>,
    shared: &crate::codebase::check_facts::CheckFactMap,
) -> Result<Vec<RuleFinding>> {
    let tsconfig = resolve_tsconfig(root, tsconfig_path)?;
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    shared_facts::check_with_prepared_facts_and_session(
        root,
        config,
        config_path,
        &tsconfig,
        shared,
        None,
        &session,
    )
}

#[test]
fn missing_forbidden_list_emits_config_error() {
    let root = fixture("forbidden-dependencies-invalid");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = check(&root, &config, None).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert!(findings[0]
        .message
        .contains("forbiddenModules or forbiddenFiles"));
}

#[test]
fn invalid_glob_pattern_emits_config_finding() {
    let root = fixture("forbidden-dependencies-basic");
    let opts = Options {
        roots: vec!["entrypoints/api.mts".to_string()],
        forbidden_modules: vec!["[invalid".to_string()],
        forbidden_files: vec![],
        relationships: vec![],
        ..Default::default()
    };
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = crate::codebase::dependencies::graph::DepGraph::build_with_plan(
        &root,
        &tsconfig,
        crate::codebase::dependencies::graph::GraphBuildPlan::imports_and_workspace(),
    )
    .unwrap();
    let findings = check_application(&root, &opts, &graph).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert!(
        findings[0].message.contains("invalid glob pattern"),
        "message should mention invalid glob pattern: {}",
        findings[0].message
    );
}

#[test]
fn invalid_glob_pattern_in_forbidden_files_emits_config_finding() {
    let root = fixture("forbidden-dependencies-basic");
    let opts = Options {
        roots: vec!["entrypoints/api.mts".to_string()],
        forbidden_modules: vec![],
        forbidden_files: vec!["[invalid".to_string()],
        relationships: vec![],
        ..Default::default()
    };
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = crate::codebase::dependencies::graph::DepGraph::build_with_plan(
        &root,
        &tsconfig,
        crate::codebase::dependencies::graph::GraphBuildPlan::imports_and_workspace(),
    )
    .unwrap();
    let findings = check_application(&root, &opts, &graph).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, ".no-mistakes.yml");
    assert!(
        findings[0].message.contains("invalid glob pattern"),
        "message should mention invalid glob pattern: {}",
        findings[0].message
    );
}

#[test]
fn nonexistent_root_is_silently_skipped() {
    let root = fixture("forbidden-dependencies-basic");
    let opts = Options {
        roots: vec!["entrypoints/does-not-exist.mts".to_string()],
        forbidden_modules: vec!["sharp".to_string()],
        forbidden_files: vec![],
        relationships: vec![],
        ..Default::default()
    };
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = crate::codebase::dependencies::graph::DepGraph::build_with_plan(
        &root,
        &tsconfig,
        crate::codebase::dependencies::graph::GraphBuildPlan::imports_and_workspace(),
    )
    .unwrap();
    let findings = check_application(&root, &opts, &graph).unwrap();
    assert!(
        findings.is_empty(),
        "nonexistent root should produce no findings"
    );
}

#[test]
fn queue_job_nodes_are_not_matched() {
    use crate::codebase::dependencies::graph::test_support::from_typed_maps;
    use crate::codebase::dependencies::{EdgeKind, NodeId};
    let root = fixture("forbidden-dependencies-basic");
    let root_file = crate::codebase::ts_resolver::normalize_path(&root.join("entrypoints/api.mts"));
    let queue_file = crate::codebase::ts_resolver::normalize_path(&root.join("jobs/queue.mts"));
    let root_node = NodeId::file(root_file.clone());
    let queue_node = NodeId::queue_job(queue_file.clone(), "process");
    let forward = [(
        root_node.clone(),
        vec![(queue_node.clone(), EdgeKind::QueueEnqueue)],
    )]
    .into_iter()
    .collect();
    let reverse = [(
        queue_node.clone(),
        vec![(root_node.clone(), EdgeKind::QueueEnqueue)],
    )]
    .into_iter()
    .collect();
    let graph = from_typed_maps(root.clone(), forward, reverse);
    let opts = Options {
        roots: vec!["entrypoints/api.mts".to_string()],
        forbidden_modules: vec!["*".to_string()],
        forbidden_files: vec!["**".to_string()],
        relationships: vec![],
        ..Default::default()
    };
    let findings = check_application(&root, &opts, &graph).unwrap();
    assert!(findings.is_empty(), "QueueJob nodes should not be matched");
}

#[test]
fn no_rule_configured_returns_empty() {
    let root = fixture("forbidden-dependencies-basic");
    let raw = "rules: []";
    let cfg: crate::config::v2::NoMistakesConfig = serde_yaml::from_str(raw).unwrap();
    let findings = check(&root, &cfg, None).unwrap();
    assert!(
        findings.is_empty(),
        "should return empty when rule not configured"
    );
}

#[test]
fn explicit_tsconfig_path_is_used() {
    let root = fixture("forbidden-dependencies-basic");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let tsconfig_path = root.join("tsconfig.json");
    let findings = check(&root, &config, Some(&tsconfig_path)).unwrap();
    assert_eq!(
        findings.len(),
        1,
        "should still find the violation with explicit tsconfig"
    );
}

#[test]
fn absolute_root_path_resolves() {
    let root = fixture("forbidden-dependencies-basic");
    let abs_root = root.join("entrypoints/api.mts");
    let opts = Options {
        roots: vec![abs_root.to_string_lossy().to_string()],
        forbidden_modules: vec!["sharp".to_string()],
        forbidden_files: vec![],
        relationships: vec![],
        ..Default::default()
    };
    let tsconfig = resolve_tsconfig(&root, None).unwrap();
    let graph = crate::codebase::dependencies::graph::DepGraph::build_with_plan(
        &root,
        &tsconfig,
        crate::codebase::dependencies::graph::GraphBuildPlan::imports_and_workspace(),
    )
    .unwrap();
    let findings = check_application(&root, &opts, &graph).unwrap();
    assert_eq!(
        findings.len(),
        1,
        "absolute path root should produce the same finding"
    );
    assert_eq!(
        findings[0].file, "entrypoints/api.mts",
        "file should be repo-relative even when root is absolute"
    );
}

mod config_path;
mod globs;
mod prepared_support;
