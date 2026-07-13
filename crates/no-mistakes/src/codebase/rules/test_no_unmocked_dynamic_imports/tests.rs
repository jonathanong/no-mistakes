use super::*;
use crate::codebase::dependencies::graph::test_support::from_raw_maps;
use crate::codebase::ts_resolver::load_tsconfig;
use dashmap::DashMap;
use std::collections::HashMap;

mod visibility;

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/test-no-unmocked-dynamic-imports/fixture"),
    )
}

fn resolve_tsconfig(root: &Path, explicit: Option<&Path>) -> Result<TsConfig> {
    crate::codebase::ts_resolver::resolve_tsconfig(explicit, root)
}

fn collect_reachable_findings(
    ctx: reachable::ReachableContext<'_>,
    test_file: &Path,
    mocks: &HashSet<PathBuf>,
    dependency_cache: &DashMap<PathBuf, Arc<Vec<PathBuf>>>,
) -> Vec<RuleFinding> {
    reachable::collect(ctx, test_file, mocks, dependency_cache)
        .unwrap()
        .findings
        .into_iter()
        .map(|entry| entry.finding)
        .collect()
}

#[test]
fn typed_mock_carrier_does_not_leak_mocked_leaf_into_reachability_scanning() {
    // Regression for #506: `vi.mock(import("./dep"), factory)`'s carrier is a real
    // ImportExpression AST node, so the shared dependency graph (built independently of
    // this rule's own extractor) records a dynamic-import edge from the test file to the
    // mocked leaf even when the test never dynamically imports it directly. Before the
    // `reachable::collect` fix, this made the mocked leaf's own (unmocked) internal
    // dynamic import look reachable and reportable, even though the leaf never executes
    // once mocked. Covers both the typed and string-literal mock forms.
    let root = fixture();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = check(&root, &config, None).unwrap();
    assert!(
        !findings.iter().any(|f| {
            f.file.contains("typed-mock-reachable")
                || f.file.contains("string-mock-reachable")
                || f.target.as_deref() == Some("src/typed-mock-reachable-internal.mts")
                || f.target.as_deref() == Some("src/typed-mock-reachable-leaf.mts")
        }),
        "a mocked leaf's own internals must not be scanned via reachability: {findings:?}"
    );
}

#[test]
fn fixture_reports_unmocked_transitive_and_nonliteral_dynamic_imports() {
    let root = fixture();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = check(&root, &config, None).unwrap();
    assert!(findings
        .iter()
        .any(|f| f.target.as_deref() == Some("src/unmocked-child.mts")));
    assert!(findings
        .iter()
        .any(|f| f.message.contains("dynamic import")));
    assert!(!findings
        .iter()
        .any(|f| f.file.contains("disabled.test.mts")));
    assert!(!findings
        .iter()
        .any(|f| f.target.as_deref() == Some("src/types.mts")));
    assert!(findings.iter().any(|f| {
        f.file == "tests/jest-setup-leak.test.mts"
            && f.target.as_deref() == Some("src/jest-setup-target.mts")
    }));
    assert!(!findings.iter().any(|f| {
        f.file == "src/next-dynamic-component.mts"
            && f.target.as_deref() == Some("src/dynamic-leaf.mts")
    }));
    assert!(!findings.iter().any(|f| {
        f.file == "src/next-dynamic-colocated.tsx"
            && f.target.as_deref() == Some("src/topic-edit-tabs.tsx")
    }));
    assert!(findings.iter().any(|f| {
        f.file == "src/unmocked-next-dynamic-component.mts"
            && f.target.as_deref() == Some("src/dynamic-leaf.mts")
    }));
}

#[test]
fn typed_mock_import_specifiers_cover_dependencies_and_keep_factory_imports() {
    // Regression for issue #506: `vi.mock(import("./dep"), factory)` /
    // `vi.doMock(import("./dep"), factory)` must cover the typed dependency, and a genuine
    // dynamic import written inside the factory body must still be flagged.
    let root = fixture();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = check(&root, &config, None).unwrap();
    assert!(
        !findings
            .iter()
            .any(|f| f.file == "tests/typed-mock.test.mts"),
        "typed vi.mock specifier must cover the dependency"
    );
    assert!(
        !findings
            .iter()
            .any(|f| f.file == "tests/typed-do-mock.test.mts"),
        "typed vi.doMock specifier must cover the dependency"
    );
    assert!(
        !findings
            .iter()
            .any(|f| f.target.as_deref() == Some("src/typed-covered.mts")),
        "the type-carrier import must never itself be flagged"
    );
    assert!(
        findings.iter().any(|f| {
            f.file == "tests/typed-mock-factory.test.mts"
                && f.target.as_deref() == Some("src/typed-factory-leaf.mts")
        }),
        "a genuine dynamic import inside the mock factory body must still be flagged"
    );
}

#[test]
fn next_line_disable_and_unresolved_import_branches_are_reported() {
    let root = fixture();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let disabled = root.join("tests").join("next-line-disabled.test.mts");
    let source = std::fs::read_to_string(&disabled).unwrap();
    assert!(has_disable_comment(&source, 5, RULE_ID));

    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig);
    let graph = from_raw_maps(root.clone(), Default::default(), Default::default());
    let mocks = HashSet::new();
    let dependency_cache = DashMap::new();
    let mut findings = Vec::new();
    let mut context = DynamicCheckContext {
        root: &root,
        file: &disabled,
        resolver: &resolver,
        graph: &graph,
        mocks: &mocks,
        dependency_cache: &dependency_cache,
        findings: &mut findings,
    };
    check_dynamic_import(
        &mut context,
        ast::DynamicImport {
            specifier: Some("./missing.mts".to_string()),
            line: 1,
        },
    );
    assert_eq!(findings[0].import.as_deref(), Some("./missing.mts"));

    let mut mocks = HashSet::new();
    mocks.insert(PathBuf::from("external-lib"));
    let mut findings = Vec::new();
    let mut context = DynamicCheckContext {
        root: &root,
        file: &disabled,
        resolver: &resolver,
        graph: &graph,
        mocks: &mocks,
        dependency_cache: &dependency_cache,
        findings: &mut findings,
    };
    check_dynamic_import(
        &mut context,
        ast::DynamicImport {
            specifier: Some("external-lib".to_string()),
            line: 1,
        },
    );
    assert!(findings.is_empty());

    assert!(check(&root, &config, None)
        .unwrap()
        .iter()
        .all(|f| !f.file.contains("next-line-disabled")));
}

#[test]
fn mocked_dynamic_import_target_skips_transitive_dependency_checks() {
    let root = fixture();
    let tsconfig = load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let resolver = ImportResolver::new(&tsconfig);
    let graph = DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::all()).unwrap();
    let test_file = root.join("tests").join("good.test.mts");
    let target = root.join("src").join("lazy.mts");
    let mut mocks = HashSet::new();
    mocks.insert(target);
    let dependency_cache = DashMap::new();
    let mut findings = Vec::new();
    let mut context = DynamicCheckContext {
        root: &root,
        file: &test_file,
        resolver: &resolver,
        graph: &graph,
        mocks: &mocks,
        dependency_cache: &dependency_cache,
        findings: &mut findings,
    };
    check_dynamic_import(
        &mut context,
        ast::DynamicImport {
            specifier: Some("../src/lazy.mts".to_string()),
            line: 1,
        },
    );
    assert!(findings.is_empty());
}

#[test]
fn reachable_dependencies_respect_skips_and_disable_comments() {
    let root = fixture();
    let tsconfig = load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let resolver = ImportResolver::new(&tsconfig);
    let mocks = HashSet::new();

    for (case, dependency, skip_directories) in [
        (
            "reachable-disabled-file",
            "src/reachable-disabled-file.mts",
            Vec::new(),
        ),
        (
            "reachable-disabled-line",
            "src/reachable-disabled-line.mts",
            Vec::new(),
        ),
        (
            "reachable-skipped-prefix",
            "src/generated/reachable-skipped.mts",
            vec!["src/generated".to_string()],
        ),
    ] {
        let test_file = root.join("cases").join(format!("{case}.case.mts"));
        let dependency = root.join(dependency);
        let mut forward = HashMap::new();
        forward.insert(test_file.clone(), vec![dependency]);
        let graph = from_raw_maps(root.clone(), forward, Default::default());
        let mut config = NoMistakesConfig::default();
        config.filesystem.skip_directories = skip_directories;
        let dependency_cache = DashMap::new();
        let findings = collect_reachable_findings(
            reachable::ReachableContext {
                root: &root,
                config: &config,
                resolver: &resolver,
                graph: &graph,
                shared: None,
                file_cache: None,
            },
            &test_file,
            &mocks,
            &dependency_cache,
        );

        assert!(findings.is_empty(), "{case} should not report findings");
    }
}

#[test]
fn repeated_dynamic_import_target_uses_dependency_cache() {
    let root = fixture();
    let tsconfig = load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let resolver = ImportResolver::new(&tsconfig);
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let test_file = root.join("tests").join("bad.test.mts");
    let mocks = HashSet::new();
    let dependency_cache = DashMap::new();
    let mut findings = Vec::new();
    let mut context = DynamicCheckContext {
        root: &root,
        file: &test_file,
        resolver: &resolver,
        graph: &graph,
        mocks: &mocks,
        dependency_cache: &dependency_cache,
        findings: &mut findings,
    };
    check_dynamic_import(
        &mut context,
        ast::DynamicImport {
            specifier: Some("../src/lazy.mts".to_string()),
            line: 1,
        },
    );
    let target = root.join("src").join("lazy.mts");
    let cached_deps = dependency_cache
        .get(&target)
        .map(|r| r.clone())
        .expect("target should be cached after first call");
    let expected_deps = runtime_deps(&graph, target.clone());
    assert_eq!(*cached_deps, expected_deps);

    let cache_len = dependency_cache.len();
    check_dynamic_import(
        &mut context,
        ast::DynamicImport {
            specifier: Some("../src/lazy.mts".to_string()),
            line: 1,
        },
    );
    assert_eq!(dependency_cache.len(), cache_len);
    assert!(!context.findings.is_empty());
}

#[test]
fn reachable_check_shared_skips_dep_with_disable_file_comment() {
    let root = fixture();
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig);
    let test_file = root.join("cases").join("reachable-disabled-file.case.mts");
    let dep = root.join("src").join("reachable-disabled-file.mts");
    let mut forward = HashMap::new();
    forward.insert(test_file.clone(), vec![dep.clone()]);
    let graph = from_raw_maps(root.clone(), forward, Default::default());
    let dep_source = std::fs::read_to_string(&dep).unwrap();
    let dep_facts = ast::extract(&dep, &dep_source).unwrap();
    let mut shared_ts = HashMap::new();
    shared_ts.insert(
        dep.clone(),
        crate::codebase::check_facts::CheckFileFacts {
            source: Some(dep_source),
            dynamic_imports: Some(dep_facts),
            ..Default::default()
        },
    );
    let shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![dep],
        ts: shared_ts,
        ..Default::default()
    };
    let mocks = HashSet::new();
    let dependency_cache = DashMap::new();
    let config = crate::config::v2::NoMistakesConfig::default();
    let findings = collect_reachable_findings(
        reachable::ReachableContext {
            root: &root,
            config: &config,
            resolver: &resolver,
            graph: &graph,
            shared: Some(&shared),
            file_cache: None,
        },
        &test_file,
        &mocks,
        &dependency_cache,
    );
    assert!(findings.is_empty());
}

#[test]
fn reachable_check_uses_shared_facts_without_disk_read() {
    // Performance regression test: when shared facts are available for a dep,
    // reachable source analysis must use them instead of reading from disk.
    // A nonexistent dep path proves no disk access occurred.
    let root = fixture();
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig);
    let test_file = root.join("tests").join("bad.test.mts");
    let fake_dep = root.join("src").join("nonexistent-dep.mts");
    let mut forward = HashMap::new();
    forward.insert(test_file.clone(), vec![fake_dep.clone()]);
    let graph = from_raw_maps(root.clone(), forward, Default::default());
    let mut shared_ts = HashMap::new();
    shared_ts.insert(
        fake_dep.clone(),
        crate::codebase::check_facts::CheckFileFacts {
            source: Some("export const x = 1".to_string()),
            dynamic_imports: Some(ast::TestFacts::default()),
            ..Default::default()
        },
    );
    let shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![fake_dep.clone()],
        ts: shared_ts,
        ..Default::default()
    };
    let mocks = HashSet::new();
    let dependency_cache = DashMap::new();
    let config = crate::config::v2::NoMistakesConfig::default();
    collect_reachable_findings(
        reachable::ReachableContext {
            root: &root,
            config: &config,
            resolver: &resolver,
            graph: &graph,
            shared: Some(&shared),
            file_cache: None,
        },
        &test_file,
        &mocks,
        &dependency_cache,
    );
    // dep was not on disk — success proves shared facts were used, not disk
    assert!(!fake_dep.exists());
}

#[test]
fn reachable_check_falls_back_to_disk_when_dep_facts_incomplete() {
    // reachable.rs:54 — closing `}` of `if let (Some(source), Some(facts))`.
    // When a dep is in shared.ts but source/dynamic_imports is None, fall through to disk.
    let root = fixture();
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig);
    let test_file = root.join("tests").join("good.test.mts");
    let dep = root.join("src").join("child.mts");
    let mut forward = HashMap::new();
    forward.insert(test_file.clone(), vec![dep.clone()]);
    let graph = from_raw_maps(root.clone(), forward, Default::default());
    let mut shared_ts = HashMap::new();
    // dep is in shared.ts but with source=None (incomplete facts)
    shared_ts.insert(
        dep.clone(),
        crate::codebase::check_facts::CheckFileFacts {
            source: None,
            dynamic_imports: None,
            ..Default::default()
        },
    );
    let shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![dep],
        ts: shared_ts,
        ..Default::default()
    };
    let mocks = HashSet::new();
    let dependency_cache = DashMap::new();
    let config = crate::config::v2::NoMistakesConfig::default();
    let findings = collect_reachable_findings(
        reachable::ReachableContext {
            root: &root,
            config: &config,
            resolver: &resolver,
            graph: &graph,
            shared: Some(&shared),
            file_cache: None,
        },
        &test_file,
        &mocks,
        &dependency_cache,
    );
    assert!(findings.is_empty());
}

#[test]
fn check_inner_propagates_reachable_dep_disk_error() {
    let root = fixture();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let tsconfig = load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let test_file = root.join("tests").join("bad.test.mts");
    let unreadable = root.join("src").join("unreadable.mts");
    let mut forward = HashMap::new();
    forward.insert(test_file.clone(), vec![unreadable]);
    let graph = from_raw_maps(root.clone(), forward, Default::default());
    let files = vec![test_file];
    let manual_mocks = HashSet::new();
    let error = check_inner(&root, &config, &files, &tsconfig, &graph, &manual_mocks).unwrap_err();
    assert!(error.to_string().contains("failed to read dependency file"));
}

#[test]
fn resolve_tsconfig_covers_explicit_and_default_paths() {
    let root = fixture();
    assert!(resolve_tsconfig(&root, Some(&root.join("tsconfig.json")))
        .unwrap()
        .base_url
        .is_some());
    let temp = tempfile::tempdir().unwrap();
    assert!(resolve_tsconfig(temp.path(), None)
        .unwrap()
        .base_url
        .is_none());
}

#[test]
fn resolve_mock_specifiers_keeps_unresolved_specifier_keys() {
    let root = fixture();
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig);
    let mocks =
        resolve_mock_specifiers(&["external".to_string()], &root.join("test.mts"), &resolver);
    assert!(mocks.contains(&PathBuf::from("external")));
}
