use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

// Output cases moved to the cli_pkgcfg_snapshot modules.

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/workspace-package-cycles")
            .join(name),
    )
}

fn package_dependencies(path: &Path, dependency_types: &[&str]) -> BTreeSet<String> {
    crate::codebase::package_deps::dependency_names(path, dependency_types)
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
fn cycle_detection_ignores_external_edges_inside_cyclic_components() {
    let mut graph = BTreeMap::new();
    graph.insert(
        "a".to_string(),
        BTreeSet::from(["b".to_string(), "outside".to_string()]),
    );
    graph.insert("b".to_string(), BTreeSet::from(["a".to_string()]));
    graph.insert("outside".to_string(), BTreeSet::new());

    assert_eq!(
        scc::cycle_keys(&graph),
        BTreeSet::from(["a -> b".to_string()])
    );
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
    assert_eq!(scc::canonical_cycle(""), "");
}

#[test]
fn empty_workspace_and_wrapped_cycle_helpers() {
    let root = fixture_root("invalid-package-json");
    let files = vec![root.join("packages/api/package.json")];

    assert_eq!(scc::canonical_cycle("b -> a -> b"), "a -> b");
    assert_eq!(scc::canonical_cycle("a -> "), "");

    let mut invalid = NoMistakesConfig::default();
    invalid.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("true").unwrap(),
        ..Default::default()
    });
    assert!(check_with_files(&root, &invalid, &files).is_err());
}

#[test]
fn version_range_match_follows_workspace_linking_rules() {
    assert!(super::range::version_satisfies("*", "1.2.0"));
    assert!(super::range::version_satisfies("^1.0.0", "1.2.0"));
    assert!(!super::range::version_satisfies("^1.0.0", "2.0.0"));
    assert!(!super::range::version_satisfies("^2.0.0", "1.0.0"));
    assert!(super::range::version_satisfies("~1.2.0", "1.2.9"));
    assert!(!super::range::version_satisfies("~1.2.0", "1.3.0"));
    assert!(super::range::version_satisfies(">=1.2.0 <2.0.0", "1.4.0"));
    assert!(!super::range::version_satisfies(">=1.2.0 <2.0.0", "2.0.0"));
    assert!(super::range::version_satisfies(">1.2.0", "1.2.1"));
    assert!(!super::range::version_satisfies(">1.2.0", "1.2.0"));
    assert!(super::range::version_satisfies("<=1.2.0", "1.2.0"));
    assert!(!super::range::version_satisfies("<1.2.0", "1.2.0"));
    assert!(super::range::version_satisfies("^0.2.3", "0.2.9"));
    assert!(!super::range::version_satisfies("^0.2.3", "0.3.0"));
    assert!(super::range::version_satisfies("^0.0.3", "0.0.3"));
    assert!(!super::range::version_satisfies("^0.0.3", "0.0.4"));
    assert!(!super::range::version_satisfies("^1.0.0", "1.0.0-rc.1"));
    assert!(!super::range::version_satisfies("latest", "1.0.0"));
    assert!(super::range::version_satisfies("1.2.3 || ^2.0.0", "2.1.0"));
    assert!(super::range::version_satisfies("x", "1.2.0"));
    assert!(super::range::version_satisfies("X", "9.0.0"));
    assert!(super::range::version_satisfies(">=1.0.0 *", "1.2.0"));
    assert!(super::range::version_satisfies("=1.2.3", "1.2.3"));
    assert!(super::range::version_satisfies("=1.2.3", "1.2.3+build.1"));
    assert!(!super::range::version_satisfies("=1.2.3", "1.2.4"));
    assert!(!super::range::version_satisfies("=1.2.3", "1.2.3-rc.1"));
    assert!(super::range::version_satisfies("1", "1.9.0"));
    assert!(!super::range::version_satisfies("1", "2.0.0"));
    assert!(super::range::version_satisfies("1.2", "1.2.9"));
    assert!(!super::range::version_satisfies("1.2", "1.3.0"));
    assert!(super::range::version_satisfies("1.2.3", "1.2.3"));
    assert!(!super::range::version_satisfies("1.2.3", "1.2.4"));
    assert!(!super::range::version_satisfies("2", "1.0.0"));
    assert!(super::range::version_satisfies("~1", "1.9.0"));
    assert!(!super::range::version_satisfies("~1.2.0", "1.1.0"));
    assert!(!super::range::version_satisfies("~1.2.0", "1.2.0-rc.1"));
    assert!(super::range::version_satisfies("1.2.x", "1.2.9"));
    assert!(super::range::version_satisfies("1.X", "1.9.0"));
    assert!(!super::range::version_satisfies("1.2.x", "1.2.0-rc.1"));
    assert!(!super::range::version_satisfies("1.x.foo", "1.0.0"));
    assert!(!super::range::version_satisfies("2.x", "1.0.0"));
    assert!(!super::range::version_satisfies(">=nope", "1.0.0"));
    assert!(!super::range::version_satisfies("1.0.0", "nope"));
    assert!(!super::range::version_satisfies("1.2.3.4", "1.2.3"));
    assert!(!super::range::version_satisfies("1.0.0", "1.2.3.4"));
}

#[test]
fn non_string_versions_are_not_workspace_edges() {
    let root = fixture_root("non-string-version");
    let files = [
        root.join("package.json"),
        root.join("packages/a/package.json"),
        root.join("packages/b/package.json"),
    ];
    let sources = super::super::source_store_for_files(&files);
    assert!(super::edges::package_version(&root.join("packages/missing"), &sources).is_none());
}

#[test]
fn local_paths_count_only_when_they_are_the_package_directory() {
    let root = fixture_root("file-cycle");
    let package_a = root.join("packages/a");
    let package_b = root.join("packages/b");
    let absolute = package_b.to_str().expect("fixture path is unicode");
    assert!(super::edges::path_is_package_dir(
        absolute, &package_a, &package_b
    ));
    assert!(!super::edges::path_is_package_dir(
        "", &package_a, &package_b
    ));
    assert!(!super::edges::path_is_package_dir(
        package_a.to_str().expect("fixture path is unicode"),
        &package_a,
        &package_b
    ));
    let manifest = package_b.join("package.json");
    assert!(super::edges::path_is_package_dir(
        manifest.to_str().expect("fixture path is unicode"),
        &package_a,
        &package_b
    ));
}
