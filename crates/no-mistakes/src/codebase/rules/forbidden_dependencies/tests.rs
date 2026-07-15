use super::*;

#[test]
fn prepared_forbidden_dependencies_never_enters_legacy_discovery_fallback() {
    let source = include_str!("shared.rs");
    let body = source
        .split("fn check_with_prepared_facts_and_graph(")
        .nth(1)
        .expect("prepared forbidden-dependencies body");

    assert!(!body.contains("return super::check_with_config"));
}
use std::path::{Path, PathBuf};

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
    check_rule_application(root, &config, &config.rules[0], opts, graph, None)
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
fn basic_forbidden_module_fails() {
    let root = fixture("forbidden-dependencies-basic");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = check(&root, &config, None).unwrap();
    assert_eq!(findings.len(), 1);
    let f = &findings[0];
    assert_eq!(f.rule, RULE_ID);
    assert_eq!(f.file, "entrypoints/api.mts");
    assert_eq!(f.target.as_deref(), Some("sharp"));
    assert!(f.message.contains("forbidden module"));
    assert!(f.message.contains("sharp"));
    assert!(f.message.contains("Reproduce:"));
    assert!(f.message.contains("no-mistakes dependencies"));
    assert!(f.message.contains("--target-module"));
}

#[test]
fn passes_fixture_has_no_findings() {
    let root = fixture("forbidden-dependencies-passes");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = check(&root, &config, None).unwrap();
    assert!(
        findings.is_empty(),
        "expected no findings but got {findings:?}"
    );
}

#[test]
fn glob_module_pattern_matches() {
    let root = fixture("forbidden-dependencies-glob-module");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = check(&root, &config, None).unwrap();
    assert!(
        findings
            .iter()
            .any(|f| f.target.as_deref() == Some("@scope/heavy")),
        "expected finding for @scope/heavy but got {findings:?}"
    );
}

#[test]
fn forbidden_file_is_detected() {
    let root = fixture("forbidden-dependencies-file");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = check(&root, &config, None).unwrap();
    assert!(
        findings
            .iter()
            .any(|f| f.target.as_deref().is_some_and(|t| t.contains("worker"))),
        "expected finding for worker.mts but got {findings:?}"
    );
    let f = findings
        .iter()
        .find(|f| f.target.as_deref().is_some_and(|t| t.contains("worker")))
        .unwrap();
    assert!(f.message.contains("Reproduce:"));
    assert!(f.message.contains("--filter"));
}

#[test]
fn type_import_relationship_fires_on_type_import() {
    let root = fixture("forbidden-dependencies-relationships");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = check(&root, &config, None).unwrap();
    assert!(
        findings
            .iter()
            .any(|f| f.target.as_deref() == Some("sharp")),
        "expected finding for sharp via type import but got {findings:?}"
    );
}

#[test]
fn multiple_applications_each_fire_independently() {
    let root = fixture("forbidden-dependencies-multi");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = check(&root, &config, None).unwrap();
    assert!(
        findings
            .iter()
            .any(|f| f.file == "entrypoints/api.mts" && f.target.as_deref() == Some("sharp")),
        "expected finding for api.mts -> sharp but got {findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|f| f.file == "entrypoints/worker.mts" && f.target.as_deref() == Some("canvas")),
        "expected finding for worker.mts -> canvas but got {findings:?}"
    );
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
fn directory_root_is_silently_skipped() {
    let root = fixture("forbidden-dependencies-basic");
    let opts = Options {
        roots: vec!["entrypoints".to_string()],
        forbidden_modules: vec!["sharp".to_string()],
        forbidden_files: vec![],
        relationships: vec![],
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
        "directory root should produce no findings (not a file)"
    );
}

#[test]
fn all_relationships_is_same_as_omitted() {
    let root = fixture("forbidden-dependencies-basic");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();

    let findings_empty_rels = check(&root, &config, None).unwrap();

    // Override relationships via Options directly — `relationships: [all]` should equal `relationships: []`.
    let opts_all = Options {
        roots: vec!["entrypoints/api.mts".to_string()],
        forbidden_modules: vec!["sharp".to_string()],
        forbidden_files: vec![],
        relationships: vec![crate::codebase::dependencies::RelationshipArg::All],
    };
    let tsconfig = resolve_tsconfig(&root, None).unwrap();
    let plan = crate::codebase::dependencies::graph::GraphBuildPlan::from_allowed(
        relationship_filter(&opts_all.relationships).as_ref(),
    );
    let graph =
        crate::codebase::dependencies::graph::DepGraph::build_with_plan(&root, &tsconfig, plan)
            .unwrap();
    let findings_all = check_application(&root, &opts_all, &graph).unwrap();
    assert_eq!(findings_empty_rels.len(), findings_all.len());
}

#[test]
fn queue_job_nodes_are_not_matched() {
    use crate::codebase::dependencies::graph::test_support::from_typed_maps;
    use crate::codebase::dependencies::{EdgeKind, NodeId};
    let root = fixture("forbidden-dependencies-basic");
    let root_file = crate::codebase::ts_resolver::normalize_path(&root.join("entrypoints/api.mts"));
    let queue_file = crate::codebase::ts_resolver::normalize_path(&root.join("jobs/queue.mts"));
    let root_node = NodeId::File(root_file.clone());
    let queue_node = NodeId::QueueJob {
        queue_file: queue_file.clone(),
        job: "process".to_string(),
    };
    let forward = std::collections::HashMap::from([(
        root_node.clone(),
        vec![(queue_node.clone(), EdgeKind::QueueEnqueue)],
    )]);
    let reverse = std::collections::HashMap::from([(
        queue_node.clone(),
        vec![(root_node.clone(), EdgeKind::QueueEnqueue)],
    )]);
    let graph = from_typed_maps(root.clone(), forward, reverse);
    let opts = Options {
        roots: vec!["entrypoints/api.mts".to_string()],
        forbidden_modules: vec!["*".to_string()],
        forbidden_files: vec!["**".to_string()],
        relationships: vec![],
    };
    let findings = check_application(&root, &opts, &graph).unwrap();
    assert!(findings.is_empty(), "QueueJob nodes should not be matched");
}

#[test]
fn source_filter_excludes_matching_forbidden_root() {
    let root = fixture("forbidden-dependencies-basic");
    let config = NoMistakesConfig {
        rules: vec![crate::config::v2::schema::RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(crate::config::v2::schema::RuleScope::Repository),
            exclude: vec!["entrypoints/api.mts".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };
    let opts = Options {
        roots: vec!["entrypoints/api.mts".to_string()],
        forbidden_modules: vec!["sharp".to_string()],
        forbidden_files: vec![],
        relationships: vec![],
    };
    let tsconfig = resolve_tsconfig(&root, None).unwrap();
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        crate::codebase::dependencies::graph::GraphBuildPlan::imports_and_workspace(),
    )
    .unwrap();

    let findings =
        check_rule_application(&root, &config, &config.rules[0], &opts, &graph, None).unwrap();

    assert!(findings.is_empty());
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

/// Regression test: `check_with_config` (used by both `run_check` and
/// `check_with_facts`'s missing-graph-universe fallback) must
/// resolve the `DepGraph`'s `GraphConfigOptions` from the given
/// `config_path`, not silently fall back to default discovery — `check`
/// itself delegates to `check_with_config(.., None, ..)`, so this also
/// covers that `check_with_config(.., Some(path), ..)` genuinely differs.
///
/// Reuses the `graph-default-route-config`/`graph-empty-route-config`
/// fixture pair from the graph module's own config_path tests: the former's
/// `.no-mistakes.yml` configures a real `backendPattern` (so `src/client.ts`
/// reaches `backend/api/users.mts` via a `RouteRef` edge), the latter's
/// configures an empty one (so it doesn't).
#[test]
fn check_with_config_uses_explicit_config_path() {
    let root = fixture("graph-default-route-config");
    let empty_config = fixture("graph-empty-route-config").join(".no-mistakes.yml");
    let raw = r#"
rules:
  - rule: forbidden-dependencies
    scope: repository
    options:
      roots:
        - src/client.ts
      forbiddenFiles:
        - backend/api/users.mts
      relationships:
        - route
"#;
    let config: NoMistakesConfig = serde_yaml::from_str(raw).unwrap();

    let default_findings = check_with_config(&root, &config, None, None).unwrap();
    assert!(
        !default_findings.is_empty(),
        "default-discovered config (this fixture's own .no-mistakes.yml) should register the backend route pattern and produce a finding via the RouteRef edge"
    );

    let explicit_findings = check_with_config(&root, &config, Some(&empty_config), None).unwrap();
    assert!(
        explicit_findings.is_empty(),
        "passing the explicit empty-pattern config must be honored, not silently ignored in favor of default discovery"
    );
}
