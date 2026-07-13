use super::*;
use crate::playwright::config::Settings;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

fn settings(selector_include: Vec<String>) -> Settings {
    Settings {
        frontend_root: "web/app".to_string(),
        playwright_configs: vec![],
        project: None,
        test_include: vec![],
        test_exclude: vec![],
        ignore_routes: vec![],
        rewrites: vec![],
        navigation_helpers: vec![],
        selector_attributes: vec!["data-pw".to_string()],
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: vec!["web/app".to_string()],
        selector_include,
        selector_exclude: vec![],
    }
}

fn collect_for_test(
    root: &Path,
    settings: &Settings,
    routes: &[routes::Route],
) -> anyhow::Result<BTreeMap<Arc<String>, BTreeSet<Arc<String>>>> {
    let tsconfig =
        crate::playwright::analysis::pipeline_setup::load_route_import_tsconfig(root, settings)?;
    let source_files = collect_route_source_files(root, settings)?;
    let mut graph_file_paths = crate::codebase::dependencies::graph::GraphFiles::discover(root)
        .all()
        .to_vec();
    graph_file_paths.extend_from_slice(&source_files.graph_files);
    graph_file_paths.sort();
    graph_file_paths.dedup();
    let graph_files =
        crate::codebase::dependencies::graph::GraphFiles::from_files(graph_file_paths);
    let graph =
        crate::codebase::dependencies::graph::DepGraph::build_with_plan_files_config_and_facts(
            root,
            &tsconfig,
            crate::codebase::dependencies::graph::GraphBuildPlan {
                route_imports: true,
                ..Default::default()
            },
            &graph_files,
            None,
            None,
        );
    collect_route_reachable_files(root, settings, routes, &graph, &source_files)
}

fn root_route(root: &Path) -> routes::Route {
    routes::Route {
        file: root.join("web/app/page.tsx"),
        pattern: "/".to_string(),
    }
}

#[test]
fn route_reachability_honors_selector_include() {
    let root = crate::playwright::test_support::fixture_path(&[
        "nextjs-selectors",
        "selector-text-locator",
    ]);
    let reachable = collect_for_test(
        &root,
        &settings(vec!["web/app/components/unreachable-*.tsx".to_string()]),
        &[root_route(&root)],
    )
    .expect("route reachability should collect");
    assert!(reachable
        .get(&Arc::new("web/app/page.tsx".to_string()))
        .is_some_and(BTreeSet::is_empty));
}

#[test]
fn route_reachability_resolves_runtime_imports_and_excludes_types() {
    let root = crate::playwright::test_support::fixture_path(&[
        "nextjs-selectors",
        "selector-text-locator",
    ]);
    let reachable = collect_for_test(&root, &settings(vec![]), &[root_route(&root)])
        .expect("route reachability should collect");
    let files = reachable
        .get(&Arc::new("web/app/page.tsx".to_string()))
        .expect("route should have reachable files");
    for expected in [
        "web/app/components/discuss-button.tsx",
        "web/app/components/reexported-button.tsx",
        "web/app/components/export-all-button.tsx",
    ] {
        assert!(
            files.contains(&Arc::new(expected.to_string())),
            "missing {expected}"
        );
    }
    assert!(!files.contains(&Arc::new(
        "web/app/components/type-only-button.tsx".to_string()
    )));
}

#[test]
fn route_reachability_includes_layout_and_template_only() {
    let root =
        crate::playwright::test_support::fixture_path(&["nextjs-selectors", "route-wrappers"]);
    let reachable = collect_for_test(&root, &settings(vec![]), &[root_route(&root)])
        .expect("route reachability should collect");
    let files = reachable
        .get(&Arc::new("web/app/page.tsx".to_string()))
        .expect("route should have reachable files");
    assert!(files.contains(&Arc::new(
        "web/app/components/layout-button.tsx".to_string()
    )));
    assert!(files.contains(&Arc::new(
        "web/app/components/template-button.tsx".to_string()
    )));
    assert!(!files.contains(&Arc::new("web/app/components/error-button.tsx".to_string())));
}

#[test]
fn route_reachability_uses_frontend_tsconfig_and_literal_dynamic_imports() {
    let root =
        crate::playwright::test_support::fixture_path(&["nextjs-selectors", "frontend-tsconfig"]);
    let reachable = collect_for_test(&root, &settings(vec![]), &[root_route(&root)])
        .expect("route reachability should collect");
    let files = reachable
        .get(&Arc::new("web/app/page.tsx".to_string()))
        .expect("route should have reachable files");
    for expected in [
        "web/app/components/alias-button.tsx",
        "web/app/components/dynamic-button.tsx",
        "web/app/components/template-button.tsx",
        "web/app/components/wrapped-button.tsx",
        "web/app/components/wrapped-template-button.tsx",
        "web/app/components/[id]-button.tsx",
        "web/app/fixtures/fixture-button.tsx",
        "web/app/components/cycle-a.ts",
        "web/app/components/cycle-b.ts",
    ] {
        assert!(
            files.contains(&Arc::new(expected.to_string())),
            "missing {expected}"
        );
    }
    assert!(!files.contains(&Arc::new(
        "web/app/components/required-button.tsx".to_string()
    )));
}

#[test]
fn standalone_and_shared_fact_route_reachability_match() {
    let root =
        crate::playwright::test_support::fixture_path(&["nextjs-selectors", "frontend-tsconfig"]);
    let settings = settings(vec![]);
    let route = root_route(&root);
    let standalone = collect_for_test(&root, &settings, std::slice::from_ref(&route))
        .expect("standalone reachability collects");
    let source_files = collect_route_source_files(&root, &settings).expect("sources collect");
    let mut files = crate::codebase::ts_source::discover_files(&root, &[]);
    files.extend_from_slice(&source_files.graph_files);
    files.sort();
    files.dedup();
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            graph: crate::codebase::ts_source::facts::TsFactPlan {
                imports: true,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let graph = crate::playwright::analysis::pipeline_setup::build_route_import_graph(
        &root,
        &settings,
        Some(&facts),
        &source_files.graph_files,
    )
    .expect("shared-fact graph builds");
    let shared = collect_route_reachable_files(&root, &settings, &[route], &graph, &source_files)
        .expect("shared reachability collects");

    assert_eq!(shared, standalone);
}

#[test]
fn route_reachability_surfaces_only_reached_parse_errors() {
    let root =
        crate::playwright::test_support::fixture_path(&["react-traits-components", "bad-file"]);
    let route = routes::Route {
        file: root.join("app/components/Broken.tsx"),
        pattern: "/broken".to_string(),
    };
    let error = collect_for_test(&root, &settings(vec![]), &[route])
        .expect_err("malformed route files should surface import parse errors");
    assert!(error.to_string().contains("Broken.tsx"));
}

#[test]
fn route_reachability_ignores_unreached_parse_errors() {
    let root =
        crate::playwright::test_support::fixture_path(&["react-traits-components", "bad-file"]);
    let route = routes::Route {
        file: root.join("web/app/page.tsx"),
        pattern: "/".to_string(),
    };
    collect_for_test(&root, &settings(vec![]), &[route])
        .expect("an unrelated malformed source must not fail route reachability");
}
