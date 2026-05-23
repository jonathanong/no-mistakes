use super::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase-analysis")
            .join(name),
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
fn invalid_glob_pattern_returns_error() {
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
    let result = check_application(&root, &opts, &graph);
    assert!(result.is_err(), "expected error for invalid glob pattern");
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
fn no_rule_configured_returns_empty() {
    let root = fixture("forbidden-dependencies-basic");
    // Build a config that has no forbidden-dependencies rule at all
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    // Override: use a fixture with no forbidden-dependencies config to get an empty rule list
    let empty_root = fixture("forbidden-dependencies-passes");
    // passes fixture HAS the rule — use a fixture without any rules
    let no_rule_root = fixture("forbidden-dependencies-basic");
    // Directly confirm early return: load the passes fixture which still has the rule
    // Instead, build a config from scratch with no rules via yaml
    let raw = "rules: []";
    let cfg: crate::config::v2::NoMistakesConfig = serde_yaml::from_str(raw).unwrap();
    let findings = check(&root, &cfg, None).unwrap();
    assert!(
        findings.is_empty(),
        "should return empty when rule not configured"
    );
    let _ = (config, empty_root, no_rule_root);
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
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    // Build opts with an absolute path for the root
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
    let _ = config;
}
