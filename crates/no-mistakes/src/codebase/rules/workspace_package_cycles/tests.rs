use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/workspace-package-cycles")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

#[test]
fn reports_workspace_dependency_cycles() {
    let root = fixture_root("cycle");
    let files = vec![
        root.join("package.json"),
        root.join("packages/api/package.json"),
        root.join("packages/domain/package.json"),
        root.join("packages/ui/package.json"),
    ];
    let findings = check_with_files(&root, &config("{}"), &files).unwrap();

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "packages/api/package.json");
    assert!(findings[0]
        .message
        .contains("@x/api -> @x/domain -> @x/api"));
}

#[test]
fn allowlist_suppresses_known_cycle() {
    let root = fixture_root("cycle");
    let files = vec![
        root.join("package.json"),
        root.join("packages/api/package.json"),
        root.join("packages/domain/package.json"),
        root.join("packages/ui/package.json"),
    ];
    let findings = check_with_files(
        &root,
        &config("allowlist: [\"@x/domain -> @x/api -> @x/domain\"]\n"),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn dependency_type_filter_can_ignore_dev_dependency_cycles() {
    let root = fixture_root("dev-cycle");
    let files = vec![
        root.join("package.json"),
        root.join("packages/api/package.json"),
        root.join("packages/domain/package.json"),
    ];
    let findings =
        check_with_files(&root, &config("dependencyTypes: [dependencies]\n"), &files).unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn dense_cycle_graph_reports_bounded_cycles_per_cyclic_edge() {
    let mut graph = BTreeMap::new();
    graph.insert(
        "a".to_string(),
        BTreeSet::from(["b".to_string(), "c".to_string()]),
    );
    graph.insert(
        "b".to_string(),
        BTreeSet::from(["a".to_string(), "c".to_string()]),
    );
    graph.insert(
        "c".to_string(),
        BTreeSet::from(["a".to_string(), "b".to_string()]),
    );

    let cycles = scc::cycle_keys(&graph);

    assert_eq!(
        cycles,
        BTreeSet::from([
            "a -> b".to_string(),
            "a -> c".to_string(),
            "b -> c".to_string(),
        ])
    );
}

#[test]
fn overlapping_component_reports_cycles_beyond_an_allowlisted_pair() {
    let mut graph = BTreeMap::new();
    graph.insert("a".to_string(), BTreeSet::from(["b".to_string()]));
    graph.insert(
        "b".to_string(),
        BTreeSet::from(["a".to_string(), "c".to_string()]),
    );
    graph.insert("c".to_string(), BTreeSet::from(["b".to_string()]));

    let mut cycles = scc::cycle_keys(&graph);
    cycles.remove("a -> b");

    assert_eq!(cycles, BTreeSet::from(["b -> c".to_string()]));
}

#[test]
fn cycle_detection_handles_self_cycles_and_external_edges() {
    let mut graph = BTreeMap::new();
    graph.insert(
        "a".to_string(),
        BTreeSet::from(["a".to_string(), "outside".to_string()]),
    );
    graph.insert("b".to_string(), BTreeSet::from(["outside".to_string()]));

    assert_eq!(scc::cycle_keys(&graph), BTreeSet::from(["a".to_string()]));
}

#[test]
fn cycle_detection_returns_empty_for_acyclic_graph() {
    let mut graph = BTreeMap::new();
    graph.insert("a".to_string(), BTreeSet::from(["b".to_string()]));
    graph.insert("b".to_string(), BTreeSet::new());

    assert!(scc::cycle_keys(&graph).is_empty());
}

#[test]
fn package_dependency_helpers_tolerate_missing_and_invalid_files() {
    let root = fixture_root("invalid-package-json");

    assert!(package_dependencies(&root.join("missing/package.json"), &["dependencies"]).is_empty());
    assert!(
        package_dependencies(&root.join("packages/api/package.json"), &["dependencies"]).is_empty()
    );
    assert_eq!(canonical_cycle(""), "");
}
